use crate::transformed::Transformed;
use crate::{Builder, Director, Result};
use geometry::*;
use topology::*;
use std::iter::FromIterator;

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
        let wire =  Wire::from_iter(&[self, edge1, edge2.inverse(), edge3]);
        director.get_builder().plane(wire)
    }
}

fn sub_wire_sweep(
    wire0: &Wire, wire1: &Wire, builder: &mut Builder) -> Result<Shell> {
    let mut columns = Vec::new();
    for (vertex0, vertex1) in wire0.vertex_iter().zip(wire1.vertex_iter()) {
        columns.push(builder.line(vertex0, vertex1)?);
    }
    let mut shell = Shell::new();
    for (i, (edge0, edge1)) in wire0.edge_iter().zip(wire1.edge_iter()).enumerate() {
        let wire = Wire::from_iter(&[
            *edge0,
            columns[(i + 1) % columns.len()],
            edge1.inverse(),
            columns[i].inverse(),
        ]);
        shell.push(builder.plane(wire)?);
    }
    Ok(shell)
}

impl TSweep for Wire {
    type Output = Shell;
    fn tsweep(self, vector: &Vector3, director: &mut Director) -> Result<Shell> {
        let wire = self.translated(vector, director)?;
        sub_wire_sweep(&self, &wire, &mut director.get_builder())
    }
}

impl TSweep for Face {
    type Output = Solid;
    fn tsweep(mut self, vector: &Vector3, director: &mut Director) -> Result<Solid> {
        let face = self.translated(vector, director)?;
        let mut shell = director
            .building(|builder| sub_wire_sweep(&self.boundary(), &face.boundary(), builder))?;
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
    mut shell0: Shell,
    vector: &Vector3,
    director: &mut Director,
) -> Result<Solid>
{
    let mut shell1 = shell0.translated(vector, director)?;
    let wires0 = shell0.extract_boundaries();
    let wires1 = shell1.extract_boundaries();
    let mut new_shell = Shell::new();
    for (wire0, wire1) in wires0.iter().zip(&wires1) {
        let mut shell = sub_wire_sweep(wire0, wire1, &mut director.get_builder())?;
        new_shell.append(&mut shell);
    }
    for face in shell0.face_iter_mut() {
        director.reverse_face(face);
    }
    new_shell.append(&mut shell0);
    new_shell.append(&mut shell1);
    Ok(Solid::new(vec![new_shell]))
}
