use crate::{Origin, Tolerance, Vector};
use std::cmp::Ordering;

impl Vector {
    /// constructor.
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// assert_eq!(v[0], 1.0);
    /// assert_eq!(v[1], 2.0);
    /// assert_eq!(v[2], 3.0);
    /// assert_eq!(v[3], 4.0);
    /// ```
    #[inline(always)]
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Vector { Vector { x: [x, y, z, w] } }

    /// construct a vector whose 4th component is 1.0.
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v = Vector::new3(1.0, 2.0, 3.0);
    /// assert_eq!(v, Vector::new(1.0, 2.0, 3.0, 1.0));
    /// ```
    #[inline(always)]
    pub fn new3(x: f64, y: f64, z: f64) -> Vector { Vector { x: [x, y, z, 1.0] } }

    /// construct by a reference of array.
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v = Vector::by_array_ref(&[1.0, 2.0, 3.0, 4.0]);
    /// assert_eq!(v, Vector::new(1.0, 2.0, 3.0, 4.0));
    /// ```
    #[inline(always)]
    pub fn by_array_ref(arr: &[f64; 4]) -> Vector { Vector { x: arr.clone() } }

    /// construct a vector whose components are 0.0.
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v = Vector::zero();
    /// assert_eq!(v, Vector::new(0.0, 0.0, 0.0, 0.0));
    /// ```
    #[inline(always)]
    pub const fn zero() -> Vector {
        Vector {
            x: [0.0, 0.0, 0.0, 0.0],
        }
    }

    /// as_slice
    #[inline(always)]
    pub fn as_slice(&self) -> &[f64] { &self.x }

    /// as_mut_slice
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [f64] { &mut self.x }

    /// other copy to self
    #[inline(always)]
    pub fn assign(&mut self, other: &Vector) {
        self[0] = other[0];
        self[1] = other[1];
        self[2] = other[2];
        self[3] = other[3];
    }

    /// square of norm
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// assert_eq!(v.norm2(), 30.0);
    /// ```
    #[inline(always)]
    pub fn norm2(&self) -> f64 { self * self }

    /// norm
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v = Vector::new(3.0, 0.0, 4.0, 0.0);
    /// assert_eq!(v.norm(), 5.0);
    /// ```
    #[inline(always)]
    pub fn norm(&self) -> f64 { self.norm2().sqrt() }

    /// project to 3D affine plane w = 1
    #[inline(always)]
    pub fn projection(&self) -> Vector {
        Vector::new(self[0] / self[3], self[1] / self[3], self[2] / self[3], 1.0)
    }

    /// culculate cosine similarity
    /// ```
    /// use truck_geometry::Vector;
    /// let vec0 = Vector::new(1.0, 0.0, 0.0, 0.0);
    /// let vec1 = Vector::new(0.0, 2.0, 0.0, 0.0);
    /// assert_eq!(vec0.cos_similarity(vec1), 0.0);
    /// ```
    #[inline(always)]
    pub fn cos_angle(&self, other: &Vector) -> f64 {
        let norm = self.norm();
        let vec0 = if norm.so_small() {
            return 0.0;
        } else {
            self / norm
        };
        let norm = self.norm();
        let vec1 = if norm.so_small() {
            return 0.0;
        } else {
            other / norm
        };
        vec0 * vec1
    }

    /// culculate angle of two vectors
    pub fn angle(&self, other: &Vector) -> f64 {
        self.cos_angle(other).acos()
    }

    /// curve-derivation projection.  
    /// For a curve x(t) = (x_0(t), x_1(t), x_2(t), x_3(t)), calculate the derivation
    /// of the projected curve (x_0 / x_3, x_1 / x_3, x_2 / x_3, 1.0).
    /// # Arguments
    /// * `self` - x(t)
    /// * `der` - x'(t)
    #[inline(always)]
    pub fn derivation_projection(&self, der: &Vector) -> Vector {
        let denom = self[3] * self[3];
        Vector::new(
            (der[0] * self[3] - self[0] * der[3]) / denom,
            (der[1] * self[3] - self[1] * der[3]) / denom,
            (der[2] * self[3] - self[2] * der[3]) / denom,
            0.0,
        )
    }

    #[inline(always)]
    pub fn derivation2_projection(&self, der: &Vector, der2: &Vector) -> Vector {
        let s = self[3] * self[3];
        let q = s * s;
        let k = 2.0 * self[3] * der[3];
        Vector::new(
            ((der2[0] * self[3] - self[0] * der2[3]) * s
                - (der[0] * self[3] - self[0] * der[3]) * k)
                / q,
            ((der2[1] * self[3] - self[1] * der2[3]) * s
                - (der[1] * self[3] - self[1] * der[3]) * k)
                / q,
            ((der2[2] * self[3] - self[2] * der2[3]) * s
                - (der[2] * self[3] - self[2] * der[3]) * k)
                / q,
            0.0,
        )
    }

    #[inline(always)]
    pub fn nan_to_zero(&mut self) -> &Vector {
        for val in self.x.iter_mut() {
            if *val == f64::NAN {
                *val = 0.0;
            }
        }
        &*self
    }
}

impl Tolerance for Vector {
    #[inline(always)]
    fn near(&self, other: &Vector) -> bool {
        self[0].near(&other[0])
            && self[1].near(&other[1])
            && self[2].near(&other[2])
            && self[3].near(&other[3])
    }

    #[inline(always)]
    fn near2(&self, other: &Vector) -> bool {
        self[0].near2(&other[0])
            && self[1].near2(&other[1])
            && self[2].near2(&other[2])
            && self[3].near2(&other[3])
    }
}

impl Origin for Vector {
    const ORIGIN: Vector = Vector::zero();
}

impl std::convert::From<[f64; 4]> for Vector {
    /// construct by array
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v = Vector::from([1.0, 2.0, 3.0, 4.0]);
    /// assert_eq!(v, Vector::new(1.0, 2.0, 3.0, 4.0));
    /// ```
    #[inline(always)]
    fn from(arr: [f64; 4]) -> Vector { Vector { x: arr } }
}

impl std::convert::From<Vector> for [f64; 4] {
    /// into the array
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let arr: [f64; 4] = v.into();
    /// assert_eq!(arr, [1.0, 2.0, 3.0, 4.0]);
    /// ```
    #[inline(always)]
    fn from(vec: Vector) -> [f64; 4] { vec.x }
}

impl std::convert::From<Vector> for [f32; 4] {
    /// into the `f32` array
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let arr: [f32; 4] = v.into();
    /// assert_eq!(arr, [1.0, 2.0, 3.0, 4.0]);
    /// ```
    #[inline(always)]
    fn from(vec: Vector) -> [f32; 4] {
        [vec.x[0] as f32, vec.x[1] as f32, vec.x[2] as f32, vec.x[3] as f32]
    }
}

impl std::ops::Index<usize> for Vector {
    type Output = f64;

    /// access each component
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// assert_eq!(v[0], 1.0);
    /// assert_eq!(v[1], 2.0);
    /// assert_eq!(v[2], 3.0);
    /// assert_eq!(v[3], 4.0);
    /// ```
    #[inline(always)]
    fn index(&self, idx: usize) -> &f64 { &self.x[idx] }
}

impl std::ops::IndexMut<usize> for Vector {
    /// access each component
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let mut v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// v0[1] = 3.0;
    /// assert_eq!(v0, Vector::new(1.0, 3.0, 3.0, 4.0));
    /// ```
    #[inline(always)]
    fn index_mut(&mut self, idx: usize) -> &mut f64 { &mut self.x[idx] }
}

impl std::ops::AddAssign<&Vector> for Vector {
    /// add_assign for each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let mut v = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// v += &Vector::new(1.0, -3.0, -2.0, 3.0);
    /// assert_eq!(v, Vector::new(2.0, -1.0, 1.0, 7.0));
    /// ```
    #[inline(always)]
    fn add_assign(&mut self, rhs: &Vector) {
        self[0] += rhs[0];
        self[1] += rhs[1];
        self[2] += rhs[2];
        self[3] += rhs[3];
    }
}

impl std::ops::AddAssign<Vector> for Vector {
    /// add_assign for each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let mut v = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// v += Vector::new(1.0, -3.0, -2.0, 3.0);
    /// assert_eq!(v, Vector::new(2.0, -1.0, 1.0, 7.0));
    /// ```
    #[inline(always)]
    fn add_assign(&mut self, rhs: Vector) { self.add_assign(&rhs); }
}

impl std::ops::Add<&Vector> for &Vector {
    type Output = Vector;

    /// add each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v1 = Vector::new(1.0, -3.0, -2.0, 3.0);
    /// let v = &v0 + &v1;
    /// assert_eq!(v, Vector::new(2.0, -1.0, 1.0, 7.0));
    /// ```
    #[inline(always)]
    fn add(self, other: &Vector) -> Self::Output {
        let mut res = self.clone();
        res += other;
        res
    }
}

impl std::ops::Add<&Vector> for Vector {
    type Output = Vector;

    /// add each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v1 = Vector::new(1.0, -3.0, -2.0, 3.0);
    /// let v = v0 + &v1;
    /// assert_eq!(v, Vector::new(2.0, -1.0, 1.0, 7.0));
    /// ```
    #[inline(always)]
    fn add(mut self, other: &Vector) -> Self::Output {
        self += other;
        self
    }
}

impl std::ops::Add<Vector> for &Vector {
    type Output = Vector;

    /// add each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v1 = Vector::new(1.0, -3.0, -2.0, 3.0);
    /// let v = &v0 + v1;
    /// assert_eq!(v, Vector::new(2.0, -1.0, 1.0, 7.0));
    /// ```
    #[inline(always)]
    fn add(self, other: Vector) -> Self::Output { self + &other }
}

impl std::ops::Add<Vector> for Vector {
    type Output = Vector;

    /// add each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v1 = Vector::new(1.0, -3.0, -2.0, 3.0);
    /// let v = v0 + v1;
    /// assert_eq!(v, Vector::new(2.0, -1.0, 1.0, 7.0));
    /// ```
    #[inline(always)]
    fn add(self, other: Vector) -> Self::Output { self + &other }
}

impl std::ops::SubAssign<&Vector> for Vector {
    /// sub_assign for each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let mut v = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// v -= &Vector::new(1.0, -3.0, -2.0, 3.0);
    /// assert_eq!(v, Vector::new(0.0, 5.0, 5.0, 1.0));
    /// ```
    #[inline(always)]
    fn sub_assign(&mut self, rhs: &Vector) {
        self[0] -= rhs[0];
        self[1] -= rhs[1];
        self[2] -= rhs[2];
        self[3] -= rhs[3];
    }
}

impl std::ops::SubAssign<Vector> for Vector {
    /// sub_assign for each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let mut v = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// v -= Vector::new(1.0, -3.0, -2.0, 3.0);
    /// assert_eq!(v, Vector::new(0.0, 5.0, 5.0, 1.0));
    /// ```
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Vector) { self.sub_assign(&rhs); }
}

impl std::ops::Sub<&Vector> for &Vector {
    type Output = Vector;

    /// subtract each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v1 = Vector::new(1.0, -3.0, -2.0, 3.0);
    /// let v = &v0 - &v1;
    /// assert_eq!(v, Vector::new(0.0, 5.0, 5.0, 1.0));
    /// ```
    #[inline(always)]
    fn sub(self, other: &Vector) -> Self::Output {
        let mut res = self.clone();
        res -= other;
        res
    }
}

impl std::ops::Sub<Vector> for &Vector {
    type Output = Vector;

    /// subtract each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v1 = Vector::new(1.0, -3.0, -2.0, 3.0);
    /// let v = &v0 - v1;
    /// assert_eq!(v, Vector::new(0.0, 5.0, 5.0, 1.0));
    /// ```
    #[inline(always)]
    fn sub(self, other: Vector) -> Self::Output { self - &other }
}

impl std::ops::Sub<&Vector> for Vector {
    type Output = Vector;

    /// subtract each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v1 = Vector::new(1.0, -3.0, -2.0, 3.0);
    /// let v = v0 - &v1;
    /// assert_eq!(v, Vector::new(0.0, 5.0, 5.0, 1.0));
    /// ```
    #[inline(always)]
    fn sub(mut self, other: &Vector) -> Self::Output {
        self -= other;
        self
    }
}

impl std::ops::Sub<Vector> for Vector {
    type Output = Vector;

    /// subtract each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v1 = Vector::new(1.0, -3.0, -2.0, 3.0);
    /// let v = v0 - v1;
    /// assert_eq!(v, Vector::new(0.0, 5.0, 5.0, 1.0));
    /// ```
    #[inline(always)]
    fn sub(self, other: Vector) -> Self::Output { self - &other }
}

impl std::ops::MulAssign<f64> for Vector {
    /// mul_assign for each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let mut v = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// v *= 2.0;
    /// assert_eq!(v, Vector::new(2.0, 4.0, 6.0, 8.0));
    /// ```
    #[inline(always)]
    fn mul_assign(&mut self, scalar: f64) {
        for a in self.as_mut_slice() {
            *a *= scalar;
        }
    }
}

impl std::ops::Mul<f64> for &Vector {
    type Output = Vector;

    /// multiply for each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v = &v * 2.0;
    /// assert_eq!(v, Vector::new(2.0, 4.0, 6.0, 8.0));
    /// ```
    #[inline(always)]
    fn mul(self, scalar: f64) -> Vector {
        let mut res = self.clone();
        res *= scalar;
        res
    }
}

impl std::ops::Mul<f64> for Vector {
    type Output = Vector;

    /// multiply for each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v = v * 2.0;
    /// assert_eq!(v, Vector::new(2.0, 4.0, 6.0, 8.0));
    /// ```
    #[inline(always)]
    fn mul(mut self, scalar: f64) -> Vector {
        self *= scalar;
        self
    }
}

impl std::ops::Mul<&Vector> for f64 {
    type Output = Vector;

    /// multiply for each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v = 2.0 * &v;
    /// assert_eq!(v, Vector::new(2.0, 4.0, 6.0, 8.0));
    /// ```
    #[inline(always)]
    fn mul(self, vector: &Vector) -> Vector { vector * self }
}

impl std::ops::Mul<Vector> for f64 {
    type Output = Vector;

    /// multiply for each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v = 2.0 * v;
    /// assert_eq!(v, Vector::new(2.0, 4.0, 6.0, 8.0));
    /// ```
    #[inline(always)]
    fn mul(self, vector: Vector) -> Vector { vector * self }
}

impl std::ops::Mul<&Vector> for &Vector {
    type Output = f64;

    /// inner product
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v1 = Vector::new(1.0, -3.0, -2.0, 3.0);
    /// assert_eq!(&v0 * &v1, 1.0);
    /// ```
    #[inline(always)]
    fn mul(self, other: &Vector) -> f64 {
        self[0] * other[0] + self[1] * other[1] + self[2] * other[2] + self[3] * other[3]
    }
}

impl std::ops::Mul<Vector> for &Vector {
    type Output = f64;

    /// inner product
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v1 = Vector::new(1.0, -3.0, -2.0, 3.0);
    /// assert_eq!(&v0 * v1, 1.0);
    /// ```
    #[inline(always)]
    fn mul(self, other: Vector) -> f64 { self * &other }
}

impl std::ops::Mul<&Vector> for Vector {
    type Output = f64;

    /// inner product
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v1 = Vector::new(1.0, -3.0, -2.0, 3.0);
    /// assert_eq!(v0 * &v1, 1.0);
    /// ```
    #[inline(always)]
    fn mul(self, other: &Vector) -> f64 { &self * other }
}

impl std::ops::Mul<Vector> for Vector {
    type Output = f64;

    /// inner product
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v1 = Vector::new(1.0, -3.0, -2.0, 3.0);
    /// assert_eq!(v0 * v1, 1.0);
    /// ```
    #[inline(always)]
    fn mul(self, other: Vector) -> f64 { &self * &other }
}

impl std::ops::DivAssign<f64> for Vector {
    /// div_assign for each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let mut v = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// v /= 2.0;
    /// assert_eq!(v, Vector::new(0.5, 1.0, 1.5, 2.0));
    /// ```
    #[inline(always)]
    fn div_assign(&mut self, scalar: f64) {
        for a in self.as_mut_slice() {
            *a /= scalar;
        }
    }
}

impl std::ops::Div<f64> for &Vector {
    type Output = Vector;

    /// divide for each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v = &v / 2.0;
    /// assert_eq!(v, Vector::new(0.5, 1.0, 1.5, 2.0));
    /// ```
    #[inline(always)]
    fn div(self, scalar: f64) -> Vector {
        let mut res = self.clone();
        res /= scalar;
        res
    }
}

impl std::ops::Div<f64> for Vector {
    type Output = Vector;

    /// divide for each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v = v / 2.0;
    /// assert_eq!(v, Vector::new(0.5, 1.0, 1.5, 2.0));
    /// ```
    #[inline(always)]
    fn div(mut self, scalar: f64) -> Vector {
        self /= scalar;
        self
    }
}

impl std::ops::Neg for &Vector {
    type Output = Vector;

    /// neg for each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v = -&v;
    /// assert_eq!(v, Vector::new(-1.0, -2.0, -3.0, -4.0));
    /// ```
    #[inline(always)]
    fn neg(self) -> Self::Output { -1.0 * self }
}

impl std::ops::Neg for Vector {
    type Output = Vector;

    /// neg for each components
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v = -v;
    /// assert_eq!(v, Vector::new(-1.0, -2.0, -3.0, -4.0));
    /// ```
    #[inline(always)]
    fn neg(self) -> Self::Output { -1.0 * self }
}

impl std::ops::RemAssign<&Vector> for Vector {
    /// Hadamard product
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let mut v = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// v %= &Vector::new(8.0, 7.0, 6.0, 5.0);
    /// assert_eq!(v, Vector::new(8.0, 14.0, 18.0, 20.0));
    /// ```
    #[inline(always)]
    fn rem_assign(&mut self, other: &Vector) {
        self[0] *= other[0];
        self[1] *= other[1];
        self[2] *= other[2];
        self[3] *= other[3];
    }
}

impl std::ops::Rem<&Vector> for &Vector {
    type Output = Vector;

    /// Hadamard product
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v1 = Vector::new(8.0, 7.0, 6.0, 5.0);
    /// let v = &v0 % &v1;
    /// assert_eq!(v, Vector::new(8.0, 14.0, 18.0, 20.0));
    /// ```
    #[inline(always)]
    fn rem(self, other: &Vector) -> Vector {
        let mut res = self.clone();
        res %= other;
        res
    }
}

impl std::ops::Rem<Vector> for &Vector {
    type Output = Vector;

    /// Hadamard product
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v1 = Vector::new(8.0, 7.0, 6.0, 5.0);
    /// let v = &v0 % v1;
    /// assert_eq!(v, Vector::new(8.0, 14.0, 18.0, 20.0));
    /// ```
    #[inline(always)]
    fn rem(self, other: Vector) -> Vector { self % &other }
}

impl std::ops::Rem<&Vector> for Vector {
    type Output = Vector;

    /// Hadamard product
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v1 = Vector::new(8.0, 7.0, 6.0, 5.0);
    /// let v = v0 % &v1;
    /// assert_eq!(v, Vector::new(8.0, 14.0, 18.0, 20.0));
    /// ```
    #[inline(always)]
    fn rem(mut self, other: &Vector) -> Vector {
        self %= &other;
        self
    }
}

impl std::ops::Rem<Vector> for Vector {
    type Output = Vector;

    /// Hadamard product
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
    /// let v1 = Vector::new(8.0, 7.0, 6.0, 5.0);
    /// let v = v0 % v1;
    /// assert_eq!(v, Vector::new(8.0, 14.0, 18.0, 20.0));
    /// ```
    #[inline(always)]
    fn rem(self, other: Vector) -> Vector { self % &other }
}

impl std::ops::BitXor<&Vector> for &Vector {
    type Output = Vector;

    /// cross product for the first three componets.  
    /// The 3rd component is the norm of the above three components.
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v0 = Vector::new3(1.0, 0.0, 0.0);
    /// let v1 = Vector::new3(0.0, 1.0, 0.0);
    /// let v = &v0 ^ &v1;
    /// assert_eq!(v, Vector::new(0.0, 0.0, 1.0, 1.0));
    /// ```
    #[inline(always)]
    fn bitxor(self, other: &Vector) -> Vector {
        let x = self[1] * other[2] - self[2] * other[1];
        let y = self[2] * other[0] - self[0] * other[2];
        let z = self[0] * other[1] - self[1] * other[0];
        Vector::new(x, y, z, (x * x + y * y + z * z).sqrt())
    }
}

impl std::ops::BitXor<&Vector> for Vector {
    type Output = Vector;

    /// cross product for the first three componets.  
    /// The 3rd component is always `0.0`.
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v0 = Vector::new3(1.0, 0.0, 0.0);
    /// let v1 = Vector::new3(0.0, 1.0, 0.0);
    /// let v = v0 ^ &v1;
    /// assert_eq!(v, Vector::new(0.0, 0.0, 1.0, 1.0));
    /// ```
    #[inline(always)]
    fn bitxor(self, other: &Vector) -> Vector { &self ^ other }
}

impl std::ops::BitXor<Vector> for &Vector {
    type Output = Vector;

    /// cross product for the first three componets.  
    /// The 3rd component is always `0.0`.
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v0 = Vector::new3(1.0, 0.0, 0.0);
    /// let v1 = Vector::new3(0.0, 1.0, 0.0);
    /// let v = &v0 ^ v1;
    /// assert_eq!(v, Vector::new(0.0, 0.0, 1.0, 1.0));
    /// ```
    #[inline(always)]
    fn bitxor(self, other: Vector) -> Vector { self ^ &other }
}

impl std::ops::BitXor<Vector> for Vector {
    type Output = Vector;

    /// cross product for the first three componets.  
    /// The 3rd component is always `0.0`.
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v0 = Vector::new3(1.0, 0.0, 0.0);
    /// let v1 = Vector::new3(0.0, 1.0, 0.0);
    /// let v = v0 ^ &v1;
    /// assert_eq!(v, Vector::new(0.0, 0.0, 1.0, 1.0));
    /// ```
    #[inline(always)]
    fn bitxor(self, other: Vector) -> Vector { &self ^ &other }
}

impl std::ops::BitXorAssign<&Vector> for Vector {
    /// cross product for the first three componets.  
    /// The 3rd component is always `0.0`.
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let mut v = Vector::new3(1.0, 0.0, 0.0);
    /// let v0 = Vector::new3(0.0, 1.0, 0.0);
    /// v ^= &v0;
    /// assert_eq!(v, Vector::new(0.0, 0.0, 1.0, 1.0));
    /// ```
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: &Vector) { *self = &*self ^ rhs; }
}

impl std::ops::BitXorAssign<Vector> for Vector {
    /// cross product for the first three componets.  
    /// The 3rd component is always `0.0`.
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let mut v = Vector::new3(1.0, 0.0, 0.0);
    /// let v0 = Vector::new3(0.0, 1.0, 0.0);
    /// v ^= v0;
    /// assert_eq!(v, Vector::new(0.0, 0.0, 1.0, 1.0));
    /// ```
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Vector) { self.bitxor_assign(&rhs); }
}

impl std::cmp::PartialOrd for Vector {
    /// expression order
    /// ```
    /// use truck_geometry::Vector;
    /// let v0 = Vector::new(1.0, 0.0, 0.0, 0.0);
    /// let v1 = Vector::new(0.0, 100.0, 0.0, 0.0);
    /// let v2 = Vector::new(1.0, 0.0, 1.0, 0.0);
    /// assert!(v0 == v0);
    /// assert!(v0 > v1);
    /// assert!(v0 < v2);
    /// ```
    #[inline(always)]
    fn partial_cmp(&self, rhs: &Vector) -> Option<Ordering> {
        let res = self[0].partial_cmp(&rhs[0]);
        if res != Some(Ordering::Equal) {
            return res;
        }
        let res = self[1].partial_cmp(&rhs[1]);
        if res != Some(Ordering::Equal) {
            return res;
        }
        let res = self[2].partial_cmp(&rhs[2]);
        if res != Some(Ordering::Equal) {
            return res;
        }
        let res = self[3].partial_cmp(&rhs[3]);
        res
    }
}

impl std::fmt::Display for Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("[{}    {}  {}  {}]", self[0], self[1], self[2], self[3]))
    }
}


