use crate::MeshHandler;
use geometry::{Matrix, Vector};

impl MeshHandler {
    pub fn triangulate(&mut self) -> &mut Self {
        let mesh = &mut self.mesh;
        for quad in &mesh.quad_faces {
            mesh.tri_faces
                .push([quad[0].clone(), quad[1].clone(), quad[3].clone()]);
            mesh.tri_faces
                .push([quad[2].clone(), quad[3].clone(), quad[1].clone()]);
        }
        for poly in &mesh.other_faces {
            for i in 2..poly.len() {
                mesh.tri_faces
                    .push([poly[0].clone(), poly[i - 1].clone(), poly[i].clone()]);
            }
        }
        mesh.quad_faces = Vec::new();

        self
    }

    pub fn quadrangulate(&mut self, tol: f64) -> &mut Self {
        let face_adjacency = self.face_adjacency();
        let mut used = vec![false; self.mesh.tri_faces.len()];
        let mesh = &mut self.mesh;

        let mut new_tri = Vec::new();
        for (i, face) in mesh.tri_faces.iter().enumerate() {
            if used[i] {
                continue;
            }

            used[i] = true;

            let v0 = &mesh.vertices[face[0][0]];
            let v1 = &mesh.vertices[face[1][0]];
            let v2 = &mesh.vertices[face[2][0]];

            let mut passed = Vec::new();
            for j in &face_adjacency[i] {
                if used[*j] {
                    continue;
                }
                let other = &mesh.tri_faces[*j];
                for k in 0..3 {
                    if face.iter().find(|arr| arr[0] == other[k][0]).is_none() {
                        let v3 = &mesh.vertices[other[k][0]];
                        let (det0, det1, det2, dot) = divide(v0, v1, v2, v3);
                        if det2 < tol {
                            passed.push((*j, other[k].clone(), det0, det1, dot));
                        }
                        break;
                    }
                }
            }
            if passed.is_empty() {
                new_tri.push(face.clone());
            } else {
                let pass = passed
                    .iter()
                    .min_by(|a, b| a.4.abs().partial_cmp(&b.4.abs()).unwrap())
                    .unwrap();
                let a = pass.2;
                let b = pass.3;
                let v0 = face[0].clone();
                let v1 = face[1].clone();
                let v2 = face[2].clone();
                let v3 = pass.1.clone();
                if a < 0.0 && b < 0.0 {
                    new_tri.push(face.clone());
                    new_tri.push(mesh.tri_faces[pass.0].clone());
                } else if a > 0.0 && b < 0.0 && a + b < 1.0 {
                    mesh.quad_faces.push([v0, v3, v1, v2]);
                } else if a < 0.0 && b > 0.0 && a + b < 1.0 {
                    mesh.quad_faces.push([v0, v1, v2, v3]);
                } else {
                    mesh.quad_faces.push([v0, v1, v3, v2]);
                }
                used[pass.0] = true;
            }
        }
        mesh.tri_faces = new_tri;

        self
    }
}

fn divide(v0: &[f64; 3], v1: &[f64; 3], v2: &[f64; 3], v3: &[f64; 3]) -> (f64, f64, f64, f64) {
    let v0 = Vector::new3(v0[0], v0[1], v0[2]);
    let v1 = Vector::new3(v1[0], v1[1], v1[2]);
    let v2 = Vector::new3(v2[0], v2[1], v2[2]);
    let v3 = Vector::new3(v3[0], v3[1], v3[2]);

    let org = Vector::new3(0.0, 0.0, 0.0);
    let vec0 = v1 - &v0;
    let vec1 = v2 - &v0;
    let mut vec2 = &vec0 ^ &vec1;
    vec2[3] = 0.0;
    
    let mat = Matrix::by_rows_ref(&vec0, &vec1, &vec2, &org);
    let vec3 = v3 - &v0;
    match mat.solve(&vec3) {
        Ok(det) => (det[0], det[1], det[2], vec0.cos_angle(&vec1)),
        Err(_) => (0.0, 0.0, 1.0, 0.0),
    }
}
