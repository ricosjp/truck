use crate::MeshHandler;
use geometry::Vector3;

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
                        if let Some((idx, dot)) = divide(v0, v1, v2, v3, tol) {
                            passed.push((*j, k, idx, dot));
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
                    .min_by(|a, b| a.3.abs().partial_cmp(&b.3.abs()).unwrap())
                    .unwrap();
                let v = [
                    face[0].clone(),
                    face[1].clone(),
                    face[2].clone(),
                    mesh.tri_faces[pass.0][pass.1].clone(),
                ];
                let idx = pass.2;
                mesh.quad_faces.push([v[idx[0]], v[idx[1]], v[idx[2]], v[idx[3]]]);
                used[pass.0] = true;
            }
        }
        mesh.tri_faces = new_tri;

        self
    }
}

fn divide(v0: &Vector3, v1: &Vector3, v2: &Vector3, v3: &Vector3, tol: f64) -> Option<([usize; 4], f64)> {
    let vec0 = v1 - v0;
    let vec1 = v2 - v0;
    let vec2 = &vec0 ^ &vec1;
    
    let vec3 = v3 - v0;
    match vec3.divide(&vec0, &vec1, &vec2) {
        Some(det) => {
            if det[2] > tol {
                None
            }else if det[0] > 0.0 && det[1] < 0.0 && det[0] + det[1] < 1.0 {
                Some(([0, 3, 1, 2], vec1.cos_angle(&vec3)))
            } else if det[0] < 0.0 && det[1] > 0.0 && det[0] + det[1] < 1.0 {
                Some(([0, 1, 2, 3], vec0.cos_angle(&vec3)))
            } else if det[0] > 0.0 && det[1] > 0.0 {
                Some(([0, 1, 3, 2], vec0.cos_angle(&vec1)))
            } else {
                None
            }
        },
        None => None,
    }
}
