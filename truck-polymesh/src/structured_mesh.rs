use crate::*;
use errors::Error;
type Result<T> = std::result::Result<T, Error>;

impl StandardVertex {
    #[inline(always)]
    fn tuple(x: usize, uv: bool, nor: bool) -> StandardVertex {
        let pos = x;
        let uv = if uv { Some(x) } else { None };
        let nor = if nor { Some(x) } else { None };
        StandardVertex { pos, uv, nor }
    }
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
        check_vectors_regularity(&u_div, &v_div)?;
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

    /// Returns the matrix of all positions.
    #[inline(always)]
    pub fn positions(&self) -> &Vec<Vec<Point3>> { &self.positions }

    /// Returns the vector of the mutable references to the rows of the positions matrix.
    #[inline(always)]
    pub fn positions_mut(&mut self) -> Vec<&mut [Point3]> {
        self.positions.iter_mut().map(|arr| arr.as_mut()).collect()
    }

    /// Returns the divisions of uv coordinates.
    #[inline(always)]
    pub fn uv_division(&self) -> Option<(&Vec<f64>, &Vec<f64>)> {
        self.uv_division.as_ref().map(|tuple| (&tuple.0, &tuple.1))
    }

    /// Returns the mutable slice of uv coordinates division.
    #[inline(always)]
    pub fn uv_division_mut(&mut self) -> Option<(&mut [f64], &mut [f64])> {
        self.uv_division
            .as_mut()
            .map(|tuple| (tuple.0.as_mut(), tuple.1.as_mut()))
    }

    /// Returns the matrix of all normals.
    #[inline(always)]
    pub fn normals(&self) -> Option<&Vec<Vec<Vector3>>> { self.normals.as_ref() }

    /// Returns the vector of the mutable references to the rows of the normals matrix.
    #[inline(always)]
    pub fn normals_mut(&mut self) -> Option<Vec<&mut [Vector3]>> {
        self.normals
            .as_mut()
            .map(|normals| normals.iter_mut().map(|arr| arr.as_mut()).collect())
    }

    /// Creates new polygon mesh by destructing `self`.
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
        let uv_coords = uv_division
            .map(move |(udiv, vdiv)| {
                udiv.into_iter()
                    .flat_map(|u| vdiv.iter().map(move |v| Vector2::new(u, *v)))
                    .collect()
            })
            .unwrap_or_else(Vec::new);
        let normals = normals
            .map(|n| n.into_iter().flatten().collect())
            .unwrap_or_else(Vec::new);
        let uv = !uv_coords.is_empty();
        let nor = !normals.is_empty();
        let quad_faces: Vec<_> = (1..m)
            .flat_map(|i| (1..n).map(move |j| (i, j)))
            .map(move |(i, j)| {
                [
                    StandardVertex::tuple((i - 1) * n + j - 1, uv, nor),
                    StandardVertex::tuple(i * n + j - 1, uv, nor),
                    StandardVertex::tuple(i * n + j, uv, nor),
                    StandardVertex::tuple((i - 1) * n + j, uv, nor),
                ]
            })
            .collect();
        let faces = Faces {
            quad_faces,
            ..Default::default()
        };
        PolygonMesh {
            attributes: StandardAttributes {
                positions,
                uv_coords,
                normals,
            },
            faces,
        }
    }
}

#[inline(always)]
fn check_matrix_regularity<T>(matrix: &[Vec<T>]) -> Result<()> {
    for arr in matrix {
        if arr.len() != matrix[0].len() {
            return Err(Error::IrregularArray);
        }
    }
    Ok(())
}

#[inline(always)]
fn check_matrices_compatibility<S, T>(matrix0: &[Vec<S>], matrix1: &[Vec<T>]) -> Result<()> {
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
fn check_vectors_regularity(vec0: &[f64], vec1: &[f64]) -> Result<()> {
    for i in 1..vec0.len() {
        if vec0[i - 1] > vec0[i] {
            return Err(Error::UnsortedDivision);
        }
    }
    for i in 1..vec1.len() {
        if vec1[i - 1] > vec1[i] {
            return Err(Error::UnsortedDivision);
        }
    }
    Ok(())
}

#[inline(always)]
fn check_matrix_vectors_compatibility<T>(
    matrix: &[Vec<T>],
    vec0: &[f64],
    vec1: &[f64],
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
