use crate::errors::Error;
use crate::{PolygonMesh, StructuredMesh};
use geometry::Vector3;

impl StructuredMesh {
    pub fn new(
        positions: Vec<Vec<Vector3>>,
        (u_div, v_div): (Vec<f64>, Vec<f64>),
        normals: Vec<Vec<Vector3>>,
    ) -> StructuredMesh
    {
        if positions.len() != u_div.len() || normals.len() != u_div.len() {
            panic!("{}", Error::DifferentLengthArrays);
        }
        for arr in &positions {
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
            positions: positions,
            uv_division: (u_div, v_div),
            normals: normals,
        }
    }

    pub fn new_unchecked(
        positions: Vec<Vec<Vector3>>,
        (u_div, v_div): (Vec<f64>, Vec<f64>),
        normals: Vec<Vec<Vector3>>,
    ) -> StructuredMesh
    {
        StructuredMesh {
            positions: positions,
            uv_division: (u_div, v_div),
            normals: normals,
        }
    }
    pub fn by_positions(positions: Vec<Vec<Vector3>>) -> StructuredMesh {
        for arr in &positions {
            if arr.len() != positions[0].len() {
                panic!("{}", Error::IrregularArray);
            }
        }

        StructuredMesh {
            positions,
            uv_division: (Vec::new(), Vec::new()),
            normals: Vec::new(),
        }
    }

    pub fn destruct(self) -> PolygonMesh {
        let StructuredMesh {
            positions,
            uv_division: (udiv, vdiv),
            normals,
        } = self;
        let m = positions.len();
        let n = positions[0].len();
        let positions = positions
            .into_iter()
            .flat_map(move |vec| vec.into_iter())
            .collect();
        let uv_coords = udiv
            .iter()
            .flat_map(|u| vdiv.iter().map(move |v| vector!(*u, *v)))
            .collect();
        let normals = normals
            .into_iter()
            .flat_map(move |vec| vec.into_iter())
            .collect();
        let quad_faces = (1..m)
            .flat_map(|i| (1..n).map(move |j| (i, j)))
            .map(move |(i, j)| {
                [
                    [(i - 1) * n + j - 1; 3],
                    [i * n + j - 1; 3],
                    [i * n + j; 3],
                    [(i - 1) * n + j; 3],
                ]
            })
            .collect();
        PolygonMesh {
            positions,
            uv_coords,
            normals,
            tri_faces: Vec::new(),
            quad_faces,
            other_faces: Vec::new(),
        }
    }
}
