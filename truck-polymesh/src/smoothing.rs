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
            .for_each(|normal| *normal = normal.normalize())
    }
    fn add_naive_normals(&mut self, overwrite: bool) {
        let mut mesh = self.debug_editor();
        let (positions, normals, faces) = (&mesh.positions, &mut mesh.normals, &mut mesh.faces);
        if overwrite {
            normals.clear()
        }
        faces.face_iter_mut().for_each(|face| {
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
        self.reflect_normal_clusters(vnmap);
    }
}

impl PolygonMesh {
    fn clustering_noraml_faces(&self, inf: f64) -> HashMap<usize, Vec<Vec<FaceNormal>>> {
        let positions = self.positions();
        let mut vnmap = HashMap::new();
        for (i, face) in self.face_iter().enumerate() {
            for j in 2..face.len() {
                add_face_normal(positions, i, face, 0, j - 1, j, &mut vnmap, inf);
            }
        }
        vnmap
    }

    fn reflect_normal_clusters(&mut self, vnmap: HashMap<usize, Vec<Vec<FaceNormal>>>) {
        let mut new_normals = Vec::new();
        for (pos_id, vecs) in vnmap.iter() {
            for vec in vecs {
                let mut tmp = get_normal_sum(vec);
                tmp /= tmp.magnitude();
                new_normals.push(tmp);
                let normal_id = new_normals.len() - 1;
                for FaceNormal { face_id, .. } in vec {
                    if face_id < &self.tri_faces().len() {
                        signup_vertex_normal(*pos_id, *face_id, normal_id, self.tri_faces_mut());
                    } else if face_id < &(self.tri_faces().len() + self.quad_faces().len()) {
                        let i = face_id - self.tri_faces().len();
                        signup_vertex_normal(*pos_id, i, normal_id, self.quad_faces_mut());
                    } else {
                        let i = face_id - self.tri_faces().len() - self.quad_faces().len();
                        signup_vertex_normal(*pos_id, i, normal_id, self.other_faces_mut());
                    }
                }
            }
        }

        self.normals = new_normals;
    }
}

#[derive(Clone)]
struct FaceNormal {
    face_id: usize,
    normal: Vector3,
}

impl FaceNormal {
    fn new(positions: &Vec<Point3>, face: &[Vertex], face_id: usize) -> FaceNormal {
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

fn get_normal_sum(normals: &Vec<FaceNormal>) -> Vector3 {
    normals
        .iter()
        .fold(Vector3::zero(), |sum, x| sum + &x.normal)
}

fn add_face_normal<I: AsRef<[Vertex]>>(
    positions: &[Point3],
    face_id: usize,
    face: I,
    idx0: usize,
    idx1: usize,
    idx2: usize,
    vnmap: &mut HashMap<usize, Vec<Vec<FaceNormal>>>,
    inf: f64,
) {
    let face = face.as_ref();
    let vec0 = &positions[face[idx1][0]] - &positions[face[idx0][0]];
    let vec1 = &positions[face[idx2][0]] - &positions[face[idx0][0]];
    let normal = vec0.cross(vec1);
    let face_normal = FaceNormal { face_id, normal };
    add_to_vnmap(face[idx0][0], face_normal.clone(), vnmap, inf);
    add_to_vnmap(face[idx1][0], face_normal.clone(), vnmap, inf);
    add_to_vnmap(face[idx2][0], face_normal, vnmap, inf);
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
                let tmp = get_normal_sum(&*vec);
                if face_normal.normal.cos_angle(&tmp) > inf {
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

fn signup_vertex_normal<T: AsMut<[Vertex]>>(
    pos_id: usize,
    face_id: usize,
    normal_id: usize,
    face_list: &mut [T],
) {
    let j = (0..face_list.len())
        .find(|j| face_list[face_id].as_mut()[*j][0] == pos_id)
        .unwrap();
    face_list[face_id].as_mut()[j][2] = normal_id;
}
