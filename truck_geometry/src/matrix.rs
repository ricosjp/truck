use crate::{Tolerance, Origin, Vector, Matrix, Result};
use crate::errors::Error;

impl Matrix {
    /// construct a matrix by rows
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v1 = Vector::new(5.0, 6.0, 7.0, 8.0);
    /// let v2 = Vector::new(-1.0, -2.0, -3.0, -4.0);
    /// let v3 = Vector::new(-5.0, -6.0, -7.0, -8.0);
    /// let mat1 = Matrix::by_rows(v0, v1, v2, v3);
    /// assert_eq!(mat0, mat1);
    /// ```
    #[inline(always)]
    pub fn new(ar0: [f64; 4], ar1: [f64; 4], ar2: [f64; 4], ar3: [f64; 4]) -> Matrix {
        Matrix {
            a: [ar0, ar1, ar2, ar3],
        }
    }

    /// construct a matrix by rows
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v1 = Vector::new(5.0, 6.0, 7.0, 8.0);
    /// let v2 = Vector::new(-1.0, -2.0, -3.0, -4.0);
    /// let v3 = Vector::new(-5.0, -6.0, -7.0, -8.0);
    /// let mat1 = Matrix::by_rows(v0, v1, v2, v3);
    /// assert_eq!(mat0, mat1);
    /// ```
    #[inline(always)]
    pub fn by_rows(v0: Vector, v1: Vector, v2: Vector, v3: Vector) -> Matrix {
        Matrix::new(v0.into(), v1.into(), v2.into(), v3.into())
    }

    /// construct a matrix by rows
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// use truck_geometry::Matrix;
    /// let v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v1 = Vector::new(5.0, 6.0, 7.0, 8.0);
    /// let v2 = Vector::new(-1.0, -2.0, -3.0, -4.0);
    /// let v3 = Vector::new(-5.0, -6.0, -7.0, -8.0);
    /// let mat0 = Matrix::by_rows_ref(&v0, &v1, &v2, &v3);
    /// let mat1 = Matrix::by_rows(v0, v1, v2, v3);
    /// assert_eq!(mat0, mat1);
    /// ```
    #[inline(always)]
    pub fn by_rows_ref(v0: &Vector, v1: &Vector, v2: &Vector, v3: &Vector) -> Matrix {
        Matrix::new(
            v0.clone().into(),
            v1.clone().into(),
            v2.clone().into(),
            v3.clone().into(),
        )
    }

    /// construct a matrix by columns
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// use truck_geometry::Matrix;
    /// let v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v1 = Vector::new(5.0, 6.0, 7.0, 8.0);
    /// let v2 = Vector::new(-1.0, -2.0, -3.0, -4.0);
    /// let v3 = Vector::new(-5.0, -6.0, -7.0, -8.0);
    /// let mat0 = Matrix::by_rows_ref(&v0, &v1, &v2, &v3);
    /// let v0 = Vector::new(1.0, 5.0, -1.0, -5.0);
    /// let v1 = Vector::new(2.0, 6.0, -2.0, -6.0);
    /// let v2 = Vector::new(3.0, 7.0, -3.0, -7.0);
    /// let v3 = Vector::new(4.0, 8.0, -4.0, -8.0);
    /// let mat1 = Matrix::by_columns(&v0, &v1, &v2, &v3);
    /// assert_eq!(mat0, mat1);
    /// ```
    #[inline(always)]
    pub fn by_columns(v0: &Vector, v1: &Vector, v2: &Vector, v3: &Vector) -> Matrix {
        Matrix {
            a: [
                [v0[0], v1[0], v2[0], v3[0]],
                [v0[1], v1[1], v2[1], v3[1]],
                [v0[2], v1[2], v2[2], v3[2]],
                [v0[3], v1[3], v2[3], v3[3]],
            ],
        }
    }

    /// construct the zero matrix
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let zero = Matrix::zero();
    /// assert_eq!(&mat * &zero, zero);
    /// ```
    #[inline(always)]
    pub const fn zero() -> Matrix {
        Matrix {
            a: [
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
            ],
        }
    }

    /// construct the identity matrix
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let id = Matrix::identity();
    /// assert_eq!(&mat * id, mat);
    /// ```
    #[inline(always)]
    pub const fn identity() -> Matrix {
        Matrix {
            a: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// construct a diagonal matrix
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// use truck_geometry::Matrix;
    /// let v0 = Vector::new(1.0, 1.0, 1.0, 1.0);
    /// let mat = Matrix::diagonal(2.0, 3.0, 4.0, 5.0);
    /// let v = mat * v0;
    /// assert_eq!(v, Vector::new(2.0, 3.0, 4.0, 5.0));
    /// ```
    #[inline(always)]
    pub fn diagonal(a: f64, b: f64, c: f64, d: f64) -> Matrix {
        Matrix {
            a: [
                [a, 0.0, 0.0, 0.0],
                [0.0, b, 0.0, 0.0],
                [0.0, 0.0, c, 0.0],
                [0.0, 0.0, 0.0, d],
            ],
        }
    }
    
    /// extract a row vector
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// assert_eq!(mat.row(1), Vector::new(5.0, 6.0, 7.0, 8.0));
    /// ```
    #[inline(always)]
    pub fn row(&self, idx: usize) -> Vector { Vector::from(&self[idx]) }

    /// extract a column vector
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// assert_eq!(mat.column(0), Vector::new(1.0, 5.0, -1.0, -5.0));
    /// ```
    #[inline(always)]
    pub fn column(&self, idx: usize) -> Vector {
        Vector::new(self[0][idx], self[1][idx], self[2][idx], self[3][idx])
    }

    /// transpose
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// use truck_geometry::Matrix;
    /// let v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v1 = Vector::new(5.0, 6.0, 7.0, 8.0);
    /// let v2 = Vector::new(-1.0, -2.0, -3.0, -4.0);
    /// let v3 = Vector::new(-5.0, -6.0, -7.0, -8.0);
    /// let mat0 = Matrix::by_rows_ref(&v0, &v1, &v2, &v3);
    /// let mat1 = Matrix::by_columns(&v0, &v1, &v2, &v3);
    /// assert_eq!(mat0.transpose(), mat1);
    /// ```
    #[inline(always)]
    pub fn transpose(&self) -> Matrix {
        Matrix::by_rows(
            self.column(0),
            self.column(1),
            self.column(2),
            self.column(3),
        )
    }
    
    /// calculate trace
    /// #Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// assert_eq!(mat.trace(), -4.0);
    /// ```
    #[inline(always)]
    pub fn trace(&self) -> f64 { self[0][0] + self[1][1] + self[2][2] + self[3][3] }

    /// calculate determinant
    /// #Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, -3.0, 3.0, 2.0];
    /// let ar1 = [4.0, 3.0, -2.0, 1.0];
    /// let ar2 = [5.0, 2.0, 1.0, 3.0];
    /// let ar3 = [5.0, 6.0, 1.0, 2.0];
    /// let mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// assert_eq!(mat.determinant(), -28.0);
    /// ```
    #[inline(always)]
    pub fn determinant(&self) -> f64 {
        self[0][0] * self[1][1] * self[2][2] * self[3][3]
            + self[0][0] * self[2][1] * self[3][2] * self[1][3]
            + self[0][0] * self[3][1] * self[1][2] * self[2][3]
            - self[0][0] * self[3][1] * self[2][2] * self[1][3]
            - self[0][0] * self[2][1] * self[1][2] * self[3][3]
            - self[0][0] * self[1][1] * self[3][2] * self[2][3]
            - self[1][0] * self[0][1] * self[2][2] * self[3][3]
            - self[1][0] * self[2][1] * self[3][2] * self[0][3]
            - self[1][0] * self[3][1] * self[0][2] * self[2][3]
            + self[1][0] * self[3][1] * self[2][2] * self[0][3]
            + self[1][0] * self[2][1] * self[0][2] * self[3][3]
            + self[1][0] * self[0][1] * self[3][2] * self[2][3]
            + self[2][0] * self[0][1] * self[1][2] * self[3][3]
            + self[2][0] * self[1][1] * self[3][2] * self[0][3]
            + self[2][0] * self[3][1] * self[0][2] * self[1][3]
            - self[2][0] * self[3][1] * self[1][2] * self[0][3]
            - self[2][0] * self[1][1] * self[0][2] * self[3][3]
            - self[2][0] * self[0][1] * self[3][2] * self[1][3]
            - self[3][0] * self[0][1] * self[1][2] * self[2][3]
            - self[3][0] * self[1][1] * self[2][2] * self[0][3]
            - self[3][0] * self[2][1] * self[0][2] * self[1][3]
            + self[3][0] * self[2][1] * self[1][2] * self[0][3]
            + self[3][0] * self[1][1] * self[0][2] * self[2][3]
            + self[3][0] * self[0][1] * self[2][2] * self[1][3]
    }

    /// calculate `x: Vector` such that `x * self = b`
    pub fn solve(&self, b: &Vector) -> Result<Vector> {
        let det = self.determinant();
        if det.so_small() {
            return Err(Error::IrregularMatrix(self.clone()));
        }

        let vec0 = self.row(0);
        let vec1 = self.row(1);
        let vec2 = self.row(2);
        let vec3 = self.row(3);
        let det0 = Matrix::by_rows_ref(&b, &vec1, &vec2, &vec3).determinant() / det;
        let det1 = Matrix::by_rows_ref(&vec0, &b, &vec2, &vec3).determinant() / det;
        let det2 = Matrix::by_rows_ref(&vec0, &vec1, &b, &vec3).determinant() / det;
        let det3 = Matrix::by_rows_ref(&vec0, &vec1, &vec2, &b).determinant() / det;

        Ok(Vector::new(det0, det1, det2, det3))
    }

    /// calculate inverse  
    /// If `self` is not invartible, return `Err()`.
    /// #Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// use truck_geometry::Tolerance;
    /// let ar0 = [1.0, -3.0, 3.0, 2.0];
    /// let ar1 = [4.0, 3.0, -2.0, 1.0];
    /// let ar2 = [5.0, 2.0, 1.0, 3.0];
    /// let ar3 = [5.0, 6.0, 1.0, 2.0];
    /// let mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let mat_inv = mat.inverse().unwrap();
    /// Matrix::assert_near2(&(mat * mat_inv), &Matrix::identity());
    /// ```
    #[inline(always)]
    pub fn inverse(&self) -> Result<Matrix> {
        let det = self.determinant();
        if det.so_small() {
            return Err(Error::IrregularMatrix(self.clone()));
        }

        let vec0 = self.row(0);
        let vec1 = self.row(1);
        let vec2 = self.row(2);
        let vec3 = self.row(3);
        let e0 = Vector::new(1.0, 0.0, 0.0, 0.0);
        let e1 = Vector::new(0.0, 1.0, 0.0, 0.0);
        let e2 = Vector::new(0.0, 0.0, 1.0, 0.0);
        let e3 = Vector::new(0.0, 0.0, 0.0, 1.0);
        let mat00 = Matrix::by_rows_ref(&e0, &vec1, &vec2, &vec3);
        let mat01 = Matrix::by_rows_ref(&vec0, &e0, &vec2, &vec3);
        let mat02 = Matrix::by_rows_ref(&vec0, &vec1, &e0, &vec3);
        let mat03 = Matrix::by_rows_ref(&vec0, &vec1, &vec2, &e0);
        let mat10 = Matrix::by_rows_ref(&e1, &vec1, &vec2, &vec3);
        let mat11 = Matrix::by_rows_ref(&vec0, &e1, &vec2, &vec3);
        let mat12 = Matrix::by_rows_ref(&vec0, &vec1, &e1, &vec3);
        let mat13 = Matrix::by_rows_ref(&vec0, &vec1, &vec2, &e1);
        let mat20 = Matrix::by_rows_ref(&e2, &vec1, &vec2, &vec3);
        let mat21 = Matrix::by_rows_ref(&vec0, &e2, &vec2, &vec3);
        let mat22 = Matrix::by_rows_ref(&vec0, &vec1, &e2, &vec3);
        let mat23 = Matrix::by_rows_ref(&vec0, &vec1, &vec2, &e2);
        let mat30 = Matrix::by_rows_ref(&e3, &vec1, &vec2, &vec3);
        let mat31 = Matrix::by_rows_ref(&vec0, &e3, &vec2, &vec3);
        let mat32 = Matrix::by_rows_ref(&vec0, &vec1, &e3, &vec3);
        let mat33 = Matrix::by_rows_ref(&vec0, &vec1, &vec2, &e3);
        Ok(Matrix::new(
            [
                mat00.determinant(),
                mat01.determinant(),
                mat02.determinant(),
                mat03.determinant(),
            ],
            [
                mat10.determinant(),
                mat11.determinant(),
                mat12.determinant(),
                mat13.determinant(),
            ],
            [
                mat20.determinant(),
                mat21.determinant(),
                mat22.determinant(),
                mat23.determinant(),
            ],
            [
                mat30.determinant(),
                mat31.determinant(),
                mat32.determinant(),
                mat33.determinant(),
            ],
        ) / det)
    }
    
    /// square of norm
    /// #Examples
    /// ```
    /// // mat.norm2() coincides with (mat * mat.transpose()).trace()
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, -3.0, 3.0, 2.0];
    /// let ar1 = [4.0, 3.0, -2.0, 1.0];
    /// let ar2 = [5.0, 2.0, 1.0, 3.0];
    /// let ar3 = [5.0, 6.0, 1.0, 2.0];
    /// let mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let mat0 = &mat * mat.transpose();
    /// assert_eq!(mat0.trace(), mat.norm2());
    /// ```
    #[inline(always)]
    pub fn norm2(&self) -> f64 {
        let mut norm2 = 0.0;
        for arr in &self.a {
            norm2 += arr.iter().fold(0.0, |sum, d| sum + d * d);
        }
        norm2
    }

    /// norm
    #[inline(always)]
    pub fn norm(&self) -> f64 { self.norm2().sqrt() }

    /// Iwasawa decomposition. `(K,A,N)`
    /// * `K`: an orthogonal matrix
    /// * `A`: a diagonal matrix
    /// * `N`: a unipotent upper triangle matrix
    /// #Examples 
    /// ```
    /// use truck_geometry::Matrix;
    /// use truck_geometry::Tolerance;
    /// let ar0 = [1.0, -3.0, 3.0, 2.0];
    /// let ar1 = [4.0, 3.0, -2.0, 1.0];
    /// let ar2 = [5.0, 2.0, 1.0, 3.0];
    /// let ar3 = [5.0, 6.0, 1.0, 2.0];
    /// let mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let (k, a, n) = mat.iwasawa_decomposition().unwrap();
    ///
    /// // k is an orthogonal matrix
    /// let trans = k.transpose();
    /// let inv = k.inverse().unwrap();
    /// println!("{:?} {:?}", trans, inv);
    /// Matrix::assert_near2(&trans, &inv);
    ///
    /// // a is a diagonal matrix
    /// for i in 0..4 {
    ///     for j in 0..4 {
    ///        if i != j { assert_eq!(a[i][j], 0.0); }
    ///     }
    /// }
    /// 
    /// // n is a unipotent upper triangle matrix
    /// for i in 0..4 {
    ///     for j in 0..i {
    ///         assert_eq!(n[i][j], 0.0);
    ///     }
    /// }
    ///
    /// // k * a * n coinsides with mat
    /// Matrix::assert_near2(&(k * a * n), &mat);
    /// ```
    pub fn iwasawa_decomposition(&self) -> Result<(Matrix, Matrix, Matrix)> {
        let v0 = self.column(0);
        let v1 = self.column(1);
        let v2 = self.column(2);
        let v3 = self.column(3);

        let mut n = Matrix::identity();
        let u0 = v0.clone();
        let a0 = u0.norm();
        if a0.so_small() {
            return Err(Error::IrregularMatrix(self.clone()));
        }
        n[0][1] = (&v1 * &u0) / (a0 * a0);
        let u1 = &v1 - n[0][1] * &u0;
        let a1 = u1.norm();
        if a1.so_small() {
            return Err(Error::IrregularMatrix(self.clone()));
        }
        n[0][2] = (&v2 * &u0) / (a0 * a0);
        n[1][2] = (&v2 * &u1) / (a1 * a1);
        let u2 = &v2 - n[0][2] * &u0 - n[1][2] * &u1;
        let a2 = u2.norm();
        if a2.so_small() {
            return Err(Error::IrregularMatrix(self.clone()));
        }
        n[0][3] = (&v3 * &u0) / (a0 * a0);
        n[1][3] = (&v3 * &u1) / (a1 * a1);
        n[2][3] = (&v3 * &u2) / (a2 * a2);
        let u3 = &v3 - n[0][3] * &u0 - n[1][3] * &u1 - n[2][3] * &u2;
        let a3 = u3.norm();
        if a3.so_small() {
            return Err(Error::IrregularMatrix(self.clone()));
        }

        let k = Matrix::by_columns(&(u0 / a0), &(u1 / a1), &(u2 / a2), &(u3 / a3));
        let a = Matrix::diagonal(a0, a1, a2, a3);

        Ok((k, a, n))
    }
}

impl Tolerance for Matrix {
    #[inline(always)]
    fn near(&self, other: &Matrix) -> bool {
        self[0][0].near(&other[0][0])
            && self[0][1].near(&other[0][1])
            && self[0][2].near(&other[0][2])
            && self[0][3].near(&other[0][3])
            && self[1][0].near(&other[1][0])
            && self[1][1].near(&other[1][1])
            && self[1][2].near(&other[1][2])
            && self[1][3].near(&other[1][3])
            && self[2][0].near(&other[2][0])
            && self[2][1].near(&other[2][1])
            && self[2][2].near(&other[2][2])
            && self[2][3].near(&other[2][3])
            && self[3][0].near(&other[3][0])
            && self[3][1].near(&other[3][1])
            && self[3][2].near(&other[3][2])
            && self[3][3].near(&other[3][3])
    }

    #[inline(always)]
    fn near2(&self, other: &Matrix) -> bool {
        self[0][0].near2(&other[0][0])
            && self[0][1].near2(&other[0][1])
            && self[0][2].near2(&other[0][2])
            && self[0][3].near2(&other[0][3])
            && self[1][0].near2(&other[1][0])
            && self[1][1].near2(&other[1][1])
            && self[1][2].near2(&other[1][2])
            && self[1][3].near2(&other[1][3])
            && self[2][0].near2(&other[2][0])
            && self[2][1].near2(&other[2][1])
            && self[2][2].near2(&other[2][2])
            && self[2][3].near2(&other[2][3])
            && self[3][0].near2(&other[3][0])
            && self[3][1].near2(&other[3][1])
            && self[3][2].near2(&other[3][2])
            && self[3][3].near2(&other[3][3])
    }
}

impl Origin for Matrix {
    const ORIGIN : Matrix = Matrix::zero();
}

impl std::ops::Index<usize> for Matrix {
    type Output = [f64; 4];

    /// access each row
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// assert_eq!(mat[1], [5.0, 6.0, 7.0, 8.0]);
    /// ```
    #[inline(always)]
    fn index(&self, idx: usize) -> &[f64; 4] { &self.a[idx] }
}

impl std::ops::IndexMut<usize> for Matrix {
    /// access each row
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mut mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// mat[1][2] = 3.0;
    /// assert_eq!(mat[1], [5.0, 6.0, 3.0, 8.0]);
    /// ```
    #[inline(always)]
    fn index_mut(&mut self, idx: usize) -> &mut [f64; 4] { &mut self.a[idx] }
}

impl std::ops::AddAssign<&Matrix> for Matrix {
    /// add_assign for each components
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mut mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [9.0, 10.0, 11.0, 12.0];
    /// let ar1 = [13.0, 14.0, 15.0, 16.0];
    /// let ar2 = [-9.0, -10.0, -11.0, -12.0];
    /// let ar3 = [-13.0, -14.0, -15.0, -16.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [10.0, 12.0, 14.0, 16.0];
    /// let ar1 = [18.0, 20.0, 22.0, 24.0];
    /// let ar2 = [-10.0, -12.0, -14.0, -16.0];
    /// let ar3 = [-18.0, -20.0, -22.0, -24.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// mat += &mat0;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn add_assign(&mut self, rhs: &Matrix) {
        self[0][0] += rhs[0][0];
        self[0][1] += rhs[0][1];
        self[0][2] += rhs[0][2];
        self[0][3] += rhs[0][3];
        self[1][0] += rhs[1][0];
        self[1][1] += rhs[1][1];
        self[1][2] += rhs[1][2];
        self[1][3] += rhs[1][3];
        self[2][0] += rhs[2][0];
        self[2][1] += rhs[2][1];
        self[2][2] += rhs[2][2];
        self[2][3] += rhs[2][3];
        self[3][0] += rhs[3][0];
        self[3][1] += rhs[3][1];
        self[3][2] += rhs[3][2];
        self[3][3] += rhs[3][3];
    }
}

impl std::ops::AddAssign<Matrix> for Matrix {
    /// add_assgin for each components
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mut mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [9.0, 10.0, 11.0, 12.0];
    /// let ar1 = [13.0, 14.0, 15.0, 16.0];
    /// let ar2 = [-9.0, -10.0, -11.0, -12.0];
    /// let ar3 = [-13.0, -14.0, -15.0, -16.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [10.0, 12.0, 14.0, 16.0];
    /// let ar1 = [18.0, 20.0, 22.0, 24.0];
    /// let ar2 = [-10.0, -12.0, -14.0, -16.0];
    /// let ar3 = [-18.0, -20.0, -22.0, -24.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// mat += mat0;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn add_assign(&mut self, rhs: Matrix) { self.add_assign(&rhs); }
}

impl std::ops::Add<&Matrix> for &Matrix {
    type Output = Matrix;

    /// add each components
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [9.0, 10.0, 11.0, 12.0];
    /// let ar1 = [13.0, 14.0, 15.0, 16.0];
    /// let ar2 = [-9.0, -10.0, -11.0, -12.0];
    /// let ar3 = [-13.0, -14.0, -15.0, -16.0];
    /// let mat1 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [10.0, 12.0, 14.0, 16.0];
    /// let ar1 = [18.0, 20.0, 22.0, 24.0];
    /// let ar2 = [-10.0, -12.0, -14.0, -16.0];
    /// let ar3 = [-18.0, -20.0, -22.0, -24.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let mat = &mat0 + &mat1;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn add(self, other: &Matrix) -> Matrix {
        let mut res = self.clone();
        res += other;
        res
    }
}

impl std::ops::Add<Matrix> for &Matrix {
    type Output = Matrix;

    /// add each components
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [9.0, 10.0, 11.0, 12.0];
    /// let ar1 = [13.0, 14.0, 15.0, 16.0];
    /// let ar2 = [-9.0, -10.0, -11.0, -12.0];
    /// let ar3 = [-13.0, -14.0, -15.0, -16.0];
    /// let mat1 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [10.0, 12.0, 14.0, 16.0];
    /// let ar1 = [18.0, 20.0, 22.0, 24.0];
    /// let ar2 = [-10.0, -12.0, -14.0, -16.0];
    /// let ar3 = [-18.0, -20.0, -22.0, -24.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let mat = &mat0 + mat1;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn add(self, other: Matrix) -> Matrix { self + &other }
}

impl std::ops::Add<&Matrix> for Matrix {
    type Output = Matrix;

    /// add each components
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [9.0, 10.0, 11.0, 12.0];
    /// let ar1 = [13.0, 14.0, 15.0, 16.0];
    /// let ar2 = [-9.0, -10.0, -11.0, -12.0];
    /// let ar3 = [-13.0, -14.0, -15.0, -16.0];
    /// let mat1 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [10.0, 12.0, 14.0, 16.0];
    /// let ar1 = [18.0, 20.0, 22.0, 24.0];
    /// let ar2 = [-10.0, -12.0, -14.0, -16.0];
    /// let ar3 = [-18.0, -20.0, -22.0, -24.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let mat = mat0 + &mat1;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn add(self, other: &Matrix) -> Matrix { &self + other }
}

impl std::ops::Add<Matrix> for Matrix {
    type Output = Matrix;

    /// add each components
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [9.0, 10.0, 11.0, 12.0];
    /// let ar1 = [13.0, 14.0, 15.0, 16.0];
    /// let ar2 = [-9.0, -10.0, -11.0, -12.0];
    /// let ar3 = [-13.0, -14.0, -15.0, -16.0];
    /// let mat1 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [10.0, 12.0, 14.0, 16.0];
    /// let ar1 = [18.0, 20.0, 22.0, 24.0];
    /// let ar2 = [-10.0, -12.0, -14.0, -16.0];
    /// let ar3 = [-18.0, -20.0, -22.0, -24.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let mat = mat0 + mat1;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn add(self, other: Matrix) -> Matrix { &self + &other }
}

impl std::ops::SubAssign<&Matrix> for Matrix {
    /// add_assign for each components
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [16.0, 15.0, 14.0, 13.0];
    /// let ar1 = [12.0, 11.0, 10.0, 9.0];
    /// let ar2 = [-16.0, -15.0, -14.0, -13.0];
    /// let ar3 = [-12.0, -11.0, -10.0, -9.0];
    /// let mut mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [15.0, 13.0, 11.0, 9.0];
    /// let ar1 = [7.0, 5.0, 3.0, 1.0];
    /// let ar2 = [-15.0, -13.0, -11.0, -9.0];
    /// let ar3 = [-7.0, -5.0, -3.0, -1.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// mat -= &mat0;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn sub_assign(&mut self, rhs: &Matrix) {
        self[0][0] -= rhs[0][0];
        self[0][1] -= rhs[0][1];
        self[0][2] -= rhs[0][2];
        self[0][3] -= rhs[0][3];
        self[1][0] -= rhs[1][0];
        self[1][1] -= rhs[1][1];
        self[1][2] -= rhs[1][2];
        self[1][3] -= rhs[1][3];
        self[2][0] -= rhs[2][0];
        self[2][1] -= rhs[2][1];
        self[2][2] -= rhs[2][2];
        self[2][3] -= rhs[2][3];
        self[3][0] -= rhs[3][0];
        self[3][1] -= rhs[3][1];
        self[3][2] -= rhs[3][2];
        self[3][3] -= rhs[3][3];
    }
}

impl std::ops::SubAssign<Matrix> for Matrix {
    /// sub_assgin for each components
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [16.0, 15.0, 14.0, 13.0];
    /// let ar1 = [12.0, 11.0, 10.0, 9.0];
    /// let ar2 = [-16.0, -15.0, -14.0, -13.0];
    /// let ar3 = [-12.0, -11.0, -10.0, -9.0];
    /// let mut mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [15.0, 13.0, 11.0, 9.0];
    /// let ar1 = [7.0, 5.0, 3.0, 1.0];
    /// let ar2 = [-15.0, -13.0, -11.0, -9.0];
    /// let ar3 = [-7.0, -5.0, -3.0, -1.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// mat -= mat0;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Matrix) { self.sub_assign(&rhs); }
}

impl std::ops::Sub<&Matrix> for &Matrix {
    type Output = Matrix;

    /// sub each components
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [16.0, 15.0, 14.0, 13.0];
    /// let ar1 = [12.0, 11.0, 10.0, 9.0];
    /// let ar2 = [-16.0, -15.0, -14.0, -13.0];
    /// let ar3 = [-12.0, -11.0, -10.0, -9.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat1 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [15.0, 13.0, 11.0, 9.0];
    /// let ar1 = [7.0, 5.0, 3.0, 1.0];
    /// let ar2 = [-15.0, -13.0, -11.0, -9.0];
    /// let ar3 = [-7.0, -5.0, -3.0, -1.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let mat = &mat0 - &mat1;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn sub(self, other: &Matrix) -> Matrix {
        let mut res = self.clone();
        res -= other;
        res
    }
}

impl std::ops::Sub<Matrix> for &Matrix {
    type Output = Matrix;

    /// sub each components
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [16.0, 15.0, 14.0, 13.0];
    /// let ar1 = [12.0, 11.0, 10.0, 9.0];
    /// let ar2 = [-16.0, -15.0, -14.0, -13.0];
    /// let ar3 = [-12.0, -11.0, -10.0, -9.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat1 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [15.0, 13.0, 11.0, 9.0];
    /// let ar1 = [7.0, 5.0, 3.0, 1.0];
    /// let ar2 = [-15.0, -13.0, -11.0, -9.0];
    /// let ar3 = [-7.0, -5.0, -3.0, -1.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let mat = &mat0 - mat1;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn sub(self, other: Matrix) -> Matrix { self - &other }
}

impl std::ops::Sub<&Matrix> for Matrix {
    type Output = Matrix;

    /// sub each components
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [16.0, 15.0, 14.0, 13.0];
    /// let ar1 = [12.0, 11.0, 10.0, 9.0];
    /// let ar2 = [-16.0, -15.0, -14.0, -13.0];
    /// let ar3 = [-12.0, -11.0, -10.0, -9.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat1 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [15.0, 13.0, 11.0, 9.0];
    /// let ar1 = [7.0, 5.0, 3.0, 1.0];
    /// let ar2 = [-15.0, -13.0, -11.0, -9.0];
    /// let ar3 = [-7.0, -5.0, -3.0, -1.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let mat = mat0 - &mat1;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn sub(self, other: &Matrix) -> Matrix { &self - other }
}

impl std::ops::Sub<Matrix> for Matrix {
    type Output = Matrix;

    /// sub each components
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [16.0, 15.0, 14.0, 13.0];
    /// let ar1 = [12.0, 11.0, 10.0, 9.0];
    /// let ar2 = [-16.0, -15.0, -14.0, -13.0];
    /// let ar3 = [-12.0, -11.0, -10.0, -9.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat1 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [15.0, 13.0, 11.0, 9.0];
    /// let ar1 = [7.0, 5.0, 3.0, 1.0];
    /// let ar2 = [-15.0, -13.0, -11.0, -9.0];
    /// let ar3 = [-7.0, -5.0, -3.0, -1.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let mat = mat0 - mat1;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn sub(self, other: Matrix) -> Matrix { &self - &other }
}

impl std::ops::Mul<&Matrix> for &Matrix {
    type Output = Matrix;

    /// multiply matrices
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, 5.0, 2.0];
    /// let ar2 = [-6.0, -1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, -4.0, 5.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, -2.0, -2.0];
    /// let ar2 = [1.0, 1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, 4.0, -3.0];
    /// let mat1 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [2.0, 1.0, 6.0, -9.0];
    /// let ar1 = [2.0, 3.0, -14.0, -9.0];
    /// let ar2 = [-8.0, -19.0, -3.0, -28.0];
    /// let ar3 = [-2.0, -18.0, 36.0, -15.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let mat = &mat0 * &mat1;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn mul(self, other: &Matrix) -> Matrix {
        let res00 = self[0][0] * other[0][0]
            + self[0][1] * other[1][0]
            + self[0][2] * other[2][0]
            + self[0][3] * other[3][0];
        let res01 = self[0][0] * other[0][1]
            + self[0][1] * other[1][1]
            + self[0][2] * other[2][1]
            + self[0][3] * other[3][1];
        let res02 = self[0][0] * other[0][2]
            + self[0][1] * other[1][2]
            + self[0][2] * other[2][2]
            + self[0][3] * other[3][2];
        let res03 = self[0][0] * other[0][3]
            + self[0][1] * other[1][3]
            + self[0][2] * other[2][3]
            + self[0][3] * other[3][3];
        let res10 = self[1][0] * other[0][0]
            + self[1][1] * other[1][0]
            + self[1][2] * other[2][0]
            + self[1][3] * other[3][0];
        let res11 = self[1][0] * other[0][1]
            + self[1][1] * other[1][1]
            + self[1][2] * other[2][1]
            + self[1][3] * other[3][1];
        let res12 = self[1][0] * other[0][2]
            + self[1][1] * other[1][2]
            + self[1][2] * other[2][2]
            + self[1][3] * other[3][2];
        let res13 = self[1][0] * other[0][3]
            + self[1][1] * other[1][3]
            + self[1][2] * other[2][3]
            + self[1][3] * other[3][3];
        let res20 = self[2][0] * other[0][0]
            + self[2][1] * other[1][0]
            + self[2][2] * other[2][0]
            + self[2][3] * other[3][0];
        let res21 = self[2][0] * other[0][1]
            + self[2][1] * other[1][1]
            + self[2][2] * other[2][1]
            + self[2][3] * other[3][1];
        let res22 = self[2][0] * other[0][2]
            + self[2][1] * other[1][2]
            + self[2][2] * other[2][2]
            + self[2][3] * other[3][2];
        let res23 = self[2][0] * other[0][3]
            + self[2][1] * other[1][3]
            + self[2][2] * other[2][3]
            + self[2][3] * other[3][3];
        let res30 = self[3][0] * other[0][0]
            + self[3][1] * other[1][0]
            + self[3][2] * other[2][0]
            + self[3][3] * other[3][0];
        let res31 = self[3][0] * other[0][1]
            + self[3][1] * other[1][1]
            + self[3][2] * other[2][1]
            + self[3][3] * other[3][1];
        let res32 = self[3][0] * other[0][2]
            + self[3][1] * other[1][2]
            + self[3][2] * other[2][2]
            + self[3][3] * other[3][2];
        let res33 = self[3][0] * other[0][3]
            + self[3][1] * other[1][3]
            + self[3][2] * other[2][3]
            + self[3][3] * other[3][3];
        Matrix {
            a: [
                [res00, res01, res02, res03],
                [res10, res11, res12, res13],
                [res20, res21, res22, res23],
                [res30, res31, res32, res33],
            ],
        }
    }
}

impl std::ops::Mul<Matrix> for &Matrix {
    type Output = Matrix;

    /// multiply matrices
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, 5.0, 2.0];
    /// let ar2 = [-6.0, -1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, -4.0, 5.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, -2.0, -2.0];
    /// let ar2 = [1.0, 1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, 4.0, -3.0];
    /// let mat1 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [2.0, 1.0, 6.0, -9.0];
    /// let ar1 = [2.0, 3.0, -14.0, -9.0];
    /// let ar2 = [-8.0, -19.0, -3.0, -28.0];
    /// let ar3 = [-2.0, -18.0, 36.0, -15.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let mat = &mat0 * mat1;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn mul(self, other: Matrix) -> Matrix { self * &other }
}

impl std::ops::Mul<&Matrix> for Matrix {
    type Output = Matrix;

    /// multiply matrices
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, 5.0, 2.0];
    /// let ar2 = [-6.0, -1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, -4.0, 5.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, -2.0, -2.0];
    /// let ar2 = [1.0, 1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, 4.0, -3.0];
    /// let mat1 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [2.0, 1.0, 6.0, -9.0];
    /// let ar1 = [2.0, 3.0, -14.0, -9.0];
    /// let ar2 = [-8.0, -19.0, -3.0, -28.0];
    /// let ar3 = [-2.0, -18.0, 36.0, -15.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let mat = mat0 * &mat1;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn mul(self, other: &Matrix) -> Matrix { &self * other }
}

impl std::ops::Mul<Matrix> for Matrix {
    type Output = Matrix;

    /// multiply matrices
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, 5.0, 2.0];
    /// let ar2 = [-6.0, -1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, -4.0, 5.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, -2.0, -2.0];
    /// let ar2 = [1.0, 1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, 4.0, -3.0];
    /// let mat1 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [2.0, 1.0, 6.0, -9.0];
    /// let ar1 = [2.0, 3.0, -14.0, -9.0];
    /// let ar2 = [-8.0, -19.0, -3.0, -28.0];
    /// let ar3 = [-2.0, -18.0, 36.0, -15.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let mat = mat0 * mat1;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn mul(self, other: Matrix) -> Matrix { &self * &other }
}

impl std::ops::MulAssign<&Matrix> for Matrix {
    /// multiply matrices
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, 5.0, 2.0];
    /// let ar2 = [-6.0, -1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, -4.0, 5.0];
    /// let mut mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, -2.0, -2.0];
    /// let ar2 = [1.0, 1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, 4.0, -3.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [2.0, 1.0, 6.0, -9.0];
    /// let ar1 = [2.0, 3.0, -14.0, -9.0];
    /// let ar2 = [-8.0, -19.0, -3.0, -28.0];
    /// let ar3 = [-2.0, -18.0, 36.0, -15.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// mat *= &mat0;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn mul_assign(&mut self, rhs: &Matrix) { *self = &*self * rhs; }
}

impl std::ops::MulAssign<Matrix> for Matrix {
    /// multiply matrices
    /// # Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, 5.0, 2.0];
    /// let ar2 = [-6.0, -1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, -4.0, 5.0];
    /// let mut mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, -2.0, -2.0];
    /// let ar2 = [1.0, 1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, 4.0, -3.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [2.0, 1.0, 6.0, -9.0];
    /// let ar1 = [2.0, 3.0, -14.0, -9.0];
    /// let ar2 = [-8.0, -19.0, -3.0, -28.0];
    /// let ar3 = [-2.0, -18.0, 36.0, -15.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// mat *= mat0;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Matrix) { self.mul_assign(&rhs); }
}

impl std::ops::Mul<&Vector> for &Matrix {
    type Output = Vector;

    /// multiply a matrix and a column vector
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, 5.0, 2.0];
    /// let ar2 = [-6.0, -1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, -4.0, 5.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let vec0 = Vector::new(4.0, -2.0, 1.0, -3.0);
    /// let ans_vec = Vector::new(-9.0, -9.0, -28.0, -15.0);
    /// let vec = &mat0 * &vec0;
    /// assert_eq!(vec, ans_vec);
    /// ```
    #[inline(always)]
    fn mul(self, vector: &Vector) -> Vector {
        let v0 = self[0][0] * vector[0]
            + self[0][1] * vector[1]
            + self[0][2] * vector[2]
            + self[0][3] * vector[3];
        let v1 = self[1][0] * vector[0]
            + self[1][1] * vector[1]
            + self[1][2] * vector[2]
            + self[1][3] * vector[3];
        let v2 = self[2][0] * vector[0]
            + self[2][1] * vector[1]
            + self[2][2] * vector[2]
            + self[2][3] * vector[3];
        let v3 = self[3][0] * vector[0]
            + self[3][1] * vector[1]
            + self[3][2] * vector[2]
            + self[3][3] * vector[3];
        Vector::new(v0, v1, v2, v3)
    }
}

impl std::ops::Mul<Vector> for &Matrix {
    type Output = Vector;

    /// multiply a matrix and a column vector
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, 5.0, 2.0];
    /// let ar2 = [-6.0, -1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, -4.0, 5.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let vec0 = Vector::new(4.0, -2.0, 1.0, -3.0);
    /// let ans_vec = Vector::new(-9.0, -9.0, -28.0, -15.0);
    /// let vec = &mat0 * vec0;
    /// assert_eq!(vec, ans_vec);
    /// ```
    #[inline(always)]
    fn mul(self, vector: Vector) -> Vector { self * &vector }
}

impl std::ops::Mul<&Vector> for Matrix {
    type Output = Vector;

    /// multiply a matrix and a column vector
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, 5.0, 2.0];
    /// let ar2 = [-6.0, -1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, -4.0, 5.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let vec0 = Vector::new(4.0, -2.0, 1.0, -3.0);
    /// let ans_vec = Vector::new(-9.0, -9.0, -28.0, -15.0);
    /// let vec = mat0 * &vec0;
    /// assert_eq!(vec, ans_vec);
    /// ```
    #[inline(always)]
    fn mul(self, vector: &Vector) -> Vector { &self * vector }
}

impl std::ops::Mul<Vector> for Matrix {
    type Output = Vector;

    /// multiply a matrix and a column vector
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, 5.0, 2.0];
    /// let ar2 = [-6.0, -1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, -4.0, 5.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let vec0 = Vector::new(4.0, -2.0, 1.0, -3.0);
    /// let ans_vec = Vector::new(-9.0, -9.0, -28.0, -15.0);
    /// let vec = mat0 * vec0;
    /// assert_eq!(vec, ans_vec);
    /// ```
    #[inline(always)]
    fn mul(self, vector: Vector) -> Vector { &self * &vector }
}

impl std::ops::Mul<&Matrix> for &Vector {
    type Output = Vector;

    /// multiply a row vector and a matrix
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// use truck_geometry::Matrix;
    /// let vec0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, -2.0, -2.0];
    /// let ar2 = [1.0, 1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, 4.0, -3.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ans_vec = Vector::new(2.0, 1.0, 6.0, -9.0);
    /// let vec = &vec0 * &mat0;
    /// assert_eq!(vec, ans_vec);
    /// ```
    #[inline(always)]
    fn mul(self, matrix: &Matrix) -> Vector {
        let v0 = matrix[0][0] * self[0]
            + matrix[1][0] * self[1]
            + matrix[2][0] * self[2]
            + matrix[3][0] * self[3];
        let v1 = matrix[0][1] * self[0]
            + matrix[1][1] * self[1]
            + matrix[2][1] * self[2]
            + matrix[3][1] * self[3];
        let v2 = matrix[0][2] * self[0]
            + matrix[1][2] * self[1]
            + matrix[2][2] * self[2]
            + matrix[3][2] * self[3];
        let v3 = matrix[0][3] * self[0]
            + matrix[1][3] * self[1]
            + matrix[2][3] * self[2]
            + matrix[3][3] * self[3];
        Vector::new(v0, v1, v2, v3)
    }
}

impl std::ops::Mul<Matrix> for &Vector {
    type Output = Vector;

    /// multiply a row vector and a matrix
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// use truck_geometry::Matrix;
    /// let vec0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, -2.0, -2.0];
    /// let ar2 = [1.0, 1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, 4.0, -3.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ans_vec = Vector::new(2.0, 1.0, 6.0, -9.0);
    /// let vec = &vec0 * mat0;
    /// assert_eq!(vec, ans_vec);
    /// ```
    #[inline(always)]
    fn mul(self, matrix: Matrix) -> Vector { self * &matrix }
}

impl std::ops::Mul<&Matrix> for Vector {
    type Output = Vector;

    /// multiply a row vector and a matrix
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// use truck_geometry::Matrix;
    /// let vec0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, -2.0, -2.0];
    /// let ar2 = [1.0, 1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, 4.0, -3.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ans_vec = Vector::new(2.0, 1.0, 6.0, -9.0);
    /// let vec = vec0 * &mat0;
    /// assert_eq!(vec, ans_vec);
    /// ```
    #[inline(always)]
    fn mul(self, matrix: &Matrix) -> Vector { &self * matrix }
}

impl std::ops::Mul<Matrix> for Vector {
    type Output = Vector;

    /// multiply a row vector and a matrix
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// use truck_geometry::Matrix;
    /// let vec0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, -2.0, -2.0];
    /// let ar2 = [1.0, 1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, 4.0, -3.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ans_vec = Vector::new(2.0, 1.0, 6.0, -9.0);
    /// let vec = vec0 * mat0;
    /// assert_eq!(vec, ans_vec);
    /// ```
    #[inline(always)]
    fn mul(self, matrix: Matrix) -> Vector { &self * &matrix }
}

impl std::ops::MulAssign<&Matrix> for Vector {
    /// multiply a row vector and a matrix
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// use truck_geometry::Matrix;
    /// let mut vec = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, -2.0, -2.0];
    /// let ar2 = [1.0, 1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, 4.0, -3.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ans_vec = Vector::new(2.0, 1.0, 6.0, -9.0);
    /// vec *= &mat0;
    /// assert_eq!(vec, ans_vec);
    /// ```
    #[inline(always)]
    fn mul_assign(&mut self, matrix: &Matrix) { *self = &*self * matrix }
}

impl std::ops::MulAssign<Matrix> for Vector {
    /// multiply a row vector and a matrix
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// use truck_geometry::Matrix;
    /// let mut vec = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [-1.0, 2.0, -2.0, -2.0];
    /// let ar2 = [1.0, 1.0, -3.0, 1.0];
    /// let ar3 = [0.0, -2.0, 4.0, -3.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ans_vec = Vector::new(2.0, 1.0, 6.0, -9.0);
    /// vec *= mat0;
    /// assert_eq!(vec, ans_vec);
    /// ```
    #[inline(always)]
    fn mul_assign(&mut self, matrix: Matrix) { self.mul_assign(&matrix); }
}

impl std::ops::MulAssign<f64> for Matrix {
    /// multiply for each components
    /// #Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mut mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [2.0, 4.0, 6.0, 8.0];
    /// let ar1 = [10.0, 12.0, 14.0, 16.0];
    /// let ar2 = [-2.0, -4.0, -6.0, -8.0];
    /// let ar3 = [-10.0, -12.0, -14.0, -16.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// mat *= 2.0;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn mul_assign(&mut self, scalar: f64) {
        self[0][0] *= scalar;
        self[0][1] *= scalar;
        self[0][2] *= scalar;
        self[0][3] *= scalar;
        self[1][0] *= scalar;
        self[1][1] *= scalar;
        self[1][2] *= scalar;
        self[1][3] *= scalar;
        self[2][0] *= scalar;
        self[2][1] *= scalar;
        self[2][2] *= scalar;
        self[2][3] *= scalar;
        self[3][0] *= scalar;
        self[3][1] *= scalar;
        self[3][2] *= scalar;
        self[3][3] *= scalar;
    }
}

impl std::ops::Mul<f64> for &Matrix {
    type Output = Matrix;

    /// multiply for each components
    /// #Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [2.0, 4.0, 6.0, 8.0];
    /// let ar1 = [10.0, 12.0, 14.0, 16.0];
    /// let ar2 = [-2.0, -4.0, -6.0, -8.0];
    /// let ar3 = [-10.0, -12.0, -14.0, -16.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let mat = &mat0 * 2.0;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn mul(self, scalar: f64) -> Matrix {
        let mut res = self.clone();
        res *= scalar;
        res
    }
}

impl std::ops::Mul<f64> for Matrix {
    type Output = Matrix;

    /// multiply for each components
    /// #Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [2.0, 4.0, 6.0, 8.0];
    /// let ar1 = [10.0, 12.0, 14.0, 16.0];
    /// let ar2 = [-2.0, -4.0, -6.0, -8.0];
    /// let ar3 = [-10.0, -12.0, -14.0, -16.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let mat = mat0 * 2.0;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn mul(self, scalar: f64) -> Matrix { &self * scalar }
}

impl std::ops::Mul<&Matrix> for f64 {
    type Output = Matrix;

    /// multiply for each components
    /// #Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [2.0, 4.0, 6.0, 8.0];
    /// let ar1 = [10.0, 12.0, 14.0, 16.0];
    /// let ar2 = [-2.0, -4.0, -6.0, -8.0];
    /// let ar3 = [-10.0, -12.0, -14.0, -16.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let mat = 2.0 * &mat0;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn mul(self, matrix: &Matrix) -> Matrix { matrix * self }
}

impl std::ops::Mul<Matrix> for f64 {
    type Output = Matrix;

    /// multiply for each components
    /// #Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [2.0, 4.0, 6.0, 8.0];
    /// let ar1 = [10.0, 12.0, 14.0, 16.0];
    /// let ar2 = [-2.0, -4.0, -6.0, -8.0];
    /// let ar3 = [-10.0, -12.0, -14.0, -16.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let mat = mat0 * 2.0;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn mul(self, matrix: Matrix) -> Matrix { matrix * self }
}

impl std::ops::DivAssign<f64> for Matrix {
    /// divide for each components
    /// #Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mut mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [0.5, 1.0, 1.5, 2.0];
    /// let ar1 = [2.5, 3.0, 3.5, 4.0];
    /// let ar2 = [-0.5, -1.0, -1.5, -2.0];
    /// let ar3 = [-2.5, -3.0, -3.5, -4.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// mat /= 2.0;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn div_assign(&mut self, scalar: f64) {
        self[0][0] /= scalar;
        self[0][1] /= scalar;
        self[0][2] /= scalar;
        self[0][3] /= scalar;
        self[1][0] /= scalar;
        self[1][1] /= scalar;
        self[1][2] /= scalar;
        self[1][3] /= scalar;
        self[2][0] /= scalar;
        self[2][1] /= scalar;
        self[2][2] /= scalar;
        self[2][3] /= scalar;
        self[3][0] /= scalar;
        self[3][1] /= scalar;
        self[3][2] /= scalar;
        self[3][3] /= scalar;
    }
}

impl std::ops::Div<f64> for &Matrix {
    type Output = Matrix;

    /// divide for each components
    /// #Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [0.5, 1.0, 1.5, 2.0];
    /// let ar1 = [2.5, 3.0, 3.5, 4.0];
    /// let ar2 = [-0.5, -1.0, -1.5, -2.0];
    /// let ar3 = [-2.5, -3.0, -3.5, -4.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let mat = &mat0 / 2.0;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn div(self, scalar: f64) -> Matrix {
        let mut res = self.clone();
        res /= scalar;
        res
    }
}

impl std::ops::Div<f64> for Matrix {
    type Output = Matrix;

    /// divide for each components
    /// #Examples
    /// ```
    /// use truck_geometry::Matrix;
    /// let ar0 = [1.0, 2.0, 3.0, 4.0];
    /// let ar1 = [5.0, 6.0, 7.0, 8.0];
    /// let ar2 = [-1.0, -2.0, -3.0, -4.0];
    /// let ar3 = [-5.0, -6.0, -7.0, -8.0];
    /// let mat0 = Matrix::new(ar0, ar1, ar2, ar3);
    /// let ar0 = [0.5, 1.0, 1.5, 2.0];
    /// let ar1 = [2.5, 3.0, 3.5, 4.0];
    /// let ar2 = [-0.5, -1.0, -1.5, -2.0];
    /// let ar3 = [-2.5, -3.0, -3.5, -4.0];
    /// let ans_mat = Matrix::new(ar0, ar1, ar2, ar3);
    /// let mat = mat0 / 2.0;
    /// assert_eq!(mat, ans_mat);
    /// ```
    #[inline(always)]
    fn div(self, scalar: f64) -> Matrix { &self / scalar }
}

impl std::fmt::Display for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "[{}    {}  {}  {}]\n[{}  {}  {}  {}]\n[{}  {}  {}  {}]\n[{}  {}  {}  {}]\n",
            self[0][0], self[0][1], self[0][2], self[0][3],
            self[1][0], self[1][1], self[1][2], self[1][3],
            self[2][0], self[2][1], self[2][2], self[2][3],
            self[3][0], self[3][1], self[3][2], self[3][3],
            ))
    }
}
