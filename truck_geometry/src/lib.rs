//! # Overview
//! `truck_geometry` is a crate for describing geometrical information.
//! It contains definision basic mathematical objects, vectors and matrices.

#![warn(
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

use std::fmt::Debug;

/// vector
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Vector<T>(T);

/// 2-dim vector
///
/// The cross product `^` returns the signed area of the parallelogram streched by the two vectors.
/// ```
/// use truck_geometry::*;
/// assert_eq!(vector!(2, 3) ^ vector!(4, 7), 2.0);
/// ```
pub type Vector2 = Vector<[f64; 2]>;
/// 3-dim vector
///
/// The cross product is represented by the operator `^`.
/// ```
/// use truck_geometry::*;
/// assert_eq!(vector!(1, 2, 3) ^ vector!(2, 4, 7), vector!(2, -1, 0));
/// ```
pub type Vector3 = Vector<[f64; 3]>;
/// 4-dim vector
///
/// To creates the rational vector by a 3-dimentional coordinate, use `rvector!()`.
/// ```
/// use truck_geometry::*;
/// assert_eq!(rvector!(1, 2, 3), vector!(1, 2, 3, 1));
/// ```
pub type Vector4 = Vector<[f64; 4]>;

/// square matrix
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Matrix<T, M>(M, std::marker::PhantomData<fn() -> T>);
/// 2x2 matrix
pub type Matrix2 = Matrix<[f64; 2], [Vector2; 2]>;
/// 3x3 matrix
pub type Matrix3 = Matrix<[f64; 3], [Vector3; 3]>;
/// 4x4 matrix
pub type Matrix4 = Matrix<[f64; 4], [Vector4; 4]>;

/// Abstraction of the arrays which is the entity of `Vector` or `Matrix`.
pub trait EntityArray<T>:
    Sized + Clone + PartialEq + AsRef<[T]> + AsMut<[T]> + Debug + Default {
    /// the array of all components are `0.0`.
    const ORIGIN: Self;
    /// the array of all components are `f64::INFINITY`.
    const INFINITY: Self;
    /// the array of all components are `f64::NEG_INFINITY`.
    const NEG_INFINITY: Self;
    /// Creats array from iterator. Panic occurs if the length of iterator is too short.
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self;
}

/// knot vector
#[derive(Clone, PartialEq, Debug)]
pub struct KnotVec(Vec<f64>);

/// bounding box
#[derive(Clone, PartialEq, Debug)]
pub struct BoundingBox<T>(Vector<T>, Vector<T>);

/// Defines a tolerance in the whole package
pub trait Tolerance: Sized + Debug {
    /// general tolerance
    const TOLERANCE: f64 = 1.0e-7;

    /// general tolerance of square order
    const TOLERANCE2: f64 = Self::TOLERANCE * Self::TOLERANCE;

    /// The "distance" is less than EPSILON.
    fn near(&self, other: &Self) -> bool;

    /// The "distance" is less than EPSILON2.
    fn near2(&self, other: &Self) -> bool;

    /// assert if `one` is not near `other`.
    fn assert_near(one: &Self, other: &Self) {
        assert!(
            one.near(other),
            "not near two values.\nvalue0: {:?}\nvalue1: {:?}",
            one,
            other
        );
    }

    /// assertion if `one` is not near `other` in square order.
    fn assert_near2(one: &Self, other: &Self) {
        assert!(
            one.near2(other),
            "not near two values in square order.\nvalue0: {:?}\nvalue1: {:?}",
            one,
            other
        );
    }
}

/// The structs defined the origin. `f64`, `Vector`, and so on.
pub trait Origin: Tolerance {
    /// origin point
    const ORIGIN: Self;

    /// Rounds the value by tolerance
    /// # Example
    /// ```
    /// use truck_geometry::*;
    /// assert_eq!(1.23456789_f64.round_by_tolerance(), &1.2345678);
    /// ```
    fn round_by_tolerance(&mut self) -> &mut Self;

    /// near origin
    #[inline(always)]
    fn so_small(&self) -> bool { self.near(&Self::ORIGIN) }

    /// near origin in square order
    #[inline(always)]
    fn so_small2(&self) -> bool { self.near2(&Self::ORIGIN) }
}

/// B-spline curve
/// # Examples
/// ```
/// use truck_geometry::*;
///
/// // the knot vector
/// let knot_vec = KnotVec::from(
///     vec![0.0, 0.0, 0.0, 0.25, 0.25, 0.5, 0.5, 0.75, 0.75, 1.0, 1.0, 1.0]
/// );
///
/// // sign up the control points in the vector of all points
/// let ctrl_pts = vec![ // the vector of the indices of control points
///     vector!(0, -2, 0, 2),
///     vector!(1, -1, 0, 1),
///     vector!(1, 0, 0, 1),
///     vector!(1, 1, 0, 1),
///     vector!(0, 2, 0, 2),
///     vector!(-1, 1, 0, 1),
///     vector!(-1, 0, 0, 1),
///     vector!(-1, -1, 0, 1),
///     vector!(0, -2, 0, 2),
/// ];
///
/// // construct the B-spline curve
/// let bspline = BSplineCurve::new(knot_vec, ctrl_pts);
///
/// // This B-spline curve is a nurbs representation of the unit circle.
/// const N : usize = 100; // sample size in test
/// for i in 0..N {
///     let t = 1.0 / (N as f64) * (i as f64);
///     let v = bspline.subs(t); // We can use the instances as a function.
///     let c = (v[0] / v[3]).powi(2) + (v[1] / v[3]).powi(2);
///     f64::assert_near2(&c, &1.0);
/// }
/// ```
#[derive(Clone, PartialEq, Debug)]
pub struct BSplineCurve<T> {
    knot_vec: KnotVec,              // the knot vector
    control_points: Vec<Vector<T>>, // the indices of control points
}

/// 4-dimensional B-spline surface
/// # Examples
/// ```
/// use truck_geometry::*;
/// const N : usize = 100; // sample size in test
///
/// // the knot vectors
/// let knot_vec0 = KnotVec::bezier_knot(3);
/// let knot_vec1 = KnotVec::from(
///     vec![0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 1.0, 1.0, 1.0]
/// );
/// let knot_vecs = (knot_vec0, knot_vec1);
///
/// // the control points
/// let mut v = vec![vec![Vector::zero(); 7]; 4];
/// v[0][0] = rvector!(0, 0, 1);
/// v[0][1] = &v[0][0] / 3.0;
/// v[0][2] = v[0][1].clone();
/// v[0][3] = v[0][0].clone();
/// v[0][4] = v[0][1].clone();
/// v[0][5] = v[0][1].clone();
/// v[0][6] = v[0][0].clone();
/// v[1][0] = rvector!(2, 0, 1) / 3.0;
/// v[1][1] = rvector!(2, 4, 1) / 9.0;
/// v[1][2] = rvector!(-2, 4, 1) / 9.0;
/// v[1][3] = rvector!(-2, 0, 1) / 3.0;
/// v[1][4] = rvector!(-2, -4, 1) / 9.0;
/// v[1][5] = rvector!(2, -4, 1) / 9.0;
/// v[1][6] = rvector!(2, 0, 1) / 3.0;
/// v[2][0] = rvector!(2, 0, -1) / 3.0;
/// v[2][1] = rvector!(2, 4, -1) / 9.0;
/// v[2][2] = rvector!(-2, 4, -1) / 9.0;
/// v[2][3] = rvector!(-2, 0, -1) / 3.0;
/// v[2][4] = rvector!(-2, -4, -1) / 9.0;
/// v[2][5] = rvector!(2, -4, -1) / 9.0;
/// v[2][6] = rvector!(2, 0, -1) / 3.0;
/// v[3][0] = rvector!(0, 0, -1);
/// v[3][1] = &v[3][0] / 3.0;
/// v[3][2] = v[3][1].clone();
/// v[3][3] = v[3][0].clone();
/// v[3][4] = v[3][1].clone();
/// v[3][5] = v[3][1].clone();
/// v[3][6] = v[3][0].clone();
///
/// // cunstruct the B-spline curve
/// let bspline = BSplineSurface::new(knot_vecs, v);
///
/// // This B-spline curve is a nurbs representation of the unit sphere.
/// for i in 0..N {
///     for j in 0..N {
///         let u = 1.0 / (N as f64) * (i as f64);
///         let v = 1.0 / (N as f64) * (j as f64);
///         let v = bspline.subs(u, v); // We can use the instances as a function.
///         let c = (v[0] / v[3]).powi(2) + (v[1] / v[3]).powi(2) + (v[2] / v[3]).powi(2);
///         f64::assert_near2(&c, &1.0);
///     }
/// }
/// ```
#[derive(Clone, PartialEq, Debug)]
pub struct BSplineSurface<T> {
    knot_vecs: (KnotVec, KnotVec),
    control_points: Vec<Vec<Vector<T>>>,
}

/// Error handler for [`Error`](./errors/enum.Error.html)
pub type Result<T> = std::result::Result<T, crate::errors::Error>;

#[macro_use]
#[doc(hidden)]
pub mod vector;
#[doc(hidden)]
pub mod bounding_box;
#[doc(hidden)]
pub mod bspcurve;
/// Defines some iterators on control points of B-spline surface.
pub mod bspsurface;
/// Enumerats `Error`.
pub mod errors;
#[doc(hidden)]
pub mod knot_vec;
/// Defines some traits: `MatrixEntity` and `Determinant`.
pub mod matrix;
#[doc(hidden)]
pub mod tolerance;
