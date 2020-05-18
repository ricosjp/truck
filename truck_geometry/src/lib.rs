#![feature(unboxed_closures, fn_traits, is_sorted)]

/// 4 dimentional vector
#[derive(Clone, PartialEq, Debug)]
pub struct Vector {
    x: [f64; 4],
}

/// 4x4 matrix
#[derive(Clone, PartialEq, Debug)]
pub struct Matrix {
    a: [[f64; 4]; 4],
}

/// knot vector
#[derive(Clone, PartialEq, Debug)]
pub struct KnotVec {
    entity: std::vec::Vec<f64>,
}

/// define a tolerance in the whole package
pub trait Tolerance: Sized + std::fmt::Debug {
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

pub trait Origin: Tolerance {
    /// origin point
    const ORIGIN: Self;

    /// near origin
    #[inline(always)]
    fn so_small(&self) -> bool { self.near(&Self::ORIGIN) }

    /// near origin in square order
    #[inline(always)]
    fn so_small2(&self) -> bool { self.near2(&Self::ORIGIN) }
}

/// 4-dimensional B-spline curve
/// # Examples
/// ```
/// use truck_geometry::*;
/// const N : usize = 100; // sample size in test
///
/// // the knot vector
/// let knot_vec = KnotVec::from(vec![0.0, 0.0, 0.0, 0.25, 0.25, 0.5, 0.5, 0.75, 0.75, 1.0, 1.0, 1.0]);
///
/// // sign up the control points in the vector of all points
/// let control_points = vec![ // the vector of the indices of control points
///     Vector::new(0.0, -2.0, 0.0, 2.0),
///     Vector::new(1.0, -1.0, 0.0, 1.0),
///     Vector::new(1.0, 0.0, 0.0, 1.0),
///     Vector::new(1.0, 1.0, 0.0, 1.0),
///     Vector::new(0.0, 2.0, 0.0, 2.0),
///     Vector::new(-1.0, 1.0, 0.0, 1.0),
///     Vector::new(-1.0, 0.0, 0.0, 1.0),
///     Vector::new(-1.0, -1.0, 0.0, 1.0),
///     Vector::new(0.0, -2.0, 0.0, 2.0),
/// ];
///
/// // cunstruct the B-spline curve
/// let bspline = BSplineCurve::new(knot_vec, control_points);
///
/// // This B-spline curve is a nurbs representation of the unit circle.
/// for i in 0..N {
///     let t = 1.0 / (N as f64) * (i as f64);
///     let v = bspline(t); // We can use the instances as a function.
///     let c = (v[0] / v[3]).powi(2) + (v[1] / v[3]).powi(2);
///     f64::assert_near2(&c, &1.0);
/// }
/// ```
#[derive(Clone, PartialEq, Debug)]
pub struct BSplineCurve {
    knot_vec: KnotVec,           // the knot vector
    control_points: Vec<Vector>, // the indices of control points
    derivation: Option<Box<BSplineCurve>>,
}

/// 4-dimensional B-spline surface
/// # Examples
/// ```
/// use truck_geometry::*;
/// const N : usize = 100; // sample size in test
///
/// // the knot vectors
/// let knot_vec0 = KnotVec::from(vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0]);
/// let knot_vec1 = KnotVec::from(vec![0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 1.0, 1.0, 1.0]);
/// let knot_vecs = (knot_vec0, knot_vec1);
///
/// // the control points
/// let mut v = vec![vec![Vector::zero(); 7]; 4];
/// v[0][0] = Vector::new3(0.0, 0.0, 1.0);
/// v[0][1] = &v[0][0] / 3.0;
/// v[0][2] = v[0][1].clone();
/// v[0][3] = v[0][0].clone();
/// v[0][4] = v[0][1].clone();
/// v[0][5] = v[0][1].clone();
/// v[0][6] = v[0][0].clone();
/// v[1][0] = Vector::new3(2.0, 0.0, 1.0) / 3.0;
/// v[1][1] = Vector::new3(2.0, 4.0, 1.0) / 9.0;
/// v[1][2] = Vector::new3(-2.0, 4.0, 1.0) / 9.0;
/// v[1][3] = Vector::new3(-2.0, 0.0, 1.0) / 3.0;
/// v[1][4] = Vector::new3(-2.0, -4.0, 1.0) / 9.0;
/// v[1][5] = Vector::new3(2.0, -4.0, 1.0) / 9.0;
/// v[1][6] = Vector::new3(2.0, 0.0, 1.0) / 3.0;
/// v[2][0] = Vector::new3(2.0, 0.0, -1.0) / 3.0;
/// v[2][1] = Vector::new3(2.0, 4.0, -1.0) / 9.0;
/// v[2][2] = Vector::new3(-2.0, 4.0, -1.0) / 9.0;
/// v[2][3] = Vector::new3(-2.0, 0.0, -1.0) / 3.0;
/// v[2][4] = Vector::new3(-2.0, -4.0, -1.0) / 9.0;
/// v[2][5] = Vector::new3(2.0, -4.0, -1.0) / 9.0;
/// v[2][6] = Vector::new3(2.0, 0.0, -1.0) / 3.0;
/// v[3][0] = Vector::new3(0.0, 0.0, -1.0);
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
///         let s = 1.0 / (N as f64) * (i as f64);
///         let t = 1.0 / (N as f64) * (j as f64);
///         let v = bspline(s, t); // We can use the instances as a function.
///         let c = (v[0] / v[3]).powi(2) + (v[1] / v[3]).powi(2) + (v[2] / v[3]).powi(2);
///         f64::assert_near2(&c, &1.0);
///     }
/// }
/// ```
#[derive(Clone, PartialEq, Debug)]
pub struct BSplineSurface {
    knot_vecs: (KnotVec, KnotVec),
    control_points: Vec<Vec<Vector>>,
    first_derivation: Option<Box<BSplineSurface>>,
    second_derivation: Option<Box<BSplineSurface>>,
}

pub type Result<T> = std::result::Result<T, crate::errors::Error>;

pub mod bspcurve;
pub mod bspsurface;
pub mod errors;
pub mod knot_vec;
pub mod matrix;
pub mod tolerance;
pub mod vector;
