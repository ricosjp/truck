use crate::elements::{GeometricalElement, TopologicalElement};
use crate::Result;
use crate::*;
use std::collections::HashMap;
use topology::*;

pub trait Transformed: Sized {
    #[doc(hidden)]
    fn mapped<F0: Fn(&mut Vector4), F1: Fn(&mut BSplineCurve), F2: Fn(&mut BSplineSurface)>(
        &self,
        vector_closure: &F0,
        curve_closure: &F1,
        surface_closure: &F2,
        director: &mut Director,
    ) -> Result<Self>;
    #[doc(hidden)]
    fn copy(&self, director: &mut Director) -> Result<Self> {
        self.mapped(&|_| {}, &|_| {}, &|_| {}, director)
    }
    #[doc(hidden)]
    fn transformed(&self, trsf: Matrix4, director: &mut Director) -> Result<Self> {
        self.mapped(
            &|v| *v = trsf * *v,
            &|c| c.transform_control_points(|v| *v = trsf * *v),
            &|s| s.transform_control_points(|v| *v = trsf * *v),
            director,
        )
    }
    #[doc(hidden)]
    fn translated(&self, vector: Vector3, director: &mut Director) -> Result<Self> {
        self.transformed(Matrix4::from_translation(vector), director)
    }
    #[doc(hidden)]
    fn rotated(
        &self,
        origin: Point3,
        axis: Vector3,
        angle: Rad,
        director: &mut Director,
    ) -> Result<Self>
    {
        let trsf0 = Matrix4::from_translation(-origin.to_vec());
        let trsf1 = Matrix4::from_axis_angle(axis, angle);
        let trsf2 = Matrix4::from_translation(origin.to_vec());
        self.transformed(trsf2 * trsf1 * trsf0, director)
    }

    #[doc(hidden)]
    fn scaled(&self, origin: Point3, scalars: Vector3, director: &mut Director) -> Result<Self> {
        let trsf0 = Matrix4::from_translation(-origin.to_vec());
        let trsf1 = Matrix4::from_nonuniform_scale(scalars[0], scalars[1], scalars[2]);
        let trsf2 = Matrix4::from_translation(origin.to_vec());
        self.transformed(trsf2 * trsf1 * trsf0, director)
    }
}

impl Transformed for Vertex {
    #[doc(hidden)]
    fn mapped<F0: Fn(&mut Vector4), F1: Fn(&mut BSplineCurve), F2: Fn(&mut BSplineSurface)>(
        &self,
        vector_closure: &F0,
        _: &F1,
        _: &F2,
        director: &mut Director,
    ) -> Result<Self>
    {
        let mut pt = director
            .get_geometry(self)
            .ok_or(self.no_geometry())?
            .clone();
        vector_closure(&mut pt);
        Ok(pt.create_topology(director))
    }
}

impl Transformed for Edge {
    #[doc(hidden)]
    fn mapped<F0: Fn(&mut Vector4), F1: Fn(&mut BSplineCurve), F2: Fn(&mut BSplineSurface)>(
        &self,
        vector_closure: &F0,
        curve_closure: &F1,
        surface_closure: &F2,
        director: &mut Director,
    ) -> Result<Self>
    {
        let v0 = self.absolute_front().mapped(
            vector_closure,
            curve_closure,
            surface_closure,
            director,
        )?;
        let v1 = self.absolute_back().mapped(
            vector_closure,
            curve_closure,
            surface_closure,
            director,
        )?;
        let mut curve = director
            .get_geometry(self)
            .ok_or(self.no_geometry())?
            .clone();
        curve_closure(&mut curve);
        let new_edge = Edge::try_new(v0, v1)?;
        director.attach(&new_edge, curve);
        if self.absolute_front() == self.front() {
            Ok(new_edge)
        } else {
            Ok(new_edge.inverse())
        }
    }
}

impl Transformed for Wire {
    #[doc(hidden)]
    fn mapped<F0: Fn(&mut Vector4), F1: Fn(&mut BSplineCurve), F2: Fn(&mut BSplineSurface)>(
        &self,
        vector_closure: &F0,
        curve_closure: &F1,
        surface_closure: &F2,
        director: &mut Director,
    ) -> Result<Self>
    {
        let mut vertex_map: HashMap<Vertex, Vertex> = HashMap::new();
        for v in self.vertex_iter() {
            if vertex_map.get(&v).is_none() {
                let vert = v.mapped(vector_closure, curve_closure, surface_closure, director)?;
                vertex_map.insert(v, vert);
            }
        }
        let mut wire = Wire::new();
        let mut edge_map: HashMap<usize, Edge> = HashMap::new();
        for edge in self.edge_iter() {
            if let Some(new_edge) = edge_map.get(&edge.id()) {
                if edge.absolute_front() == edge.front() {
                    wire.push_back(*new_edge);
                } else {
                    wire.push_back(new_edge.inverse());
                }
            } else {
                let vertex0 = *vertex_map.get(&edge.absolute_front()).unwrap();
                let vertex1 = *vertex_map.get(&edge.absolute_back()).unwrap();
                let mut curve = director
                    .get_geometry(edge)
                    .ok_or(edge.no_geometry())?
                    .clone();
                curve_closure(&mut curve);
                let new_edge = Edge::new_unchecked(vertex0, vertex1);
                director.attach(&new_edge, curve);
                if edge.absolute_front() == edge.front() {
                    wire.push_back(new_edge);
                } else {
                    wire.push_back(new_edge.inverse());
                }
                edge_map.insert(edge.id(), new_edge);
            }
        }
        Ok(wire)
    }
}

impl Transformed for Face {
    #[doc(hidden)]
    fn mapped<F0: Fn(&mut Vector4), F1: Fn(&mut BSplineCurve), F2: Fn(&mut BSplineSurface)>(
        &self,
        vector_closure: &F0,
        curve_closure: &F1,
        surface_closure: &F2,
        director: &mut Director,
    ) -> Result<Self>
    {
        let wire =
            self.boundary()
                .mapped(vector_closure, curve_closure, surface_closure, director)?;
        let face = Face::new_unchecked(wire);
        let mut surface = director
            .get_geometry(self)
            .ok_or(self.no_geometry())?
            .clone();
        surface_closure(&mut surface);
        director.attach(&face, surface);
        Ok(face)
    }
}

impl Transformed for Shell {
    #[doc(hidden)]
    fn mapped<F0: Fn(&mut Vector4), F1: Fn(&mut BSplineCurve), F2: Fn(&mut BSplineSurface)>(
        &self,
        vector_closure: &F0,
        curve_closure: &F1,
        surface_closure: &F2,
        director: &mut Director,
    ) -> Result<Self>
    {
        let mut shell = Shell::new();
        let mut vmap: HashMap<Vertex, Vertex> = HashMap::new();
        let vertex_iter = self
            .iter()
            .flat_map(|face| face.absolute_boundary().vertex_iter());
        for vertex in vertex_iter {
            if vmap.get(&vertex).is_none() {
                let new_vertex =
                    vertex.mapped(vector_closure, curve_closure, surface_closure, director)?;
                vmap.insert(vertex, new_vertex);
            }
        }
        let mut edge_map: HashMap<usize, Edge> = HashMap::new();
        for face in self.face_iter() {
            let mut wire = Wire::new();
            for edge in face.boundary_iter() {
                if let Some(new_edge) = edge_map.get(&edge.id()) {
                    if edge.absolute_front() == edge.front() {
                        wire.push_back(*new_edge);
                    } else {
                        wire.push_back(new_edge.inverse());
                    }
                } else {
                    let v0 = vmap.get(&edge.absolute_front()).unwrap();
                    let v1 = vmap.get(&edge.absolute_back()).unwrap();
                    let mut curve = director
                        .get_geometry(&edge)
                        .ok_or(edge.no_geometry())?
                        .clone();
                    curve_closure(&mut curve);
                    let new_edge = Edge::new_unchecked(*v0, *v1);
                    director.attach(&new_edge, curve);
                    if edge.absolute_front() == edge.front() {
                        wire.push_back(new_edge);
                    } else {
                        wire.push_back(new_edge.inverse());
                    }
                    edge_map.insert(edge.id(), new_edge);
                }
            }
            let new_face = Face::new_unchecked(wire);
            let mut surface = director
                .get_geometry(face)
                .ok_or(face.no_geometry())?
                .clone();
            surface_closure(&mut surface);
            director.attach(&new_face, surface);
            shell.push(new_face);
        }
        Ok(shell)
    }
}

impl Transformed for Solid {
    #[doc(hidden)]
    fn mapped<F0: Fn(&mut Vector4), F1: Fn(&mut BSplineCurve), F2: Fn(&mut BSplineSurface)>(
        &self,
        vector_closure: &F0,
        curve_closure: &F1,
        surface_closure: &F2,
        director: &mut Director,
    ) -> Result<Self>
    {
        let mut vec = Vec::new();
        for shell in self.boundaries() {
            vec.push(shell.mapped(vector_closure, curve_closure, surface_closure, director)?);
        }
        Ok(Solid::new_unchecked(vec))
    }
}
