use crate::*;
use std::collections::HashMap;

pub trait NormalFilters {
    /// First, assign `None` to the `nor` index of the vertices that has a normal of zero length,
    /// and then normalize all normals.
    fn normalize_normals(&mut self);
    /// Adds face normals to each vertices.
    /// # Arguments
    /// - If `overwrite == true`, clear all normals and update all normals in vertices.
    /// - If `overwrite == false`, add normals only for `nor` is `None`.
    fn add_naive_normals(&mut self, overwrite: bool);
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
    fn add_smooth_normal(&mut self, tol_ang: f64, overwrite: bool);
}

impl NormalFilters for PolygonMesh {
    fn normalize_normals(&mut self) {
        let mut mesh = self.debug_editor();
        let (normals, faces) = (&mut mesh.normals, &mut mesh.faces);
        faces.face_iter_mut().flatten().for_each(|v| {
            if let Some(idx) = v.nor {
                if normals[idx].magnitude2().so_small2() {
                    v.nor = None;
                }
            }
        });
        normals
            .iter_mut()
            .for_each(move |normal| *normal = normal.normalize())
    }
    fn add_naive_normals(&mut self, overwrite: bool) {
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
    }
    fn add_smooth_normal(&mut self, tol_ang: f64, overwrite: bool) {
        let vnmap = self.clustering_noraml_faces(tol_ang.cos());
        self.reflect_normal_clusters(vnmap, overwrite);
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
struct FaceNormal {
    face_id: usize,
    normal: Vector3,
}

impl FaceNormal {
    fn new(positions: &[Point3], face: &[Vertex], face_id: usize) -> FaceNormal {
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
