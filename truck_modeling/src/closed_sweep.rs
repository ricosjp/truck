use crate::*;
use std::collections::HashMap;
const PI: Rad<f64> = Rad(std::f64::consts::PI);

pub trait CompleteRSweep {
    type RSweeped;
    fn rsweep(&self, origin: Point3, axis: Vector3) -> Self::RSweeped;
}

impl CompleteRSweep for Vertex {
    type RSweeped = Wire;
    fn rsweep(&self, origin: Point3, axis: Vector3) -> Self::RSweeped {
        let v = builder::rotated(self, origin, axis, PI);
        let edge0 = create_edge(self, &v, origin, axis);
        let edge1 = create_edge(&v, self, origin, axis);
        vec![edge0, edge1].into()
    }
}

fn sub_sweep_edge(edge0: &Edge, edge1: &Edge, origin: Point3, axis: Vector3) -> Face {
    let edge2 = create_edge(edge0.front(), edge1.front(), origin, axis);
    let edge3 = create_edge(edge0.back(), edge1.back(), origin, axis);
    let wire0 = Wire::from(vec![edge0.clone(), edge3, edge1.inverse(), edge2.inverse()]);
    let surface0 = create_surface(edge0, edge1, origin, axis);
    Face::debug_new(vec![wire0], surface0)
}

impl CompleteRSweep for Edge {
    type RSweeped = Shell;
    fn rsweep(&self, origin: Point3, axis: Vector3) -> Self::RSweeped {
        let edge = builder::rotated(self, origin, axis, PI);
        vec![
            sub_sweep_edge(self, &edge, origin, axis),
            sub_sweep_edge(&edge, self, origin, axis),
        ]
        .into()
    }
}

fn new_fedge_bedge(
    v0: &Vertex,
    v1: &Vertex,
    origin: Point3,
    axis: Vector3,
    vemap: &mut HashMap<VertexID, (Edge, Edge)>,
) -> (Edge, Edge)
{
    let curve0 = geom_impls::circle_arc(*v0.lock_point().unwrap(), origin, axis, PI);
    let curve1 = geom_impls::circle_arc(*v1.lock_point().unwrap(), origin, axis, PI);
    let fedge = Edge::debug_new(v0, v1, curve0);
    let bedge = Edge::debug_new(v1, v0, curve1);
    vemap.insert(v0.id(), (fedge.clone(), bedge.clone()));
    (fedge, bedge)
}

fn sub_sweep_wire(
    edge0: &Edge,
    edge1: &Edge,
    origin: Point3,
    axis: Vector3,
    vemap: &mut HashMap<VertexID, (Edge, Edge)>,
) -> (Face, Face)
{
    let (fedge0, bedge0) = match vemap.get(&edge0.front().id()) {
        Some((fedge, bedge)) => (fedge.clone(), bedge.clone()),
        None => new_fedge_bedge(edge0.front(), edge1.front(), origin, axis, vemap),
    };
    let (fedge1, bedge1) = match vemap.get(&edge0.back().id()) {
        Some((fedge, bedge)) => (fedge.clone(), bedge.clone()),
        None => new_fedge_bedge(edge0.back(), edge1.back(), origin, axis, vemap),
    };
    let wire0 = Wire::from(vec![
        edge0.clone(),
        fedge1.clone(),
        edge1.inverse(),
        fedge0.inverse(),
    ]);
    let wire1 = Wire::from(vec![
        edge1.clone(),
        bedge1.clone(),
        edge0.inverse(),
        bedge0.inverse(),
    ]);
    let surface0 = geom_impls::rsweep_surface(&*edge0.lock_curve().unwrap(), origin, axis, PI);
    let surface1 = geom_impls::rsweep_surface(&*edge1.lock_curve().unwrap(), origin, axis, PI);
    (
        Face::debug_new(vec![wire0], surface0),
        Face::debug_new(vec![wire1], surface1),
    )
}

impl CompleteRSweep for Wire {
    type RSweeped = Shell;
    fn rsweep(&self, origin: Point3, axis: Vector3) -> Self::RSweeped {
        let wire = builder::rotated(self, origin, axis, PI);
        let mut vemap = HashMap::<VertexID, (Edge, Edge)>::new();
        let mut shell = Shell::new();
        for (edge0, edge1) in self.edge_iter().zip(wire.edge_iter()) {
            let (face0, face1) = sub_sweep_wire(edge0, edge1, origin, axis, &mut vemap);
            shell.push(face0);
            shell.push(face1);
        }
        shell
    }
}

impl CompleteRSweep for Face {
    type RSweeped = Solid;
    fn rsweep(&self, origin: Point3, axis: Vector3) -> Self::RSweeped {
        let shells: Vec<Shell> = self
            .boundary_iters()
            .into_iter()
            .flatten()
            .map(|wire| wire.rsweep(origin, axis))
            .collect();
        Solid::new(shells)
    }
}

impl CompleteRSweep for Shell {
    type RSweeped = Vec<topology::Result<Solid>>;
    fn rsweep(&self, origin: Point3, axis: Vector3) -> Self::RSweeped {
        self.connected_components().into_iter().map(|shell| {
            let shells: Vec<Shell> = shell.extract_boundaries()
                .into_iter()
                .map(|wire| wire.rsweep(origin, axis))
                .collect();
            Solid::try_new(shells)
        }).collect()
    }
}

fn create_edge(v0: &Vertex, v1: &Vertex, origin: Point3, axis: Vector3) -> Edge {
    let curve = geom_impls::circle_arc(*v0.lock_point().unwrap(), origin, axis, PI);
    Edge::debug_new(v0, v1, curve)
}

fn create_surface(edge0: &Edge, _: &Edge, origin: Point3, axis: Vector3) -> BSplineSurface {
    geom_impls::rsweep_surface(&*edge0.lock_curve().unwrap(), origin, axis, PI)
}
