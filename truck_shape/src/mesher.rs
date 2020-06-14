use crate::{Builder, Director, Mesher};
use polymesh::{PolygonMesh, StructuredMesh};
use std::collections::HashMap;
use topology::*;

impl<'a> Mesher<'a> {
    pub fn meshing<T: Meshed>(&self, meshed: &T, tol: f64) -> T::MeshType {
        meshed.meshing(tol, &self.director)
    }
    pub fn mesh_to_shape(&mut self, mesh: &PolygonMesh) -> Shell {
        let mut builder = self.director.get_builder();
        let v: Vec<_> = mesh
            .positions
            .iter()
            .map(|v| builder.vertex(v.clone()).unwrap())
            .collect();

        let mut shell = Shell::new();
        let mut edges: HashMap<(Vertex, Vertex), Edge> = HashMap::new();
        for face in &mesh.tri_faces {
            let i = face[0][0];
            let j = face[1][0];
            let k = face[2][0];
            let mut wire = Wire::new();
            wire.push_back(create_edge(v[i], v[j], &mut edges, &mut builder));
            wire.push_back(create_edge(v[j], v[k], &mut edges, &mut builder));
            wire.push_back(create_edge(v[k], v[i], &mut edges, &mut builder));
            shell.push(builder.plane(wire).unwrap());
        }
        for face in &mesh.quad_faces {
            let i = face[0][0];
            let j = face[1][0];
            let k = face[2][0];
            let l = face[3][0];
            let mut wire = Wire::new();
            wire.push_back(create_edge(v[i], v[j], &mut edges, &mut builder));
            wire.push_back(create_edge(v[j], v[k], &mut edges, &mut builder));
            wire.push_back(create_edge(v[k], v[l], &mut edges, &mut builder));
            wire.push_back(create_edge(v[l], v[i], &mut edges, &mut builder));
            shell.push(builder.plane(wire).unwrap());
        }
        for face in &mesh.other_faces {
            let idx: Vec<_> = face.iter().map(|x| x[0]).collect();
            let mut wire = Wire::new();
            for i in 0..=idx.len() {
                let idx0 = idx[i];
                let idx1 = idx[(i + 1) % idx.len()];
                wire.push_back(create_edge(v[idx0], v[idx1], &mut edges, &mut builder));
            }
            shell.push(builder.plane(wire).unwrap());
        }

        shell
    }
}

fn create_edge(
    v0: Vertex,
    v1: Vertex,
    edges: &mut HashMap<(Vertex, Vertex), Edge>,
    builder: &mut Builder,
) -> Edge
{
    let min = if v0.id() < v1.id() { v0 } else { v1 };
    let max = if v0.id() > v1.id() { v0 } else { v1 };
    let edge = match edges.get(&(min, max)) {
        Some(edge) => edge.clone(),
        None => {
            let edge = builder.line(min, max).unwrap();
            edges.insert((min, max), edge);
            edge
        }
    };

    if v0 == min {
        edge.clone()
    } else {
        edge.inverse()
    }
}

pub trait Meshed {
    type MeshType;
    fn meshing(&self, tol: f64, director: &Director) -> Self::MeshType;
}

impl Meshed for Face {
    type MeshType = StructuredMesh;
    fn meshing(&self, tol: f64, director: &Director) -> StructuredMesh {
        StructuredMesh::from_surface(director.get_geometry(self).unwrap(), tol)
    }
}

impl Meshed for Shell {
    type MeshType = PolygonMesh;
    fn meshing(&self, tol: f64, director: &Director) -> PolygonMesh {
        let mut mesh = PolygonMesh::default();
        for face in self.iter() {
            let counter = mesh.positions.len();
            let mut tmp = face.meshing(tol, director).destruct();
            mesh.positions.append(&mut tmp.positions);
            mesh.uv_coords.append(&mut tmp.uv_coords);
            mesh.normals.append(&mut tmp.normals);
            for face in tmp.quad_faces.iter_mut() {
                for vert in face.iter_mut() {
                    vert[0] += counter;
                    vert[2] += counter;
                }
                mesh.quad_faces.push(face.clone());
            }
        }
        mesh
    }
}

impl Meshed for Solid {
    type MeshType = PolygonMesh;
    fn meshing(&self, tol: f64, director: &Director) -> PolygonMesh {
        let mut mesh = PolygonMesh::default();
        for shell in self.boundaries() {
            let counter = mesh.positions.len();
            let mut tmp = shell.meshing(tol, director);
            mesh.positions.append(&mut tmp.positions);
            mesh.uv_coords.append(&mut tmp.uv_coords);
            mesh.normals.append(&mut tmp.normals);
            for face in tmp.quad_faces.iter_mut() {
                for vert in face.iter_mut() {
                    vert[0] += counter;
                    vert[2] += counter;
                }
                mesh.quad_faces.push(face.clone());
            }
        }
        mesh
    }
}
