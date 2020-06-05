use crate::Result;
use crate::*;
use geometry::*;
use std::collections::HashMap;
use topology::*;

pub trait TopologicalElement: Sized {
    type Sweeped: Sized;
    fn copy(&self, director: &mut Director) -> Result<Self>;
    fn transformed(&self, director: &mut Director, trsf: &Transform) -> Result<Self>;
    fn translated(&self, director: &mut Director, vector: &Vector3) -> Result<Self> {
        self.transformed(director, &Transform::translate(vector))
    }
    fn rotated(
        &self,
        director: &mut Director,
        origin: &Vector3,
        axis: &Vector3,
        angle: f64,
    ) -> Result<Self>
    {
        let trsf0 = Transform::translate(&-origin);
        let trsf1 = Transform::rotate(axis, angle);
        let trsf2 = Transform::translate(origin);
        self.transformed(director, &(trsf0 * trsf1 * trsf2))
    }
    fn scaled(&self, director: &mut Director, origin: &Vector3, scalars: &Vector3) -> Result<Self> {
        let trsf0 = Transform::translate(&-origin);
        let trsf1 = Transform::scale(scalars);
        let trsf2 = Transform::translate(origin);
        self.transformed(director, &(trsf0 * trsf1 * trsf2))
    }
    fn tsweep(&self, director: &mut Director, vector: &Vector3) -> Result<Self::Sweeped>;
}

impl Director {
    pub fn create_copy<T: TopologicalElement>(&mut self, elem: &T) -> Result<T> { elem.copy(self) }

    pub fn create_transformed<T: TopologicalElement>(
        &mut self,
        elem: &T,
        trsf: &Transform,
    ) -> Result<T>
    {
        elem.transformed(self, trsf)
    }

    pub fn create_translated<T: TopologicalElement>(
        &mut self,
        elem: &T,
        vector: &Vector3,
    ) -> Result<T>
    {
        elem.translated(self, vector)
    }

    pub fn create_rotated<T: TopologicalElement>(
        &mut self,
        elem: &T,
        origin: &Vector3,
        axis: &Vector3,
        angle: f64,
    ) -> Result<T>
    {
        elem.rotated(self, origin, axis, angle)
    }

    pub fn create_scaled<T: TopologicalElement>(
        &mut self,
        elem: &T,
        origin: &Vector3,
        scalar: &Vector3,
    ) -> Result<T>
    {
        elem.scaled(self, origin, scalar)
    }

    pub fn tsweep<T: TopologicalElement>(
        &mut self,
        elem: &T,
        vector: &Vector3,
    ) -> Result<T::Sweeped>
    {
        elem.tsweep(self, vector)
    }
}

impl TopologicalElement for Vertex {
    type Sweeped = Edge;
    fn copy(&self, director: &mut Director) -> Result<Vertex> {
        Ok(director.create_vertex(director.get_point(self)?.clone()))
    }

    fn transformed(&self, director: &mut Director, trsf: &Transform) -> Result<Vertex> {
        let pt = director.get_point(self)? * &trsf.0;
        Ok(director.create_vertex(pt))
    }

    fn tsweep(&self, director: &mut Director, vector: &Vector3) -> Result<Edge> {
        let vertex = director.create_translated(self, vector)?;
        director.line(*self, vertex)
    }
}

impl TopologicalElement for Edge {
    type Sweeped = Face;
    fn copy(&self, director: &mut Director) -> Result<Edge> {
        let v0 = director.create_copy(&self.absolute_front())?;
        let v1 = director.create_copy(&self.absolute_back())?;
        let curve = director.get_curve(self)?.clone();
        let new_edge = director.create_edge(v0, v1, curve);
        if self.absolute_front() == self.front() {
            new_edge
        } else {
            new_edge.map(|x| x.inverse())
        }
    }

    fn transformed(&self, director: &mut Director, trsf: &Transform) -> Result<Edge> {
        let v0 = director.create_transformed(&self.absolute_front(), trsf)?;
        let v1 = director.create_transformed(&self.absolute_back(), trsf)?;
        let mut curve = director.get_curve(self)?.clone();
        curve *= &trsf.0;
        let new_edge = director.create_edge(v0, v1, curve);
        if self.absolute_front() == self.front() {
            new_edge
        } else {
            new_edge.map(|x| x.inverse())
        }
    }

    fn tsweep(&self, director: &mut Director, vector: &Vector3) -> Result<Face> {
        let edge2 = self.translated(director, vector)?;
        let edge1 = director.line(self.back(), edge2.back())?;
        let edge3 = director.line(edge2.front(), self.front())?;
        let wire = Wire::by_slice(&[*self, edge1, edge2.inverse(), edge3]);
        director.plane(wire)
    }
}

impl TopologicalElement for Wire {
    type Sweeped = Shell;
    fn copy(&self, director: &mut Director) -> Result<Wire> {
        let mut vertex_map: HashMap<Vertex, Vertex> = HashMap::new();
        for v in self.vertex_iter() {
            if vertex_map.get(&v).is_none() {
                let vert = director.create_copy(&v)?;
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
                let new_edge = if wire.is_empty() {
                    director.create_copy(edge)?
                } else {
                    let vertex0 = wire.back_vertex().unwrap();
                    let vertex1 = *vertex_map.get(&edge.back()).unwrap();
                    let curve = director.get_curve(&edge)?.clone();
                    if edge.absolute_front() == edge.front() {
                        director.create_edge(vertex0, vertex1, curve)?
                    } else {
                        director.create_edge(vertex1, vertex0, curve)?.inverse()
                    }
                };
                wire.push_back(new_edge);
                edge_map.insert(edge.id(), new_edge);
            }
        }
        Ok(wire)
    }
    fn transformed(&self, director: &mut Director, trsf: &Transform) -> Result<Wire> {
        let mut vertex_map: HashMap<Vertex, Vertex> = HashMap::new();
        for v in self.vertex_iter() {
            if vertex_map.get(&v).is_none() {
                let vert = director.create_transformed(&v, trsf)?;
                vertex_map.insert(v, vert);
            }
        }
        let mut wire = Wire::new();
        for edge in self.edge_iter() {
            let vertex0 = *vertex_map.get(&edge.front()).unwrap();
            let vertex1 = *vertex_map.get(&edge.back()).unwrap();
            let mut curve = director.get_curve(&edge)?.clone();
            curve *= &trsf.0;
            let new_edge = if edge.absolute_front() == edge.front() {
                director.create_edge(vertex0, vertex1, curve)?
            } else {
                director.create_edge(vertex1, vertex0, curve)?.inverse()
            };
            wire.push_back(new_edge);
        }
        Ok(wire)
    }

    fn tsweep(&self, director: &mut Director, vector: &Vector3) -> Result<Shell> {
        let wire = self.translated(director, vector)?;
        let mut columns = Vec::new();
        for (edge0, edge1) in self.edge_iter().zip(wire.edge_iter()) {
            columns.push(director.line(edge0.front(), edge1.front())?);
        }
        if !self.is_closed() {
            if let (Some(vertex0), Some(vertex1)) = (self.back_vertex(), wire.back_vertex()) {
                columns.push(director.line(vertex0, vertex1)?);
            }
        }
        let mut shell = Shell::new();
        for i in 0..wire.len() {
            let edge0 = self[i];
            let edge1 = columns[(i + 1) % wire.len()];
            let edge2 = wire[i].inverse();
            let edge3 = columns[i].inverse();
            let wire = Wire::by_slice(&[edge0, edge1, edge2, edge3]);
            shell.push(director.plane(wire)?);
        }
        Ok(shell)
    }
}

impl TopologicalElement for Face {
    type Sweeped = Solid;
    fn copy(&self, director: &mut Director) -> Result<Face> {
        let boundary = director.create_copy(self.boundary())?;
        let surface = director.get_surface(self)?.clone();
        director.create_face(boundary, surface)
    }

    fn transformed(&self, director: &mut Director, trsf: &Transform) -> Result<Face> {
        let boundary = director.create_transformed(self.boundary(), trsf)?;
        let mut surface = director.get_surface(self)?.clone();
        surface *= &trsf.0;
        director.create_face(boundary, surface)
    }

    fn tsweep(&self, director: &mut Director, vector: &Vector3) -> Result<Solid> {
        let mut surface = director.get_surface(self)?.clone();
        let same_direction = compare_direction(&mut surface, vector);
        let face = self.translated(director, vector)?;
        let (face0, face1) = match same_direction {
            true => (self, &face),
            false => (&face, self),
        };
        let wire0 = face0.boundary();
        let wire1 = face1.boundary();
        let mut columns = Vec::new();
        for (edge0, edge1) in wire0.edge_iter().zip(wire1.edge_iter()) {
            columns.push(director.line(edge0.front(), edge1.front())?);
        }
        let mut shell = Shell::new();
        for i in 0..wire0.len() {
            let edge0 = wire0[i];
            let edge1 = columns[(i + 1) % wire0.len()];
            let edge2 = wire1[i].inverse();
            let edge3 = columns[i].inverse();
            let wire = Wire::by_slice(&[edge0, edge1, edge2, edge3]);
            shell.push(director.plane(wire)?);
        }
        shell.push(face);
        let mut wire = self.boundary().clone();
        wire.inverse();
        let mut surface = director.get_surface(self)?.clone();
        surface.swap_axes();
        shell.push(director.create_face(wire, surface)?);
        Ok(Solid::new(vec![shell]))
    }
}

fn compare_direction(surface: &mut BSplineSurface, vector: &Vector3) -> bool {
    let (knot_vec0, knot_vec1) = surface.knot_vecs();
    let u = knot_vec0[0] + knot_vec0.range_length() / 2.0;
    let v = knot_vec1[0] + knot_vec1.range_length() / 2.0;
    let normal = surface.normal_vector(u, v);
    let normal = Vector3::new(normal[0], normal[1], normal[2]);
    normal * vector > 0.0
}

