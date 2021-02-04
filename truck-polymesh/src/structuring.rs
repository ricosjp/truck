use crate::*;

/// triangulation, quadrangulation, give a structure
pub trait StructuringFilter {
    /// triangulate all n-gons
    /// # Examples
    /// ```
    /// use truck_polymesh::prelude::*;
    /// 
    /// // cube consisting quad faces
    /// let positions = vec![
    ///     Point3::new(0.0, 0.0, 0.0),
    ///     Point3::new(1.0, 0.0, 0.0),
    ///     Point3::new(1.0, 1.0, 0.0),
    ///     Point3::new(0.0, 1.0, 0.0),
    ///     Point3::new(0.0, 0.0, 1.0),
    ///     Point3::new(1.0, 0.0, 1.0),
    ///     Point3::new(1.0, 1.0, 1.0),
    ///     Point3::new(0.0, 1.0, 1.0),
    /// ];
    /// let faces = Faces::from_iter(&[
    ///     &[3, 2, 1, 0], &[0, 1, 5, 4], &[1, 2, 6, 5],
    ///     &[2, 3, 7, 6], &[3, 0, 4, 7], &[4, 5, 6, 7],
    /// ]);
    /// let mut mesh = PolygonMesh::new(positions, Vec::new(), Vec::new(), faces);
    /// 
    /// // the number of face becomes twice since each quadrangle decompose into two triangles.
    /// assert_eq!(mesh.faces().len(), 6);
    /// mesh.triangulate();
    /// assert_eq!(mesh.faces().len(), 12);
    /// ```
    fn triangulate(&mut self) -> &mut Self;
    /// join two triangles into one quadrangle.
    /// # Arguments
    /// * `plane_tol` - the tolerance for determining that four points are in the same plane
    /// * `score_tol` - The upper limit of the score to determine if the four points form an uncrushed rectangle
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
    /// # Examples
    /// ```
    /// use truck_polymesh::prelude::*;
    /// 
    /// // cube consisting tri_faces
    /// let positions = vec![
    ///     Point3::new(0.0, 0.0, 0.0),
    ///     Point3::new(1.0, 0.0, 0.0),
    ///     Point3::new(1.0, 1.0, 0.0),
    ///     Point3::new(0.0, 1.0, 0.0),
    ///     Point3::new(0.0, 0.0, 1.0),
    ///     Point3::new(1.0, 0.0, 1.0),
    ///     Point3::new(1.0, 1.0, 1.0),
    ///     Point3::new(0.0, 1.0, 1.0),
    /// ];
    /// let faces = Faces::from_iter(&[
    ///     &[3, 2, 0], &[1, 0, 2], &[0, 1, 4], &[5, 4, 1],
    ///     &[1, 2, 5], &[6, 5, 2], &[2, 3, 6], &[7, 6, 3],
    ///     &[3, 0, 7], &[4, 7, 0], &[4, 5, 7], &[6, 7, 5],
    /// ]);
    /// let mut mesh = PolygonMesh::new(positions, Vec::new(), Vec::new(), faces);
    /// 
    /// // The number of faces becomes a half since each pair of triangles is combined.
    /// assert_eq!(mesh.faces().len(), 12);
    /// mesh.quadrangulate(0.01, 0.1);
    /// assert_eq!(mesh.faces().len(), 6);
    /// ```
    fn quadrangulate(&mut self, plane_tol: f64, score_tol: f64) -> &mut Self;
}

impl StructuringFilter for PolygonMesh {
    fn triangulate(&mut self) -> &mut Self {
        let mut tri_faces = self.faces().tri_faces().clone();
        for quad in self.faces().quad_faces() {
            tri_faces.push(get_tri(quad, 0, 1, 3));
            tri_faces.push(get_tri(quad, 2, 3, 1));
        }
        for poly in self.faces().other_faces() {
            for i in 2..poly.len() {
                tri_faces.push(get_tri(poly, 0, i - 1, i));
            }
        }
        *self.debug_editor().faces = Faces::from_tri_and_quad_faces(tri_faces, Vec::new());
        self
    }
    fn quadrangulate(&mut self, plane_tol: f64, score_tol: f64) -> &mut Self {
        let list = self.create_face_edge_list(plane_tol, score_tol);
        self.reflect_face_edge_list(list);
        self
    }
}

impl PolygonMesh {
    fn create_face_edge_list(&self, plane_tol: f64, score_tol: f64) -> Vec<FaceEdge> {
        let face_adjacency = self.faces().face_adjacency();
        let mut passed = Vec::new();
        for i in 0..self.faces().tri_faces().len() {
            for j in &face_adjacency[i] {
                if i > *j {
                    continue;
                } else if let Some(face_edge) = self.get_face_edge(i, *j, plane_tol, score_tol) {
                    passed.push(face_edge);
                }
            }
        }
        passed.sort_by(|x, y| x.score.partial_cmp(&y.score).unwrap());
        passed
    }

    fn reflect_face_edge_list(&mut self, list: Vec<FaceEdge>) {
        let mut used = vec![false; self.faces().tri_faces().len()];
        let mut quad_faces = self.faces().quad_faces().clone();
        quad_faces.extend(list.into_iter().filter_map(|face_edge| {
            let (i, j) = face_edge.faces;
            if used[i] || used[j] {
                None
            } else {
                used[i] = true;
                used[j] = true;
                Some(face_edge.positions)
            }
        }));
        let tri_faces = self.faces().tri_faces();
        let tri_faces = used
            .into_iter()
            .enumerate()
            .filter_map(move |(i, flag)| match flag {
                true => None,
                false => Some(tri_faces[i]),
            })
            .collect::<Vec<_>>();
        *self.debug_editor().faces = Faces::from_tri_and_quad_faces(tri_faces, quad_faces);
    }
}

struct FaceEdge {
    faces: (usize, usize),
    positions: [Vertex; 4],
    score: f64,
}

impl PolygonMesh {
    fn get_face_edge(
        &self,
        face0_id: usize,
        face1_id: usize,
        plane_tol: f64,
        score_tol: f64,
    ) -> Option<FaceEdge> {
        let face0 = self.faces().tri_faces()[face0_id];
        let face1 = self.faces().tri_faces()[face1_id];

        let k = (0..3)
            .find(|k| face0.iter().find(|x| x.pos == face1[*k].pos).is_none())
            .unwrap();
        let vec0 = self.positions()[face0[1].pos] - self.positions()[face0[0].pos];
        let vec1 = self.positions()[face0[2].pos] - self.positions()[face0[0].pos];
        let mut n = vec0.cross(vec1);
        n /= n.magnitude();
        let vec2 = self.positions()[face1[k].pos] - self.positions()[face0[0].pos];
        let mat = Matrix3::from_cols(vec0.clone(), vec1.clone(), n.clone());
        let coef = mat.invert().unwrap() * vec2;

        if coef[2] > plane_tol {
            None
        } else if coef[0] > 0.0 && coef[1] > 0.0 {
            let score = calc_score(&vec0, &(&vec2 - &vec0), &(&vec1 - &vec2), &vec1);
            if score < score_tol {
                Some(FaceEdge {
                    faces: (face0_id, face1_id),
                    positions: [
                        face0[0].clone(),
                        face0[1].clone(),
                        face1[k].clone(),
                        face0[2].clone(),
                    ],
                    score,
                })
            } else {
                None
            }
        } else if coef[0] < 0.0 && coef[1] > 0.0 && coef[0] + coef[1] < 1.0 {
            let score = calc_score(&vec0, &(&vec1 - &vec0), &(&vec2 - &vec1), &vec2);
            if score < score_tol {
                Some(FaceEdge {
                    faces: (face0_id, face1_id),
                    positions: [
                        face0[0].clone(),
                        face0[1].clone(),
                        face0[2].clone(),
                        face1[k].clone(),
                    ],
                    score,
                })
            } else {
                None
            }
        } else if coef[0] > 0.0 && coef[1] < 0.0 && coef[0] + coef[1] < 1.0 {
            let score = calc_score(&vec2, &(&vec0 - &vec2), &(&vec1 - &vec0), &vec1);
            if score < score_tol {
                Some(FaceEdge {
                    faces: (face0_id, face1_id),
                    positions: [
                        face0[0].clone(),
                        face1[k].clone(),
                        face0[1].clone(),
                        face0[2].clone(),
                    ],
                    score,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[inline(always)]
fn calc_score(edge0: &Vector3, edge1: &Vector3, edge2: &Vector3, edge3: &Vector3) -> f64 {
    edge0.cos_angle(edge1).abs()
        + edge1.cos_angle(edge2).abs()
        + edge2.cos_angle(edge3).abs()
        + edge3.cos_angle(edge0).abs()
}

#[inline(always)]
fn get_tri<T: Clone>(face: &[T], idx0: usize, idx1: usize, idx2: usize) -> [T; 3] {
    [face[idx0].clone(), face[idx1].clone(), face[idx2].clone()]
}

trait CosAngle {
    fn cos_angle(&self, other: &Self) -> f64;
}

impl CosAngle for Vector3 {
    fn cos_angle(&self, other: &Self) -> f64 {
        self.dot(*other) / (self.magnitude() * other.magnitude())
    }
}
