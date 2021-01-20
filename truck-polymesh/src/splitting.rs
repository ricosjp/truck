use crate::errors::Error;
use crate::*;
use crate::prelude::*;
use std::collections::HashMap;
use std::f64::consts::PI;

impl Faces {
    /// create the adjacency list of the faces  
    /// # Remarks
    /// the indices of faces are serial number of all kinds of faces, i.e.
    /// * the `i`th quad face is identified as the `i + tri_faces.len()`th face, and
    /// * the `i`th other face is identified as the `i + tri_faces.len() + quad_faces.len()`th face.
    pub(super) fn face_adjacency(&self) -> Vec<Vec<usize>> {
        let len = self.len();
        let mut face_adjacency = vec![Vec::<usize>::new(); len];
        let mut edge_face_map: HashMap<[(usize, Option<usize>); 2], usize> = HashMap::new();
        for (i, face) in self.face_iter().enumerate() {
            face.windows(2)
                .chain(std::iter::once([face[face.len() - 1], face[0]].as_ref()))
                .for_each(|v| {
                    signup_adjacency(i, v[0], v[1], &mut face_adjacency, &mut edge_face_map)
                })
        }
        face_adjacency
    }
}

/// splitting the global faces
impl PolygonMesh {
    /// extract the each components of faces.  
    /// Two vertices are reagarded as the same if and only if they have the same
    /// position and the normal vector respectively.
    /// # Remarks
    /// the indices of faces are serial number of all kinds of faces, i.e.
    /// * the `i`th quad face is identified as the `i + tri_faces.len()`th face, and
    /// * the `i`th other face is identified as the `i + tri_faces.len() + quad_faces.len()`th face.
    pub fn clustering_faces(&self) -> Vec<Vec<usize>> {
        if self.normals().is_empty() {
            panic!("{}", Error::NoNormal);
        }
        let face_adjacency = self.faces().face_adjacency();
        get_components(&face_adjacency)
    }

    /// separate all faces into two clusters, one with `func` returning true and the other with false.
    /// # Returns
    /// (the vector of all faces `f` which `func(f) == true`, the one of the other faces)
    /// # Remarks
    /// the indices of faces are serial number of all kinds of faces, i.e.
    /// * the `i`th quad face is identified as the `i + tri_faces.len()`th face, and
    /// * the `i`th other face is identified as the `i + tri_faces.len() + quad_faces.len()`th face.
    pub fn faces_into_two_clusters<F: Fn(&[Vertex]) -> bool>(
        &self,
        func: F,
    ) -> (Vec<usize>, Vec<usize>) {
        let mut true_faces = Vec::new();
        let mut false_faces = Vec::new();
        for (i, face) in self.face_iter().enumerate() {
            match func(face) {
                true => true_faces.push(i),
                false => false_faces.push(i),
            }
        }
        (true_faces, false_faces)
    }

    #[doc(hidden)]
    pub fn extract_planes(&self, tol: f64) -> (Vec<usize>, Vec<usize>) {
        self.faces_into_two_clusters(|face: &[Vertex]| {
            is_in_the_plane(self.positions(), self.normals(), face, tol * tol)
        })
    }

    #[doc(hidden)]
    pub fn clustering_faces_by_gcurvature(
        &self,
        threshold: f64,
        preferred_upper: bool,
    ) -> (Vec<usize>, Vec<usize>) {
        let gcurve = self.get_gcurve();
        self.faces_into_two_clusters(|face: &[Vertex]| {
            is_signed_up_upper(face, &gcurve, preferred_upper, threshold)
        })
    }

    #[doc(hidden)]
    pub fn create_mesh_by_face_indices(&self, indices: &[usize]) -> PolygonMesh {
        let positions = self.positions.clone();
        let uv_coords = self.uv_coords.clone();
        let normals = self.normals.clone();
        let faces = Faces::from_iter(indices.iter().map(|i| &self.faces()[*i]));
        let mut mesh = PolygonMesh::new(positions, uv_coords, normals, faces);
        mesh.remove_unused_attrs();
        mesh
    }

    #[doc(hidden)]
    pub fn get_gcurve(&self) -> Vec<f64> {
        let positions = self.positions();
        let mut angles = vec![0.0; positions.len()];
        let mut weights = vec![0.0; positions.len()];
        for face in self.tri_faces() {
            angles[face[0].pos] += get_angle(positions, face, 0, 1, 2);
            angles[face[1].pos] += get_angle(positions, face, 1, 2, 0);
            angles[face[2].pos] += get_angle(positions, face, 2, 0, 1);
            add_weights(&mut weights, positions, face);
        }
        for face in self.quad_faces() {
            angles[face[0].pos] += get_angle(positions, face, 0, 1, 3);
            angles[face[1].pos] += get_angle(positions, face, 1, 2, 0);
            angles[face[2].pos] += get_angle(positions, face, 2, 3, 1);
            angles[face[3].pos] += get_angle(positions, face, 3, 0, 1);
            add_weights(&mut weights, positions, face);
        }
        for face in self.other_faces() {
            let n = face.len() - 1;
            angles[face[0].pos] += get_angle(positions, face, 0, 1, n);
            for i in 1..n {
                angles[face[i].pos] += get_angle(positions, face, i, i + 1, i - 1);
            }
            angles[face[n].pos] += get_angle(positions, face, n, 0, n - 1);
            add_weights(&mut weights, positions, face);
        }

        angles
            .into_iter()
            .zip(weights)
            .map(|(ang, weight)| (PI * 2.0 - ang) / weight)
            .collect()
    }
}

fn signup_adjacency(
    i: usize,
    v0: Vertex,
    v1: Vertex,
    face_adjacency: &mut Vec<Vec<usize>>,
    edge_face_map: &mut HashMap<[(usize, Option<usize>); 2], usize>,
) {
    let edge = if v0.pos < v1.pos {
        [(v0.pos, v0.nor), (v1.pos, v1.nor)]
    } else {
        [(v1.pos, v1.nor), (v0.pos, v1.nor)]
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

fn is_in_the_plane(
    positions: &[Point3],
    normals: &[Vector3],
    face: &[Vertex],
    tol2: f64,
) -> bool {
    let vec0 = &positions[face[1].pos] - &positions[face[0].pos];
    let vec1 = &positions[face[2].pos] - &positions[face[0].pos];
    let mut n = vec0.cross(vec1);
    n /= n.magnitude();
    for v in face {
        if (&n - &normals[v.nor.unwrap()]).magnitude2() < tol2 {
            return true;
        }
    }
    false
}

fn is_signed_up_upper(
    face: &[Vertex],
    gcurve: &Vec<f64>,
    preferred_upper: bool,
    threshold: f64,
) -> bool {
    if preferred_upper {
        match face.as_ref().iter().find(|v| gcurve[v.pos] > threshold) {
            Some(_) => true,
            None => false,
        }
    } else {
        match face.as_ref().iter().find(|v| gcurve[v.pos] < threshold) {
            Some(_) => false,
            None => true,
        }
    }
}

fn get_angle(
    positions: &[Point3],
    face: &[Vertex],
    idx0: usize,
    idx1: usize,
    idx2: usize,
) -> f64 {
    let vec0 = &positions[face[idx1].pos] - &positions[face[idx0].pos];
    let vec1 = &positions[face[idx2].pos] - &positions[face[idx0].pos];
    vec0.angle(vec1).0
}

fn add_weights(weights: &mut Vec<f64>, positions: &[Point3], face: &[Vertex]) {
    let area = (2..face.len()).fold(0.0, |sum, i| {
        let vec0 = &positions[face[i - 1].pos] - &positions[face[0].pos];
        let vec1 = &positions[face[i].pos] - &positions[face[0].pos];
        sum + (vec0.cross(vec1)).magnitude() / 2.0
    }) / (face.len() as f64);
    for v in face {
        weights[v.pos] += area;
    }
}
