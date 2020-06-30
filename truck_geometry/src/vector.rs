use crate::*;
use std::cmp::Ordering;
use std::ops::*;

pub trait VectorEntity:
    Sized + Clone + PartialEq + AsRef<[f64]> + AsMut<[f64]> + std::fmt::Debug + std::default::Default
{
    const ORIGIN: Self;
}

macro_rules! impl_entity_array {
    ($($dim: expr), *) => {
        $(
            impl VectorEntity for [f64; $dim] {
                const ORIGIN: Self = [0.0; $dim];
            }
            impl From<Vector<[f64; $dim]>> for [f64; $dim] {
                #[inline(always)]
                fn from(vec: Vector<[f64; $dim]>) -> [f64; $dim] { vec.0 }
            }
            impl From<Vector<[f64; $dim]>> for [f32; $dim] {
                #[inline(always)]
                fn from(vec: Vector<[f64; $dim]>) -> [f32; $dim] {
                    let mut res = <[f32; $dim]>::default();
                    res.iter_mut().zip(&vec).for_each(|(a, b)| *a = *b as f32);
                    res
                }
            }
        )*
    };
}
impl_entity_array!(2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13);
impl_entity_array!(14, 15, 16, 17, 18, 19, 20, 21, 22, 23);
impl_entity_array!(24, 25, 26, 27, 28, 29, 30, 31, 32);

/// Creates a new `Vector<[f64; N]>`.
/// # Arguments
/// - `$x`: Variadic arguments. `Into<f64>` has to be implemented.
/// ```
/// use truck_geometry::*;
/// fn type_of<T>(_: &T) -> &'static str { std::any::type_name::<T>() }
/// let v = vector_new!(1, 2.0, 3, -4.0_f32);
/// assert_eq!(type_of(&v), "truck_geometry::Vector<[f64; 4]>");
/// assert_eq!(v[0], 1.0);
/// assert_eq!(v[1], 2.0);
/// assert_eq!(v[2], 3.0);
/// assert_eq!(v[3], -4.0);
/// ```
#[macro_export]
macro_rules! vector_new {
    ($($x: expr), *) => {
        $crate::Vector::from([$($x.into()), *])
    };
}

impl<T: VectorEntity> Deref for Vector<T> {
    type Target = [f64];
    #[inline(always)]
    fn deref(&self) -> &[f64] { self.0.as_ref() }
}

impl<T: VectorEntity> DerefMut for Vector<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [f64] { self.0.as_mut() }
}

impl<T: VectorEntity> AsRef<[f64]> for Vector<T> {
    #[inline(always)]
    fn as_ref(&self) -> &[f64] { self.0.as_ref() }
}

impl<T: VectorEntity> AsMut<[f64]> for Vector<T> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut [f64] { self.0.as_mut() }
}

impl<'a, T: VectorEntity> IntoIterator for &'a Vector<T> {
    type Item = &'a f64;
    type IntoIter = std::slice::Iter<'a, f64>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl<'a, T: VectorEntity> IntoIterator for &'a mut Vector<T> {
    type Item = &'a mut f64;
    type IntoIter = std::slice::IterMut<'a, f64>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter { self.iter_mut() }
}

impl<T: VectorEntity> From<T> for Vector<T> {
    #[inline(always)]
    fn from(arr: T) -> Vector<T> { Vector(arr) }
}

impl<T: VectorEntity> From<&T> for Vector<T> {
    #[inline(always)]
    fn from(arr: &T) -> Vector<T> { Vector(arr.clone()) }
}

impl<T: VectorEntity> AddAssign<&Vector<T>> for Vector<T> {
    /// Adds and assigns for each components
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let mut v = vector_new!(1, 2, 3, 4);
    /// v += &vector_new!(1, -3, -2, 3);
    /// assert_eq!(v, vector_new!(2, -1, 1, 7));
    /// ```
    #[inline(always)]
    fn add_assign(&mut self, other: &Vector<T>) {
        self.iter_mut().zip(other).for_each(move |(a, b)| *a += b);
    }
}

impl<T: VectorEntity> AddAssign<Vector<T>> for Vector<T> {
    /// Adds and assigns for each components
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let mut v = vector_new!(1, 2, 3, 4);
    /// v += vector_new!(1, -3, -2, 3);
    /// assert_eq!(v, vector_new!(2, -1, 1, 7));
    /// ```
    #[inline(always)]
    fn add_assign(&mut self, other: Self) { *self += &other }
}

impl<T: VectorEntity> Add<&Vector<T>> for Vector<T> {
    type Output = Vector<T>;
    /// Adds each components
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v0 = vector_new!(1, 2, 3, 4);
    /// let v1 = vector_new!(1, -3, -2, 3);
    /// assert_eq!(v0 + &v1, vector_new!(2, -1, 1, 7));
    /// ```
    #[inline(always)]
    fn add(mut self, other: &Vector<T>) -> Vector<T> {
        self += other;
        self
    }
}

impl<T: VectorEntity> Add<&Vector<T>> for &Vector<T> {
    type Output = Vector<T>;
    /// Adds each components
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v0 = vector_new!(1, 2, 3, 4);
    /// let v1 = vector_new!(1, -3, -2, 3);
    /// assert_eq!(&v0 + &v1, vector_new!(2, -1, 1, 7));
    /// ```
    #[inline(always)]
    fn add(self, other: &Vector<T>) -> Vector<T> { self.clone() + other }
}

impl<T: VectorEntity> Add<Vector<T>> for &Vector<T> {
    type Output = Vector<T>;
    /// Adds each components
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v0 = vector_new!(1, 2, 3, 4);
    /// let v1 = vector_new!(1, -3, -2, 3);
    /// assert_eq!(&v0 + v1, vector_new!(2, -1, 1, 7));
    /// ```
    #[inline(always)]
    fn add(self, other: Vector<T>) -> Vector<T> { other + self }
}

impl<T: VectorEntity> Add<Vector<T>> for Vector<T> {
    type Output = Vector<T>;
    /// Adds each components
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v0 = vector_new!(1, 2, 3, 4);
    /// let v1 = vector_new!(1, -3, -2, 3);
    /// assert_eq!(v0 + v1, vector_new!(2, -1, 1, 7));
    /// ```
    #[inline(always)]
    fn add(self, other: Vector<T>) -> Vector<T> { self + &other }
}

impl<T: VectorEntity> SubAssign<&Vector<T>> for Vector<T> {
    /// Subtracts and assigns for each components
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let mut v = vector_new!(1, 2, 3, 4);
    /// v -= &vector_new!(1, -3, -2, 3);
    /// assert_eq!(v, vector_new!(0, 5, 5, 1));
    /// ```
    #[inline(always)]
    fn sub_assign(&mut self, other: &Vector<T>) {
        self.iter_mut().zip(other).for_each(move |(a, b)| *a -= b);
    }
}

impl<T: VectorEntity> SubAssign<Vector<T>> for Vector<T> {
    /// Subtracts and assigns for each components
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let mut v = vector_new!(1, 2, 3, 4);
    /// v -= vector_new!(1, -3, -2, 3);
    /// assert_eq!(v, vector_new!(0, 5, 5, 1));
    /// ```
    #[inline(always)]
    fn sub_assign(&mut self, other: Self) { *self -= &other }
}

impl<T: VectorEntity> Sub<&Vector<T>> for Vector<T> {
    type Output = Vector<T>;
    /// Subtracts each components
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v0 = vector_new!(1, 2, 3, 4);
    /// let v1 = vector_new!(1, -3, -2, 3);
    /// assert_eq!(v0 - &v1, vector_new!(0, 5, 5, 1));
    /// ```
    #[inline(always)]
    fn sub(mut self, other: &Vector<T>) -> Vector<T> {
        self -= other;
        self
    }
}

impl<T: VectorEntity> Sub<&Vector<T>> for &Vector<T> {
    type Output = Vector<T>;
    /// Subtracts each components
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v0 = vector_new!(1, 2, 3, 4);
    /// let v1 = vector_new!(1, -3, -2, 3);
    /// assert_eq!(&v0 - &v1, vector_new!(0, 5, 5, 1));
    /// ```
    #[inline(always)]
    fn sub(self, other: &Vector<T>) -> Vector<T> { self.clone() - other }
}

impl<T: VectorEntity> Sub<Vector<T>> for &Vector<T> {
    type Output = Vector<T>;
    /// Subtracts each components
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v0 = vector_new!(1, 2, 3, 4);
    /// let v1 = vector_new!(1, -3, -2, 3);
    /// assert_eq!(&v0 - v1, vector_new!(0, 5, 5, 1));
    /// ```
    #[inline(always)]
    fn sub(self, other: Vector<T>) -> Vector<T> { -(other - self) }
}

impl<T: VectorEntity> Sub<Vector<T>> for Vector<T> {
    type Output = Vector<T>;
    /// Subtracts each components
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v0 = vector_new!(1, 2, 3, 4);
    /// let v1 = vector_new!(1, -3, -2, 3);
    /// assert_eq!(v0 - v1, vector_new!(0, 5, 5, 1));
    /// ```
    #[inline(always)]
    fn sub(self, other: Vector<T>) -> Vector<T> { self - &other }
}

impl<T: VectorEntity> MulAssign<f64> for Vector<T> {
    /// Multiplies and assigns for each components
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let mut v = vector_new!(1, 2, 3, 4);
    /// v *= 2.0;
    /// assert_eq!(v, vector_new!(2, 4, 6, 8));
    /// ```
    #[inline(always)]
    fn mul_assign(&mut self, rhs: f64) { self.iter_mut().for_each(move |a| *a *= rhs); }
}

impl<T: VectorEntity> Mul<f64> for Vector<T> {
    type Output = Vector<T>;
    /// Multiplies and assigns for each components
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v = vector_new!(1, 2, 3, 4);
    /// assert_eq!(v * 2.0, vector_new!(2, 4, 6, 8));
    /// ```
    #[inline(always)]
    fn mul(mut self, scalar: f64) -> Vector<T> {
        self *= scalar;
        self
    }
}

impl<'a, T: VectorEntity> Mul<f64> for &'a Vector<T> {
    type Output = Vector<T>;
    /// Multiplies and assigns for each components
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v = vector_new!(1, 2, 3, 4);
    /// assert_eq!(&v * 2.0, vector_new!(2, 4, 6, 8));
    /// ```
    #[inline(always)]
    fn mul(self, scalar: f64) -> Vector<T> { self.clone() * scalar }
}

impl<T: VectorEntity> Mul<&Vector<T>> for f64 {
    type Output = Vector<T>;
    /// Multiplies and assigns for each components
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v = vector_new!(1, 2, 3, 4);
    /// assert_eq!(2.0 * &v, vector_new!(2, 4, 6, 8));
    /// ```
    #[inline(always)]
    fn mul(self, vector: &Vector<T>) -> Vector<T> { vector * self }
}

impl<T: VectorEntity> Mul<Vector<T>> for f64 {
    type Output = Vector<T>;
    /// Multiplies and assigns for each components
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v = vector_new!(1, 2, 3, 4);
    /// assert_eq!(2.0 * v, vector_new!(2, 4, 6, 8));
    /// ```
    #[inline(always)]
    fn mul(self, vector: Vector<T>) -> Vector<T> { vector * self }
}

impl<T: VectorEntity> Mul<&Vector<T>> for &Vector<T> {
    type Output = f64;
    /// inner product
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v0 = vector_new!(1, 2, 3, 4);
    /// let v1 = vector_new!(1, -3, -2, 3);
    /// assert_eq!(&v0 * &v1, 1.0);
    /// ```
    #[inline(always)]
    fn mul(self, other: &Vector<T>) -> f64 { self.iter().zip(other).map(move |(a, b)| a * b).sum() }
}

impl<T: VectorEntity> Mul<Vector<T>> for &Vector<T> {
    type Output = f64;
    /// inner product
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v0 = vector_new!(1, 2, 3, 4);
    /// let v1 = vector_new!(1, -3, -2, 3);
    /// assert_eq!(&v0 * v1, 1.0);
    /// ```
    #[inline(always)]
    fn mul(self, other: Vector<T>) -> f64 { self * &other }
}

impl<T: VectorEntity> Mul<&Vector<T>> for Vector<T> {
    type Output = f64;
    /// inner product
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v0 = vector_new!(1, 2, 3, 4);
    /// let v1 = vector_new!(1, -3, -2, 3);
    /// assert_eq!(v0 * &v1, 1.0);
    /// ```
    #[inline(always)]
    fn mul(self, other: &Vector<T>) -> f64 { &self * other }
}

impl<T: VectorEntity> Mul<Vector<T>> for Vector<T> {
    type Output = f64;
    /// inner product
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v0 = vector_new!(1, 2, 3, 4);
    /// let v1 = vector_new!(1, -3, -2, 3);
    /// assert_eq!(v0 * v1, 1.0);
    /// ```
    #[inline(always)]
    fn mul(self, other: Vector<T>) -> f64 { &self * &other }
}

impl<T: VectorEntity> DivAssign<f64> for Vector<T> {
    /// Divides and assigns for each components
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let mut v = vector_new!(1.0, 2.0, 3.0, 4.0);
    /// v /= 2.0;
    /// assert_eq!(v, vector_new!(0.5, 1.0, 1.5, 2.0));
    /// ```
    #[inline(always)]
    fn div_assign(&mut self, rhs: f64) { self.iter_mut().for_each(move |a| *a /= rhs); }
}

impl<T: VectorEntity> Div<f64> for Vector<T> {
    type Output = Vector<T>;
    /// Divides for each components
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v = vector_new!(1.0, 2.0, 3.0, 4.0);
    /// assert_eq!(v / 2.0, vector_new!(0.5, 1.0, 1.5, 2.0));
    /// ```
    #[inline(always)]
    fn div(mut self, scalar: f64) -> Vector<T> {
        self /= scalar;
        self
    }
}

impl<T: VectorEntity> Div<f64> for &Vector<T> {
    type Output = Vector<T>;
    /// Divides for each components
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v = vector_new!(1.0, 2.0, 3.0, 4.0);
    /// assert_eq!(&v / 2.0, vector_new!(0.5, 1.0, 1.5, 2.0));
    /// ```
    #[inline(always)]
    fn div(self, scalar: f64) -> Vector<T> { self.clone() / scalar }
}

impl<T: VectorEntity> Neg for Vector<T> {
    type Output = Vector<T>;
    /// Returns the negative vector
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v = vector_new!(1, 2, 3, 4);
    /// assert_eq!(-v, vector_new!(-1, -2, -3, -4));
    /// ```
    #[inline(always)]
    fn neg(mut self) -> Vector<T> {
        self.iter_mut().for_each(move |a| *a = -*a);
        self
    }
}

impl<T: VectorEntity> Neg for &Vector<T> {
    type Output = Vector<T>;
    /// Returns the negative vector
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v = vector_new!(1, 2, 3, 4);
    /// assert_eq!(-&v, vector_new!(-1, -2, -3, -4));
    /// ```
    #[inline(always)]
    fn neg(self) -> Vector<T> { -self.clone() }
}

impl<T: VectorEntity> RemAssign<&Vector<T>> for Vector<T> {
    /// Hadamard product
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let mut v = vector_new!(1, 2, 3, 4);
    /// v %= &vector_new!(8, 7, 6, 5);
    /// assert_eq!(v, vector_new!(8, 14, 18, 20));
    /// ```
    #[inline(always)]
    fn rem_assign(&mut self, other: &Vector<T>) {
        self.iter_mut().zip(other).for_each(move |(a, b)| *a *= b);
    }
}

impl<T: VectorEntity> Rem<&Vector<T>> for Vector<T> {
    type Output = Vector<T>;
    /// Hadamard product
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v0 = vector_new!(1, 2, 3, 4);
    /// let v1 = vector_new!(8, 7, 6, 5);
    /// assert_eq!(v0 % &v1, vector_new!(8, 14, 18, 20));
    /// ```
    #[inline(always)]
    fn rem(mut self, other: &Vector<T>) -> Vector<T> {
        self %= &other;
        self
    }
}

impl<T: VectorEntity> Rem<&Vector<T>> for &Vector<T> {
    type Output = Vector<T>;
    /// Hadamard product
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v0 = vector_new!(1, 2, 3, 4);
    /// let v1 = vector_new!(8, 7, 6, 5);
    /// assert_eq!(&v0 % &v1, vector_new!(8, 14, 18, 20));
    /// ```
    #[inline(always)]
    fn rem(self, other: &Vector<T>) -> Vector<T> { self.clone() % other }
}

impl<T: VectorEntity> Rem<Vector<T>> for &Vector<T> {
    type Output = Vector<T>;
    /// Hadamard product
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v0 = vector_new!(1, 2, 3, 4);
    /// let v1 = vector_new!(8, 7, 6, 5);
    /// assert_eq!(&v0 % v1, vector_new!(8, 14, 18, 20));
    /// ```
    #[inline(always)]
    fn rem(self, other: Vector<T>) -> Vector<T> { other % self }
}

impl<T: VectorEntity> Rem<Vector<T>> for Vector<T> {
    type Output = Vector<T>;
    /// Hadamard product
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v0 = vector_new!(1, 2, 3, 4);
    /// let v1 = vector_new!(8, 7, 6, 5);
    /// assert_eq!(v0 % v1, vector_new!(8, 14, 18, 20));
    /// ```
    #[inline(always)]
    fn rem(self, other: Vector<T>) -> Vector<T> { self % &other }
}

impl<T: VectorEntity> PartialOrd for Vector<T> {
    /// expression order
    /// ```
    /// # use truck_geometry::*;
    /// let v0 = vector_new!(1, 0, 0, 0);
    /// let v1 = vector_new!(0, 100, 0, 0);
    /// let v2 = vector_new!(1, 0, 1, 0);
    /// assert!(v0 == v0);
    /// assert!(v0 > v1);
    /// assert!(v0 < v2);
    /// ```
    #[inline(always)]
    fn partial_cmp(&self, rhs: &Vector<T>) -> Option<Ordering> {
        let mut res = None;
        for (a, b) in self.iter().zip(rhs) {
            res = a.partial_cmp(b);
            if res != Some(Ordering::Equal) {
                return res;
            }
        }
        res
    }
}

impl<T: VectorEntity> Index<usize> for Vector<T> {
    type Output = f64;
    #[inline(always)]
    fn index(&self, idx: usize) -> &f64 { &self.as_ref()[idx] }
}

impl<T: VectorEntity> IndexMut<usize> for Vector<T> {
    #[inline(always)]
    fn index_mut(&mut self, idx: usize) -> &mut f64 { &mut self.as_mut()[idx] }
}

impl<T: VectorEntity> Tolerance for Vector<T> {
    /// The components of each of the two vectors are close enough.
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let eps = f64::TOLERANCE / 2.0;
    /// let v0 = vector_new!(0.0, 0.0);
    /// let v1 = vector_new!(eps, eps);
    /// let v2 = vector_new!(1.0, 0.0);
    /// let v3 = vector_new!(0.0, 1.0);
    /// assert!(v0.near(&v1));
    /// assert!(!v0.near(&v2));
    /// assert!(!v0.near(&v3));
    /// ```
    #[inline(always)]
    fn near(&self, other: &Self) -> bool {
        self.iter()
            .zip(other.as_ref())
            .fold(true, |sum, (a, b)| sum && a.near(&b))
    }

    /// The components of each of the two vectors are close enough.
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let eps = f64::TOLERANCE2 / 2.0;
    /// let v0 = vector_new!(0.0, 0.0);
    /// let v1 = vector_new!(eps, eps);
    /// let v2 = vector_new!(1.0, 0.0);
    /// let v3 = vector_new!(0.0, 1.0);
    /// assert!(v0.near(&v1));
    /// assert!(!v0.near(&v2));
    /// assert!(!v0.near(&v3));
    /// ```
    #[inline(always)]
    fn near2(&self, other: &Self) -> bool {
        self.iter()
            .zip(other.as_ref())
            .fold(true, |sum, (a, b)| sum && a.near2(&b))
    }
}

impl<T: VectorEntity> Origin for Vector<T> {
    const ORIGIN: Self = Self(T::ORIGIN);
}

impl<T: VectorEntity> Vector<T> {
    /// Creates a vector whose components are 0.0.
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// assert_eq!(Vector4::zero(), vector_new!(0, 0, 0, 0));
    /// ```
    #[inline(always)]
    pub fn zero() -> Self { Self::ORIGIN }

    /// square of norm
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// assert_eq!(vector_new!(1, 2, 3, 4).norm2(), 30.0);
    /// ```
    #[inline(always)]
    pub fn norm2(&self) -> f64 { self * self }

    /// norm
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// assert_eq!(vector_new!(3, 4).norm(), 5.0);
    /// ```
    pub fn norm(&self) -> f64 { self.norm2().sqrt() }

    /// Returns the cosine of the angle of the two vectors
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let vec0 = vector_new!(1, 2, 3, 4);
    /// let vec1 = vector_new!(-3, 4, 2, 1);
    /// assert_eq!(vec0.cos_angle(&vec1), 0.5);
    /// ```
    /// # Remarks
    /// If the norm of `self` or `other` is zero, then returns `std::f64::NAN`.
    /// ```
    /// # use truck_geometry::*;
    /// assert!(Vector4::zero().cos_angle(&vector_new!(1, 2, 3, 4)).is_nan());
    /// ```
    #[inline(always)]
    pub fn cos_angle(&self, other: &Self) -> f64 { (self * other) / (self.norm() * other.norm()) }

    /// Returns the angle of two vectors
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use std::f64::consts::PI;
    ///
    /// let vec0 = vector_new!(1, 2, 3, 4);
    /// let vec1 = vector_new!(-3, 4, 2, 1);
    /// f64::assert_near2(&vec0.angle(&vec1), &(PI / 3.0));
    /// ```
    #[inline(always)]
    pub fn angle(&self, other: &Self) -> f64 { self.cos_angle(other).acos() }
    /// Returns the projection to the plane whose the last component is `1.0`.
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v = vector_new!(8, 4, 6, 2).projection();
    /// assert_eq!(v, vector_new!(4, 2, 3, 1));
    /// ```
    #[inline(always)]
    pub fn projection(&self) -> Self { self / self[self.len() - 1] }

    /// Returns the derivation of the projected curve.
    ///
    /// For a curve c(t) = (c_0(t), c_1(t), c_2(t), c_3(t)), returns the derivation
    /// of the projected curve (c_0 / c_3, c_1 / c_3, c_2 / c_3, 1.0).
    /// # Arguments
    /// * `self` - the point of the curve c(t)
    /// * `der` - the derivation c'(t) of the curve
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// // calculate the derivation at t = 1.5
    /// let t = 1.5;
    /// // the curve: c(t) = (t^2, t^3, t^4, t)
    /// let pt = vector_new!(t * t, t * t * t, t * t * t * t, t);
    /// // the derivation: c'(t) = (2t, 3t^2, 4t^3, 1)
    /// let der = vector_new!(2.0 * t, 3.0 * t * t, 4.0 * t * t * t, 1.0);
    /// // the projected curve: \bar{c}(t) = (t, t^2, t^3, 1)
    /// // the derivation of the proj'ed curve: \bar{c}'(t) = (1, 2t, 3t^2, 0)
    /// let ans = vector_new!(1.0, 2.0 * t, 3.0 * t * t, 0.0);
    /// assert_eq!(pt.derivation_projection(&der), ans);
    /// ```
    #[inline(always)]
    pub fn derivation_projection(&self, der: &Self) -> Self {
        let self_last = self[self.len() - 1];
        let coef = der[der.len() - 1] / self_last / self_last;
        (der / self_last) - (self * coef)
    }

    /// Returns the derivation of the projected curve.
    ///
    /// For a curve c(t) = (c_0(t), c_1(t), c_2(t), c_3(t)), returns the 2nd ordered derivation
    /// of the projected curve (c_0 / c_3, c_1 / c_3, c_2 / c_3, 1.0).
    /// # Arguments
    /// * `self` - the point of the curve c(t)
    /// * `der` - the derivation c'(t) of the curve
    /// * `der2` - the 2nd ordered derivation c''(t) of the curve
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// // calculate the derivation at t = 1.5
    /// let t = 1.5;
    /// // the curve: c(t) = (t^2, t^3, t^4, t)
    /// let pt = vector_new!(t * t, t * t * t, t * t * t * t, t);
    /// // the derivation: c'(t) = (2t, 3t^2, 4t^3, 1)
    /// let der = vector_new!(2.0 * t, 3.0 * t * t, 4.0 * t * t * t, 1.0);
    /// // the 2nd ord. deri.: c''(t) = (2, 6t, 12t^2, 0)
    /// let der2 = vector_new!(2.0, 6.0 * t, 12.0 * t * t, 0.0);
    /// // the projected curve: \bar{c}(t) = (t, t^2, t^3, 1)
    /// // the derivation of the proj'ed curve: \bar{c}'(t) = (1, 2t, 3t^2, 0)
    /// // the 2nd ord. deri. of the proj'ed curve: \bar{c}''(t) = (0, 2, 6t, 0)
    /// let ans = vector_new!(0.0, 2.0, 6.0 * t, 0.0);
    /// assert_eq!(pt.derivation2_projection(&der, &der2), ans);
    /// ```
    #[inline(always)]
    pub fn derivation2_projection(&self, der: &Self, der2: &Self) -> Self {
        let self_last = self[self.len() - 1];
        let self_last2 = self_last * self_last;
        let der_last = der[der.len() - 1];
        let coef1 = 2.0 * der_last / self_last2;
        let coef2 = (2.0 * der_last * der_last - der2[der2.len() - 1] * self_last)
            / (self_last * self_last2);
        (der2 / self_last) - (der * coef1) + (self * coef2)
    }
}

impl BitXor<&Vector4> for &Vector4 {
    type Output = Vector4;

    /// cross product for the first three componets.  
    /// The 3rd component is the norm of the above three components.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v0 = Vector4::new3(1.0, 0.0, 0.0);
    /// let v1 = Vector4::new3(0.0, 1.0, 0.0);
    /// let v = &v0 ^ &v1;
    /// assert_eq!(v, vector_new!(0.0, 0.0, 1.0, 1.0));
    /// ```
    #[inline(always)]
    fn bitxor(self, other: &Vector4) -> Vector4 {
        let x = self[1] * other[2] - self[2] * other[1];
        let y = self[2] * other[0] - self[0] * other[2];
        let z = self[0] * other[1] - self[1] * other[0];
        vector_new!(x, y, z, (x * x + y * y + z * z).sqrt())
    }
}

impl BitXor<&Vector3> for &Vector3 {
    type Output = Vector3;
    /// cross product
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v0 = vector_new!(1, 2, 3);
    /// let v1 = vector_new!(2, 4, 7);
    /// assert_eq!(&v0 ^ &v1, vector_new!(2, -1, 0));
    /// ```
    #[inline(always)]
    fn bitxor(self, other: &Vector3) -> Vector3 {
        let x = self[1] * other[2] - self[2] * other[1];
        let y = self[2] * other[0] - self[0] * other[2];
        let z = self[0] * other[1] - self[1] * other[0];
        vector_new!(x, y, z)
    }
}

impl BitXor<&Vector2> for &Vector2 {
    type Output = f64;
    /// Returns the signed area of a parallelogram stretched by two vectors.
    /// # Examples
    /// ```
    /// # use truck_geometry::*;
    /// let v0 = vector_new!(2, 3);
    /// let v1 = vector_new!(4, 7);
    /// assert_eq!(&v0 ^ &v1, 2.0);
    /// ```
    #[inline(always)]
    fn bitxor(self, other: &Vector2) -> f64 { self[0] * other[1] - self[1] * other[0] }
}

macro_rules! impl_bitxor_others {
    ($classname: ident) => {
        impl BitXor<&$classname> for $classname {
            type Output = <&'static $classname as BitXor<&'static $classname>>::Output;
            /// cross product
            /// # Examples
            /// ```
            /// # use truck_geometry::*;
            /// let v0 = vector_new!(1, 2, 3);
            /// let v1 = vector_new!(2, 4, 7);
            /// assert_eq!(v0 ^ &v1, vector_new!(2, -1, 0));
            /// ```
            #[inline(always)]
            fn bitxor(self, other: &$classname) -> Self::Output { &self ^ other }
        }

        impl BitXor<$classname> for &$classname {
            type Output = <&'static $classname as BitXor<&'static $classname>>::Output;
            /// cross product
            /// # Examples
            /// ```
            /// # use truck_geometry::*;
            /// let v0 = vector_new!(1, 2, 3);
            /// let v1 = vector_new!(2, 4, 7);
            /// assert_eq!(&v0 ^ v1, vector_new!(2, -1, 0));
            /// ```
            #[inline(always)]
            fn bitxor(self, other: $classname) -> Self::Output { self ^ &other }
        }

        impl BitXor<$classname> for $classname {
            type Output = <&'static $classname as BitXor<&'static $classname>>::Output;
            /// cross product
            /// # Examples
            /// ```
            /// # use truck_geometry::*;
            /// let v0 = vector_new!(1, 2, 3);
            /// let v1 = vector_new!(2, 4, 7);
            /// assert_eq!(v0 ^ v1, vector_new!(2, -1, 0));
            /// ```
            #[inline(always)]
            fn bitxor(self, other: $classname) -> Self::Output { &self ^ &other }
        }
    };
}

impl_bitxor_others!(Vector4);
impl_bitxor_others!(Vector3);
impl_bitxor_others!(Vector2);

macro_rules! impl_bitxor_assign {
    ($classname: ty) => {
        impl BitXorAssign<&$classname> for $classname {
            /// cross product
            /// # Examples
            /// ```
            /// # use truck_geometry::*;
            /// let mut v = vector_new!(1, 2, 3);
            /// v ^= &vector_new!(2, 4, 7);
            /// assert_eq!(v, vector_new!(2, -1, 0));
            /// ```
            #[inline(always)]
            fn bitxor_assign(&mut self, rhs: &$classname) { *self = &*self ^ rhs; }
        }

        impl BitXorAssign<$classname> for $classname {
            /// cross product
            /// # Examples
            /// ```
            /// # use truck_geometry::*;
            /// let mut v = vector_new!(1, 2, 3);
            /// v ^= vector_new!(2, 4, 7);
            /// assert_eq!(v, vector_new!(2, -1, 0));
            /// ```
            #[inline(always)]
            fn bitxor_assign(&mut self, rhs: $classname) { self.bitxor_assign(&rhs); }
        }
    };
}

impl_bitxor_assign!(Vector4);
impl_bitxor_assign!(Vector3);

macro_rules! impl_lesser_convert {
    ($higher_vector: ty, $lesser_vector: ty) => {
        impl From<$higher_vector> for $lesser_vector {
            /// canonical projection from higher dimensional vector to lower dimensional
            #[inline(always)]
            fn from(vector: $higher_vector) -> $lesser_vector {
                let mut res = <$lesser_vector>::zero();
                res.iter_mut().zip(&vector).for_each(move |(a, b)| *a = *b);
                res
            }
        }
    };
}

impl_lesser_convert!(Vector4, Vector3);
impl_lesser_convert!(Vector4, Vector2);
impl_lesser_convert!(Vector3, Vector2);

impl std::fmt::Display for Vector4 {
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

impl Vector4 {
    /// construct a vector whose 4th component is 1.0.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v = Vector::new3(1.0, 2.0, 3.0);
    /// assert_eq!(v, vector_new!(1.0, 2.0, 3.0, 1.0));
    /// ```
    #[inline(always)]
    pub fn new3<T: Into<f64>>(x: T, y: T, z: T) -> Vector4 {
        vector_new!(x, y, z, 1)
    }
}

impl Vector3 {
    #[inline(always)]
    pub fn volume(&self, v0: &Vector3, v1: &Vector3) -> f64 {
        self[0] * (v0[1] * v1[2] - v0[2] * v1[1])
            + self[1] * (v0[2] * v1[0] - v0[0] * v1[2])
            + self[2] * (v0[0] * v1[1] - v0[1] * v1[0])
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
        Some(vector_new!(x, y, z))
    }
}
