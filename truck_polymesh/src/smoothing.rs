use crate::*;
use geometry::{InnerSpace, Zero};
use std::collections::HashMap;

/// mesh smoothing filters
impl MeshHandler {
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
    pub fn add_smooth_normal(&mut self, tol_ang: f64) -> &mut Self {
        let vnmap = self.clustering_noraml_faces(tol_ang.cos());
        self.reflect_normal_clusters(vnmap);
        self
    }

    fn clustering_noraml_faces(&self, inf: f64) -> HashMap<usize, Vec<Vec<FaceNormal>>> {
        let positions = &self.mesh.positions;
        let mut vnmap = HashMap::new();
        for (i, face) in self.mesh.tri_faces.iter().enumerate() {
            add_face_normal(positions, i, face, 0, 1, 2, &mut vnmap, inf);
        }
        for (mut i, face) in self.mesh.quad_faces.iter().enumerate() {
            i += self.mesh.tri_faces.len();
            add_face_normal(positions, i, face, 0, 1, 3, &mut vnmap, inf);
            add_face_normal(positions, i, face, 2, 3, 1, &mut vnmap, inf);
        }
        for (mut i, face) in self.mesh.other_faces.iter().enumerate() {
            i += self.mesh.tri_faces.len() + self.mesh.quad_faces.len();
            for j in 2..face.len() {
                add_face_normal(positions, i, face, 0, j - 1, j, &mut vnmap, inf);
            }
        }
        vnmap
    }

    fn reflect_normal_clusters(&mut self, vnmap: HashMap<usize, Vec<Vec<FaceNormal>>>) {
        let tri_faces = &mut self.mesh.tri_faces;
        let quad_faces = &mut self.mesh.quad_faces;
        let other_faces = &mut self.mesh.other_faces;

        let mut new_normals = Vec::new();
        for (pos_id, vecs) in vnmap.iter() {
            for vec in vecs {
                let mut tmp = get_normal_sum(vec);
                tmp /= tmp.magnitude();
                new_normals.push(tmp);
                let normal_id = new_normals.len() - 1;
                for FaceNormal { face_id, .. } in vec {
                    if face_id < &tri_faces.len() {
                        signup_vertex_normal(*pos_id, *face_id, normal_id, tri_faces);
                    } else if face_id < &(tri_faces.len() + quad_faces.len()) {
                        let i = face_id - tri_faces.len();
                        signup_vertex_normal(*pos_id, i, normal_id, quad_faces);
                    } else {
                        let i = face_id - tri_faces.len() - quad_faces.len();
                        signup_vertex_normal(*pos_id, i, normal_id, other_faces);
                    }
                }
            }
        }

        self.mesh.normals = new_normals;
    }
}

#[derive(Clone)]
struct FaceNormal {
    face_id: usize,
    normal: Vector3,
}

fn get_normal_sum(normals: &Vec<FaceNormal>) -> Vector3 {
    normals
        .iter()
        .fold(Vector3::zero(), |sum, x| sum + &x.normal)
}

fn add_face_normal(
    positions: &Vec<Point3>,
    face_id: usize,
    face: &[[usize; 3]],
    idx0: usize,
    idx1: usize,
    idx2: usize,
    vnmap: &mut HashMap<usize, Vec<Vec<FaceNormal>>>,
    inf: f64,
)
{
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
)
{
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

fn signup_vertex_normal<T: AsMut<[[usize; 3]]>>(
    pos_id: usize,
    face_id: usize,
    normal_id: usize,
    face_list: &mut [T],
)
{
    let j = (0..face_list.len())
        .find(|j| face_list[face_id].as_mut()[*j][0] == pos_id)
        .unwrap();
    face_list[face_id].as_mut()[j][2] = normal_id;
}
