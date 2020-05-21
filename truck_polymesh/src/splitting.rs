use crate::errors::Error;
use crate::{MeshHandler, PolygonMesh};
use std::collections::HashMap;

impl MeshHandler {
    pub fn clustering_face(&self) -> Vec<Vec<usize>> {
        if self.mesh.normals.is_empty() {
            panic!("{}", Error::NoNormal);
        }
        let face_adjacency = self.face_adjacency();
        get_components(&face_adjacency)
    }

    pub fn create_mesh_by_face_indices(&self, indices: &Vec<usize>) -> PolygonMesh {
        let mut mesh = PolygonMesh::default();
        mesh.vertices = self.mesh.vertices.clone();
        mesh.uv_coords = self.mesh.uv_coords.clone();
        mesh.normals = self.mesh.normals.clone();
        for i in indices {
            if *i < self.mesh.tri_faces.len() {
                mesh.tri_faces.push(self.mesh.tri_faces[*i].clone());
            } else {
                let i = *i - self.mesh.tri_faces.len();
                if i < self.mesh.quad_faces.len() {
                    mesh.quad_faces.push(self.mesh.quad_faces[i].clone());
                } else {
                    let i = i - self.mesh.quad_faces.len();
                    mesh.other_faces.push(self.mesh.other_faces[i].clone());
                }
            }
        }

        let mut handler = MeshHandler::new(mesh);
        handler.remove_unused_attrs();
        handler.mesh
    }

    pub fn face_adjacency(&self) -> Vec<Vec<usize>> {
        let mesh = &self.mesh;
        let mut face_adjacency =
            vec![
                Vec::<usize>::new();
                mesh.tri_faces.len() + mesh.quad_faces.len() + mesh.other_faces.len()
            ];

        let mut edge_face_map: HashMap<[[usize; 2]; 2], usize> = HashMap::new();
        for (i, face) in mesh.tri_faces.iter().enumerate() {
            signup_adjacency(i, face[0], face[1], &mut face_adjacency, &mut edge_face_map);
            signup_adjacency(i, face[1], face[2], &mut face_adjacency, &mut edge_face_map);
            signup_adjacency(i, face[2], face[0], &mut face_adjacency, &mut edge_face_map);
        }
        for (i, face) in mesh.quad_faces.iter().enumerate() {
            let i = i + mesh.tri_faces.len();
            signup_adjacency(i, face[0], face[1], &mut face_adjacency, &mut edge_face_map);
            signup_adjacency(i, face[1], face[2], &mut face_adjacency, &mut edge_face_map);
            signup_adjacency(i, face[2], face[3], &mut face_adjacency, &mut edge_face_map);
            signup_adjacency(i, face[3], face[0], &mut face_adjacency, &mut edge_face_map);
        }
        for (i, face) in mesh.other_faces.iter().enumerate() {
            let i = i + mesh.tri_faces.len() + mesh.quad_faces.len();
            let n = face.len();
            for j in 1..n {
                signup_adjacency(
                    i,
                    face[j - 1],
                    face[j],
                    &mut face_adjacency,
                    &mut edge_face_map,
                );
            }
            signup_adjacency(
                i,
                face[n - 1],
                face[0],
                &mut face_adjacency,
                &mut edge_face_map,
            );
        }
        face_adjacency
    }
}

fn signup_adjacency(
    i: usize,
    face0: [usize; 3],
    face1: [usize; 3],
    face_adjacency: &mut Vec<Vec<usize>>,
    edge_face_map: &mut HashMap<[[usize; 2]; 2], usize>,
)
{
    let edge = if face0[0] < face1[0] {
        [[face0[0], face0[2]], [face1[0], face1[2]]]
    } else {
        [[face1[0], face1[2]], [face0[0], face0[2]]]
    };
    match edge_face_map.get(&edge) {
        Some(j) => {
            face_adjacency[i].push(*j);
            face_adjacency[*j].push(i);
        }
        None => {
            edge_face_map.insert(edge, i);
        }
    }
}

fn get_components(adjacency: &Vec<Vec<usize>>) -> Vec<Vec<usize>> {
    let mut unchecked = vec![true; adjacency.len()];
    let mut components = Vec::new();
    loop {
        let first = match unchecked.iter().position(|x| *x) {
            Some(idx) => idx,
            None => return components,
        };
        let mut stack = vec![first];
        let mut component = vec![first];
        unchecked[first] = false;
        while !stack.is_empty() {
            let cursor = stack.pop().unwrap();
            for i in &adjacency[cursor] {
                if unchecked[*i] {
                    unchecked[*i] = false;
                    component.push(*i);
                    stack.push(*i);
                }
            }
        }
        components.push(component);
    }
}
