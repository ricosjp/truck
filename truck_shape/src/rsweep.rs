use crate::geom_impls::*;
use crate::transformed::Transformed;
use crate::{Director, Result};
use geometry::*;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::f64::consts::PI;
use topology::*;

pub trait RSweep: Sized {
    type Output: Sized;
    fn partial_rsweep(
        self,
        origin: &Vector3,
        axis: &Vector3,
        angle: f64,
        builder: &mut Director,
    ) -> Result<Self::Output>;
    fn full_rsweep(
        self,
        origin: &Vector3,
        axis: &Vector3,
        orientation: bool,
        builder: &mut Director,
    ) -> Result<Self::Output>;
    fn rsweep(
        self,
        origin: &Vector3,
        axis: &Vector3,
        angle: f64,
        builder: &mut Director,
    ) -> Result<Self::Output>
    {
        if 2.0 * PI - angle.abs() > Vector::TOLERANCE {
            self.partial_rsweep(origin, axis, angle, builder)
        } else {
            self.full_rsweep(origin, axis, angle > 0.0, builder)
        }
    }
}

impl RSweep for Vertex {
    type Output = Wire;
    fn partial_rsweep(
        self,
        origin: &Vector3,
        axis: &Vector3,
        angle: f64,
        director: &mut Director,
    ) -> Result<Wire>
    {
        let pt = director.try_get_geometry(&self)?;
        let curve = circle_arc(pt, origin, axis, angle);
        let v = self.rotated(origin, axis, angle, director)?;
        let edge = Edge::new_unchecked(self, v);
        director.attach(&edge, curve);
        Ok(Wire::from(vec![edge]))
    }
    fn full_rsweep(
        self,
        origin: &Vector3,
        axis: &Vector3,
        orientation: bool,
        director: &mut Director,
    ) -> Result<Wire>
    {
        let pt = director.try_get_geometry(&self)?;
        let curve0 = circle_arc(pt, origin, axis, PI);
        let curve1 = circle_arc(pt, origin, axis, -PI);
        let v = self.rotated(origin, axis, PI, director)?;
        let edge0 = Edge::new_unchecked(self, v);
        let edge1 = Edge::new_unchecked(self, v);
        director.attach(&edge0, curve0);
        director.attach(&edge1, curve1);
        if orientation {
            Ok(Wire::from(vec![edge0, edge1.inverse()]))
        } else {
            Ok(Wire::from(vec![edge1, edge0.inverse()]))
        }
    }
}

impl RSweep for Edge {
    type Output = Shell;
    fn partial_rsweep(
        self,
        origin: &Vector3,
        axis: &Vector3,
        angle: f64,
        director: &mut Director,
    ) -> Result<Shell>
    {
        let edge2 = self.rotated(origin, axis, angle, director)?;
        let (wire, surface) = sub_partial_sweep_edge(&self, &edge2, origin, axis, angle, director)?;
        let face = Face::new_unchecked(wire);
        director.attach(&face, surface);
        Ok(vec![face].into())
    }
    fn full_rsweep(
        self,
        origin: &Vector3,
        axis: &Vector3,
        orientation: bool,
        director: &mut Director,
    ) -> Result<Shell>
    {
        let edge2 = self.rotated(origin, axis, PI, director)?;
        let (mut wire0, mut surface0) =
            sub_partial_sweep_edge(&self, &edge2, origin, axis, PI, director)?;
        let (mut wire1, mut surface1) =
            sub_partial_sweep_edge(&self, &edge2, origin, axis, -PI, director)?;
        if orientation {
            wire1.invert();
            surface1.swap_axes();
        } else {
            wire0.invert();
            surface0.swap_axes();
        }
        let face0 = Face::new_unchecked(wire0);
        let face1 = Face::new_unchecked(wire1);
        director.attach(&face0, surface0);
        director.attach(&face1, surface1);
        Ok(vec![face0, face1].into())
    }
}

fn sub_partial_sweep_edge(
    edge0: &Edge,
    edge2: &Edge,
    origin: &Vector3,
    axis: &Vector3,
    angle: f64,
    director: &mut Director,
) -> Result<(Wire, BSplineSurface)>
{
    let pt = director.try_get_geometry(&edge0.back())?;
    let curve1 = circle_arc(&pt, origin, axis, angle);
    let edge1 = Edge::new_unchecked(edge0.back(), edge2.back());
    director.attach(&edge1, curve1);
    let pt = director.try_get_geometry(&edge0.front())?;
    let curve3 = circle_arc(&pt, origin, axis, angle);
    let edge3 = Edge::new_unchecked(edge0.front(), edge2.front());
    director.attach(&edge3, curve3);
    let wire0 = Wire::from_iter(&[*edge0, edge1, edge2.inverse(), edge3.inverse()]);
    let curve0 = director.try_get_geometry(edge0)?;
    let surface0 = rsweep_surface(&curve0, origin, axis, angle);
    Ok((wire0, surface0))
}

impl RSweep for Wire {
    type Output = Shell;
    fn partial_rsweep(
        self,
        origin: &Vector3,
        axis: &Vector3,
        angle: f64,
        director: &mut Director,
    ) -> Result<Shell>
    {
        let wire = self.rotated(origin, axis, angle, director)?;
        connect_by_circle_arc(&self, &wire, origin, axis, angle, director)
    }
    fn full_rsweep(
        self,
        origin: &Vector3,
        axis: &Vector3,
        orientation: bool,
        director: &mut Director,
    ) -> Result<Shell>
    {
        let wire = self.rotated(origin, axis, PI, director)?;
        let angle = if orientation { PI } else { -PI };
        let mut shell0 = connect_by_circle_arc(&self, &wire, origin, axis, angle, director)?;
        let mut shell1 = connect_by_circle_arc(&wire, &self, origin, axis, angle, director)?;
        shell0.append(&mut shell1);
        Ok(shell0)
    }
}

fn connect_by_circle_arc(
    wire0: &Wire,
    wire1: &Wire,
    origin: &Vector3,
    axis: &Vector3,
    angle: f64,
    director: &mut Director,
) -> Result<Shell>
{
    let mut vemap: HashMap<Vertex, Edge> = HashMap::new();
    for (v0, v1) in wire0.vertex_iter().zip(wire1.vertex_iter()) {
        if vemap.get(&v0).is_none() {
            let pt0 = director.try_get_geometry(&v0)?;
            let curve = circle_arc(&pt0, origin, axis, angle);
            let edge = Edge::new_unchecked(v0, v1);
            director.attach(&edge, curve);
            vemap.insert(v0, edge);
        }
    }
    let mut shell = Shell::new();
    for (edge0, edge2) in wire0.edge_iter().zip(wire1.edge_iter()) {
        let edge1 = vemap.get(&edge0.back()).unwrap();
        let edge3 = vemap.get(&edge0.front()).unwrap();
        let wire1 = Wire::from(vec![*edge0, *edge1, edge2.inverse(), edge3.inverse()]);
        let face = Face::new_unchecked(wire1);
        let curve = director.try_get_geometry(edge0)?;
        let surface = rsweep_surface(curve, origin, axis, angle);
        director.attach(&face, surface);
        shell.push(face);
    }
    Ok(shell)
}

impl RSweep for Face {
    type Output = Solid;
    fn partial_rsweep(
        mut self,
        origin: &Vector3,
        axis: &Vector3,
        angle: f64,
        director: &mut Director,
    ) -> Result<Solid>
    {
        let face = self.rotated(origin, axis, angle, director)?;
        let mut shell = connect_by_circle_arc(
            &self.boundary(),
            &face.boundary(),
            origin,
            axis,
            angle,
            director,
        )?;
        director.reverse_face(&mut self);
        shell.push(self);
        shell.push(face);
        Ok(Solid::new(vec![shell]))
    }
    fn full_rsweep(
        self,
        origin: &Vector3,
        axis: &Vector3,
        orientation: bool,
        builder: &mut Director,
    ) -> Result<Solid>
    {
        let wire = self.into_boundary();
        let shell = wire.full_rsweep(origin, axis, orientation, builder)?;
        Ok(Solid::new(vec![shell]))
    }
}

impl RSweep for Shell {
    type Output = Vec<Solid>;
    fn partial_rsweep(
        self,
        origin: &Vector3,
        axis: &Vector3,
        angle: f64,
        director: &mut Director,
    ) -> Result<Vec<Solid>>
    {
        let mut res = Vec::new();
        for mut shell0 in self.connected_components() {
            let mut shell1 = shell0.rotated(origin, axis, angle, director)?;
            let mut new_shell =
                connected_shell_sweep(&shell0, &shell1, origin, axis, angle, director)?;
            for face in shell0.face_iter_mut() {
                director.reverse_face(face);
            }
            new_shell.append(&mut shell0);
            new_shell.append(&mut shell1);
            res.push(Solid::new_unchecked(new_shell.connected_components()));
        }
        Ok(res)
    }
    fn full_rsweep(
        self,
        origin: &Vector3,
        axis: &Vector3,
        orientation: bool,
        director: &mut Director,
    ) -> Result<Vec<Solid>>
    {
        let mut res = Vec::new();
        for shell0 in self.connected_components() {
            let angle = if orientation { PI } else { -PI };
            let shell1 = shell0.rotated(origin, axis, angle, director)?;
            let mut new_shell =
                connected_shell_sweep(&shell0, &shell1, origin, axis, angle, director)?;
            new_shell.append(&mut connected_shell_sweep(
                &shell1, &shell0, origin, axis, angle, director,
            )?);
            res.push(Solid::new_unchecked(new_shell.connected_components()));
        }
        Ok(res)
    }
}

fn connected_shell_sweep(
    shell0: &Shell,
    shell1: &Shell,
    origin: &Vector3,
    axis: &Vector3,
    angle: f64,
    director: &mut Director,
) -> Result<Shell>
{
    let wires0 = shell0.extract_boundaries();
    let wires1 = shell1.extract_boundaries();
    let mut new_shell = Shell::new();
    for (wire0, wire1) in wires0.iter().zip(&wires1) {
        let mut shell = connect_by_circle_arc(wire0, wire1, origin, axis, angle, director)?;
        new_shell.append(&mut shell);
    }
    Ok(new_shell)
}
