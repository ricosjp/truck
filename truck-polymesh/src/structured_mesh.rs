use crate::errors::Error;
use crate::*;

macro_rules! create_quad_faces {
    ($m: expr, $n: expr, $size: tt) => {
        Faces {
            quad_faces: (1..$m)
                .flat_map(|i| (1..$n).map(move |j| (i, j)))
                .map(move |(i, j)| {
                    [
                        [(i - 1) * $n + j - 1; $size],
                        [i * $n + j - 1; $size],
                        [i * $n + j; $size],
                        [(i - 1) * $n + j; $size],
                    ]
                })
                .collect(),
            ..Default::default()
        }
    };
}

impl StructuredMesh {
    /// Creates a structured polygon without `uv_division` and `normal`.
    #[inline(always)]
    pub fn from_positions(positions: Vec<Vec<Point3>>) -> StructuredMesh {
        StructuredMesh::try_from_positions(positions).unwrap_or_else(|e| panic!("{:?}", e))
    }
    /// Creates a structured polygon without `uv_division` and `normal`.
    #[inline(always)]
    pub fn try_from_positions(positions: Vec<Vec<Point3>>) -> Result<StructuredMesh> {
        check_matrix_regularity(&positions)?;
        Ok(StructuredMesh::from_positions_unchecked(positions))
    }

    /// Creates a structured polygon without `uv_division` and `normal`.
    #[inline(always)]
    pub fn from_positions_unchecked(positions: Vec<Vec<Point3>>) -> StructuredMesh {
        StructuredMesh {
            positions,
            uv_division: None,
            normals: None,
        }
    }
    /// Creates a structured polygon without normals.
    #[inline(always)]
    pub fn from_positions_and_uvs(
        positions: Vec<Vec<Point3>>,
        (u_div, v_div): (Vec<f64>, Vec<f64>),
    ) -> StructuredMesh {
        StructuredMesh::try_from_positions_and_uvs(positions, (u_div, v_div))
            .unwrap_or_else(|e| panic!("{:?}", e))
    }

    /// Creates a structured polygon without normals.
    #[inline(always)]
    pub fn try_from_positions_and_uvs(
        positions: Vec<Vec<Point3>>,
        (u_div, v_div): (Vec<f64>, Vec<f64>),
    ) -> Result<StructuredMesh> {
        check_matrix_vectors_compatibility(&positions, &u_div, &v_div)?;
        Ok(StructuredMesh::from_positions_and_uvs_unchecked(
            positions,
            (u_div, v_div),
        ))
    }
    /// Creates a structured polygon without normals.
    #[inline(always)]
    pub fn from_positions_and_uvs_unchecked(
        positions: Vec<Vec<Point3>>,
        uv_divisions: (Vec<f64>, Vec<f64>),
    ) -> StructuredMesh {
        StructuredMesh {
            positions,
            uv_division: Some(uv_divisions),
            normals: None,
        }
    }
    /// Creates a structured polygon without uv divisions.
    #[inline(always)]
    pub fn from_positions_and_normals(
        positions: Vec<Vec<Point3>>,
        normals: Vec<Vec<Vector3>>,
    ) -> StructuredMesh {
        StructuredMesh::try_from_positions_and_normals(positions, normals)
            .unwrap_or_else(|e| panic!("{:?}", e))
    }
    /// Creates a structured polygon without uv divisions.
    #[inline(always)]
    pub fn try_from_positions_and_normals(
        positions: Vec<Vec<Point3>>,
        normals: Vec<Vec<Vector3>>,
    ) -> Result<StructuredMesh> {
        check_matrix_regularity(&positions)?;
        check_matrices_compatibility(&positions, &normals)?;
        Ok(StructuredMesh::from_positions_and_normals_unchecked(
            positions, normals,
        ))
    }
    /// Creates a structured polygon without uv divisions.
    #[inline(always)]
    pub fn from_positions_and_normals_unchecked(
        positions: Vec<Vec<Point3>>,
        normals: Vec<Vec<Vector3>>,
    ) -> StructuredMesh {
        StructuredMesh {
            positions,
            uv_division: None,
            normals: Some(normals),
        }
    }

    /// Creates new structured mesh.
    /// Checks whether the size of vectors are compatible before creation.
    #[inline(always)]
    pub fn new(
        positions: Vec<Vec<Point3>>,
        uv_division: (Vec<f64>, Vec<f64>),
        normals: Vec<Vec<Vector3>>,
    ) -> StructuredMesh {
        StructuredMesh::try_new(positions, uv_division, normals)
            .unwrap_or_else(|e| panic!("{:?}", e))
    }
    /// Creates new structured mesh.
    /// Checks whether the size of vectors are compatible before creation.
    #[inline(always)]
    pub fn try_new(
        positions: Vec<Vec<Point3>>,
        (u_div, v_div): (Vec<f64>, Vec<f64>),
        normals: Vec<Vec<Vector3>>,
    ) -> Result<StructuredMesh> {
        check_matrix_vectors_compatibility(&positions, &u_div, &v_div)?;
        check_matrix_vectors_compatibility(&normals, &u_div, &v_div)?;
        check_vectors_regularity(&u_div, &v_div)?;
        Ok(StructuredMesh::new_unchecked(
            positions,
            (u_div, v_div),
            normals,
        ))
    }

    /// Creates new structured mesh.
    /// Does not check whether the size of vectors are compatible before creation.
    #[inline(always)]
    pub fn new_unchecked(
        positions: Vec<Vec<Point3>>,
        uv_division: (Vec<f64>, Vec<f64>),
        normals: Vec<Vec<Vector3>>,
    ) -> StructuredMesh {
        StructuredMesh {
            positions,
            uv_division: Some(uv_division),
            normals: Some(normals),
        }
    }
    /// Creates new polygon by destructing `self`.
    #[inline(always)]
    pub fn destruct(self) -> PolygonMesh {
        let StructuredMesh {
            positions,
            uv_division,
            normals,
        } = self;
        let m = positions.len();
        let n = positions[0].len();
        let positions = positions.into_iter().flatten().collect();
        let uv_coords = uv_division.map(move |(udiv, vdiv)| {
            udiv.into_iter()
                .flat_map(|u| vdiv.iter().map(move |v| Vector2::new(u, *v)))
                .collect()
        });
        let normals = normals.map(|n| n.into_iter().flatten().collect());
        match (uv_coords, normals) {
            (None, None) => {
                let faces = create_quad_faces!(m, n, 1);
                PolygonMesh::Positions { positions, faces }
            }
            (Some(uv_coords), None) => PolygonMesh::Textured {
                positions,
                uv_coords,
                faces: create_quad_faces!(m, n, 2),
            },
            (None, Some(normals)) => PolygonMesh::WithNormals {
                positions,
                normals,
                faces: create_quad_faces!(m, n, 2),
            },
            (Some(uv_coords), Some(normals)) => PolygonMesh::Complete {
                positions,
                uv_coords,
                normals,
                faces: create_quad_faces!(m, n, 3),
            },
        }
    }
}

#[inline(always)]
fn check_matrix_regularity<T>(matrix: &Vec<Vec<T>>) -> Result<()> {
    for arr in matrix {
        if arr.len() != matrix[0].len() {
            return Err(Error::IrregularArray);
        }
    }
    Ok(())
}

#[inline(always)]
fn check_matrices_compatibility<S, T>(matrix0: &Vec<Vec<S>>, matrix1: &Vec<Vec<T>>) -> Result<()> {
    if matrix0.len() != matrix1.len() {
        return Err(Error::DifferentLengthArrays);
    }
    for arr in matrix0 {
        if arr.len() != matrix1[0].len() {
            return Err(Error::DifferentLengthArrays);
        }
    }
    Ok(())
}

#[inline(always)]
fn check_vectors_regularity(vec0: &Vec<f64>, vec1: &Vec<f64>) -> Result<()> {
    for i in 1..vec0.len() {
        if vec0[i - 1] > vec0[i] {
            panic!("{}", Error::UnsortedDivision);
        }
    }
    for i in 1..vec1.len() {
        if vec1[i - 1] > vec1[i] {
            panic!("{}", Error::UnsortedDivision);
        }
    }
    Ok(())
}

#[inline(always)]
fn check_matrix_vectors_compatibility<T>(
    matrix: &Vec<Vec<T>>,
    vec0: &Vec<f64>,
    vec1: &Vec<f64>,
) -> Result<()> {
    if matrix.len() != vec0.len() {
        return Err(Error::DifferentLengthArrays);
    }
    for arr in matrix {
        if arr.len() != vec1.len() {
            return Err(Error::DifferentLengthArrays);
        }
    }
    Ok(())
}
