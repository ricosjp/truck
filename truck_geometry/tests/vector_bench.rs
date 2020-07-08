use std::cmp::Ordering;
use truck_geometry::{Origin, Tolerance};

macro_rules! vector_define {
    ($classname: ident, $dim: expr) => {
        /// vector
        #[derive(Clone, PartialEq, Debug)]
        pub struct $classname([f64; $dim]);
    };
}

vector_define!(Vector, 4);
vector_define!(Vector3, 3);
vector_define!(Vector2, 2);

macro_rules! impl_vector_new {
    ($classname: ty, $dim: expr, $($field: ident), *) => {
        impl $classname {
            /// constructor.
            pub fn new<T: Into<f64>>($($field: T), *) -> Self {
                Self([$($field.into()), *])
            }
        }
    };
}

impl_vector_new!(Vector, 4, x, y, z, w);
impl_vector_new!(Vector3, 3, x, y, z);
impl_vector_new!(Vector2, 2, u, v);

macro_rules! impl_vector {
    ($classname: ty, $dim: expr, $($num: expr), *) => {
        impl $classname {
            /// construct by a reference of array.
            #[inline(always)]
            pub fn by_array_ref(arr: &[f64; $dim]) -> Self { Self(arr.clone()) }

            /// construct a vector whose components are 0.0.
            pub const fn zero() -> Self { Self([0.0; $dim]) }

            /// as_slice
            #[inline(always)]
            pub fn as_slice(&self) -> &[f64] { &self.0 }

            /// as_mut_slice
            #[inline(always)]
            pub fn as_mut_slice(&mut self) -> &mut [f64] { &mut self.0 }

            /// other copy to self
            #[inline(always)]
            pub fn assign(&mut self, other: &$classname) {
                $(self[$num] = other[$num];)*
            }

            /// square of norm
            #[inline(always)]
            pub fn norm2(&self) -> f64 { self * self }

            /// norm
            #[inline(always)]
            pub fn norm(&self) -> f64 { self.norm2().sqrt() }

            /// culculate cosine of the angle of the two vectors
            /// ```
            /// use truck_geometry::Vector;
            /// let vec0 = Vector::new(1.0, 0.0, 0.0, 0.0);
            /// let vec1 = Vector::new(0.0, 2.0, 0.0, 0.0);
            /// assert_eq!(vec0.cos_angle(&vec1), 0.0);
            /// ```
            #[inline(always)]
            pub fn cos_angle(&self, other: &Self) -> f64 {
                let norm = self.norm();
                let vec0 = if norm.so_small() {
                    return 0.0;
                } else {
                    self / norm
                };
                let norm = other.norm();
                let vec1 = if norm.so_small() {
                    return 0.0;
                } else {
                    other / norm
                };
                vec0 * vec1
            }

            /// culculate the angle of two vectors
            #[inline(always)]
            pub fn angle(&self, other: &Self) -> f64 { self.cos_angle(other).acos() }

            /// project to 3D affine plane w = 1
            #[inline(always)]
            pub fn projection(&self) -> Self {
                Self::new($(self[$num] / self[$dim - 1]),*)
            }

            /// curve-derivation projection.
            /// For a curve x(t) = (x_0(t), x_1(t), x_2(t), x_3(t)), calculate the derivation
            /// of the projected curve (x_0 / x_3, x_1 / x_3, x_2 / x_3, 1.0).
            /// # Arguments
            /// * `self` - x(t)
            /// * `der` - x'(t)
            #[inline(always)]
            pub fn derivation_projection(&self, der: &Self) -> Self {
                let denom = self[$dim - 1] * self[$dim - 1];
                Self::new($((der[$num] * self[$dim - 1] - self[$num] * der[$dim - 1]) / denom),*)
            }

            #[inline(always)]
            pub fn derivation2_projection(&self, der: &Self, der2: &Self) -> Self {
                let s = self[$dim - 1] * self[$dim - 1];
                let q = s * s;
                let k = 2.0 * self[$dim - 1] * der[$dim - 1];
                Self::new($(
                        ((der2[$num] * self[$dim - 1] - self[$num] * der2[$dim - 1]) * s
                         - (der[$num] * self[$dim - 1] - self[$num] * der[$dim - 1]) * k) / q),*)
            }

            #[inline(always)]
            pub fn nan_to_zero(&mut self) -> &Self {
                $(if self[$num] == f64::NAN {
                    self[$num] = 0.0;
                })*
                &*self
            }
        }

        impl std::ops::Index<usize> for $classname {
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
            fn index(&self, idx: usize) -> &f64 { &self.0[idx] }
        }
        impl std::ops::IndexMut<usize> for $classname {
            /// access each component
            /// # Examples
            /// ```
            /// use truck_geometry::Vector;
            /// let mut v0 = Vector::new(1.0, 2.0, 3.0, 4.0);
            /// v0[1] = 3.0;
            /// assert_eq!(v0, Vector::new(1.0, 3.0, 3.0, 4.0));
            /// ```
            #[inline(always)]
            fn index_mut(&mut self, idx: usize) -> &mut f64 { &mut self.0[idx] }
        }
        impl Tolerance for $classname {
            #[inline(always)]
            fn near(&self, other: &Self) -> bool {
                true $(&& self[$num].near(&other[$num]))*
            }

            #[inline(always)]
            fn near2(&self, other: &$classname) -> bool {
                true $(&& self[$num].near2(&other[$num]))*
            }
        }

        impl Origin for $classname {
            const ORIGIN: Self = Self::zero();
            fn round_by_tolerance(&mut self) -> &mut Self {
                $(self[$num].round_by_tolerance();)*
                self
            }
        }

        impl std::convert::AsRef<[f64]> for $classname {
            #[inline(always)]
            fn as_ref(&self) -> &[f64] { self.0.as_ref() }
        }

        impl std::convert::From<[f64; $dim]> for $classname {
            /// construct by array
            /// # Examples
            /// ```
            /// use truck_geometry::Vector;
            /// let v = Vector::from([1.0, 2.0, 3.0, 4.0]);
            /// assert_eq!(v, Vector::new(1.0, 2.0, 3.0, 4.0));
            /// ```
            #[inline(always)]
            fn from(arr: [f64; $dim]) -> Self { Self(arr) }
        }
        
        impl std::convert::From<&[f64; $dim]> for $classname {
            /// construct by array
            /// # Examples
            /// ```
            /// use truck_geometry::Vector;
            /// let v = Vector::from([1.0, 2.0, 3.0, 4.0]);
            /// assert_eq!(v, Vector::new(1.0, 2.0, 3.0, 4.0));
            /// ```
            #[inline(always)]
            fn from(arr: &[f64; $dim]) -> Self { Self(arr.clone()) }
        }

        impl std::convert::From<$classname> for [f64; $dim] {
            /// into the array
            /// # Examples
            /// ```
            /// use truck_geometry::Vector;
            /// let v = Vector::new(1.0, 2.0, 3.0, 4.0);
            /// let arr: [f64; 4] = v.into();
            /// assert_eq!(arr, [1.0, 2.0, 3.0, 4.0]);
            /// ```
            #[inline(always)]
            fn from(vec: $classname) -> [f64; $dim] { vec.0 }
        }

        impl std::convert::From<$classname> for [f32; $dim] {
            /// into the `f32` array
            /// # Examples
            /// ```
            /// use truck_geometry::Vector;
            /// let v = Vector::new(1.0, 2.0, 3.0, 4.0);
            /// let arr: [f32; 4] = v.into();
            /// assert_eq!(arr, [1.0, 2.0, 3.0, 4.0]);
            /// ```
            #[inline(always)]
            fn from(vec: $classname) -> [f32; $dim] {
                [
                    $(vec.0[$num] as f32),*
                ]
            }
        }

        impl std::ops::AddAssign<&Self> for $classname {
            /// add_assign for each components
            /// # Examples
            /// ```
            /// use truck_geometry::Vector;
            /// let mut v = Vector::new(1.0, 2.0, 3.0, 4.0);
            /// v += &Vector::new(1.0, -3.0, -2.0, 3.0);
            /// assert_eq!(v, Vector::new(2.0, -1.0, 1.0, 7.0));
            /// ```
            #[inline(always)]
            fn add_assign(&mut self, rhs: &Self) {
                $(self[$num] += rhs[$num];)*
            }
        }

        impl std::ops::AddAssign<Self> for $classname {
            /// add_assign for each components
            /// # Examples
            /// ```
            /// use truck_geometry::Vector;
            /// let mut v = Vector::new(1.0, 2.0, 3.0, 4.0);
            /// v += Vector::new(1.0, -3.0, -2.0, 3.0);
            /// assert_eq!(v, Vector::new(2.0, -1.0, 1.0, 7.0));
            /// ```
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) { self.add_assign(&rhs); }
        }

        impl std::ops::Add<&$classname> for &$classname {
            type Output = $classname;

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
            fn add(self, other: &$classname) -> Self::Output {
                [$(self[$num] + other[$num]),*].into()
            }
        }

        impl std::ops::Add<&$classname> for $classname {
            type Output = $classname;

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
            fn add(mut self, other: &$classname) -> Self::Output {
                self += other;
                self
            }
        }

        impl std::ops::Add<$classname> for &$classname {
            type Output = $classname;

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
            fn add(self, other: $classname) -> Self::Output { other + self }
        }

        impl std::ops::Add<$classname> for $classname {
            type Output = $classname;

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
            fn add(self, other: $classname) -> Self::Output { self + &other }
        }

        impl std::ops::SubAssign<&$classname> for $classname {
            /// sub_assign for each components
            /// # Examples
            /// ```
            /// use truck_geometry::Vector;
            /// let mut v = Vector::new(1.0, 2.0, 3.0, 4.0);
            /// v -= Vector::new(1.0, -3.0, -2.0, 3.0);
            /// assert_eq!(v, Vector::new(0.0, 5.0, 5.0, 1.0));
            /// ```
            #[inline(always)]
            fn sub_assign(&mut self, rhs: &$classname) {
                $(self[$num] -= rhs[$num];)*
            }
        }

        impl std::ops::SubAssign<$classname> for $classname {
            /// sub_assign for each components
            /// # Examples
            /// ```
            /// use truck_geometry::Vector;
            /// let mut v = Vector::new(1.0, 2.0, 3.0, 4.0);
            /// v -= Vector::new(1.0, -3.0, -2.0, 3.0);
            /// assert_eq!(v, Vector::new(0.0, 5.0, 5.0, 1.0));
            /// ```
            #[inline(always)]
            fn sub_assign(&mut self, rhs: $classname) { self.sub_assign(&rhs); }
        }

        impl std::ops::Sub<&$classname> for &$classname {
            type Output = $classname;

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
            fn sub(self, other: &$classname) -> Self::Output { [$(self[$num] - other[$num]),*].into() }
        }

        impl std::ops::Sub<$classname> for &$classname {
            type Output = $classname;

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
            fn sub(self, other: $classname) -> Self::Output { self - &other }
        }

        impl std::ops::Sub<&$classname> for $classname {
            type Output = $classname;

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
            fn sub(mut self, other: &$classname) -> Self::Output {
                self -= other;
                self
            }
        }

        impl std::ops::Sub<$classname> for $classname {
            type Output = $classname;

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
            fn sub(self, other: $classname) -> Self::Output { self - &other }
        }

        impl std::ops::MulAssign<f64> for $classname {
            /// mul_assign for each components
            /// # Examples
            /// ```
            /// use truck_geometry::Vector;
            /// let mut v = Vector::new(1.0, 2.0, 3.0, 4.0);
            /// v *= 2.0;
            /// assert_eq!(v, Vector::new(2.0, 4.0, 6.0, 8.0));
            /// ```
            #[inline(always)]
            fn mul_assign(&mut self, scalar: f64) { $(self[$num] *= scalar;)* }
        }

        impl std::ops::Mul<f64> for &$classname {
            type Output = $classname;

            /// multiply for each components
            /// # Examples
            /// ```
            /// use truck_geometry::Vector;
            /// let v = Vector::new(1.0, 2.0, 3.0, 4.0);
            /// let v = &v * 2.0;
            /// assert_eq!(v, Vector::new(2.0, 4.0, 6.0, 8.0));
            /// ```
            #[inline(always)]
            fn mul(self, scalar: f64) -> $classname { [$(self[$num] * scalar),*].into() }
        }

        impl std::ops::Mul<f64> for $classname {
            type Output = $classname;

            /// multiply for each components
            /// # Examples
            /// ```
            /// use truck_geometry::Vector;
            /// let v = Vector::new(1.0, 2.0, 3.0, 4.0);
            /// let v = v * 2.0;
            /// assert_eq!(v, Vector::new(2.0, 4.0, 6.0, 8.0));
            /// ```
            #[inline(always)]
            fn mul(mut self, scalar: f64) -> $classname {
                self *= scalar;
                self
            }
        }

        impl std::ops::Mul<&$classname> for f64 {
            type Output = $classname;

            /// multiply for each components
            /// # Examples
            /// ```
            /// use truck_geometry::Vector;
            /// let v = Vector::new(1.0, 2.0, 3.0, 4.0);
            /// let v = 2.0 * &v;
            /// assert_eq!(v, Vector::new(2.0, 4.0, 6.0, 8.0));
            /// ```
            #[inline(always)]
            fn mul(self, vector: &$classname) -> $classname { vector * self }
        }

        impl std::ops::Mul<$classname> for f64 {
            type Output = $classname;

            /// multiply for each components
            /// # Examples
            /// ```
            /// use truck_geometry::Vector;
            /// let v = Vector::new(1.0, 2.0, 3.0, 4.0);
            /// let v = 2.0 * v;
            /// assert_eq!(v, Vector::new(2.0, 4.0, 6.0, 8.0));
            /// ```
            #[inline(always)]
            fn mul(self, vector: $classname) -> $classname { vector * self }
        }

        impl std::ops::Mul<&$classname> for &$classname {
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
            fn mul(self, other: &$classname) -> f64 { 0.0 $(+ self[$num] * other[$num])* }
        }

        impl std::ops::Mul<$classname> for &$classname {
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
            fn mul(self, other: $classname) -> f64 { self * &other }
        }

        impl std::ops::Mul<&$classname> for $classname {
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
            fn mul(self, other: &$classname) -> f64 { &self * other }
        }

        impl std::ops::Mul<$classname> for $classname {
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
            fn mul(self, other: $classname) -> f64 { &self * &other }
        }

        impl std::ops::DivAssign<f64> for $classname {
            /// div_assign for each components
            /// # Examples
            /// ```
            /// use truck_geometry::Vector;
            /// let mut v = Vector::new(1.0, 2.0, 3.0, 4.0);
            /// v /= 2.0;
            /// assert_eq!(v, Vector::new(0.5, 1.0, 1.5, 2.0));
            /// ```
            #[inline(always)]
            fn div_assign(&mut self, scalar: f64) { $(self[$num] /= scalar;)* }
        }

        impl std::ops::Div<f64> for &$classname {
            type Output = $classname;
            /// Divides for each components
            /// # Examples
            /// ```
            /// use truck_geometry::Vector;
            /// let v = Vector::new(1.0, 2.0, 3.0, 4.0);
            /// let v = &v / 2.0;
            /// assert_eq!(v, Vector::new(0.5, 1.0, 1.5, 2.0));
            /// ```
            #[inline(always)]
            fn div(self, scalar: f64) -> $classname {
                [$(self[$num] / scalar),*].into()
            }
        }

        impl std::ops::Div<f64> for $classname {
            type Output = $classname;

            /// divide for each components
            /// # Examples
            /// ```
            /// use truck_geometry::Vector;
            /// let v = Vector::new(1.0, 2.0, 3.0, 4.0);
            /// let v = v / 2.0;
            /// assert_eq!(v, Vector::new(0.5, 1.0, 1.5, 2.0));
            /// ```
            #[inline(always)]
            fn div(mut self, scalar: f64) -> $classname {
                self /= scalar;
                self
            }
        }

        impl std::ops::Neg for &$classname {
            type Output = $classname;

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

        impl std::ops::Neg for $classname {
            type Output = $classname;

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

        impl std::ops::RemAssign<&$classname> for $classname {
            /// Hadamard product
            /// # Examples
            /// ```
            /// use truck_geometry::Vector;
            /// let mut v = Vector::new(1.0, 2.0, 3.0, 4.0);
            /// v %= &Vector::new(8.0, 7.0, 6.0, 5.0);
            /// assert_eq!(v, Vector::new(8.0, 14.0, 18.0, 20.0));
            /// ```
            #[inline(always)]
            fn rem_assign(&mut self, other: &$classname) { $(self[$num] *= other[$num];)* }
        }

        impl std::ops::Rem<&$classname> for &$classname {
            type Output = $classname;

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
            fn rem(self, other: &$classname) -> $classname {
                let mut res = self.clone();
                res %= other;
                res
            }
        }

        impl std::ops::Rem<$classname> for &$classname {
            type Output = $classname;

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
            fn rem(self, other: $classname) -> $classname { self % &other }
        }

        impl std::ops::Rem<&$classname> for $classname {
            type Output = $classname;

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
            fn rem(mut self, other: &$classname) -> $classname {
                self %= &other;
                self
            }
        }

        impl std::ops::Rem<$classname> for $classname {
            type Output = $classname;

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
            fn rem(self, other: $classname) -> $classname { self % &other }
        }

        impl std::cmp::PartialOrd for $classname {
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
            fn partial_cmp(&self, rhs: &$classname) -> Option<Ordering> {
                $(
                    let res = self[$num].partial_cmp(&rhs[$num]);
                    if res != Some(Ordering::Equal) {
                        return res;
                    }
                )*
                    res
            }
        }
    };
}

impl_vector!(Vector, 4, 0, 1, 2, 3);
impl_vector!(Vector3, 3, 0, 1, 2);
impl_vector!(Vector2, 2, 0, 1);

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

impl std::ops::BitXor<&Vector3> for &Vector3 {
    type Output = Vector3;

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
    fn bitxor(self, other: &Vector3) -> Vector3 {
        let x = self[1] * other[2] - self[2] * other[1];
        let y = self[2] * other[0] - self[0] * other[2];
        let z = self[0] * other[1] - self[1] * other[0];
        Vector3::new(x, y, z)
    }
}

impl std::ops::BitXor<&Vector2> for &Vector2 {
    type Output = f64;

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
    fn bitxor(self, other: &Vector2) -> f64 { self[0] * other[1] - self[1] * other[0] }
}

macro_rules! impl_bitxor_others {
    ($classname: ident) => {
        impl std::ops::BitXor<&$classname> for $classname {
            type Output = <&'static $classname as std::ops::BitXor<&'static $classname>>::Output;

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
            fn bitxor(self, other: &$classname) -> Self::Output { &self ^ other }
        }

        impl std::ops::BitXor<$classname> for &$classname {
            type Output = <&'static $classname as std::ops::BitXor<&'static $classname>>::Output;

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
            fn bitxor(self, other: $classname) -> Self::Output { self ^ &other }
        }

        impl std::ops::BitXor<$classname> for $classname {
            type Output = <&'static $classname as std::ops::BitXor<&'static $classname>>::Output;

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
            fn bitxor(self, other: $classname) -> Self::Output { &self ^ &other }
        }
    };
}

impl_bitxor_others!(Vector);
impl_bitxor_others!(Vector3);
impl_bitxor_others!(Vector2);

macro_rules! impl_bitxor_assign {
    ($classname: ty) => {
        impl std::ops::BitXorAssign<&$classname> for $classname {
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
            fn bitxor_assign(&mut self, rhs: &$classname) { *self = &*self ^ rhs; }
        }

        impl std::ops::BitXorAssign<$classname> for $classname {
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
            fn bitxor_assign(&mut self, rhs: $classname) { self.bitxor_assign(&rhs); }
        }
    };
}

impl_bitxor_assign!(Vector);
impl_bitxor_assign!(Vector3);

macro_rules! impl_lesser_convert {
    ($higher_vector: ty, $lesser_vector: ty, $($lesser_index: expr), *) => {
        impl std::convert::From<$higher_vector> for $lesser_vector {
            #[inline(always)]
            fn from(vector: $higher_vector) -> $lesser_vector {
                <$lesser_vector>::new($(vector[$lesser_index]),*)
            }
        }
    };
}

impl_lesser_convert!(Vector, Vector3, 0, 1, 2);
impl_lesser_convert!(Vector, Vector2, 0, 1);
impl_lesser_convert!(Vector3, Vector2, 0, 1);

impl std::fmt::Display for Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "[{}    {}  {}  {}]",
            self[0], self[1], self[2], self[3]
        ))
    }
}

impl std::fmt::Display for Vector3 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("[{}    {}  {}]", self[0], self[1], self[2]))
    }
}

impl std::fmt::Display for Vector2 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("[{}    {}]", self[0], self[1]))
    }
}

impl Vector {
    /// construct a vector whose 4th component is 1.0.
    /// # Examples
    /// ```
    /// use truck_geometry::Vector;
    /// let v = Vector::new3(1.0, 2.0, 3.0);
    /// assert_eq!(v, Vector::new(1.0, 2.0, 3.0, 1.0));
    /// ```
    #[inline(always)]
    pub fn new3<T: Into<f64>>(x: T, y: T, z: T) -> Vector {
        Self([x.into(), y.into(), z.into(), 1.0])
    }
}

impl Vector3 {
    #[inline(always)]
    pub fn volume(&self, v0: &Vector3, v1: &Vector3) -> f64 {
        self[0] * v0[1] * v1[2] + self[1] * v0[2] * v1[0] + self[2] * v0[0] * v1[1]
            - self[0] * v0[2] * v1[1]
            - self[2] * v0[1] * v1[0]
            - self[1] * v0[0] * v1[2]
    }

    #[inline(always)]
    pub fn divide(&self, v0: &Vector3, v1: &Vector3, v2: &Vector3) -> Option<Vector3> {
        let det = v0.volume(v1, v2);
        if det.so_small2() {
            return None;
        }
        let x = self.volume(v1, v2) / det;
        let y = v0.volume(self, v2) / det;
        let z = v0.volume(v1, self) / det;
        Some(Vector3::new(x, y, z))
    }
}

fn measurement<F: Fn()>(closure: &F) -> i32 {
    let instant = std::time::Instant::now();
    for _ in 0..1_000 {
        closure();
    }
    let end_time = instant.elapsed();
    end_time.as_secs() as i32 * 1_000 + end_time.subsec_nanos() as i32 / 1_000_000
}

pub fn meantime<F: Fn()>(closure: F) -> i32 {
    let mut a: Vec<i32> = (0..4).map(|_| measurement(&closure)).collect();

    let mut idx = 0;
    let mut max = 0;
    for i in 0..4 {
        if a[i] > max {
            max = a[i];
            idx = i;
        }
    }
    a.remove(idx);

    let mut idx = 0;
    let mut min = 0;
    for i in 0..3 {
        if a[i] < min {
            min = a[i];
            idx = i;
        }
    }
    a.remove(idx);

    (a[0] + a[1]) / 2
}

// The benchmark of the old implementation of the vector and the new one.
// This test must run in the release build.
#[test]
#[ignore]
fn vector_bench_add_assign() {
    let old_add_assign = meantime(&old_add_assign);
    println!("old_add_assign: {}", old_add_assign);
    let new_add_assign = meantime(&new_add_assign);
    println!("new_add_assign: {}", new_add_assign);
    assert!(new_add_assign - old_add_assign < 10);
}

// The benchmark of the old implementation of the vector and the new one.
// This test must run in the release build.
#[test]
#[ignore]
fn vector_bench_add0() {
    let vecs = old_testdata();
    let old_add_assign = meantime(|| old_add(&vecs));
    println!("old_add: {}", old_add_assign);
    let vecs = new_testdata();
    let new_add_assign = meantime(|| new_add(&vecs));
    println!("new_add: {}", new_add_assign);
    assert!(new_add_assign - old_add_assign < 10);
}

const N: usize = 1_000_000;

fn new_testdata() -> (Vec<truck_geometry::Vector4>, Vec<truck_geometry::Vector4>) {
    let a: Vec<_> = (0..N)
        .map(|i| {
            let x = i as f64;
            truck_geometry::vector!(x, x + 1.0, x + 2.0, x + 3.0)
        })
        .collect();
    let b: Vec<_> = (0..N)
        .map(|i| {
            let x = i as f64;
            truck_geometry::vector!(x + 1.0, x + 2.0, x + 3.0, x)
        })
        .collect();
    (a, b)
}

fn new_add_assign() {
    let (mut a, b) = new_testdata();
    for (vec0, vec1) in a.iter_mut().zip(&b) {
        *vec0 += vec1;
    }
}

fn new_add((a, b): &(Vec<truck_geometry::Vector4>, Vec<truck_geometry::Vector4>)) {
    for (vec0, vec1) in a.iter().zip(b) {
        let _ = vec0 + vec1;
    }
}

fn old_testdata() -> (Vec<Vector>, Vec<Vector>) {
    let a: Vec<_> = (0..N)
        .map(|i| {
            let x = i as f64;
            Vector::new(x, x + 1.0, x + 2.0, x + 3.0)
        })
        .collect();
    let b: Vec<_> = (0..N)
        .map(|i| {
            let x = i as f64;
            Vector::new(x + 1.0, x + 2.0, x + 3.0, x)
        })
        .collect();
    (a, b)
}

fn old_add_assign() {
    let (mut a, b) = old_testdata();
    for (vec0, vec1) in a.iter_mut().zip(&b) {
        *vec0 += vec1;
    }
}

fn old_add((a, b): &(Vec<Vector>, Vec<Vector>)) {
    for (vec0, vec1) in a.iter().zip(b) {
        let _ = vec0 + vec1;
    }
}
