use crate::*;
use geometry::{Matrix3, Matrix4, Vector3};

impl Transform {
    #[inline(always)]
    pub fn new(mat: Matrix4) -> Transform { Transform(mat) }

    #[inline(always)]
    pub fn translate(vector: &Vector3) -> Transform {
        Transform(Matrix3::identity().affine(vector))
    }

    #[inline(always)]
    pub fn scale(scalars: &Vector3) -> Transform {
        Transform(Matrix4::diagonal(&rvector!(
            scalars[0], scalars[1], scalars[2]
        )))
    }

    #[inline(always)]
    pub fn rotate(axis: &Vector3, angle: f64) -> Transform {
        Transform(Matrix3::rotation(axis, angle).affine(&Vector3::zero()))
    }

    #[inline(always)]
    pub fn by_axes(axis_x: Vector3, axis_y: Vector3, axis_z: Vector3) -> Transform {
        Transform(matrix!(axis_x, axis_y, axis_z).affine(&Vector3::zero()))
    }

    #[inline(always)]
    pub fn identity() -> Transform { Transform(Matrix4::identity()) }
    #[inline(always)]
    pub fn inverse(&self) -> Result<Transform> { Ok(Transform(self.0.inverse())) }
    #[inline(always)]
    pub fn mul_assign_closure<'a, T: std::ops::MulAssign<&'a Transform>>(
        &'a self,
    ) -> impl Fn(&mut T) + 'a {
        move |x| {
            *x *= self;
        }
    }
}

impl Default for Transform {
    #[inline(always)]
    fn default() -> Self { Transform::identity() }
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
    fn add_assign(&mut self, vector: &Vector3) {
        self.0[3][0] += vector[0];
        self.0[3][1] += vector[1];
        self.0[3][2] += vector[2];
    }
}

impl std::ops::AddAssign<Vector3> for Transform {
    #[inline(always)]
    fn add_assign(&mut self, vector: Vector3) { 
        self.0[3][0] += vector[0];
        self.0[3][1] += vector[1];
        self.0[3][2] += vector[2];
    }
}

impl std::ops::SubAssign<&Vector3> for Transform {
    #[inline(always)]
    fn sub_assign(&mut self, vector: &Vector3) {
        self.0[3][0] -= vector[0];
        self.0[3][1] -= vector[1];
        self.0[3][2] -= vector[2];
    }
}

impl std::ops::SubAssign<Vector3> for Transform {
    #[inline(always)]
    fn sub_assign(&mut self, vector: Vector3) {
        self.0[3][0] -= vector[0];
        self.0[3][1] -= vector[1];
        self.0[3][2] -= vector[2];
    }
}

impl std::ops::MulAssign<&Transform> for Vector3 {
    #[inline(always)]
    fn mul_assign(&mut self, trsf: &Transform) {
        let mut vector = rvector!(self[0], self[1], self[2]);
        vector *= trsf;
        self[0] = vector[0] / vector[3];
        self[1] = vector[1] / vector[3];
        self[2] = vector[2] / vector[3];
    }
}

impl std::ops::MulAssign<Transform> for Vector3 {
    #[inline(always)]
    fn mul_assign(&mut self, trsf: Transform) { *self *= &trsf; }
}

impl std::convert::From<Transform> for Matrix4 {
    #[inline(always)]
    fn from(transform: Transform) -> Matrix4 { transform.0 }
}

impl std::convert::From<Matrix4> for Transform {
    #[inline(always)]
    fn from(mat: Matrix4) -> Transform { Transform(mat) }
}

macro_rules! mul_as_mat {
    ($classname: ty) => {
        impl std::ops::MulAssign<&Transform> for $classname {
            #[inline(always)]
            fn mul_assign(&mut self, trsf: &Transform) { *self *= &trsf.0; }
        }
        impl std::ops::MulAssign<Transform> for $classname {
            #[inline(always)]
            fn mul_assign(&mut self, trsf: Transform) { *self *= &trsf.0; }
        }
    };
}

mul_as_mat!(Vector);
mul_as_mat!(BSplineCurve);
mul_as_mat!(BSplineSurface);
