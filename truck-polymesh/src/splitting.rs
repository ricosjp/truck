use crate::*;
use normal_filters::FaceNormal;
use std::collections::HashMap;
use std::f64::consts::PI;

/// Splitting the faces into several clusters.
pub trait Splitting {
    /// Creates a sub mesh by the face indices.
    /// # Examples
    /// ```
    /// use truck_polymesh::prelude::*;
    /// 
    /// // cube
    /// let positions = vec![
    ///     Point3::new(0.0, 0.0, 0.0),
    ///     Point3::new(1.0, 0.0, 0.0),
    ///     Point3::new(1.0, 1.0, 0.0),
    ///     Point3::new(0.0, 1.0, 0.0),
    ///     Point3::new(0.0, 0.0, 1.0),
    ///     Point3::new(1.0, 0.0, 1.0),
    ///     Point3::new(1.0, 1.0, 1.0),
    ///     Point3::new(0.0, 1.0, 1.0),
    /// ];
    /// let faces = Faces::from_iter(&[
    ///     &[3, 2, 1, 0], &[0, 1, 5, 4], &[1, 2, 6, 5],
    ///     &[2, 3, 7, 6], &[3, 0, 4, 7], &[4, 5, 6, 7],
    /// ]);
    /// let mesh = PolygonMesh::new(positions, Vec::new(), Vec::new(), faces);
    ///
    /// let submesh = mesh.create_mesh_by_face_indices(&[0, 3, 5]);
    /// 
    /// // the same attributes vector
    /// assert_eq!(mesh.positions(), submesh.positions());
    ///
    /// let faces0 = Faces::from_iter(&[
    ///     &[3, 2, 1, 0], &[2, 3, 7, 6], &[4, 5, 6, 7],
    /// ]);
    /// assert_eq!(submesh.faces().len(), faces0.len());
    /// assert_eq!(submesh.faces()[0], faces0[0]);
    /// assert_eq!(submesh.faces()[1], faces0[1]);
    /// assert_eq!(submesh.faces()[2], faces0[2]);
    /// ```
    fn create_mesh_by_face_indices(&self, indices: &[usize]) -> PolygonMesh;
    /// Extracts polygons such that there exists its normal is the same as its face normal.
    /// # Arguments
    /// - `tol`: tolerance to be regarded as the same normal as the face normal
    /// # Returns
    /// - The first polygon consists the faces included in planes.
    /// - The second polygon is the extracted remainder.
    /// # Examples
    /// ```
    /// use truck_polymesh::prelude::*;
    /// let positions = vec![
    ///     Point3::new(0.0, 0.5, 0.0),
    ///     Point3::new(0.0, 0.5, 1.0),
    ///     Point3::new(1.0, 0.5, 1.0),
    ///     Point3::new(1.0, 0.5, 0.0),
    ///     Point3::new(0.0, 0.0, 2.0),
    ///     Point3::new(1.0, 0.0, 2.0),
    /// ];
    /// let normals = vec![
    ///     Vector3::new(0.0, 1.0, 0.0),
    ///     // displaced normals for smooth rendering
    ///     Vector3::new(0.0, 1.0, 1.0).normalize(),
    /// ];
    /// let faces = Faces::from_iter(&[
    ///     &[(0, None, Some(0)), (1, None, Some(0)), (2, None, Some(0)), (3, None, Some(0))],
    ///     &[(2, None, Some(0)), (1, None, Some(0)), (4, None, Some(1)), (5, None, Some(1))],
    /// ]);
    /// let mesh = PolygonMesh::new(positions, Vec::new(), normals, faces);
    /// let (plane, remained) = mesh.extract_planes(TOLERANCE); // TOLERANCE == 1.0e-7
    /// assert_eq!(plane.len(), 1); assert_eq!(plane[0], 0);
    /// assert_eq!(remained.len(), 1); assert_eq!(remained[0], 1);
    /// ```
    fn extract_planes(&self, tol: f64) -> (Vec<usize>, Vec<usize>);
    /// Splits into the components.
    /// # Details
    /// Two polygons are considered to be in the same component if they share an edge
    /// whose vertices has the same positions and normals.
    /// # Examples
    /// ```
    /// use truck_polymesh::prelude::*;
    /// 
    /// // cube consisting tri_faces
    /// let positions = vec![
    ///     Point3::new(0.0, 0.0, 0.0),
    ///     Point3::new(1.0, 0.0, 0.0),
    ///     Point3::new(1.0, 1.0, 0.0),
    ///     Point3::new(0.0, 1.0, 0.0),
    ///     Point3::new(0.0, 0.0, 1.0),
    ///     Point3::new(1.0, 0.0, 1.0),
    ///     Point3::new(1.0, 1.0, 1.0),
    ///     Point3::new(0.0, 1.0, 1.0),
    /// ];
    /// let faces = Faces::from_iter(&[
    ///     &[3, 2, 0], &[1, 0, 2], &[0, 1, 4], &[5, 4, 1],
    ///     &[1, 2, 5], &[6, 5, 2], &[2, 3, 6], &[7, 6, 3],
    ///     &[3, 0, 7], &[4, 7, 0], &[4, 5, 7], &[6, 7, 5],
    /// ]);
    /// let mut mesh = PolygonMesh::new(positions, Vec::new(), Vec::new(), faces);
    /// 
    /// // sign up normals
    /// mesh.add_naive_normals(true).put_together_same_attrs();
    /// 
    /// let components = mesh.into_components();
    /// // The number of components is six because the mesh is a cube.
    /// assert_eq!(components.len(), 6);
    /// ```
    fn into_components(&self) -> Vec<Vec<usize>>;
}

impl Splitting for PolygonMesh {
    fn create_mesh_by_face_indices(&self, indices: &[usize]) -> PolygonMesh {
        let positions = self.positions.clone();
        let uv_coords = self.uv_coords.clone();
        let normals = self.normals.clone();
        let faces = Faces::from_iter(indices.iter().map(|i| &self.faces()[*i]));
        PolygonMesh::new(positions, uv_coords, normals, faces)
    }

    fn extract_planes(&self, tol: f64) -> (Vec<usize>, Vec<usize>) {
        self.faces_into_two_clusters(|face: &[Vertex]| {
            is_in_the_plane(self.positions(), self.normals(), face, tol * tol)
        })
    }

    fn into_components(&self) -> Vec<Vec<usize>> {
        let face_adjacency = self.faces().face_adjacency();
        get_components(&face_adjacency)
    }
}

impl Faces {
    /// create the adjacency list of the faces  
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
    /// separate all faces into two clusters, one with `func` returning true and the other with false.
    /// # Returns
    /// (the vector of all faces `f` which `func(f) == true`, the one of the other faces)
    fn faces_into_two_clusters<F: Fn(&[Vertex]) -> bool>(
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


    /// experimental
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


    /// experimental
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
    let n = FaceNormal::new(positions, face, 0).normal;
    for v in face {
        if let Some(nor) = v.nor {
            if n.distance2(normals[nor]) < tol2 {
                return true;
            }
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
