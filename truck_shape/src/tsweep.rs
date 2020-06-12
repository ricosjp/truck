use crate::transformed::Transformed;
use crate::{Director, Result};
use geometry::*;
use std::collections::HashMap;
use topology::*;

pub trait TSweep: Sized {
    #[doc(hidden)]
    type Output: Sized;
    #[doc(hidden)]
    fn tsweep(self, vector: &Vector3, director: &mut Director) -> Result<Self::Output>;
}

impl TSweep for Vertex {
    #[doc(hidden)]
    type Output = Edge;
    #[doc(hidden)]
    fn tsweep(self, vector: &Vector3, director: &mut Director) -> Result<Edge> {
        let vertex = self.translated(vector, director)?;
        director.get_builder().line(self, vertex)
    }
}

impl TSweep for Edge {
    #[doc(hidden)]
    type Output = Face;
    #[doc(hidden)]
    fn tsweep(self, vector: &Vector3, director: &mut Director) -> Result<Face> {
        let edge2 = self.translated(vector, director)?;
        let edge1 = director.get_builder().line(self.back(), edge2.back())?;
        let edge3 = director.get_builder().line(edge2.front(), self.front())?;
        let wire = Wire::by_slice(&[self, edge1, edge2.inverse(), edge3]);
        director.get_builder().plane(wire)
    }
}

impl TSweep for Wire {
    type Output = Shell;
    fn tsweep(self, vector: &Vector3, director: &mut Director) -> Result<Shell> {
        let wire = self.translated(vector, director)?;
        let mut builder = director.get_builder();
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
    fn tsweep(mut self, vector: &Vector3, director: &mut Director) -> Result<Solid> {
        let face = self.translated(vector, director)?;
        let mut builder = director.get_builder();
        let wire0 = self.boundary();
        let wire1 = face.boundary();
        let mut columns = Vec::new();
        for (v0, v1) in wire0.vertex_iter().zip(wire1.vertex_iter()) {
            columns.push(builder.line(v0, v1)?);
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
        director.reverse_face(&mut self);
        shell.push(self);
        shell.push(face);
        Ok(Solid::new(vec![shell]))
    }
}

impl TSweep for Shell {
    type Output = Vec<Solid>;
    fn tsweep(self, vector: &Vector3, director: &mut Director) -> Result<Vec<Solid>> {
        let mut res = Vec::new();
        for shell in self.connected_components() {
            res.push(connected_shell_sweep(shell, vector, director)?)
        }
        Ok(res)
    }
}

fn connected_shell_sweep(
    shell: Shell,
    vector: &Vector3,
    director: &mut Director,
) -> Result<Solid>
{
    let mut shell0 = shell.translated(vector, director)?;
    let mut edges_counters: HashMap<usize, usize> = HashMap::new();
    let edge_iter = shell
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
    let edge_iter0 = shell
        .face_iter()
        .flat_map(|face| face.boundary().edge_iter());
    let edge_iter1 = shell0
        .face_iter()
        .flat_map(|face| face.boundary().edge_iter());
    let mut builder = director.get_builder();
    let mut new_shell = Shell::new();
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
    new_shell.append(&mut shell0);
    for mut face in shell.face_into_iter() {
        director.reverse_face(&mut face);
        new_shell.push(face);
    }
    Ok(Solid::new(vec![new_shell]))
}
