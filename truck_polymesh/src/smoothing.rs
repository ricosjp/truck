use crate::MeshHandler;
use geometry::Vector3;
use std::collections::HashMap;

impl MeshHandler {
    /// add the smooth normal vectors to the mesh.
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
        for (i, face) in mesh.other_faces.iter().enumerate() {
            let i = i + mesh.tri_faces.len() + mesh.quad_faces.len();
            for j in 2..face.len() {
                add_normal_one_face(
                    &mesh.vertices,
                    i,
                    face[0][0],
                    face[j - 1][0],
                    face[j][0],
                    &mut vnmap,
                    inf,
                );
            }
        }

        let mut normals = Vec::new();
        for (vert, vecs) in vnmap.iter() {
            for vec in vecs {
                let mut tmp = vec.iter().fold(Vector3::zero(), |sum, (_, n)| sum + n);
                tmp /= tmp.norm();
                normals.push(tmp);
                let idx = normals.len() - 1;
                for (i, _) in vec {
                    if i < &mesh.tri_faces.len() {
                        let j = (0..3).find(|j| mesh.tri_faces[*i][*j][0] == *vert).unwrap();
                        mesh.tri_faces[*i][j][2] = idx;
                    } else if i < &(mesh.tri_faces.len() + mesh.quad_faces.len()) {
                        let i = i - mesh.tri_faces.len();
                        let j = (0..4).find(|j| mesh.quad_faces[i][*j][0] == *vert).unwrap();
                        mesh.quad_faces[i][j][2] = idx;
                    } else {
                        let i = i - mesh.tri_faces.len() - mesh.quad_faces.len();
                        let j = (0..mesh.other_faces[i].len())
                            .find(|j| mesh.other_faces[i][*j][0] == *vert)
                            .unwrap();
                        mesh.other_faces[i][j][2] = idx;
                    }
                }
            }
        }

        mesh.normals = normals;
        self
    }
}

fn add_normal_one_face(
    vertices: &Vec<Vector3>,
    i: usize,
    fv0: usize,
    fv1: usize,
    fv2: usize,
    vnmap: &mut HashMap<usize, Vec<Vec<(usize, Vector3)>>>,
    inf: f64,
)
{
    let org = &vertices[fv0];
    let vec0 = &vertices[fv1] - org;
    let vec1 = &vertices[fv2] - org;
    let n = vec0 ^ vec1;
    add_to_vnmap(fv0, i, n.clone(), vnmap, inf);
    add_to_vnmap(fv1, i, n.clone(), vnmap, inf);
    add_to_vnmap(fv2, i, n, vnmap, inf);
}

fn add_to_vnmap(
    vert: usize,
    face: usize,
    normal: Vector3,
    vnmap: &mut HashMap<usize, Vec<Vec<(usize, Vector3)>>>,
    inf: f64,
)
{
    let normal_normal = &normal / normal.norm();
    match vnmap.get_mut(&vert) {
        Some(vecs) => {
            for vec in vecs.iter_mut() {
                let mut tmp = vec.iter().fold(Vector3::zero(), |sum, (_, n)| sum + n);
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
