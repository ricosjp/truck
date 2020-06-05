use crate::Transform;
use geometry::{Matrix, Vector3};

impl Transform {
    #[inline(always)]
    pub fn new(mat: Matrix) -> Transform { Transform(mat) }

    #[inline(always)]
    pub fn translate(vector: &Vector3) -> Transform {
        let arr0 = [1.0, 0.0, 0.0, 0.0];
        let arr1 = [0.0, 1.0, 0.0, 0.0];
        let arr2 = [0.0, 0.0, 1.0, 0.0];
        let arr3 = [vector[0], vector[1], vector[2], 1.0];
        Transform(Matrix::new(arr0, arr1, arr2, arr3))
    }

    #[inline(always)]
    pub fn scale(scalars: &Vector3) -> Transform {
        let arr0 = [scalars[0], 0.0, 0.0, 0.0];
        let arr1 = [0.0, scalars[1], 0.0, 0.0];
        let arr2 = [0.0, 0.0, scalars[2], 0.0];
        let arr3 = [0.0, 0.0, 0.0, 1.0];
        Transform(Matrix::new(arr0, arr1, arr2, arr3))
    }

    #[inline(always)]
    pub fn rotate(axis: &Vector3, angle: f64) -> Transform {
        let cos = angle.cos();
        let sin = angle.sin();
        let arr0 = [
            cos + axis[0] * axis[0] * (1.0 - cos),
            axis[0] * axis[1] * (1.0 - cos) + axis[2] * sin,
            axis[2] * axis[0] * (1.0 - cos) - axis[1] * sin,
            0.0,
        ];
        let arr1 = [
            axis[0] * axis[1] * (1.0 - cos) - axis[2] * sin,
            cos + axis[1] * axis[1] * (1.0 - cos),
            axis[1] * axis[2] * (1.0 - cos) + axis[0] * sin,
            0.0,
        ];
        let arr2 = [
            axis[2] * axis[0] * (1.0 - cos) + axis[1] * sin,
            axis[1] * axis[2] * (1.0 - cos) - axis[0] * sin,
            cos + axis[1] * axis[1] * (1.0 - cos),
            0.0,
        ];
        let arr3 = [0.0, 0.0, 0.0, 1.0];
        Transform(Matrix::new(arr0, arr1, arr2, arr3))
    }
}

impl std::ops::MulAssign<&Transform> for Transform {
    #[inline(always)]
    fn mul_assign(&mut self, other: &Transform) { self.0 *= &other.0; }
}

impl std::ops::MulAssign<Transform> for Transform {
    #[inline(always)]
    fn mul_assign(&mut self, other: Transform) { self.0 *= other.0; }
}

impl std::ops::Mul<&Transform> for &Transform {
    type Output = Transform;
    #[inline(always)]
    fn mul(self, other: &Transform) -> Transform { Transform(&self.0 * &other.0) }
}

impl std::ops::Mul<&Transform> for Transform {
    type Output = Transform;
    #[inline(always)]
    fn mul(mut self, other: &Transform) -> Transform {
        self *= other;
        self
    }
}

impl std::ops::Mul<Transform> for &Transform {
    type Output = Transform;
    #[inline(always)]
    fn mul(self, other: Transform) -> Transform { other * self }
}

impl std::ops::Mul<Transform> for Transform {
    type Output = Transform;
    #[inline(always)]
    fn mul(self, other: Transform) -> Transform { self * &other }
}

impl std::ops::AddAssign<&Vector3> for Transform {
    #[inline(always)]
    fn add_assign(&mut self, vector: &Vector3) { *self *= Transform::translate(vector); }
}

impl std::ops::AddAssign<Vector3> for Transform {
    #[inline(always)]
    fn add_assign(&mut self, vector: Vector3) { *self *= Transform::translate(&vector); }
}

impl std::ops::SubAssign<&Vector3> for Transform {
    #[inline(always)]
    fn sub_assign(&mut self, vector: &Vector3) { *self *= Transform::translate(&-vector); }
}

impl std::ops::SubAssign<Vector3> for Transform {
    #[inline(always)]
    fn sub_assign(&mut self, vector: Vector3) { *self *= Transform::translate(&-vector); }
}

impl std::convert::From<Transform> for Matrix {
    #[inline(always)]
    fn from(transform: Transform) -> Matrix { transform.0 }
}

impl std::convert::From<Matrix> for Transform {
    #[inline(always)]
    fn from(mat: Matrix) -> Transform { Transform(mat) }
}
