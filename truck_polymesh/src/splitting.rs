use crate::errors::Error;
use crate::{MeshHandler, PolygonMesh};
use std::collections::HashMap;

impl MeshHandler {
    pub fn face_adjacency(&self) -> Vec<Vec<usize>> {
        let mesh = &self.mesh;
        let len = mesh.tri_faces.len() + mesh.quad_faces.len() + mesh.other_faces.len();
        let mut face_adjacency = vec![Vec::<usize>::new(); len];

        let mut edge_face_map: HashMap<[[usize; 2]; 2], usize> = HashMap::new();
        for (i, face) in mesh.tri_faces.iter().enumerate() {
            signup_adjacency(i, face, 0, 1, &mut face_adjacency, &mut edge_face_map);
            signup_adjacency(i, face, 1, 2, &mut face_adjacency, &mut edge_face_map);
            signup_adjacency(i, face, 2, 0, &mut face_adjacency, &mut edge_face_map);
        }
        for (mut i, face) in mesh.quad_faces.iter().enumerate() {
            i += mesh.tri_faces.len();
            signup_adjacency(i, face, 0, 1, &mut face_adjacency, &mut edge_face_map);
            signup_adjacency(i, face, 1, 2, &mut face_adjacency, &mut edge_face_map);
            signup_adjacency(i, face, 2, 3, &mut face_adjacency, &mut edge_face_map);
            signup_adjacency(i, face, 3, 0, &mut face_adjacency, &mut edge_face_map);
        }
        for (mut i, face) in mesh.other_faces.iter().enumerate() {
            i += mesh.tri_faces.len() + mesh.quad_faces.len();
            let n = face.len();
            for j in 1..n {
                signup_adjacency(i, face, j - 1, j, &mut face_adjacency, &mut edge_face_map);
            }
            signup_adjacency(i, face, n - 1, 0, &mut face_adjacency, &mut edge_face_map);
        }
        face_adjacency
    }

    pub fn clustering_face(&self) -> Vec<Vec<usize>> {
        if self.mesh.normals.is_empty() {
            panic!("{}", Error::NoNormal);
        }
        let face_adjacency = self.face_adjacency();
        get_components(&face_adjacency)
    }

    pub fn extract_planes(&self, tol: f64) -> (Vec<usize>, Vec<usize>) {
        let mesh = &self.mesh;
        let tol2 = tol * tol;
        let mut planes = Vec::new();
        let mut others = Vec::new();
        'tri: for (i, tri) in mesh.tri_faces.iter().enumerate() {
            let vec0 = &mesh.positions[tri[1][0]] - &mesh.positions[tri[0][0]];
            let vec1 = &mesh.positions[tri[2][0]] - &mesh.positions[tri[0][0]];
            let mut n = vec0 ^ vec1;
            n /= n.norm();
            for [_, _, idx] in tri {
                if (&n - &mesh.normals[*idx]).norm2() < tol2 {
                    planes.push(i);
                    continue 'tri;
                }
            }
            others.push(i);
        }
        (planes, others)
    }

    pub fn create_mesh_by_face_indices(&self, indices: &Vec<usize>) -> PolygonMesh {
        let mut mesh = PolygonMesh::default();
        mesh.positions = self.mesh.positions.clone();
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
}

fn signup_adjacency(
    i: usize,
    face: &[[usize; 3]],
    vidx0: usize,
    vidx1: usize,
    face_adjacency: &mut Vec<Vec<usize>>,
    edge_face_map: &mut HashMap<[[usize; 2]; 2], usize>,
)
{
    let edge = if face[vidx0][0] < face[vidx1][0] {
        [
            [face[vidx0][0], face[vidx0][2]],
            [face[vidx1][0], face[vidx1][2]],
        ]
    } else {
        [
            [face[vidx1][0], face[vidx1][2]],
            [face[vidx0][0], face[vidx0][2]],
        ]
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

/// divide the graph to the connected components.
/// # Arguments
/// * adjacency - the adjacency matrix
/// # Return
/// * the list of the indices of faces contained in each components
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
