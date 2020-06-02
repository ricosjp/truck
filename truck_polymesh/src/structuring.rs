use crate::{get_tri, MeshHandler, PolygonMesh};
use geometry::Vector3;

/// triangulation, quadrangulation, give a structure
impl MeshHandler {
    /// triangulate all n-gons
    pub fn triangulate(&mut self) -> &mut Self {
        let mesh = &mut self.mesh;
        let tri_faces = &mut mesh.tri_faces;
        for quad in &mesh.quad_faces {
            tri_faces.push(get_tri(quad, 0, 1, 3));
            tri_faces.push(get_tri(quad, 2, 3, 1));
        }
        for poly in &mesh.other_faces {
            for i in 2..poly.len() {
                tri_faces.push(get_tri(poly, 0, i - 1, i));
            }
        }
        mesh.quad_faces = Vec::new();
        mesh.other_faces = Vec::new();

        self
    }

    /// join two triangles into one quadrangle.
    /// # Arguments
    /// * `tol` - the tolerance for determining that four points are in the same plane
    /// # Details
    /// The overview of the algorithm is the following:
    /// 1. make the list of pairs of triangles satisfying the following conditions:
    ///   * two faces are adjacent by one edge,
    ///   * the pair of faces consists four vertices in the same plane
    /// 1. for each joined quadrangle, calculate the score by the sum of the absolute value of
    /// the cosine for each angles,
    /// 1. sort the list of the pairs of triangles by the score
    /// 1. take a pair of triangles in order from the top of the list and register a new one
    /// if it doesn't conflict with the one has been already registered.
    pub fn quadrangulate(&mut self, tol: f64) -> &mut Self {
        let list = self.create_face_edge_list(tol);
        self.reflect_face_edge_list(list);
        self
    }

    fn create_face_edge_list(&self, tol: f64) -> Vec<FaceEdge> {
        let face_adjacency = self.face_adjacency();
        let mesh = &self.mesh;

        let mut passed = Vec::new();
        for i in 0..mesh.tri_faces.len() {
            for j in &face_adjacency[i] {
                if i > *j {
                    continue;
                } else if let Some(face_edge) = mesh.get_face_edge(i, *j, tol) {
                    passed.push(face_edge);
                }
            }
        }
        passed.sort_by(|x, y| x.score.partial_cmp(&y.score).unwrap());
        passed
    }

    fn reflect_face_edge_list(&mut self, list: Vec<FaceEdge>) {
        let mesh = &mut self.mesh;

        let mut used = vec![false; mesh.tri_faces.len()];
        for face_edge in list.into_iter() {
            let (i, j) = face_edge.faces;
            if used[i] || used[j] {
                continue;
            }
            used[i] = true;
            used[j] = true;
            mesh.quad_faces.push(face_edge.positions);
        }

        let mut new_tri = Vec::new();
        for (i, flag) in used.into_iter().enumerate() {
            if !flag {
                new_tri.push(mesh.tri_faces[i].clone());
            }
        }
        mesh.tri_faces = new_tri;
    }
}

struct FaceEdge {
    faces: (usize, usize),
    positions: [[usize; 3]; 4],
    score: f64,
}

impl PolygonMesh {
    fn get_face_edge(&self, face0_id: usize, face1_id: usize, tol: f64) -> Option<FaceEdge> {
        let face0 = &self.tri_faces[face0_id];
        let face1 = &self.tri_faces[face1_id];

        let k = (0..3)
            .find(|k| face0.iter().find(|x| x[0] == face1[*k][0]).is_none())
            .unwrap();
        let vec0 = &self.positions[face0[1][0]] - &self.positions[face0[0][0]];
        let vec1 = &self.positions[face0[2][0]] - &self.positions[face0[0][0]];
        let mut n = &vec0 ^ &vec1;
        n /= n.norm();
        let vec2 = &self.positions[face1[k][0]] - &self.positions[face0[0][0]];
        let coef = match vec2.divide(&vec0, &vec1, &n) {
            Some(coef) => coef,
            None => return None,
        };

        if coef[2] > tol {
            None
        } else if coef[0] > 0.0 && coef[1] > 0.0 {
            Some(FaceEdge {
                faces: (face0_id, face1_id),
                positions: [
                    face0[0].clone(),
                    face0[1].clone(),
                    face1[k].clone(),
                    face0[2].clone(),
                ],
                score: calc_score(&vec0, &(&vec2 - &vec0), &(&vec1 - &vec2), &vec1),
            })
        } else if coef[0] < 0.0 && coef[1] > 0.0 && coef[0] + coef[1] < 1.0 {
            Some(FaceEdge {
                faces: (face0_id, face1_id),
                positions: [
                    face0[0].clone(),
                    face0[1].clone(),
                    face0[2].clone(),
                    face1[k].clone(),
                ],
                score: calc_score(&vec0, &(&vec1 - &vec0), &(&vec2 - &vec1), &vec2),
            })
        } else if coef[0] > 0.0 && coef[1] < 0.0 && coef[0] + coef[1] < 1.0 {
            Some(FaceEdge {
                faces: (face0_id, face1_id),
                positions: [
                    face0[0].clone(),
                    face1[k].clone(),
                    face0[1].clone(),
                    face0[2].clone(),
                ],
                score: calc_score(&vec2, &(&vec0 - &vec2), &(&vec1 - &vec0), &vec1),
            })
        } else {
            None
        }
    }
}

fn calc_score(edge0: &Vector3, edge1: &Vector3, edge2: &Vector3, edge3: &Vector3) -> f64 {
    edge0.cos_angle(edge1).abs()
        + edge1.cos_angle(edge2).abs()
        + edge2.cos_angle(edge3).abs()
        + edge3.cos_angle(edge0).abs()
}
