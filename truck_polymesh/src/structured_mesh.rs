use crate::errors::Error;
use crate::{PolygonMesh, StructuredMesh};

impl StructuredMesh {
    pub fn new(
        points: Vec<Vec<[f64; 3]>>,
        (u_div, v_div): (Vec<f64>, Vec<f64>),
        normals: Vec<Vec<[f64; 3]>>,
    ) -> StructuredMesh
    {
        if points.len() != u_div.len() || normals.len() != u_div.len() {
            panic!("{}", Error::DifferentLengthArrays);
        }
        for arr in &points {
            if arr.len() != v_div.len() {
                panic!("{}", Error::IrregularArray);
            }
        }
        for arr in &normals {
            if arr.len() != v_div.len() {
                panic!("{}", Error::IrregularArray);
            }
        }
        for i in 1..u_div.len() {
            if u_div[i - 1] > u_div[i] {
                panic!("{}", Error::UnsortedDivision);
            }
        }
        for i in 1..v_div.len() {
            if v_div[i - 1] > v_div[i] {
                panic!("{}", Error::UnsortedDivision);
            }
        }
        StructuredMesh {
            points: points,
            uv_division: (u_div, v_div),
            normals: normals,
        }
    }

    pub fn new_unchecked(
        points: Vec<Vec<[f64; 3]>>,
        (u_div, v_div): (Vec<f64>, Vec<f64>),
        normals: Vec<Vec<[f64; 3]>>,
    ) -> StructuredMesh
    {
        StructuredMesh {
            points: points,
            uv_division: (u_div, v_div),
            normals: normals,
        }
    }
    pub fn by_points(points: Vec<Vec<[f64; 3]>>) -> StructuredMesh {
        for arr in &points {
            if arr.len() != points[0].len() {
                panic!("{}", Error::IrregularArray);
            }
        }

        StructuredMesh {
            points: points,
            uv_division: (Vec::new(), Vec::new()),
            normals: Vec::new(),
        }
    }

    pub fn destruct(self) -> PolygonMesh {
        let mut mesh = PolygonMesh::default();
        let m = self.points.len();
        let n = self.points[0].len();
        mesh.vertices = self
            .points
            .iter()
            .flat_map(|vec| vec.iter())
            .map(|x| x.clone())
            .collect();
        mesh.uv_coords = self.uv_division.0.iter()
            .flat_map(|u| {
                self.uv_division.1.iter().map(move |v| {
                    [*u, *v]
                })
            })
            .collect();
        mesh.normals = self
            .normals
            .into_iter()
            .flat_map(|vec| vec.into_iter())
            .collect();
        for i in 1..m {
            for j in 1..n {
                let face = [
                    [(i - 1) * n + j - 1; 3],
                    [i * n + j - 1; 3],
                    [i * n + j; 3],
                    [(i - 1) * n + j; 3],
                ];
                mesh.quad_faces.push(face);
            }
        }
        mesh
    }
}
