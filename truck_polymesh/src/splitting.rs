use crate::errors::Error;
use crate::{MeshHandler, PolygonMesh};
use geometry::Vector3;
use std::collections::HashMap;
use std::f64::consts::PI;

/// splitting the global faces
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
        'quad: for (mut i, quad) in mesh.quad_faces.iter().enumerate() {
            i += mesh.tri_faces.len();
            let vec0 = &mesh.positions[quad[1][0]] - &mesh.positions[quad[0][0]];
            let vec1 = &mesh.positions[quad[2][0]] - &mesh.positions[quad[0][0]];
            let mut n = vec0 ^ vec1;
            n /= n.norm();
            for [_, _, idx] in quad {
                if (&n - &mesh.normals[*idx]).norm2() < tol2 {
                    planes.push(i);
                    continue 'quad;
                }
            }
            others.push(i);
        }
        'poly: for (mut i, poly) in mesh.other_faces.iter().enumerate() {
            i += mesh.tri_faces.len() + mesh.quad_faces.len();
            let vec0 = &mesh.positions[poly[1][0]] - &mesh.positions[poly[0][0]];
            let vec1 = &mesh.positions[poly[2][0]] - &mesh.positions[poly[0][0]];
            let mut n = vec0 ^ vec1;
            n /= n.norm();
            for [_, _, idx] in poly {
                if (&n - &mesh.normals[*idx]).norm2() < tol2 {
                    planes.push(i);
                    continue 'poly;
                }
            }
            others.push(i);
        }
        (planes, others)
    }

    pub fn clustering_faces_by_gcurvature(
        &self,
        threshold: f64,
        preferred_upper: bool,
    ) -> (Vec<usize>, Vec<usize>)
    {
        let gcurve = self.get_gcurve();
        let mesh = &self.mesh;

        let mut lower = Vec::new();
        let mut upper = Vec::new();
        for (i, face) in mesh.tri_faces.iter().enumerate() {
            if preferred_upper {
                match face.iter().find(|v| gcurve[v[0]] > threshold) {
                    Some(_) => upper.push(i),
                    None => lower.push(i),
                }
            } else {
                match face.iter().find(|v| gcurve[v[0]] < threshold) {
                    Some(_) => lower.push(i),
                    None => upper.push(i),
                }
            }
        }
        for (mut i, face) in mesh.quad_faces.iter().enumerate() {
            i += mesh.tri_faces.len();
            if preferred_upper {
                match face.iter().find(|v| gcurve[v[0]] > threshold) {
                    Some(_) => upper.push(i),
                    None => lower.push(i),
                }
            } else {
                match face.iter().find(|v| gcurve[v[0]] < threshold) {
                    Some(_) => lower.push(i),
                    None => upper.push(i),
                }
            }
        }
        for (mut i, face) in mesh.other_faces.iter().enumerate() {
            i += mesh.tri_faces.len() + mesh.quad_faces.len();
            if preferred_upper {
                match face.iter().find(|v| gcurve[v[0]] > threshold) {
                    Some(_) => upper.push(i),
                    None => lower.push(i),
                }
            } else {
                match face.iter().find(|v| gcurve[v[0]] < threshold) {
                    Some(_) => lower.push(i),
                    None => upper.push(i),
                }
            }
        }

        (lower, upper)
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

    pub fn get_gcurve(&self) -> Vec<f64> {
        let positions = &self.mesh.positions;
        let mut angles = vec![0.0; positions.len()];
        let mut weights = vec![0.0; positions.len()];
        for face in &self.mesh.tri_faces {
            angles[face[0][0]] += get_angle(positions, face, 0, 1, 2);
            angles[face[1][0]] += get_angle(positions, face, 1, 2, 0);
            angles[face[2][0]] += get_angle(positions, face, 2, 0, 1);
            add_weights(&mut weights, positions, face);
        }
        for face in &self.mesh.quad_faces {
            angles[face[0][0]] += get_angle(positions, face, 0, 1, 2);
            angles[face[1][0]] += get_angle(positions, face, 1, 2, 0);
            angles[face[2][0]] += get_angle(positions, face, 2, 3, 1);
            angles[face[3][0]] += get_angle(positions, face, 3, 0, 1);
            add_weights(&mut weights, positions, face);
        }
        for face in &self.mesh.other_faces {
            let n = face.len() - 1;
            angles[face[0][0]] += get_angle(positions, face, 0, 1, n);
            for i in 1..n {
                angles[face[i][0]] += get_angle(positions, face, i, i + 1, i - 1);
            }
            angles[face[n][0]] += get_angle(positions, face, n, 0, n - 1);
            add_weights(&mut weights, positions, face);
        }

        angles.into_iter().zip(weights).map(|(ang, weight)| (PI * 2.0 - ang) / weight).collect()
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

fn get_angle(
    positions: &Vec<Vector3>,
    face: &[[usize; 3]],
    idx0: usize,
    idx1: usize,
    idx2: usize,
) -> f64
{
    let vec0 = &positions[face[idx1][0]] - &positions[face[idx0][0]];
    let vec1 = &positions[face[idx2][0]] - &positions[face[idx0][0]];
    vec0.angle(&vec1)
}

fn add_weights(
    weights: &mut Vec<f64>,
    positions: &Vec<Vector3>,
    face: &[[usize; 3]],
) {
    let area = (2..face.len()).fold(0.0, |sum, i| {
        let vec0 = &positions[face[i - 1][0]] - &positions[face[0][0]];
        let vec1 = &positions[face[i][0]] - &positions[face[0][0]];
        sum + (vec0 ^ vec1).norm() / 2.0
    }) / (face.len() as f64);
    for v in face {
        weights[v[0]] += area;
    }
}
