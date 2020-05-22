use crate::MeshHandler;
use geometry::Vector3;
use std::collections::HashMap;

impl MeshHandler {
    /// add the smooth normal vectors to the mesh.
    pub fn add_smooth_normal(&mut self, tol_ang: f64) -> &mut Self {
        let inf = tol_ang.cos();
        let positions = &self.mesh.positions;
        let tri_faces = &mut self.mesh.tri_faces;
        let quad_faces = &mut self.mesh.quad_faces;
        let other_faces = &mut self.mesh.other_faces;

        let mut vnmap = HashMap::new();
        for (i, face) in tri_faces.iter().enumerate() {
            add_face_normal(positions, i, face, 0, 1, 2, &mut vnmap, inf);
        }
        for (mut i, face) in quad_faces.iter().enumerate() {
            i += tri_faces.len();
            add_face_normal(positions, i, face, 0, 1, 3, &mut vnmap, inf);
            add_face_normal(positions, i, face, 2, 3, 1, &mut vnmap, inf);
        }
        for (mut i, face) in other_faces.iter().enumerate() {
            i += tri_faces.len() + quad_faces.len();
            for j in 2..face.len() {
                add_face_normal(positions, i, face, 0, j - 1, j, &mut vnmap, inf);
            }
        }

        let mut new_normals = Vec::new();
        for (pos_id, vecs) in vnmap.iter() {
            for vec in vecs {
                let mut tmp = get_normal_sum(vec);
                tmp /= tmp.norm();
                new_normals.push(tmp);
                let normal_id = new_normals.len() - 1;
                for FaceNormal { face_id, normal: _ } in vec {
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
        self
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
    positions: &Vec<Vector3>,
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
    let normal = vec0 ^ vec1;
    let face_normal = FaceNormal { face_id, normal };
    add_to_vnmap(face[idx0][0], face_normal.clone(), vnmap, inf);
    add_to_vnmap(face[idx1][0], face_normal.clone(), vnmap, inf);
    add_to_vnmap(face[idx2][0], face_normal, vnmap, inf);
}

fn add_to_vnmap(
    vert: usize,
    face_normal: FaceNormal,
    vnmap: &mut HashMap<usize, Vec<Vec<FaceNormal>>>,
    inf: f64,
)
{
    match vnmap.get_mut(&vert) {
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
            vnmap.insert(vert, vecs);
        }
    }
}

fn signup_vertex_normal<T: AsMut<[[usize; 3]]>>(
    pos_id: usize,
    face_id: usize,
    normal_id: usize,
    face_list: &mut [T],
) {
    let j = (0..3).find(|j| face_list[face_id].as_mut()[*j][0] == pos_id).unwrap();
    face_list[face_id].as_mut()[j][2] = normal_id;
}
