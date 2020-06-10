use crate::Result;
use crate::*;
use geometry::*;
use std::collections::HashMap;
use topology::*;

pub trait Transformed: Sized {
    fn mapped<F0: Fn(&mut Vector), F1: Fn(&mut BSplineCurve), F2: Fn(&mut BSplineSurface)>(
        &self,
        vector_closure: &F0,
        curve_closure: &F1,
        surface_closure: &F2,
        builder: &mut Builder,
    ) -> Result<Self>;
    fn copy(&self, builder: &mut Builder) -> Result<Self> {
        self.mapped(&|_| {}, &|_| {}, &|_| {}, builder)
    }
    fn transformed(&self, trsf: &Transform, builder: &mut Builder) -> Result<Self> {
        self.mapped(
            &trsf.mul_assign_closure(),
            &trsf.mul_assign_closure(),
            &trsf.mul_assign_closure(),
            builder,
        )
    }
    fn translated(&self, vector: &Vector3, builder: &mut Builder) -> Result<Self> {
        self.transformed(&Transform::translate(vector), builder)
    }
    fn rotated(
        &self,
        origin: &Vector3,
        axis: &Vector3,
        angle: f64,
        builder: &mut Builder,
    ) -> Result<Self>
    {
        let trsf0 = Transform::translate(&-origin);
        let trsf1 = Transform::rotate(axis, angle);
        let trsf2 = Transform::translate(origin);
        self.transformed(&(trsf0 * trsf1 * trsf2), builder)
    }

    fn scaled(&self, origin: &Vector3, scalars: &Vector3, builder: &mut Builder) -> Result<Self> {
        let trsf0 = Transform::translate(&-origin);
        let trsf1 = Transform::scale(scalars);
        let trsf2 = Transform::translate(origin);
        self.transformed(&(trsf0 * trsf1 * trsf2), builder)
    }
}

impl Transformed for Vertex {
    fn mapped<F0: Fn(&mut Vector), F1: Fn(&mut BSplineCurve), F2: Fn(&mut BSplineSurface)>(
        &self,
        vector_closure: &F0,
        _: &F1,
        _: &F2,
        builder: &mut Builder,
    ) -> Result<Self>
    {
        let mut pt = builder.director.try_get_geometry(self)?.clone();
        vector_closure(&mut pt);
        builder.create_topology(pt)
    }
}

impl Transformed for Edge {
    fn mapped<F0: Fn(&mut Vector), F1: Fn(&mut BSplineCurve), F2: Fn(&mut BSplineSurface)>(
        &self,
        vector_closure: &F0,
        curve_closure: &F1,
        surface_closure: &F2,
        builder: &mut Builder,
    ) -> Result<Self>
    {
        let v0 = self.absolute_front().mapped(
            vector_closure,
            curve_closure,
            surface_closure,
            builder,
        )?;
        let v1 =
            self.absolute_back()
                .mapped(vector_closure, curve_closure, surface_closure, builder)?;
        let mut curve = builder.director.try_get_geometry(self)?.clone();
        curve_closure(&mut curve);
        let new_edge = Edge::try_new(v0, v1)?;
        builder.director.attach(&new_edge, curve);
        if self.absolute_front() == self.front() {
            Ok(new_edge)
        } else {
            Ok(new_edge.inverse())
        }
    }
}

impl Transformed for Wire {
    fn mapped<F0: Fn(&mut Vector), F1: Fn(&mut BSplineCurve), F2: Fn(&mut BSplineSurface)>(
        &self,
        vector_closure: &F0,
        curve_closure: &F1,
        surface_closure: &F2,
        builder: &mut Builder,
    ) -> Result<Self>
    {
        let mut vertex_map: HashMap<Vertex, Vertex> = HashMap::new();
        for v in self.vertex_iter() {
            if vertex_map.get(&v).is_none() {
                let vert = v.mapped(vector_closure, curve_closure, surface_closure, builder)?;
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
                let mut curve = builder.director.try_get_geometry(edge)?.clone();
                curve_closure(&mut curve);
                let new_edge = Edge::new_unchecked(vertex0, vertex1);
                builder.director.attach(&new_edge, curve);
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
    fn mapped<F0: Fn(&mut Vector), F1: Fn(&mut BSplineCurve), F2: Fn(&mut BSplineSurface)>(
        &self,
        vector_closure: &F0,
        curve_closure: &F1,
        surface_closure: &F2,
        builder: &mut Builder,
    ) -> Result<Self>
    {
        let wire =
            self.boundary()
                .mapped(vector_closure, curve_closure, surface_closure, builder)?;
        let face = Face::new_unchecked(wire);
        let mut surface = builder.director.try_get_geometry(self)?.clone();
        surface_closure(&mut surface);
        builder.director.attach(&face, surface);
        Ok(face)
    }
}

impl Transformed for Shell {
    fn mapped<F0: Fn(&mut Vector), F1: Fn(&mut BSplineCurve), F2: Fn(&mut BSplineSurface)>(
        &self,
        vector_closure: &F0,
        curve_closure: &F1,
        surface_closure: &F2,
        builder: &mut Builder,
    ) -> Result<Self>
    {
        let mut shell = Shell::new();
        let mut vmap: HashMap<Vertex, Vertex> = HashMap::new();
        let vertex_iter = self
            .face_iter()
            .flat_map(|face| face.boundary().edge_iter().map(|edge| edge.front()));
        for vertex in vertex_iter {
            if vmap.get(&vertex).is_none() {
                let new_vertex =
                    vertex.mapped(vector_closure, curve_closure, surface_closure, builder)?;
                vmap.insert(vertex, new_vertex);
            }
        }
        let director = &mut builder.director;
        let mut edge_map: HashMap<usize, Edge> = HashMap::new();
        for face in self.face_iter() {
            let mut wire = Wire::new();
            for edge in face.boundary().edge_iter() {
                if let Some(new_edge) = edge_map.get(&edge.id()) {
                    if edge.absolute_front() == edge.front() {
                        wire.push_back(*new_edge);
                    } else {
                        wire.push_back(new_edge.inverse());
                    }
                } else {
                    let v0 = vmap.get(&edge.absolute_front()).unwrap();
                    let v1 = vmap.get(&edge.absolute_back()).unwrap();
                    let mut curve = director.try_get_geometry(edge)?.clone();
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
            let mut surface = director.try_get_geometry(face)?.clone();
            surface_closure(&mut surface);
            director.attach(&new_face, surface);
            shell.push(new_face);
        }
        Ok(shell)
    }
}

impl Transformed for Solid {
    fn mapped<F0: Fn(&mut Vector), F1: Fn(&mut BSplineCurve), F2: Fn(&mut BSplineSurface)>(
        &self,
        vector_closure: &F0,
        curve_closure: &F1,
        surface_closure: &F2,
        builder: &mut Builder,
    ) -> Result<Self>
    {
        let mut vec = Vec::new();
        for shell in self.boundaries() {
            vec.push(shell.mapped(vector_closure, curve_closure, surface_closure, builder)?);
        }
        Ok(Solid::new_unchecked(vec))
    }
}
