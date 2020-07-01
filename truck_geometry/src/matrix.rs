use crate::{EntityArray, Matrix, Matrix2, Matrix3, Matrix4, Origin, Tolerance, Vector};
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::ops::*;

/// an trait for the multiplication matrix
pub trait Multiplicable<T>: EntityArray<Vector<T>>
where T: EntityArray<f64> {
    #[doc(hidden)]
    fn vec_multiply(vec: &Vector<T>, mat: &Self) -> Vector<T>;
    #[doc(hidden)]
    fn multiply(&self, other: &Self) -> Self;
}

macro_rules! impl_entity_array {
    ($dim: expr) => {
        impl EntityArray<Vector<[f64; $dim]>> for [Vector<[f64; $dim]>; $dim] {
            const ORIGIN: Self = [Vector::<[f64; $dim]>::ORIGIN; $dim];
        }
        impl From<[[f64; $dim]; $dim]> for Matrix<[f64; $dim], [Vector<[f64; $dim]>; $dim]> {
            fn from(arr: [[f64; $dim]; $dim]) -> Self {
                let mut mat = Self::ORIGIN;
                mat.iter_mut().zip(&arr).for_each(|(a, b)| *a = Vector::from(b));
                mat
            }
        }
    };
    ($first: expr, $($latter: expr), *) => {
        impl_entity_array!($first);
        impl_entity_array!($($latter), *);
    };
}
impl_entity_array!(2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13);
impl_entity_array!(14, 15, 16, 17, 18, 19, 20, 21, 22, 23);
impl_entity_array!(24, 25, 26, 27, 28, 29, 30, 31, 32);

#[doc(hidden)]
#[macro_export]
macro_rules! count {
    ($a: expr) => {1_usize};
    ($a: expr, $($b: expr), *) => {
        1_usize + $crate::count!($($b), *)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! array_type {
    ($typename: ty, $($a:expr), *) => { [$typename; $crate::count!($($a), *)] };
}

#[doc(hidden)]
#[macro_export]
macro_rules! vector_type {
    ($($a:expr), *) => { $crate::Vector<[f64; $crate::count!($($a), *)]> };
}

#[macro_export]
macro_rules! matrix {
    ($($a: expr), *) => {
        $crate::Matrix::<$crate::array_type!(f64, $($a), *),
        $crate::array_type!($crate::vector_type!($($a),*), $($a), *)>::from([$($a), *])
    };
}

impl<T, M: EntityArray<Vector<T>>> Deref for Matrix<T, M> {
    type Target = [Vector<T>];
    #[inline(always)]
    fn deref(&self) -> &[Vector<T>] { self.0.as_ref() }
}

impl<T, M: EntityArray<Vector<T>>> DerefMut for Matrix<T, M> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [Vector<T>] { self.0.as_mut() }
}

impl<T, M: EntityArray<Vector<T>>> AsRef<[Vector<T>]> for Matrix<T, M> {
    #[inline(always)]
    fn as_ref(&self) -> &[Vector<T>] { self.0.as_ref() }
}

impl<T, M: EntityArray<Vector<T>>> AsMut<[Vector<T>]> for Matrix<T, M> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut [Vector<T>] { self.0.as_mut() }
}

impl<'a, T, M: EntityArray<Vector<T>>> IntoIterator for &'a Matrix<T, M> {
    type Item = &'a Vector<T>;
    type IntoIter = std::slice::Iter<'a, Vector<T>>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl<'a, T, M: EntityArray<Vector<T>>> IntoIterator for &'a mut Matrix<T, M> {
    type Item = &'a mut Vector<T>;
    type IntoIter = std::slice::IterMut<'a, Vector<T>>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter { self.iter_mut() }
}

impl<T, M> FromIterator<Vector<T>> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    /// Creates a vector by an iterator over `Vector<T>`.  
    /// If the length of the iterator is large, then the latter elements are truncated.  
    /// If the length of the iterator is small, then the latter components are made zero.
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = Vector<T>>>(iter: I) -> Matrix<T, M> {
        let mut res = Matrix::ORIGIN;
        res.iter_mut().zip(iter).for_each(|(a, b)| *a = b);
        res
    }
}

impl<'a, T, M> FromIterator<&'a Vector<T>> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    /// Creates a vector by an iterator over `Vector<T>`.  
    /// If the length of the iterator is large, then the latter elements are truncated.  
    /// If the length of the iterator is small, then the latter components are made zero.
    #[inline(always)]
    fn from_iter<I: IntoIterator<Item = &'a Vector<T>>>(iter: I) -> Matrix<T, M> {
        iter.into_iter().map(|x| x.clone()).collect()
    }
}

impl<T, M> From<M> for Matrix<T, M> {
    #[inline(always)]
    fn from(arr: M) -> Matrix<T, M> { Matrix(arr, PhantomData) }
}

impl<T, M: Clone> From<&M> for Matrix<T, M> {
    #[inline(always)]
    fn from(arr: &M) -> Matrix<T, M> { Matrix(arr.clone(), PhantomData) }
}

impl<T, M> AddAssign<&Matrix<T, M>> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    /// Adds and assigns two matrices.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut mat = matrix!(vector!(1, 2), vector!(3, -4));
    /// mat += &matrix!(vector!(-2, 1), vector!(-3, 5));
    /// assert_eq!(mat, matrix!(vector!(-1, 3), vector!(0, 1)));
    /// ```
    #[inline(always)]
    fn add_assign(&mut self, other: &Matrix<T, M>) {
        self.iter_mut().zip(other).for_each(move |(a, b)| *a += b)
    }
}

impl<T, M> AddAssign<Matrix<T, M>> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    /// Adds and assigns two matrices.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut mat = matrix!(vector!(1, 2), vector!(3, -4));
    /// mat += matrix!(vector!(-2, 1), vector!(-3, 5));
    /// assert_eq!(mat, matrix!(vector!(-1, 3), vector!(0, 1)));
    /// ```
    #[inline(always)]
    fn add_assign(&mut self, other: Matrix<T, M>) { *self += &other }
}

impl<T, M> Add<&Matrix<T, M>> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Matrix<T, M>;
    /// Adds two matrices.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat0 = matrix!(vector!(1, 2), vector!(3, -4));
    /// let mat1 = matrix!(vector!(-2, 1), vector!(-3, 5));
    /// assert_eq!(mat0 + &mat1, matrix!(vector!(-1, 3), vector!(0, 1)));
    /// ```
    #[inline(always)]
    fn add(mut self, other: &Matrix<T, M>) -> Matrix<T, M> {
        self += other;
        self
    }
}

impl<T, M> Add<&Matrix<T, M>> for &Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Matrix<T, M>;
    /// Adds two matrices.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat0 = matrix!(vector!(1, 2), vector!(3, -4));
    /// let mat1 = matrix!(vector!(-2, 1), vector!(-3, 5));
    /// assert_eq!(&mat0 + &mat1, matrix!(vector!(-1, 3), vector!(0, 1)));
    /// ```
    #[inline(always)]
    fn add(self, other: &Matrix<T, M>) -> Matrix<T, M> { self.clone() + other }
}

impl<T, M> Add<Matrix<T, M>> for &Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Matrix<T, M>;
    /// Adds two matrices.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat0 = matrix!(vector!(1, 2), vector!(3, -4));
    /// let mat1 = matrix!(vector!(-2, 1), vector!(-3, 5));
    /// assert_eq!(&mat0 + mat1, matrix!(vector!(-1, 3), vector!(0, 1)));
    /// ```
    #[inline(always)]
    fn add(self, other: Matrix<T, M>) -> Matrix<T, M> { other + self }
}

impl<T, M> Add<Matrix<T, M>> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Matrix<T, M>;
    /// Adds two matrices.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat0 = matrix!(vector!(1, 2), vector!(3, -4));
    /// let mat1 = matrix!(vector!(-2, 1), vector!(-3, 5));
    /// assert_eq!(mat0 + mat1, matrix!(vector!(-1, 3), vector!(0, 1)));
    /// ```
    #[inline(always)]
    fn add(self, other: Matrix<T, M>) -> Matrix<T, M> { self + &other }
}

impl<T, M> SubAssign<&Matrix<T, M>> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    /// Subtracts and assigns two matrices.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut mat = matrix!(vector!(1, 2), vector!(3, -4));
    /// mat -= &matrix!(vector!(-2, 1), vector!(-3, 5));
    /// assert_eq!(mat, matrix!(vector!(3, 1), vector!(6, -9)));
    /// ```
    #[inline(always)]
    fn sub_assign(&mut self, other: &Matrix<T, M>) {
        self.iter_mut().zip(other).for_each(move |(a, b)| *a -= b)
    }
}

impl<T, M> SubAssign<Matrix<T, M>> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    /// Subtracts and assigns two matrices.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut mat = matrix!(vector!(1, 2), vector!(3, -4));
    /// mat -= matrix!(vector!(-2, 1), vector!(-3, 5));
    /// assert_eq!(mat, matrix!(vector!(3, 1), vector!(6, -9)));
    /// ```
    #[inline(always)]
    fn sub_assign(&mut self, other: Matrix<T, M>) { *self -= &other }
}

impl<T, M> Sub<&Matrix<T, M>> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Matrix<T, M>;
    /// Subtracts two matrices.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat0 = matrix!(vector!(1, 2), vector!(3, -4));
    /// let mat1 = matrix!(vector!(-2, 1), vector!(-3, 5));
    /// assert_eq!(mat0 - &mat1, matrix!(vector!(3, 1), vector!(6, -9)));
    /// ```
    #[inline(always)]
    fn sub(mut self, other: &Matrix<T, M>) -> Matrix<T, M> {
        self -= other;
        self
    }
}

impl<T, M> Sub<&Matrix<T, M>> for &Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Matrix<T, M>;
    /// Subtracs two matrices.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat0 = matrix!(vector!(1, 2), vector!(3, -4));
    /// let mat1 = matrix!(vector!(-2, 1), vector!(-3, 5));
    /// assert_eq!(&mat0 - &mat1, matrix!(vector!(3, 1), vector!(6, -9)));
    /// ```
    #[inline(always)]
    fn sub(self, other: &Matrix<T, M>) -> Matrix<T, M> { self.clone() - other }
}

impl<T, M> Sub<Matrix<T, M>> for &Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Matrix<T, M>;
    /// Subtracs two matrices.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat0 = matrix!(vector!(1, 2), vector!(3, -4));
    /// let mat1 = matrix!(vector!(-2, 1), vector!(-3, 5));
    /// assert_eq!(&mat0 - mat1, matrix!(vector!(3, 1), vector!(6, -9)));
    /// ```
    #[inline(always)]
    fn sub(self, other: Matrix<T, M>) -> Matrix<T, M> { -(other - self) }
}

impl<T, M> Sub<Matrix<T, M>> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Matrix<T, M>;
    /// Subtracs two matrices.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat0 = matrix!(vector!(1, 2), vector!(3, -4));
    /// let mat1 = matrix!(vector!(-2, 1), vector!(-3, 5));
    /// assert_eq!(mat0 - mat1, matrix!(vector!(3, 1), vector!(6, -9)));
    /// ```
    #[inline(always)]
    fn sub(self, other: Matrix<T, M>) -> Matrix<T, M> { self - &other }
}

impl<T, M> Neg for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Matrix<T, M>;
    /// Returns the negative matrix
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(vector!(1, 2), vector!(3, -4));
    /// assert_eq!(-mat, matrix!(vector!(-1, -2), vector!(-3, 4)));
    /// ```
    #[inline(always)]
    fn neg(mut self) -> Matrix<T, M> {
        self.iter_mut()
            .for_each(move |a| a.iter_mut().for_each(|b| *b = -*b));
        self
    }
}

impl<T, M> Neg for &Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Matrix<T, M>;
    /// Returns the negative matrix
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(vector!(1, 2), vector!(3, -4));
    /// assert_eq!(-&mat, matrix!(vector!(-1, -2), vector!(-3, 4)));
    /// ```
    #[inline(always)]
    fn neg(self) -> Matrix<T, M> { -self.clone() }
}

impl<T, M> MulAssign<f64> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    /// Multiplies a scalar to a matrix.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut mat = matrix!(vector!(1, 2), vector!(3, -4));
    /// mat *= 2.0;
    /// assert_eq!(mat, matrix!(vector!(2, 4), vector!(6, -8)));
    /// ```
    #[inline(always)]
    fn mul_assign(&mut self, scalar: f64) { self.iter_mut().for_each(move |a| *a *= scalar) }
}

impl<T, M> Mul<f64> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Matrix<T, M>;
    /// Multiplies a scalar to a matrix.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(vector!(1, 2), vector!(3, -4));
    /// assert_eq!(mat * 2.0, matrix!(vector!(2, 4), vector!(6, -8)));
    /// ```
    #[inline(always)]
    fn mul(mut self, scalar: f64) -> Matrix<T, M> {
        self *= scalar;
        self
    }
}

impl<T, M> Mul<f64> for &Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Matrix<T, M>;
    /// Multiplies a scalar to a matrix.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(vector!(1, 2), vector!(3, -4));
    /// assert_eq!(&mat * 2.0, matrix!(vector!(2, 4), vector!(6, -8)));
    /// ```
    #[inline(always)]
    fn mul(self, scalar: f64) -> Matrix<T, M> { self.clone() * scalar }
}

impl<T, M> Mul<&Matrix<T, M>> for f64
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Matrix<T, M>;
    /// Multiplies a scalar to a matrix.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(vector!(1, 2), vector!(3, -4));
    /// assert_eq!(2.0 * &mat, matrix!(vector!(2, 4), vector!(6, -8)));
    /// ```
    #[inline(always)]
    fn mul(self, matrix: &Matrix<T, M>) -> Matrix<T, M> { matrix * self }
}

impl<T, M> Mul<Matrix<T, M>> for f64
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Matrix<T, M>;
    /// Multiplies a scalar to a matrix.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(vector!(1, 2), vector!(3, -4));
    /// assert_eq!(2.0 * &mat, matrix!(vector!(2, 4), vector!(6, -8)));
    /// ```
    #[inline(always)]
    fn mul(self, matrix: Matrix<T, M>) -> Matrix<T, M> { matrix * self }
}

impl<T, M> Mul<&Matrix<T, M>> for &Vector<T>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Vector<T>;
    /// Multiplies a matrix to the vector
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let vec = vector!(1, 2);
    /// let mat = matrix!(vector!(2, 3), vector!(4, 5));
    /// assert_eq!(&vec * &mat, vector!(10, 13));
    /// ```
    #[inline(always)]
    fn mul(self, matrix: &Matrix<T, M>) -> Vector<T> {
        let mut res = Vector::ORIGIN;
        matrix
            .iter()
            .zip(self)
            .for_each(|(vec, a)| res.iter_mut().zip(vec).for_each(move |(p, q)| *p += q * a));
        res
    }
}

impl<T, M> Mul<Matrix<T, M>> for &Vector<T>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Vector<T>;
    /// Multiplies a matrix to the vector
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let vec = vector!(1, 2);
    /// let mat = matrix!(vector!(2, 3), vector!(4, 5));
    /// assert_eq!(&vec * mat, vector!(10, 13));
    /// ```
    #[inline(always)]
    fn mul(self, matrix: Matrix<T, M>) -> Vector<T> { self * &matrix }
}

impl<T, M> Mul<&Matrix<T, M>> for Vector<T>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Vector<T>;
    /// Multiplies a matrix to the vector
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let vec = vector!(1, 2);
    /// let mat = matrix!(vector!(2, 3), vector!(4, 5));
    /// assert_eq!(&vec * mat, vector!(10, 13));
    /// ```
    #[inline(always)]
    fn mul(self, matrix: &Matrix<T, M>) -> Vector<T> { &self * matrix }
}

impl<T, M> Mul<Matrix<T, M>> for Vector<T>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Vector<T>;
    /// Multiplies a matrix to the vector
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let vec = vector!(1, 2);
    /// let mat = matrix!(vector!(2, 3), vector!(4, 5));
    /// assert_eq!(vec * mat, vector!(10, 13));
    /// ```
    #[inline(always)]
    fn mul(self, matrix: Matrix<T, M>) -> Vector<T> { &self * &matrix }
}

impl<T, M> MulAssign<&Matrix<T, M>> for Vector<T>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    /// Multiplies a matrix to the vector
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut vec = vector!(1, 2);
    /// vec *= &matrix!(vector!(2, 3), vector!(4, 5));
    /// assert_eq!(vec, vector!(10, 13));
    /// ```
    #[inline(always)]
    fn mul_assign(&mut self, matrix: &Matrix<T, M>) { *self = &*self * matrix }
}

impl<T, M> MulAssign<Matrix<T, M>> for Vector<T>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    /// Multiplies a matrix to the vector
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut vec = vector!(1, 2);
    /// vec *= matrix!(vector!(2, 3), vector!(4, 5));
    /// assert_eq!(vec, vector!(10, 13));
    /// ```
    #[inline(always)]
    fn mul_assign(&mut self, matrix: Matrix<T, M>) { *self *= &matrix }
}

impl<T, M> Mul<&Vector<T>> for &Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Vector<T>;
    /// Multiplies a matrix to the vector
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let vec = vector!(1, 2);
    /// let mat = matrix!(vector!(2, 3), vector!(4, 5));
    /// assert_eq!(&mat * &vec, vector!(8, 14));
    /// ```
    #[inline(always)]
    fn mul(self, vec: &Vector<T>) -> Vector<T> { self.iter().map(move |tmp| tmp * vec).collect() }
}

impl<T, M> Mul<&Vector<T>> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Vector<T>;
    /// Multiplies a matrix to the vector
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let vec = vector!(1, 2);
    /// let mat = matrix!(vector!(2, 3), vector!(4, 5));
    /// assert_eq!(mat * &vec, vector!(8, 14));
    /// ```
    #[inline(always)]
    fn mul(self, vec: &Vector<T>) -> Vector<T> { &self * vec }
}

impl<T, M> Mul<Vector<T>> for &Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Vector<T>;
    /// Multiplies a matrix to the vector
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let vec = vector!(1, 2);
    /// let mat = matrix!(vector!(2, 3), vector!(4, 5));
    /// assert_eq!(&mat * vec, vector!(8, 14));
    /// ```
    #[inline(always)]
    fn mul(self, vec: Vector<T>) -> Vector<T> { self * &vec }
}

impl<T, M> Mul<Vector<T>> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Vector<T>;
    /// Multiplies a matrix to the vector
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let vec = vector!(1, 2);
    /// let mat = matrix!(vector!(2, 3), vector!(4, 5));
    /// assert_eq!(mat * vec, vector!(8, 14));
    /// ```
    #[inline(always)]
    fn mul(self, vec: Vector<T>) -> Vector<T> { &self * &vec }
}

macro_rules! sub_impl_mul {
    ($a: expr, $b: expr) => {};
    ($dim: expr, $($num: expr), *) => {
        impl Multiplicable<[f64; $dim]> for [Vector<[f64; $dim]>; $dim] {
            #[inline(always)]
            fn vec_multiply(vec: &Vector<[f64; $dim]>, matrix: &Self) -> Vector<[f64; $dim]> {
                let mut res = Vector::ORIGIN;
                matrix.iter().zip(vec).for_each(|(vec, a)| {
                    res.iter_mut().zip(vec).for_each(move |(p, q)| {
                        *p += q * a;
                    });
                });
                res
            }

            #[inline(always)]
            fn multiply(&self, other: &Self) -> Self {
                let mut arr = [$(Self::vec_multiply(&self[$num], other)), *];
                arr.reverse();
                arr
            }
        }
        sub_impl_mul!($($num), *);
    };
}

sub_impl_mul!(
    32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9,
    8, 7, 6, 5, 4, 3, 2, 1, 0
);

impl<T, M> Mul<&Matrix<T, M>> for &Matrix<T, M>
where
    T: EntityArray<f64>,
    M: Multiplicable<T>,
{
    type Output = Matrix<T, M>;
    /// Multiplies a matrix to another matrix
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat0 = matrix!(vector!(1, 2), vector!(-2, 1));
    /// let mat1 = matrix!(vector!(2, 3), vector!(4, 5));
    /// assert_eq!(&mat0 * &mat1, matrix!(vector!(10, 13), vector!(0, -1)));
    /// ```
    #[inline(always)]
    fn mul(self, mat: &Matrix<T, M>) -> Matrix<T, M> { Matrix::from(self.0.multiply(&mat.0)) }
}

impl<T, M> Mul<Matrix<T, M>> for &Matrix<T, M>
where
    T: EntityArray<f64>,
    M: Multiplicable<T>,
{
    type Output = Matrix<T, M>;
    /// Multiplies a matrix to another matrix
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat0 = matrix!(vector!(1, 2), vector!(-2, 1));
    /// let mat1 = matrix!(vector!(2, 3), vector!(4, 5));
    /// assert_eq!(&mat0 * mat1, matrix!(vector!(10, 13), vector!(0, -1)));
    /// ```
    #[inline(always)]
    fn mul(self, mat: Matrix<T, M>) -> Self::Output { self * &mat }
}

impl<T, M> Mul<&Matrix<T, M>> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: Multiplicable<T>,
{
    type Output = Matrix<T, M>;
    /// Multiplies a matrix to another matrix
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat0 = matrix!(vector!(1, 2), vector!(-2, 1));
    /// let mat1 = matrix!(vector!(2, 3), vector!(4, 5));
    /// assert_eq!(mat0 * &mat1, matrix!(vector!(10, 13), vector!(0, -1)));
    /// ```
    #[inline(always)]
    fn mul(self, mat: &Matrix<T, M>) -> Self::Output { &self * mat }
}

impl<T, M> Mul<Matrix<T, M>> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: Multiplicable<T>,
{
    type Output = Matrix<T, M>;
    /// Multiplies a matrix to another matrix
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat0 = matrix!(vector!(1, 2), vector!(-2, 1));
    /// let mat1 = matrix!(vector!(2, 3), vector!(4, 5));
    /// assert_eq!(mat0 * mat1, matrix!(vector!(10, 13), vector!(0, -1)));
    /// ```
    #[inline(always)]
    fn mul(self, mat: Matrix<T, M>) -> Self::Output { &self * &mat }
}

impl<T, M> MulAssign<&Matrix<T, M>> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: Multiplicable<T>,
{
    /// Multiplies a matrix to another matrix
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut mat = matrix!(vector!(1, 2), vector!(-2, 1));
    /// mat *= &matrix!(vector!(2, 3), vector!(4, 5));
    /// assert_eq!(mat, matrix!(vector!(10, 13), vector!(0, -1)));
    /// ```
    #[inline(always)]
    fn mul_assign(&mut self, mat: &Matrix<T, M>) { *self = &*self * mat; }
}

impl<T, M> MulAssign<Matrix<T, M>> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: Multiplicable<T>,
{
    /// Multiplies a matrix to another matrix
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut mat = matrix!(vector!(1, 2), vector!(-2, 1));
    /// mat *= matrix!(vector!(2, 3), vector!(4, 5));
    /// assert_eq!(mat, matrix!(vector!(10, 13), vector!(0, -1)));
    /// ```
    #[inline(always)]
    fn mul_assign(&mut self, mat: Matrix<T, M>) { *self = &*self * &mat; }
}

impl<T, M> DivAssign<f64> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    /// Divides a matrix by a scalar.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mut mat = matrix!(vector!(1, 2), vector!(3, -4));
    /// mat /= 2.0;
    /// assert_eq!(mat, matrix!(vector!(0.5, 1), vector!(1.5, -2)));
    /// ```
    #[inline(always)]
    fn div_assign(&mut self, scalar: f64) { self.iter_mut().for_each(move |a| *a /= scalar); }
}

impl<T, M> Div<f64> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Matrix<T, M>;
    /// Divides a matrix by a scalar.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(vector!(1, 2), vector!(3, -4));
    /// assert_eq!(mat / 2.0, matrix!(vector!(0.5, 1), vector!(1.5, -2)));
    /// ```
    #[inline(always)]
    fn div(mut self, scalar: f64) -> Matrix<T, M> {
        self /= scalar;
        self
    }
}

impl<T, M> Div<f64> for &Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Matrix<T, M>;
    /// Divides a matrix by a scalar.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(vector!(1, 2), vector!(3, -4));
    /// assert_eq!(&mat / 2.0, matrix!(vector!(0.5, 1), vector!(1.5, -2)));
    /// ```
    #[inline(always)]
    fn div(self, scalar: f64) -> Matrix<T, M> { self.clone() / scalar }
}

impl<T, M> Index<usize> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    type Output = Vector<T>;
    /// Returns the `idx`th row vector.
    #[inline(always)]
    fn index(&self, idx: usize) -> &Vector<T> { &self.as_ref()[idx] }
}

impl<T, M> IndexMut<usize> for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    /// Returns the `idx`th row vector.
    #[inline(always)]
    fn index_mut(&mut self, idx: usize) -> &mut Vector<T> { &mut self.as_mut()[idx] }
}

impl<T, M> Tolerance for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    /// The components of each of the two matrices are close enough.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let eps = f64::TOLERANCE / 2.0;
    /// let v0 = matrix!(vector!(0.0, 0.0), vector!(0.0, 0.0));
    /// let v1 = matrix!(vector!(eps, -eps), vector!(-eps, eps));
    /// let v2 = matrix!(vector!(1.0, 0.0), vector!(0.0, 0.0));
    /// let v3 = matrix!(vector!(0.0, 0.0), vector!(0.0, 1.0));
    /// assert!(v0.near(&v1));
    /// assert!(!v0.near(&v2));
    /// assert!(!v0.near(&v3));
    /// ```
    #[inline(always)]
    fn near(&self, other: &Self) -> bool { self.iter().zip(other).all(move |(a, b)| a.near(b)) }

    /// The components of each of the two matrices are close enough.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let eps = f64::TOLERANCE / 2.0;
    /// let eps2 = f64::TOLERANCE2 / 2.0;
    /// let v0 = matrix!(vector!(0.0, 0.0), vector!(0.0, 0.0));
    /// let v1 = matrix!(vector!(eps2, -eps2), vector!(-eps2, eps2));
    /// let v2 = matrix!(vector!(eps, 0.0), vector!(0.0, 0.0));
    /// let v3 = matrix!(vector!(0.0, 0.0), vector!(0.0, eps));
    /// assert!(v0.near2(&v1));
    /// assert!(!v0.near2(&v2));
    /// assert!(!v0.near2(&v3));
    /// ```
    #[inline(always)]
    fn near2(&self, other: &Self) -> bool { self.iter().zip(other).all(move |(a, b)| a.near2(b)) }
}

impl<T, M> Origin for Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    const ORIGIN: Self = Self(M::ORIGIN, PhantomData);
}

impl<T, M> Matrix<T, M>
where
    T: EntityArray<f64>,
    M: EntityArray<Vector<T>>,
{
    /// Creates the zero matrix.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(vector!(1, 2), vector!(3, 4));
    /// assert_eq!(mat * Matrix2::zero(), Matrix2::zero());
    /// ```
    #[inline(always)]
    pub fn zero() -> Matrix<T, M> { Matrix::ORIGIN }

    /// Creates the identity matrix.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(vector!(1, 2), vector!(3, 4));
    /// assert_eq!(&mat * Matrix2::identity(), mat);
    /// ```
    #[inline(always)]
    pub fn identity() -> Matrix<T, M> {
        let mut mat = Matrix::ORIGIN;
        mat.iter_mut()
            .enumerate()
            .for_each(move |(i, v)| v[i] = 1.0);
        mat
    }

    /// Creates the diagonal matrix.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let vec = vector!(2, 3);
    /// let mat = Matrix2::diagonal(&vec);
    /// assert_eq!(vec * mat, vector!(4, 9));
    /// ```
    #[inline(always)]
    pub fn diagonal(diag: &Vector<T>) -> Matrix<T, M> {
        let mut mat = Matrix::ORIGIN;
        mat.iter_mut()
            .zip(diag)
            .enumerate()
            .for_each(move |(i, (vec, c))| vec[i] = *c);
        mat
    }

    /// Returns the `idx`th row vector. In fact, the result coincides with `self[idx].clone()`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(vector!(1, 2), vector!(3, 4));
    /// assert_eq!(mat.row(1), vector!(3, 4));
    /// ```
    #[inline(always)]
    pub fn row(&self, idx: usize) -> Vector<T> { self[idx].clone() }

    /// Returns the `idx`th column vector.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(vector!(1, 2), vector!(3, 4));
    /// assert_eq!(mat.column(1), vector!(2, 4));
    /// ```
    #[inline(always)]
    pub fn column(&self, idx: usize) -> Vector<T> { self.iter().map(move |vec| vec[idx]).collect() }

    /// Returns the transposed matrix.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(vector!(1, 2), vector!(3, 4));
    /// assert_eq!(mat.transpose(), matrix!(vector!(1, 3), vector!(2, 4)));
    /// ```
    #[inline(always)]
    pub fn transpose(&self) -> Matrix<T, M> {
        (0..self.len()).map(move |i| self.column(i)).collect()
    }

    /// Returns the trace of the matrix.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(vector!(1, 2), vector!(3, 4));
    /// assert_eq!(mat.trace(), 5.0);
    /// ```
    #[inline(always)]
    pub fn trace(&self) -> f64 { self.iter().enumerate().map(move |(i, vec)| vec[i]).sum() }

    /// the square of the norm.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(vector!(1, 2), vector!(3, 4));
    /// assert_eq!(mat.norm2(), 30.0);
    /// ```
    #[inline(always)]
    pub fn norm2(&self) -> f64 { self.iter().map(|vec| vec.norm2()).sum() }
    /// norm.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(vector!(1, 2), vector!(3, 4));
    /// assert_eq!(mat.norm(), 30.0_f64.sqrt());
    /// ```
    #[inline(always)]
    pub fn norm(&self) -> f64 { self.norm2().sqrt() }

    /// Returns the determinant.
    #[inline(always)]
    pub fn determinant(&self) -> f64
    where Self: Determinant<T> {
        Determinant::<T>::determinant(self)
    }
    /// Returns the adjugate matrix.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(vector!(1, 2), vector!(3, 4));
    /// assert_eq!(mat.adjugate(), matrix!(vector!(4, -2), vector!(-3, 1)));
    /// ```
    #[inline(always)]
    pub fn adjugate(&self) -> Self
    where Self: Determinant<T> {
        Determinant::<T>::adjugate(self)
    }

    /// Returns the solution of the equation `vec = x * self`.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(vector!(1, -3, 2), vector!(-4, 5, 1), vector!(6, -2, -4));
    /// let vec = vector!(1, 8, -3);
    /// Vector::assert_near2(&mat.solve(&vec), &vector!(1, 3, 2));
    /// ```
    #[inline(always)]
    pub fn solve(&self, vec: &Vector<T>) -> Vector<T>
    where Self: Determinant<T> {
        Determinant::<T>::solve(self, vec)
    }

    /// Returns the inverse matrix.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(
    ///     vector!(1, -3, 3, 2),
    ///     vector!(4, 3, -2, 1),
    ///     vector!(5, 2, 1, 3),
    ///     vector!(5, 6, 1, 2)
    /// );
    /// Matrix::assert_near2(&(&mat * mat.inverse()), &Matrix::identity());
    /// ```
    #[inline(always)]
    pub fn inverse(&self) -> Matrix<T, M>
    where Self: Determinant<T> {
        Determinant::<T>::inverse(self)
    }

    /// Iwasawa decomposition. `(N, A, K)`
    /// * `N`: a unipotent lower triangle matrix
    /// * `A`: a diagonal matrix
    /// * `K`: an orthogonal matrix
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(
    ///     vector!(1, -3, 3, 2),
    ///     vector!(4, 3, -2, 1),
    ///     vector!(5, 2, 1, 3),
    ///     vector!(5, 6, 1, 2)
    /// );
    /// let (n, a, k) = mat.iwasawa_decomposition();
    ///
    /// // k is an orthogonal matrix
    /// let trans = k.transpose();
    /// let inv = k.inverse();
    /// Matrix::assert_near2(&trans, &inv);
    ///
    /// // a is a diagonal matrix
    /// for i in 0..4 {
    ///     for j in 0..4 {
    ///        if i != j { assert_eq!(a[i][j], 0.0); }
    ///     }
    /// }
    ///
    /// // n is a unipotent lower triangle matrix
    /// for j in 0..4 {
    ///     for i in 0..j {
    ///         assert_eq!(n[i][j], 0.0);
    ///     }
    /// }
    ///
    /// // n * a * k coinsides with mat
    /// Matrix::assert_near2(&(n * a * k), &mat);
    /// ```
    pub fn iwasawa_decomposition(&self) -> (Self, Self, Self) {
        let mut n = Matrix::<T, M>::identity();
        let mut u = Matrix::<T, M>::ORIGIN;
        let mut a = Vector::<T>::ORIGIN;
        self.iter().enumerate().for_each(|(i, vec)| {
            (0..i).for_each(|j| n[i][j] = (vec * &u[j]) / (a[j] * a[j]));
            u[i] = (0..i).fold(vec.clone(), |sum, j| sum - n[i][j] * &u[j]);
            a[i] = u[i].norm();
        });
        u.iter_mut().zip(&a).for_each(|(v, k)| *v /= *k);
        (n, Matrix::diagonal(&a), u)
    }
}

pub trait Determinant<T>:
    Clone + Deref<Target = [Vector<T>]> + DerefMut + Div<f64, Output = Self> + FromIterator<Vector<T>>
where T: EntityArray<f64> {
    /// Returns the determinant.
    fn determinant(&self) -> f64;

    /// Returns the multiplication of the determinant and the solution of the equation `vec = x * self`
    #[inline(always)]
    fn presolve(&self, vec: &Vector<T>) -> Vector<T> {
        (0..self.len())
            .map(move |i| {
                let mut mat = self.clone();
                mat[i] = vec.clone();
                mat.determinant()
            })
            .collect()
    }
    /// Returns the adjugate matrix.
    #[inline(always)]
    fn adjugate(&self) -> Self {
        (0..self.len())
            .map(|i| self.presolve(&Vector::std_basis(i)))
            .collect()
    }
    /// Returns the solution of the equation `vec = x * self`.
    #[inline(always)]
    fn solve(&self, vec: &Vector<T>) -> Vector<T> { self.presolve(vec) / self.determinant() }
    /// Returns the inverse matrix
    #[inline(always)]
    fn inverse(&self) -> Self { self.adjugate() / self.determinant() }
}

impl Determinant<[f64; 2]> for Matrix2 {
    /// Returns the determinant.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(vector!(4, 5), vector!(2, 3));
    /// assert_eq!(mat.determinant(), 2.0);
    /// ```
    #[inline(always)]
    fn determinant(&self) -> f64 { self[0][0] * self[1][1] - self[0][1] * self[1][0] }
}

impl Determinant<[f64; 3]> for Matrix3 {
    /// Returns the determinant.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(vector!(2, 1, 3), vector!(-1, 5, 2), vector!(4, 1, -1));
    /// assert_eq!(mat.determinant(), -70.0);
    /// ```
    #[inline(always)]
    fn determinant(&self) -> f64 {
        self[0][0] * (self[1][1] * self[2][2] - self[1][2] * self[2][1])
            + self[1][0] * (self[2][1] * self[0][2] - self[2][2] * self[0][1])
            + self[2][0] * (self[0][1] * self[1][2] - self[0][2] * self[1][1])
    }
}

impl Determinant<[f64; 4]> for Matrix4 {
    /// Returns the determinant.
    /// # Examples
    /// ```
    /// use truck_geometry::*;
    /// let mat = matrix!(
    ///     vector!(1, -3, 3, 2),
    ///     vector!(4, 3, -2, 1),
    ///     vector!(5, 2, 1, 3),
    ///     vector!(5, 6, 1, 2)
    /// );
    /// assert_eq!(mat.determinant(), -28.0);
    /// ```
    #[inline(always)]
    fn determinant(&self) -> f64 {
        self[0][0]
            * (self[1][1] * (self[2][2] * self[3][3] - self[2][3] * self[3][2])
                + self[2][1] * (self[3][2] * self[1][3] - self[3][3] * self[1][2])
                + self[3][1] * (self[1][2] * self[2][3] - self[1][3] * self[2][2]))
            - self[1][0]
                * (self[2][1] * (self[3][2] * self[0][3] - self[3][3] * self[0][2])
                    + self[3][1] * (self[0][2] * self[2][3] - self[0][3] * self[2][2])
                    + self[0][1] * (self[2][2] * self[3][3] - self[2][3] * self[3][2]))
            + self[2][0]
                * (self[3][1] * (self[0][2] * self[1][3] - self[0][3] * self[1][2])
                    + self[0][1] * (self[1][2] * self[3][3] - self[1][3] * self[3][2])
                    + self[1][1] * (self[3][2] * self[0][3] - self[3][3] * self[0][2]))
            - self[3][0]
                * (self[0][1] * (self[1][2] * self[2][3] - self[1][3] * self[2][2])
                    + self[1][1] * (self[2][2] * self[0][3] - self[2][3] * self[0][2])
                    + self[2][1] * (self[0][2] * self[1][3] - self[0][3] * self[1][2]))
    }
}
