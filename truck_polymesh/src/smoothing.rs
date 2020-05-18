use crate::MeshHandler;
use geometry::Vector;
use std::collections::HashMap;

impl MeshHandler {
    /// add the smooth normal to the mesh.
    pub fn add_smooth_normal(&mut self, tol_ang: f64) -> &mut Self {
        let inf = tol_ang.cos();
        let mesh = &mut self.mesh;

        let mut vnmap = HashMap::new();
        for (i, face) in mesh.tri_faces.iter().enumerate() {
            add_normal_one_face(
                &mesh.vertices,
                i,
                face[0][0],
                face[1][0],
                face[2][0],
                &mut vnmap,
                inf,
            );
        }
        for (i, face) in mesh.quad_faces.iter().enumerate() {
            add_normal_one_face(
                &mesh.vertices,
                i + mesh.tri_faces.len(),
                face[0][0],
                face[1][0],
                face[3][0],
                &mut vnmap,
                inf,
            );
            add_normal_one_face(
                &mesh.vertices,
                i + mesh.tri_faces.len(),
                face[2][0],
                face[3][0],
                face[1][0],
                &mut vnmap,
                inf,
            );
        }

        let mut normals = Vec::new();
        for (vert, vecs) in vnmap.iter() {
            for vec in vecs {
                let mut tmp = vec.iter().fold(Vector::zero(), |sum, (_, n)| sum + n);
                tmp /= tmp.norm();
                normals.push([tmp[0], tmp[1], tmp[2]]);
                let idx = normals.len() - 1;
                for (i, _) in vec {
                    if i < &mesh.tri_faces.len() {
                        let j = (0..3).find(|j| mesh.tri_faces[*i][*j][0] == *vert).unwrap();
                        mesh.tri_faces[*i][j][2] = idx;
                    } else {
                        let i = i - mesh.tri_faces.len();
                        let j = (0..4).find(|j| mesh.quad_faces[i][*j][0] == *vert).unwrap();
                        mesh.quad_faces[i][j][2] = idx;
                    }
                }
            }
        }

        mesh.normals = normals;
        self
    }
}

fn add_normal_one_face(
    vertices: &Vec<[f64; 3]>,
    i: usize,
    fv0: usize,
    fv1: usize,
    fv2: usize,
    vnmap: &mut HashMap<usize, Vec<Vec<(usize, Vector)>>>,
    inf: f64,
) {
    let tmp = vertices[fv0];
    let org = Vector::new3(tmp[0], tmp[1], tmp[2]);
    let tmp = vertices[fv1];
    let vec0 = Vector::new3(tmp[0], tmp[1], tmp[2]) - &org;
    let tmp = vertices[fv2];
    let vec1 = Vector::new3(tmp[0], tmp[1], tmp[2]) - &org;
    let n = vec0 ^ vec1;
    add_to_vnmap(fv0, i, n.clone(), vnmap, inf);
    add_to_vnmap(fv1, i, n.clone(), vnmap, inf);
    add_to_vnmap(fv2, i, n, vnmap, inf);
}

fn add_to_vnmap(
    vert: usize,
    face: usize,
    mut normal: Vector,
    vnmap: &mut HashMap<usize, Vec<Vec<(usize, Vector)>>>,
    inf: f64,
) {
    let normal_normal = normal.projection();
    normal[3] = 0.0;
    match vnmap.get_mut(&vert) {
        Some(vecs) => {
            for vec in vecs.iter_mut() {
                let mut tmp = vec.iter().fold(Vector::zero(), |sum, (_, n)| sum + n);
                tmp /= tmp.norm();
                if &normal_normal * tmp > inf {
                    vec.push((face, normal));
                    return;
                }
            }
            vecs.push(vec![(face, normal)]);
        }
        None => {
            let vecs = vec![vec![(face, normal)]];
            vnmap.insert(vert, vecs);
        }
    }
}
