use crate::errors::Error;
use crate::*;
use std::collections::HashMap;
use std::f64::consts::PI;

/// splitting the global faces
impl MeshHandler {
    /// create the adjacency list of the faces  
    /// # Remarks
    /// the indices of faces are serial number of all kinds of faces, i.e.
    /// * the `i`th quad face is identified as the `i + tri_faces.len()`th face, and
    /// * the `i`th other face is identified as the `i + tri_faces.len() + quad_faces.len()`th face.
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

    /// extract the each components of faces.  
    /// Two vertices are reagarded as the same if and only if they have the same
    /// position and the normal vector respectively.
    /// # Remarks
    /// the indices of faces are serial number of all kinds of faces, i.e.
    /// * the `i`th quad face is identified as the `i + tri_faces.len()`th face, and
    /// * the `i`th other face is identified as the `i + tri_faces.len() + quad_faces.len()`th face.
    pub fn clustering_face(&self) -> Vec<Vec<usize>> {
        if self.mesh.normals.is_empty() {
            panic!("{}", Error::NoNormal);
        }
        let face_adjacency = self.face_adjacency();
        get_components(&face_adjacency)
    }

    /// separate all faces into two clusters, one with `func` returning true and the other with false.
    /// # Returns
    /// (the vector of all faces `f` which `func(f) == true`, the one of the other faces)
    /// # Remarks
    /// the indices of faces are serial number of all kinds of faces, i.e.
    /// * the `i`th quad face is identified as the `i + tri_faces.len()`th face, and
    /// * the `i`th other face is identified as the `i + tri_faces.len() + quad_faces.len()`th face.
    pub fn faces_into_two_clusters<F: Fn(&[[usize; 3]]) -> bool>(
        &self,
        func: F,
    ) -> (Vec<usize>, Vec<usize>)
    {
        let mesh = &self.mesh;
        let mut true_faces = Vec::new();
        let mut false_faces = Vec::new();
        for (i, tri) in mesh.tri_faces.iter().enumerate() {
            match func(tri) {
                true => true_faces.push(i),
                false => false_faces.push(i),
            }
        }
        for (i, quad) in mesh.quad_faces.iter().enumerate() {
            match func(quad) {
                true => true_faces.push(i + mesh.tri_faces.len()),
                false => false_faces.push(i + mesh.tri_faces.len()),
            }
        }
        for (i, poly) in mesh.other_faces.iter().enumerate() {
            match func(poly) {
                true => true_faces.push(i + mesh.tri_faces.len() + mesh.quad_faces.len()),
                false => false_faces.push(i + mesh.tri_faces.len() + mesh.quad_faces.len()),
            }
        }
        (true_faces, false_faces)
    }

    pub fn extract_planes(&self, tol: f64) -> (Vec<usize>, Vec<usize>) {
        self.faces_into_two_clusters(|face: &[[usize; 3]]| {
            is_in_the_plane(&self.mesh.positions, &self.mesh.normals, face, tol * tol)
        })
    }

    pub fn clustering_faces_by_gcurvature(
        &self,
        threshold: f64,
        preferred_upper: bool,
    ) -> (Vec<usize>, Vec<usize>)
    {
        let gcurve = self.get_gcurve();
        self.faces_into_two_clusters(|face: &[[usize; 3]]| {
            is_signed_up_upper(face, &gcurve, preferred_upper, threshold)
        })
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
            angles[face[0][0]] += get_angle(positions, face, 0, 1, 3);
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

        angles
            .into_iter()
            .zip(weights)
            .map(|(ang, weight)| (PI * 2.0 - ang) / weight)
            .collect()
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

fn is_in_the_plane(
    positions: &Vec<Point3>,
    normals: &Vec<Vector3>,
    face: &[[usize; 3]],
    tol2: f64,
) -> bool
{
    let vec0 = &positions[face[1][0]] - &positions[face[0][0]];
    let vec1 = &positions[face[2][0]] - &positions[face[0][0]];
    let mut n = vec0.cross(vec1);
    n /= n.magnitude();
    for [_, _, idx] in face {
        if (&n - &normals[*idx]).magnitude2() < tol2 {
            return true;
        }
    }
    false
}

fn is_signed_up_upper(
    face: &[[usize; 3]],
    gcurve: &Vec<f64>,
    preferred_upper: bool,
    threshold: f64,
) -> bool
{
    if preferred_upper {
        match face.as_ref().iter().find(|v| gcurve[v[0]] > threshold) {
            Some(_) => true,
            None => false,
        }
    } else {
        match face.as_ref().iter().find(|v| gcurve[v[0]] < threshold) {
            Some(_) => false,
            None => true,
        }
    }
}

fn get_angle(
    positions: &Vec<Point3>,
    face: &[[usize; 3]],
    idx0: usize,
    idx1: usize,
    idx2: usize,
) -> f64
{
    let vec0 = &positions[face[idx1][0]] - &positions[face[idx0][0]];
    let vec1 = &positions[face[idx2][0]] - &positions[face[idx0][0]];
    vec0.angle(vec1).0
}

fn add_weights(weights: &mut Vec<f64>, positions: &Vec<Point3>, face: &[[usize; 3]]) {
    let area = (2..face.len()).fold(0.0, |sum, i| {
        let vec0 = &positions[face[i - 1][0]] - &positions[face[0][0]];
        let vec1 = &positions[face[i][0]] - &positions[face[0][0]];
        sum + (vec0.cross(vec1)).magnitude() / 2.0
    }) / (face.len() as f64);
    for v in face {
        weights[v[0]] += area;
    }
}
