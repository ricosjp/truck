use crate::{Builder, Result};
use geometry::*;
use std::collections::HashMap;
use topology::*;

pub trait TSweep: Sized {
    type Output: Sized;
    fn tsweep(&self, vector: &Vector3, builder: &mut Builder) -> Result<Self::Output>;
}

impl TSweep for Vertex {
    type Output = Edge;
    fn tsweep(&self, vector: &Vector3, builder: &mut Builder) -> Result<Edge> {
        let vertex = builder.translated(self, vector)?;
        builder.line(*self, vertex)
    }
}

impl TSweep for Edge {
    type Output = Face;
    fn tsweep(&self, vector: &Vector3, builder: &mut Builder) -> Result<Face> {
        let edge2 = builder.translated(self, vector)?;
        let edge1 = builder.line(self.back(), edge2.back())?;
        let edge3 = builder.line(edge2.front(), self.front())?;
        let wire = Wire::by_slice(&[*self, edge1, edge2.inverse(), edge3]);
        builder.plane(wire)
    }
}

impl TSweep for Wire {
    type Output = Shell;
    fn tsweep(&self, vector: &Vector3, builder: &mut Builder) -> Result<Shell> {
        let wire = builder.translated(self, vector)?;
        let mut columns = Vec::new();
        for (edge0, edge1) in self.edge_iter().zip(wire.edge_iter()) {
            columns.push(builder.line(edge0.front(), edge1.front())?);
        }
        if !self.is_closed() {
            if let (Some(vertex0), Some(vertex1)) = (self.back_vertex(), wire.back_vertex()) {
                columns.push(builder.line(vertex0, vertex1)?);
            }
        }
        let mut shell = Shell::new();
        for i in 0..wire.len() {
            let edge0 = self[i];
            let edge1 = columns[(i + 1) % wire.len()];
            let edge2 = wire[i].inverse();
            let edge3 = columns[i].inverse();
            let wire = Wire::by_slice(&[edge0, edge1, edge2, edge3]);
            shell.push(builder.plane(wire)?);
        }
        Ok(shell)
    }
}

impl TSweep for Face {
    type Output = Solid;
    fn tsweep(&self, vector: &Vector3, builder: &mut Builder) -> Result<Solid> {
        let mut surface = builder.director.try_get_geometry(self)?.clone();
        let same_direction = compare_direction(&mut surface, vector);
        let face = builder.translated(self, vector)?;
        let (face0, face1) = match same_direction {
            true => (self, &face),
            false => (&face, self),
        };
        let wire0 = face0.boundary();
        let wire1 = face1.boundary();
        let mut columns = Vec::new();
        for (edge0, edge1) in wire0.edge_iter().zip(wire1.edge_iter()) {
            columns.push(builder.line(edge0.front(), edge1.front())?);
        }
        let mut shell = Shell::new();
        for i in 0..wire0.len() {
            let edge0 = wire0[i];
            let edge1 = columns[(i + 1) % wire0.len()];
            let edge2 = wire1[i].inverse();
            let edge3 = columns[i].inverse();
            let wire = Wire::by_slice(&[edge0, edge1, edge2, edge3]);
            shell.push(builder.plane(wire)?);
        }
        shell.push(face1.clone());
        let mut wire = face0.boundary().clone();
        wire.inverse();
        let mut surface = builder.director.try_get_geometry(face0)?.clone();
        surface.swap_axes();
        if !same_direction {
            builder.director.remove(&face);
        }
        let face = Face::new_unchecked(wire);
        builder.director.insert(&face, surface);
        shell.push(face);
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

impl TSweep for Shell {
    type Output = Vec<Solid>;
    fn tsweep(&self, vector: &Vector3, builder: &mut Builder) -> Result<Vec<Solid>> {
        let mut res = Vec::new();
        for shell in self.connected_components() {
            res.push(connected_shell_sweep(&shell, vector, builder)?)
        }
        Ok(res)
    }
}

fn connected_shell_sweep(shell: &Shell, vector: &Vector3, builder: &mut Builder) -> Result<Solid> {
    let mut surface = builder.director.try_get_geometry(&shell[0])?.clone();
    let same_direction = compare_direction(&mut surface, vector);
    let shell0 = builder.translated(shell, vector)?;
    let (shell0, shell1) = match same_direction {
        true => (shell, &shell0),
        false => (&shell0, shell),
    };
    let mut new_shell = shell1.clone();
    for face in shell0.face_iter() {
        let mut wire = face.boundary().clone();
        wire.inverse();
        let mut surface = builder.director.try_get_geometry(face)?.clone();
        surface.swap_axes();
        let new_face = Face::new_unchecked(wire);
        builder.director.insert(&new_face, surface);
        new_shell.push(new_face);
    }
    let mut edges_counters: HashMap<usize, usize> = HashMap::new();
    let edge_iter = shell0
        .face_iter()
        .flat_map(|face| face.boundary().edge_iter());
    for edge in edge_iter {
        match edges_counters.get_mut(&edge.id()) {
            Some(counter) => *counter += 1,
            None => {
                edges_counters.insert(edge.id(), 1);
            }
        }
    }
    let edge_iter0 = shell0
        .face_iter()
        .flat_map(|face| face.boundary().edge_iter());
    let edge_iter1 = shell1
        .face_iter()
        .flat_map(|face| face.boundary().edge_iter());
    let mut columns: HashMap<usize, Edge> = HashMap::new();
    for (edge0, edge2) in edge_iter0.zip(edge_iter1) {
        if edges_counters.get(&edge0.id()).unwrap() == &1 {
            let mut wire = Wire::new();
            wire.push_back(*edge0);
            match columns.get(&edge0.back().id()) {
                Some(got) => wire.push_back(*got),
                None => {
                    let new_edge = builder.line(edge0.back(), edge2.back())?;
                    columns.insert(edge0.back().id(), new_edge);
                    wire.push_back(new_edge);
                }
            };
            wire.push_back(edge2.inverse());
            match columns.get(&edge0.front().id()) {
                Some(got) => wire.push_back(got.inverse()),
                None => {
                    let new_edge = builder.line(edge0.front(), edge2.front())?;
                    columns.insert(edge0.front().id(), new_edge);
                    wire.push_back(new_edge.inverse());
                }
            };
            new_shell.push(builder.plane(wire)?);
        }
    }
    Ok(Solid::new(vec![new_shell]))
}
