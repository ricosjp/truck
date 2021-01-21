use crate::*;
use std::collections::HashMap;

/// Filters for adding normals
pub trait NormalFilters {
    /// Normalize all normals and assign `None` to the `nor` index of the vertices
    /// that has irregular normals.
    /// # Examples
    /// ```
    /// use truck_polymesh::prelude::*;
    /// 
    /// // Morbid data only for testing
    /// let positions = vec![Point3::new(0.0, 0.0, 0.0)];
    /// let normals = vec![
    ///     Vector3::new(100.0, 20.0, 56.0),
    ///     Vector3::new(1.0e-12, 3.536e10, std::f64::NAN),
    ///     Vector3::new(0.0, 1.0, 0.0),
    /// ];
    /// let faces = Faces::from_iter(&[
    ///     &[(0, None, Some(0)), (0, None, Some(1)), (0, None, Some(2))],
    /// ]);
    /// let mut mesh = PolygonMesh::new(positions, Vec::new(), normals, faces);
    /// 
    /// mesh.normalize_normals();
    /// assert!(mesh.normals()[0].magnitude().near(&1.0));
    /// assert_eq!(mesh.faces()[0][1].nor, None);
    /// ```
    fn normalize_normals(&mut self) -> &mut Self;
    /// Adds face normals to each vertices.
    /// # Arguments
    /// - If `overwrite == true`, clear all normals and update all normals in vertices.
    /// - If `overwrite == false`, add normals only for `nor` is `None`.
    /// # Examples
    /// Compare with the examples of [`add_smooth_normals`](./trait.NormalFilters.html#tymethod.add_smooth_normals).
    /// ```
    /// use truck_polymesh::prelude::*;
    /// let positions = vec![
    ///     Point3::new(-5.0, 0.0, 0.0),
    ///     Point3::new(0.0, 2.0, -2.0),
    ///     Point3::new(0.0, 2.0, 0.0),
    ///     Point3::new(0.0, 2.0, 2.0),
    ///     Point3::new(5.0, 0.0, 0.0),
    /// ];
    /// let faces = Faces::from_iter(&[
    ///     &[0, 2, 1], &[0, 3, 2], &[1, 2, 4], &[2, 3, 4],
    /// ]);
    /// let mut mesh = PolygonMesh::new(positions, Vec::new(), Vec::new(), faces);
    /// 
    /// mesh.add_naive_normals(true);
    /// let v0: Vertex = mesh.faces()[0][1];
    /// let v1: Vertex = mesh.faces()[3][0];
    /// 
    /// // those vertices are at position with the index 2.
    /// assert_eq!(v0.pos, 2); assert_eq!(v1.pos, 2);
    /// 
    /// // Each normal is each face normal.
    /// assert!(mesh.normals()[v0.nor.unwrap()].near(&Vector3::new(-2.0, 5.0, 0.0).normalize()));
    /// assert!(mesh.normals()[v1.nor.unwrap()].near(&Vector3::new(2.0, 5.0, 0.0).normalize()));
    /// ```
    fn add_naive_normals(&mut self, overwrite: bool) -> &mut Self;
    /// add the smooth normal vectors to the mesh.
    /// # Details
    /// For each vertex, apply the following algorithm:
    /// 1. prepare vectors that enumerate the normals of the faces containing
    /// the target vertices in order.
    /// 1. cluster each normal `n` in turn in the following manner.
    ///  * If there is an existing cluster `A` in which the angle between the weighted
    /// average of `A` and `n` is less than or equal to `tol_ang`, add `n` to `A`.
    ///  * If cluster `A` as described above does not exist,
    /// create a new cluster that contains only `n`.
    /// # Arguments
    /// - If `overwrite == true`, clear all normals and update all normals in vertices.
    /// - If `overwrite == false`, add normals only for `nor` is `None`.
    /// # Examples
    /// Compare with the examples of [`add_smooth_normals`](./trait.NormalFilters.html#tymethod.add_smooth_normals).
    /// ```
    /// use truck_polymesh::prelude::*;
    /// let positions = vec![
    ///     Point3::new(-5.0, 0.0, 0.0),
    ///     Point3::new(0.0, 2.0, -2.0),
    ///     Point3::new(0.0, 2.0, 0.0),
    ///     Point3::new(0.0, 2.0, 2.0),
    ///     Point3::new(5.0, 0.0, 0.0),
    /// ];
    /// let faces = Faces::from_iter(&[
    ///     &[0, 2, 1], &[0, 3, 2], &[1, 2, 4], &[2, 3, 4],
    /// ]);
    /// let mut mesh = PolygonMesh::new(positions, Vec::new(), Vec::new(), faces);
    /// 
    /// mesh.add_smooth_normals(0.8, true);
    /// let v0: Vertex = mesh.faces()[0][1];
    /// let v1: Vertex = mesh.faces()[3][0];
    /// 
    /// // those vertices are at position with the index 2.
    /// assert_eq!(v0.pos, 2); assert_eq!(v1.pos, 2);
    /// 
    /// // Normals are avaraged!
    /// assert!(mesh.normals()[v0.nor.unwrap()].near(&Vector3::new(0.0, 1.0, 0.0)));
    /// assert!(mesh.normals()[v1.nor.unwrap()].near(&Vector3::new(0.0, 1.0, 0.0)));
    /// 
    /// // If the tolerance is enough little, the faces are recognized as edges.
    /// mesh.add_smooth_normals(0.6, true); // Normals are overwritten!
    /// let v0: Vertex = mesh.faces()[0][1];
    /// let v1: Vertex = mesh.faces()[3][0];
    /// assert!(mesh.normals()[v0.nor.unwrap()].near(&Vector3::new(-2.0, 5.0, 0.0).normalize()));
    /// assert!(mesh.normals()[v1.nor.unwrap()].near(&Vector3::new(2.0, 5.0, 0.0).normalize()));
    /// ```
    fn add_smooth_normals(&mut self, tol_ang: f64, overwrite: bool) -> &mut Self;
}

impl NormalFilters for PolygonMesh {
    fn normalize_normals(&mut self) -> &mut Self {
        let mut mesh = self.debug_editor();
        let (normals, faces) = (&mut mesh.normals, &mut mesh.faces);
        normals
            .iter_mut()
            .for_each(move |normal| *normal = normal.normalize());
        faces.face_iter_mut().flatten().for_each(|v| {
            if let Some(idx) = v.nor {
                if !normals[idx].magnitude2().near(&1.0) {
                    v.nor = None;
                }
            }
        });
        drop(mesh);
        self
    }
    fn add_naive_normals(&mut self, overwrite: bool) -> &mut Self {
        let mut mesh = self.debug_editor();
        let (positions, normals, faces) = (&mesh.positions, &mut mesh.normals, &mut mesh.faces);
        if overwrite {
            normals.clear()
        }
        faces.face_iter_mut().for_each(move |face| {
            let normal = FaceNormal::new(positions, face, 0).normal;
            let mut added = false;
            face.iter_mut().for_each(|v| {
                if v.nor.is_none() || overwrite {
                    if !added {
                        normals.push(normal);
                        added = true;
                    }
                    v.nor = Some(normals.len() - 1);
                }
            });
        });
        drop(mesh);
        self
    }
    fn add_smooth_normals(&mut self, tol_ang: f64, overwrite: bool) -> &mut Self {
        let vnmap = self.clustering_noraml_faces(tol_ang.cos());
        self.reflect_normal_clusters(vnmap, overwrite);
        self
    }
}

impl PolygonMesh {
    fn clustering_noraml_faces(&self, inf: f64) -> HashMap<usize, Vec<Vec<FaceNormal>>> {
        let positions = self.positions();
        let mut vnmap = HashMap::new();
        self.face_iter()
            .enumerate()
            .for_each(|(i, face)| add_face_normal(positions, i, face, &mut vnmap, inf));
        vnmap
    }

    fn reflect_normal_clusters(
        &mut self,
        vnmap: HashMap<usize, Vec<Vec<FaceNormal>>>,
        overwrite: bool,
    ) {
        let mut mesh = self.debug_editor();
        let (normals, faces) = (&mut mesh.normals, &mut mesh.faces);
        if overwrite {
            normals.clear();
        }
        for (pos_id, vecs) in vnmap.into_iter() {
            for vec in vecs {
                let normal = vec
                    .iter()
                    .fold(Vector3::zero(), |sum, x| sum + x.normal)
                    .normalize();
                for FaceNormal { face_id, .. } in vec {
                    signup_vertex_normal(pos_id, face_id, normals, normal, faces, overwrite);
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct FaceNormal {
    pub face_id: usize,
    pub normal: Vector3,
}

impl FaceNormal {
    pub(super) fn new(positions: &[Point3], face: &[Vertex], face_id: usize) -> FaceNormal {
        let center = face
            .iter()
            .fold(Vector3::zero(), |sum, v| sum + positions[v.pos].to_vec())
            / face.len() as f64;
        let normal = face
            .windows(2)
            .chain(std::iter::once([face[face.len() - 1], face[0]].as_ref()))
            .fold(Vector3::zero(), |sum, v| {
                let vec0 = positions[v[0].pos].to_vec() - center;
                let vec1 = positions[v[1].pos].to_vec() - center;
                sum + vec0.cross(vec1)
            })
            .normalize();
        FaceNormal { face_id, normal }
    }
}

fn add_face_normal(
    positions: &[Point3],
    face_id: usize,
    face: &[Vertex],
    vnmap: &mut HashMap<usize, Vec<Vec<FaceNormal>>>,
    inf: f64,
) {
    let face_normal = FaceNormal::new(positions, face, face_id);
    face.iter().for_each(|v| {
        add_to_vnmap(v.pos, face_normal, vnmap, inf);
    })
}

fn add_to_vnmap(
    pos_id: usize,
    face_normal: FaceNormal,
    vnmap: &mut HashMap<usize, Vec<Vec<FaceNormal>>>,
    inf: f64,
) {
    match vnmap.get_mut(&pos_id) {
        Some(vecs) => {
            for vec in vecs.iter_mut() {
                let normal = vec
                    .iter()
                    .fold(Vector3::zero(), |sum, x| sum + x.normal)
                    .normalize();
                if face_normal.normal.dot(normal) > inf {
                    vec.push(face_normal);
                    return;
                }
            }
            vecs.push(vec![face_normal]);
        }
        None => {
            let vecs = vec![vec![face_normal]];
            vnmap.insert(pos_id, vecs);
        }
    }
}

fn signup_vertex_normal(
    pos_id: usize,
    face_id: usize,
    normals: &mut Vec<Vector3>,
    normal: Vector3,
    faces: &mut Faces,
    overwrite: bool,
) {
    let face = faces[face_id].as_mut();
    let j = (0..face.len()).find(|j| face[*j].pos == pos_id).unwrap();
    if face[j].nor.is_none() || overwrite {
        normals.push(normal);
        face[j].nor = Some(normals.len() - 1);
    }
}
