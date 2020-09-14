use crate::*;
use std::collections::HashMap;

pub trait Transformed: Sized {
    #[doc(hidden)]
    fn mapped<F0: Fn(&mut Vector4), F1: Fn(&mut BSplineCurve), F2: Fn(&mut BSplineSurface)>(
        &self,
        vector_closure: &F0,
        curve_closure: &F1,
        surface_closure: &F2,
    ) -> Self;
    #[doc(hidden)]
    fn copy(&self) -> Self {
        self.mapped(&|_| {}, &|_| {}, &|_| {})
    }
    #[doc(hidden)]
    fn transformed(&self, trsf: Matrix4) -> Self {
        self.mapped(
            &|v| *v = trsf * *v,
            &|c| c.transform_control_points(|v| *v = trsf * *v),
            &|s| s.transform_control_points(|v| *v = trsf * *v),
        )
    }
    #[doc(hidden)]
    fn translated(&self, vector: Vector3) -> Self {
        self.transformed(Matrix4::from_translation(vector))
    }
    #[doc(hidden)]
    fn rotated(
        &self,
        origin: Point3,
        axis: Vector3,
        angle: f64,
    ) -> Self
    {
        let trsf0 = Matrix4::from_translation(-origin.to_vec());
        let trsf1 = Matrix4::from_axis_angle(axis, cgmath::Rad(angle));
        let trsf2 = Matrix4::from_translation(origin.to_vec());
        self.transformed(trsf2 * trsf1 * trsf0)
    }

    #[doc(hidden)]
    fn scaled(&self, origin: Point3, scalars: Vector3) -> Self {
        let trsf0 = Matrix4::from_translation(-origin.to_vec());
        let trsf1 = Matrix4::from_nonuniform_scale(scalars[0], scalars[1], scalars[2]);
        let trsf2 = Matrix4::from_translation(origin.to_vec());
        self.transformed(trsf2 * trsf1 * trsf0)
    }
}

impl Transformed for Vertex {
    #[doc(hidden)]
    fn mapped<F0: Fn(&mut Vector4), F1: Fn(&mut BSplineCurve), F2: Fn(&mut BSplineSurface)>(
        &self,
        vector_closure: &F0,
        _: &F1,
        _: &F2,
    ) -> Self
    {
        let mut pt = *self.lock_point().unwrap();
        vector_closure(&mut pt);
        Vertex::new(pt)
    }
}

impl Transformed for Edge {
    #[doc(hidden)]
    fn mapped<F0: Fn(&mut Vector4), F1: Fn(&mut BSplineCurve), F2: Fn(&mut BSplineSurface)>(
        &self,
        vector_closure: &F0,
        curve_closure: &F1,
        surface_closure: &F2,
    ) -> Self
    {
        let v0 = self.absolute_front().mapped(
            vector_closure,
            curve_closure,
            surface_closure,
        );
        let v1 = self.absolute_back().mapped(
            vector_closure,
            curve_closure,
            surface_closure,
        );
        let mut curve = self.lock_curve().unwrap().clone();
        curve_closure(&mut curve);
        let new_edge = Edge::new(&v0, &v1, curve);
        if self.orientation() {
            new_edge
        } else {
            new_edge.inverse()
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
    ) -> Self
    {
        let mut vertex_map: HashMap<VertexID, Vertex> = HashMap::new();
        for v in self.vertex_iter() {
            if vertex_map.get(&v.id()).is_none() {
                let vert = v.mapped(vector_closure, curve_closure, surface_closure);
                vertex_map.insert(v.id(), vert);
            }
        }
        let mut wire = Wire::new();
        let mut edge_map: HashMap<EdgeID, Edge> = HashMap::new();
        for edge in self.edge_iter() {
            if let Some(new_edge) = edge_map.get(&edge.id()) {
                if edge.absolute_front() == edge.front() {
                    wire.push_back(*new_edge);
                } else {
                    wire.push_back(new_edge.inverse());
                }
            } else {
                let vertex0 = vertex_map.get(&edge.absolute_front().id()).unwrap().clone();
                let vertex1 = vertex_map.get(&edge.absolute_back().id()).unwrap().clone();
                let mut curve = edge.lock_curve().unwrap().clone();
                curve_closure(&mut curve);
                let new_edge = Edge::new(&vertex0, &vertex1, curve);
                if edge.orientation() {
                    wire.push_back(new_edge);
                } else {
                    wire.push_back(new_edge.inverse());
                }
                edge_map.insert(edge.id(), new_edge);
            }
        }
        wire
    }
}

impl Transformed for Face {
    #[doc(hidden)]
    fn mapped<F0: Fn(&mut Vector4), F1: Fn(&mut BSplineCurve), F2: Fn(&mut BSplineSurface)>(
        &self,
        vector_closure: &F0,
        curve_closure: &F1,
        surface_closure: &F2,
    ) -> Self
    {
        let wires: Vec<_> =
            self.absolute_boundaries()
                .iter()
                .map(|wire| {
                    wire.mapped(vector_closure, curve_closure, surface_closure)
                }).collect();
        let mut surface = self.lock_surface().unwrap().clone();
        surface_closure(&mut surface);
        let face = Face::new(wires, surface);
        if self.orientation() {
            face
        } else {
            face.inverse()
        }
    }
}

impl Transformed for Shell {
    #[doc(hidden)]
    fn mapped<F0: Fn(&mut Vector4), F1: Fn(&mut BSplineCurve), F2: Fn(&mut BSplineSurface)>(
        &self,
        vector_closure: &F0,
        curve_closure: &F1,
        surface_closure: &F2,
    ) -> Self
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
