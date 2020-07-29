use crate::*;
use crate::errors::Error;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::iter::FromIterator;
use std::ops::*;

macro_rules! inverse_array {
    ([] $($x: expr,) *) => { [$($x), *] };
    ([$first: expr, $($x: expr,) *] $($y: expr,) *) => {
        inverse_array!([$($x,) *] $first, $($y,)*)
    };
}

macro_rules! impl_entity_array {
    ($a: expr, $b: expr) => {};
    ($dim: expr, $($num: expr),*) => {
        impl EntityArray<f64> for [f64; $dim] {
            const ORIGIN: Self = [0.0; $dim];
            const INFINITY: Self = [f64::INFINITY; $dim];
            const NEG_INFINITY: Self = [f64::NEG_INFINITY; $dim];
            #[inline(always)]
            fn from_iter<I: IntoIterator<Item=f64>> (iter: I) -> Self {
                let mut iter = iter.into_iter();
                [$({
                    $num;
                    iter.next().expect(&format!("{}", Error::TooShortIterator))
                }),*]
            }
        }
        impl From<Vector<[f64; $dim]>> for [f64; $dim] {
            #[inline(always)]
            fn from(vec: Vector<[f64; $dim]>) -> [f64; $dim] { vec.0 }
        }
        impl From<&Vector<[f64; $dim]>> for [f64; $dim] {
            #[inline(always)]
            fn from(vec: &Vector<[f64; $dim]>) -> [f64; $dim] { vec.0.clone() }
        }
        impl From<Vector<[f64; $dim]>> for [f32; $dim] {
            #[inline(always)]
            fn from(vec: Vector<[f64; $dim]>) -> [f32; $dim] {
                inverse_array!([$(vec[$num] as f32,)*])
            }
        }
        impl From<&Vector<[f64; $dim]>> for [f32; $dim] {
            #[inline(always)]
            fn from(vec: &Vector<[f64; $dim]>) -> [f32; $dim] {
                inverse_array!([$(vec[$num] as f32,)*])
            }
        }

        impl_entity_array!($($num),*);
    };
}
impl_entity_array!(
    32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9,
    8, 7, 6, 5, 4, 3, 2, 1, 0
);

/// Creates a new `Vector<[f64; N]>`.
/// # Arguments
/// - `$x`: Variadic arguments. `Into<f64>` has to be implemented.
/// ```
/// use truck_geometry::*;
/// fn type_of<T>(_: &T) -> &'static str { std::any::type_name::<T>() }
/// let v = vector!(1, 2.0, 3, -4.0_f32);
/// assert_eq!(type_of(&v), "truck_geometry::Vector<[f64; 4]>");
/// assert_eq!(v[0], 1.0);
/// assert_eq!(v[1], 2.0);
/// assert_eq!(v[2], 3.0);
/// assert_eq!(v[3], -4.0);
/// ```
#[macro_export]
macro_rules! vector {
    ($($x: expr), *) => { $crate::Vector::from([$(Into::<f64>::into($x)), *]) };
    ($($x: expr,) *) => { $crate::Vector::from([$(Into::<f64>::into($x)), *]) };
    ($x: expr; $n: expr) => { $crate::Vector::from([Into::<f64>::into($x); $n]) };
}

/// Creates a new vector by the homogeneous coordinate.
/// i.e. creates a `n + 1`-dim vector with the final component is 1.0.
/// # Examples
/// ```
/// use truck_geometry::*;
/// assert_eq!(rvector!(1, 2, 3), vector!(1, 2, 3, 1));
/// ```
#[macro_export]
macro_rules! rvector {
    ($($x: expr), *) => { vector!($($x), *, 1) };
}

impl<T: EntityArray<f64>> Deref for Vector<T> {
    type Target = [f64];
    #[inline(always)]
    fn deref(&self) -> &[f64] { self.0.as_ref() }
}

impl<T: EntityArray<f64>> DerefMut for Vector<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [f64] { self.0.as_mut() }
}

impl<T: EntityArray<f64>> AsRef<[f64]> for Vector<T> {
    #[inline(always)]
    fn as_ref(&self) -> &[f64] { self.0.as_ref() }
}

impl<T: EntityArray<f64>> AsMut<[f64]> for Vector<T> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut [f64] { self.0.as_mut() }
}

impl<'a, T: EntityArray<f64>> IntoIterator for &'a Vector<T> {
    type Item = &'a f64;
    type IntoIter = std::slice::Iter<'a, f64>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl<'a, T: EntityArray<f64>> IntoIterator for &'a mut Vector<T> {
    type Item = &'a mut f64;
    type IntoIter = std::slice::IterMut<'a, f64>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter { self.iter_mut() }
}

impl<T> From<T> for Vector<T> {
    #[inline(always)]
    fn from(arr: T) -> Vector<T> { Vector(arr) }
}

impl<T: Clone> From<&T> for Vector<T> {
    #[inline(always)]
    fn from(arr: &T) -> Vector<T> { Vector(arr.clone()) }
}

impl<T: EntityArray<f64>> FromIterator<f64> for Vector<T> {
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = f64>>(iter: I) -> Self { Vector(T::from_iter(iter)) }
}

impl<'a, T: EntityArray<f64>> FromIterator<&'a f64> for Vector<T> {
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = &'a f64>>(iter: I) -> Self {
        iter.into_iter().map(|x| *x).collect()
    }
}

impl<T: EntityArray<f64>> AddAssign<&Vector<T>> for Vector<T> {
    /// Adds and assigns for each components
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut v = vector!(1, 2, 3, 4);
    /// v += &vector!(1, -3, -2, 3);
    /// assert_eq!(v, vector!(2, -1, 1, 7));
    /// ```
    #[inline(always)]
    fn add_assign(&mut self, other: &Vector<T>) {
        self.iter_mut().zip(other).for_each(move |(a, b)| *a += b);
    }
}

impl<T: EntityArray<f64>> AddAssign<Vector<T>> for Vector<T> {
    /// Adds and assigns for each components
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut v = vector!(1, 2, 3, 4);
    /// v += vector!(1, -3, -2, 3);
    /// assert_eq!(v, vector!(2, -1, 1, 7));
    /// ```
    #[inline(always)]
    fn add_assign(&mut self, other: Self) { *self += &other }
}

impl<T: EntityArray<f64>> Add<&Vector<T>> for Vector<T> {
    type Output = Vector<T>;
    /// Adds each components
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v0 = vector!(1, 2, 3, 4);
    /// let v1 = vector!(1, -3, -2, 3);
    /// assert_eq!(v0 + &v1, vector!(2, -1, 1, 7));
    /// ```
    #[inline(always)]
    fn add(mut self, other: &Vector<T>) -> Vector<T> {
        self += other;
        self
    }
}

impl<T: EntityArray<f64>> Add<&Vector<T>> for &Vector<T> {
    type Output = Vector<T>;
    /// Adds each components
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v0 = vector!(1, 2, 3, 4);
    /// let v1 = vector!(1, -3, -2, 3);
    /// assert_eq!(&v0 + &v1, vector!(2, -1, 1, 7));
    /// ```
    #[inline(always)]
    fn add(self, other: &Vector<T>) -> Vector<T> { self.clone() + other }
}

impl<T: EntityArray<f64>> Add<Vector<T>> for &Vector<T> {
    type Output = Vector<T>;
    /// Adds each components
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v0 = vector!(1, 2, 3, 4);
    /// let v1 = vector!(1, -3, -2, 3);
    /// assert_eq!(&v0 + v1, vector!(2, -1, 1, 7));
    /// ```
    #[inline(always)]
    fn add(self, other: Vector<T>) -> Vector<T> { other + self }
}

impl<T: EntityArray<f64>> Add<Vector<T>> for Vector<T> {
    type Output = Vector<T>;
    /// Adds each components
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v0 = vector!(1, 2, 3, 4);
    /// let v1 = vector!(1, -3, -2, 3);
    /// assert_eq!(v0 + v1, vector!(2, -1, 1, 7));
    /// ```
    #[inline(always)]
    fn add(self, other: Vector<T>) -> Vector<T> { self + &other }
}

impl<T: EntityArray<f64>> SubAssign<&Vector<T>> for Vector<T> {
    /// Subtracts and assigns for each components
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut v = vector!(1, 2, 3, 4);
    /// v -= &vector!(1, -3, -2, 3);
    /// assert_eq!(v, vector!(0, 5, 5, 1));
    /// ```
    #[inline(always)]
    fn sub_assign(&mut self, other: &Vector<T>) {
        self.iter_mut().zip(other).for_each(move |(a, b)| *a -= b);
    }
}

impl<T: EntityArray<f64>> SubAssign<Vector<T>> for Vector<T> {
    /// Subtracts and assigns for each components
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut v = vector!(1, 2, 3, 4);
    /// v -= vector!(1, -3, -2, 3);
    /// assert_eq!(v, vector!(0, 5, 5, 1));
    /// ```
    #[inline(always)]
    fn sub_assign(&mut self, other: Self) { *self -= &other }
}

impl<T: EntityArray<f64>> Sub<&Vector<T>> for Vector<T> {
    type Output = Vector<T>;
    /// Subtracts each components
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v0 = vector!(1, 2, 3, 4);
    /// let v1 = vector!(1, -3, -2, 3);
    /// assert_eq!(v0 - &v1, vector!(0, 5, 5, 1));
    /// ```
    #[inline(always)]
    fn sub(mut self, other: &Vector<T>) -> Vector<T> {
        self -= other;
        self
    }
}

impl<T: EntityArray<f64>> Sub<&Vector<T>> for &Vector<T> {
    type Output = Vector<T>;
    /// Subtracts each components
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v0 = vector!(1, 2, 3, 4);
    /// let v1 = vector!(1, -3, -2, 3);
    /// assert_eq!(&v0 - &v1, vector!(0, 5, 5, 1));
    /// ```
    #[inline(always)]
    fn sub(self, other: &Vector<T>) -> Vector<T> { self.clone() - other }
}

impl<T: EntityArray<f64>> Sub<Vector<T>> for &Vector<T> {
    type Output = Vector<T>;
    /// Subtracts each components
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v0 = vector!(1, 2, 3, 4);
    /// let v1 = vector!(1, -3, -2, 3);
    /// assert_eq!(&v0 - v1, vector!(0, 5, 5, 1));
    /// ```
    #[inline(always)]
    fn sub(self, other: Vector<T>) -> Vector<T> { -(other - self) }
}

impl<T: EntityArray<f64>> Sub<Vector<T>> for Vector<T> {
    type Output = Vector<T>;
    /// Subtracts each components
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v0 = vector!(1, 2, 3, 4);
    /// let v1 = vector!(1, -3, -2, 3);
    /// assert_eq!(v0 - v1, vector!(0, 5, 5, 1));
    /// ```
    #[inline(always)]
    fn sub(self, other: Vector<T>) -> Vector<T> { self - &other }
}

impl<T: EntityArray<f64>> MulAssign<f64> for Vector<T> {
    /// Multiplies and assigns for each components
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut v = vector!(1, 2, 3, 4);
    /// v *= 2.0;
    /// assert_eq!(v, vector!(2, 4, 6, 8));
    /// ```
    #[inline(always)]
    fn mul_assign(&mut self, rhs: f64) { self.iter_mut().for_each(move |a| *a *= rhs); }
}

impl<T: EntityArray<f64>> Mul<f64> for Vector<T> {
    type Output = Vector<T>;
    /// Multiplies and assigns for each components
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v = vector!(1, 2, 3, 4);
    /// assert_eq!(v * 2.0, vector!(2, 4, 6, 8));
    /// ```
    #[inline(always)]
    fn mul(mut self, scalar: f64) -> Vector<T> {
        self *= scalar;
        self
    }
}

impl<'a, T: EntityArray<f64>> Mul<f64> for &'a Vector<T> {
    type Output = Vector<T>;
    /// Multiplies and assigns for each components
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v = vector!(1, 2, 3, 4);
    /// assert_eq!(&v * 2.0, vector!(2, 4, 6, 8));
    /// ```
    #[inline(always)]
    fn mul(self, scalar: f64) -> Vector<T> { self.clone() * scalar }
}

impl<T: EntityArray<f64>> Mul<&Vector<T>> for f64 {
    type Output = Vector<T>;
    /// Multiplies and assigns for each components
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v = vector!(1, 2, 3, 4);
    /// assert_eq!(2.0 * &v, vector!(2, 4, 6, 8));
    /// ```
    #[inline(always)]
    fn mul(self, vector: &Vector<T>) -> Vector<T> { vector * self }
}

impl<T: EntityArray<f64>> Mul<Vector<T>> for f64 {
    type Output = Vector<T>;
    /// Multiplies and assigns for each components
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v = vector!(1, 2, 3, 4);
    /// assert_eq!(2.0 * v, vector!(2, 4, 6, 8));
    /// ```
    #[inline(always)]
    fn mul(self, vector: Vector<T>) -> Vector<T> { vector * self }
}

impl<T: EntityArray<f64>> Mul<&Vector<T>> for &Vector<T> {
    type Output = f64;
    /// inner product
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v0 = vector!(1, 2, 3, 4);
    /// let v1 = vector!(1, -3, -2, 3);
    /// assert_eq!(&v0 * &v1, 1.0);
    /// ```
    #[inline(always)]
    fn mul(self, other: &Vector<T>) -> f64 { self.iter().zip(other).map(move |(a, b)| a * b).sum() }
}

impl<T: EntityArray<f64>> Mul<Vector<T>> for &Vector<T> {
    type Output = f64;
    /// inner product
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v0 = vector!(1, 2, 3, 4);
    /// let v1 = vector!(1, -3, -2, 3);
    /// assert_eq!(&v0 * v1, 1.0);
    /// ```
    #[inline(always)]
    fn mul(self, other: Vector<T>) -> f64 { self * &other }
}

impl<T: EntityArray<f64>> Mul<&Vector<T>> for Vector<T> {
    type Output = f64;
    /// inner product
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v0 = vector!(1, 2, 3, 4);
    /// let v1 = vector!(1, -3, -2, 3);
    /// assert_eq!(v0 * &v1, 1.0);
    /// ```
    #[inline(always)]
    fn mul(self, other: &Vector<T>) -> f64 { &self * other }
}

impl<T: EntityArray<f64>> Mul<Vector<T>> for Vector<T> {
    type Output = f64;
    /// inner product
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v0 = vector!(1, 2, 3, 4);
    /// let v1 = vector!(1, -3, -2, 3);
    /// assert_eq!(v0 * v1, 1.0);
    /// ```
    #[inline(always)]
    fn mul(self, other: Vector<T>) -> f64 { &self * &other }
}

impl<T: EntityArray<f64>> DivAssign<f64> for Vector<T> {
    /// Divides and assigns for each components
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut v = vector!(1.0, 2.0, 3.0, 4.0);
    /// v /= 2.0;
    /// assert_eq!(v, vector!(0.5, 1.0, 1.5, 2.0));
    /// ```
    #[inline(always)]
    fn div_assign(&mut self, rhs: f64) { self.iter_mut().for_each(move |a| *a /= rhs); }
}

impl<T: EntityArray<f64>> Div<f64> for Vector<T> {
    type Output = Vector<T>;
    /// Divides for each components
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v = vector!(1.0, 2.0, 3.0, 4.0);
    /// assert_eq!(v / 2.0, vector!(0.5, 1.0, 1.5, 2.0));
    /// ```
    #[inline(always)]
    fn div(mut self, scalar: f64) -> Vector<T> {
        self /= scalar;
        self
    }
}

impl<T: EntityArray<f64>> Div<f64> for &Vector<T> {
    type Output = Vector<T>;
    /// Divides for each components
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v = vector!(1.0, 2.0, 3.0, 4.0);
    /// assert_eq!(&v / 2.0, vector!(0.5, 1.0, 1.5, 2.0));
    /// ```
    #[inline(always)]
    fn div(self, scalar: f64) -> Vector<T> { self.clone() / scalar }
}

impl<T: EntityArray<f64>> Neg for Vector<T> {
    type Output = Vector<T>;
    /// Returns the negative vector
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v = vector!(1, 2, 3, 4);
    /// assert_eq!(-v, vector!(-1, -2, -3, -4));
    /// ```
    #[inline(always)]
    fn neg(mut self) -> Vector<T> {
        self.iter_mut().for_each(move |a| *a = -*a);
        self
    }
}

impl<T: EntityArray<f64>> Neg for &Vector<T> {
    type Output = Vector<T>;
    /// Returns the negative vector
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v = vector!(1, 2, 3, 4);
    /// assert_eq!(-&v, vector!(-1, -2, -3, -4));
    /// ```
    #[inline(always)]
    fn neg(self) -> Vector<T> { -self.clone() }
}

impl<T: EntityArray<f64>> RemAssign<&Vector<T>> for Vector<T> {
    /// Hadamard product
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut v = vector!(1, 2, 3, 4);
    /// v %= &vector!(8, 7, 6, 5);
    /// assert_eq!(v, vector!(8, 14, 18, 20));
    /// ```
    #[inline(always)]
    fn rem_assign(&mut self, other: &Vector<T>) {
        self.iter_mut().zip(other).for_each(move |(a, b)| *a *= b);
    }
}

impl<T: EntityArray<f64>> Rem<&Vector<T>> for Vector<T> {
    type Output = Vector<T>;
    /// Hadamard product
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v0 = vector!(1, 2, 3, 4);
    /// let v1 = vector!(8, 7, 6, 5);
    /// assert_eq!(v0 % &v1, vector!(8, 14, 18, 20));
    /// ```
    #[inline(always)]
    fn rem(mut self, other: &Vector<T>) -> Vector<T> {
        self %= &other;
        self
    }
}

impl<T: EntityArray<f64>> Rem<&Vector<T>> for &Vector<T> {
    type Output = Vector<T>;
    /// Hadamard product
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v0 = vector!(1, 2, 3, 4);
    /// let v1 = vector!(8, 7, 6, 5);
    /// assert_eq!(&v0 % &v1, vector!(8, 14, 18, 20));
    /// ```
    #[inline(always)]
    fn rem(self, other: &Vector<T>) -> Vector<T> { self.clone() % other }
}

impl<T: EntityArray<f64>> Rem<Vector<T>> for &Vector<T> {
    type Output = Vector<T>;
    /// Hadamard product
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v0 = vector!(1, 2, 3, 4);
    /// let v1 = vector!(8, 7, 6, 5);
    /// assert_eq!(&v0 % v1, vector!(8, 14, 18, 20));
    /// ```
    #[inline(always)]
    fn rem(self, other: Vector<T>) -> Vector<T> { other % self }
}

impl<T: EntityArray<f64>> Rem<Vector<T>> for Vector<T> {
    type Output = Vector<T>;
    /// Hadamard product
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v0 = vector!(1, 2, 3, 4);
    /// let v1 = vector!(8, 7, 6, 5);
    /// assert_eq!(v0 % v1, vector!(8, 14, 18, 20));
    /// ```
    #[inline(always)]
    fn rem(self, other: Vector<T>) -> Vector<T> { self % &other }
}

impl<T: EntityArray<f64>> PartialOrd for Vector<T> {
    /// expression order
    /// ```
    /// use truck_geometry::*;
    /// let v0 = vector!(1, 0, 0, 0);
    /// let v1 = vector!(0, 100, 0, 0);
    /// let v2 = vector!(1, 0, 1, 0);
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

impl<T: EntityArray<f64>> Index<usize> for Vector<T> {
    type Output = f64;
    #[inline(always)]
    fn index(&self, idx: usize) -> &f64 { &self.as_ref()[idx] }
}

impl<T: EntityArray<f64>> IndexMut<usize> for Vector<T> {
    #[inline(always)]
    fn index_mut(&mut self, idx: usize) -> &mut f64 { &mut self.as_mut()[idx] }
}

impl<T: EntityArray<f64>> Tolerance for Vector<T> {
    /// The components of each of the two vectors are close enough.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let eps = f64::TOLERANCE / 2.0;
    /// let v0 = vector!(0.0, 0.0);
    /// let v1 = vector!(eps, eps);
    /// let v2 = vector!(1.0, 0.0);
    /// let v3 = vector!(0.0, 1.0);
    /// assert!(v0.near(&v1));
    /// assert!(!v0.near(&v2));
    /// assert!(!v0.near(&v3));
    /// ```
    #[inline(always)]
    fn near(&self, other: &Self) -> bool { self.iter().zip(other).all(move |(a, b)| a.near(b)) }

    /// The components of each of the two vectors are close enough.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let eps = f64::TOLERANCE / 2.0;
    /// let eps2 = f64::TOLERANCE2 / 2.0;
    /// let v0 = vector!(0.0, 0.0);
    /// let v1 = vector!(eps2, eps2);
    /// let v2 = vector!(eps, 0.0);
    /// let v3 = vector!(0.0, eps);
    /// assert!(v0.near(&v1));
    /// assert!(!v0.near2(&v2));
    /// assert!(!v0.near2(&v3));
    /// ```
    #[inline(always)]
    fn near2(&self, other: &Self) -> bool { self.iter().zip(other).all(move |(a, b)| a.near2(&b)) }
}

impl<T: EntityArray<f64>> Origin for Vector<T> {
    const ORIGIN: Self = Self(T::ORIGIN);
    fn round_by_tolerance(&mut self) -> &mut Self {
        self.iter_mut().for_each(|val| {
            val.round_by_tolerance();
        });
        self
    }
}

impl<T: EntityArray<f64>> Vector<T> {
    /// Creates a vector whose components are 0.0.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// assert_eq!(Vector4::zero(), vector!(0, 0, 0, 0));
    /// ```
    #[inline(always)]
    pub fn zero() -> Self { Self::ORIGIN }

    /// Creates the `i`th vector of the standard basis.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// assert_eq!(Vector4::std_basis(1), vector!(0, 1, 0, 0));
    /// ```
    #[inline(always)]
    pub fn std_basis(idx: usize) -> Vector<T> {
        let mut res = Self::ORIGIN;
        res[idx] = 1.0;
        res
    }

    /// square of norm
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// assert_eq!(vector!(1, 2, 3, 4).norm2(), 30.0);
    /// ```
    #[inline(always)]
    pub fn norm2(&self) -> f64 { self * self }

    /// norm
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// assert_eq!(vector!(3, 4).norm(), 5.0);
    /// ```
    pub fn norm(&self) -> f64 { self.norm2().sqrt() }

    /// Returns the cosine of the angle of the two vectors
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let vec0 = vector!(1, 2, 3, 4);
    /// let vec1 = vector!(-3, 4, 2, 1);
    /// assert_eq!(vec0.cos_angle(&vec1), 0.5);
    /// ```
    /// # Remarks
    /// If the norm of `self` or `other` is zero, then returns `std::f64::NAN`.
    /// ```
    /// use truck_geometry::*;
    /// assert!(Vector4::zero().cos_angle(&vector!(1, 2, 3, 4)).is_nan());
    /// ```
    #[inline(always)]
    pub fn cos_angle(&self, other: &Self) -> f64 { (self * other) / (self.norm() * other.norm()) }

    /// Returns the angle of two vectors
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// use std::f64::consts::PI;
    ///
    /// let vec0 = vector!(1, 2, 3, 4);
    /// let vec1 = vector!(-3, 4, 2, 1);
    /// f64::assert_near2(&vec0.angle(&vec1), &(PI / 3.0));
    /// ```
    #[inline(always)]
    pub fn angle(&self, other: &Self) -> f64 { self.cos_angle(other).acos() }
    /// Returns the projection to the plane whose the last component is `1.0`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v = vector!(8, 4, 6, 2).rational_projection();
    /// assert_eq!(v, vector!(4, 2, 3, 1));
    /// ```
    #[inline(always)]
    pub fn rational_projection(&self) -> Self { self / self[self.len() - 1] }

    /// Returns the derivation of the rational curve.
    ///
    /// For a curve c(t) = (c_0(t), c_1(t), c_2(t), c_3(t)), returns the derivation
    /// of the projected curve (c_0 / c_3, c_1 / c_3, c_2 / c_3, 1.0).
    /// # Arguments
    /// * `self` - the point of the curve c(t)
    /// * `der` - the derivation c'(t) of the curve
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// // calculate the derivation at t = 1.5
    /// let t = 1.5;
    /// // the curve: c(t) = (t^2, t^3, t^4, t)
    /// let pt = vector!(t * t, t * t * t, t * t * t * t, t);
    /// // the derivation: c'(t) = (2t, 3t^2, 4t^3, 1)
    /// let der = vector!(2.0 * t, 3.0 * t * t, 4.0 * t * t * t, 1.0);
    /// // the projected curve: \bar{c}(t) = (t, t^2, t^3, 1)
    /// // the derivation of the proj'ed curve: \bar{c}'(t) = (1, 2t, 3t^2, 0)
    /// let ans = vector!(1.0, 2.0 * t, 3.0 * t * t, 0.0);
    /// assert_eq!(pt.rational_derivation(&der), ans);
    /// ```
    #[inline(always)]
    pub fn rational_derivation(&self, der: &Self) -> Self {
        let self_last = self[self.len() - 1];
        let coef = der[der.len() - 1] / self_last / self_last;
        (der / self_last) - (self * coef)
    }

    /// Returns the derivation of the rational curve.
    ///
    /// For a curve c(t) = (c_0(t), c_1(t), c_2(t), c_3(t)), returns the 2nd ordered derivation
    /// of the projected curve (c_0 / c_3, c_1 / c_3, c_2 / c_3, 1.0).
    /// # Arguments
    /// * `self` - the point of the curve c(t)
    /// * `der` - the derivation c'(t) of the curve
    /// * `der2` - the 2nd ordered derivation c''(t) of the curve
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// // calculate the derivation at t = 1.5
    /// let t = 1.5;
    /// // the curve: c(t) = (t^2, t^3, t^4, t)
    /// let pt = vector!(t * t, t * t * t, t * t * t * t, t);
    /// // the derivation: c'(t) = (2t, 3t^2, 4t^3, 1)
    /// let der = vector!(2.0 * t, 3.0 * t * t, 4.0 * t * t * t, 1.0);
    /// // the 2nd ord. deri.: c''(t) = (2, 6t, 12t^2, 0)
    /// let der2 = vector!(2.0, 6.0 * t, 12.0 * t * t, 0.0);
    /// // the projected curve: \bar{c}(t) = (t, t^2, t^3, 1)
    /// // the derivation of the proj'ed curve: \bar{c}'(t) = (1, 2t, 3t^2, 0)
    /// // the 2nd ord. deri. of the proj'ed curve: \bar{c}''(t) = (0, 2, 6t, 0)
    /// let ans = vector!(0.0, 2.0, 6.0 * t, 0.0);
    /// assert_eq!(pt.rational_derivation2(&der, &der2), ans);
    /// ```
    #[inline(always)]
    pub fn rational_derivation2(&self, der: &Self, der2: &Self) -> Self {
        let self_last = self[self.len() - 1];
        let self_last2 = self_last * self_last;
        let der_last = der[der.len() - 1];
        let coef1 = 2.0 * der_last / self_last2;
        let coef2 = (2.0 * der_last * der_last - der2[der2.len() - 1] * self_last)
            / (self_last * self_last2);
        (der2 / self_last) - (der * coef1) + (self * coef2)
    }
}

impl BitXor<&Vector3> for &Vector3 {
    type Output = Vector3;
    /// cross product
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v0 = vector!(1, 2, 3);
    /// let v1 = vector!(2, 4, 7);
    /// assert_eq!(&v0 ^ &v1, vector!(2, -1, 0));
    /// ```
    #[inline(always)]
    fn bitxor(self, other: &Vector3) -> Vector3 {
        let x = self[1] * other[2] - self[2] * other[1];
        let y = self[2] * other[0] - self[0] * other[2];
        let z = self[0] * other[1] - self[1] * other[0];
        vector!(x, y, z)
    }
}

impl BitXor<&Vector2> for &Vector2 {
    type Output = f64;
    /// Returns the signed area of a parallelogram stretched by two vectors.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let v0 = vector!(2, 3);
    /// let v1 = vector!(4, 7);
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
            /// use truck_geometry::*;
            /// let v0 = vector!(1, 2, 3);
            /// let v1 = vector!(2, 4, 7);
            /// assert_eq!(v0 ^ &v1, vector!(2, -1, 0));
            /// ```
            #[inline(always)]
            fn bitxor(self, other: &$classname) -> Self::Output { &self ^ other }
        }

        impl BitXor<$classname> for &$classname {
            type Output = <&'static $classname as BitXor<&'static $classname>>::Output;
            /// cross product
            /// # Examples
            /// ```
            /// use truck_geometry::*;
            /// let v0 = vector!(1, 2, 3);
            /// let v1 = vector!(2, 4, 7);
            /// assert_eq!(&v0 ^ v1, vector!(2, -1, 0));
            /// ```
            #[inline(always)]
            fn bitxor(self, other: $classname) -> Self::Output { self ^ &other }
        }

        impl BitXor<$classname> for $classname {
            type Output = <&'static $classname as BitXor<&'static $classname>>::Output;
            /// cross product
            /// # Examples
            /// ```
            /// use truck_geometry::*;
            /// let v0 = vector!(1, 2, 3);
            /// let v1 = vector!(2, 4, 7);
            /// assert_eq!(v0 ^ v1, vector!(2, -1, 0));
            /// ```
            #[inline(always)]
            fn bitxor(self, other: $classname) -> Self::Output { &self ^ &other }
        }
    };
}
impl_bitxor_others!(Vector3);
impl_bitxor_others!(Vector2);

impl BitXorAssign<&Vector3> for Vector3 {
    /// cross product
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut v = vector!(1, 2, 3);
    /// v ^= &vector!(2, 4, 7);
    /// assert_eq!(v, vector!(2, -1, 0));
    /// ```
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: &Vector3) { *self = &*self ^ rhs; }
}

impl BitXorAssign<Vector3> for Vector3 {
    /// cross product
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut v = vector!(1, 2, 3);
    /// v ^= vector!(2, 4, 7);
    /// assert_eq!(v, vector!(2, -1, 0));
    /// ```
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Vector3) { self.bitxor_assign(&rhs); }
}

macro_rules! sub_impl_lesser_convert {
    ($a: expr, $b: expr) => {
        /// Truncates the latter components.
        impl From<Vector<[f64; $a]>> for Vector<[f64; $b]> {
            fn from(vec: Vector<[f64; $a]>) -> Vector<[f64; $b]> {
                Vector::from(<[f64; $b]>::try_from(&vec.0[0..$b]).unwrap())
            }
        }
    };
}

macro_rules! impl_lesser_convert {
    ($a: expr) => {};
    ($a: expr, $($b: expr), *) => {
        $(sub_impl_lesser_convert!($a, $b);)*
        impl_lesser_convert!($($b), *);
    };
}

impl_lesser_convert!(
    32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9,
    8, 7, 6, 5, 4, 3, 2
);

#[test]
fn test_lesser_convert() {
    let vector = vector!(1, 2, 3, 4);
    assert_eq!(vector!(1, 2), Vector2::from(vector));
}

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
