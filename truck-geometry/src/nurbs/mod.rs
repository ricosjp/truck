//! NURBS (Non-Uniform Rational B-Spline) curves and surfaces.
//!
//! A B-spline is a piecewise polynomial curve or surface defined by:
//! - **Degree** — polynomial order (1 = linear, 2 = quadratic, 3 = cubic).
//! - **Control points** — define the shape; the curve is attracted toward them
//!   but generally does not pass through them (except at endpoints of clamped knot vectors).
//! - **Knot vector** — a non-decreasing sequence of parameter values that determines
//!   where each polynomial piece begins and ends. Knot count = control points + degree + 1.
//!
//! A NURBS curve is a rational B-spline: each control point carries a **weight**,
//! allowing exact representation of conics (circles, ellipses). When all weights are
//! equal the curve reduces to a non-rational B-spline.

use crate::{prelude::*, *};
use truck_base::cgmath64::control_point::ControlPoint;

/// A non-decreasing sequence of parameter values defining the piecewise structure of a B-spline.
///
/// The knot vector length is always `control_points.len() + degree + 1`.
/// Repeated (multiple) knots reduce continuity at that parameter value;
/// a knot with multiplicity equal to degree creates a C⁰ joint.
#[derive(Clone, PartialEq, Debug, Default, Serialize)]
pub struct KnotVector(Vec<f64>);

/// B-spline curve
/// # Examples
/// ```
/// use truck_geometry::prelude::*;
///
/// // the knot vector
/// let knot_vec = KnotVector::from(
///     vec![0.0, 0.0, 0.0, 0.25, 0.25, 0.5, 0.5, 0.75, 0.75, 1.0, 1.0, 1.0]
/// );
///
/// // sign up the control points in the vector of all points
/// let control_points = vec![ // the vector of the indices of control points
///     Vector4::new(0.0, -2.0, 0.0, 2.0),
///     Vector4::new(1.0, -1.0, 0.0, 1.0),
///     Vector4::new(1.0, 0.0, 0.0, 1.0),
///     Vector4::new(1.0, 1.0, 0.0, 1.0),
///     Vector4::new(0.0, 2.0, 0.0, 2.0),
///     Vector4::new(-1.0, 1.0, 0.0, 1.0),
///     Vector4::new(-1.0, 0.0, 0.0, 1.0),
///     Vector4::new(-1.0, -1.0, 0.0, 1.0),
///     Vector4::new(0.0, -2.0, 0.0, 2.0),
/// ];
///
/// // construct the B-spline curve
/// let bspline = BsplineCurve::new(knot_vec, control_points);
///
/// // This B-spline curve is a nurbs representation of the unit circle.
/// const N : usize = 100; // sample size in test
/// for i in 0..N {
///     let t = 1.0 / (N as f64) * (i as f64);
///     let v = bspline.subs(t); // We can use the instances as a function.
///     let c = (v[0] / v[3]).powi(2) + (v[1] / v[3]).powi(2);
///     assert_near2!(c, 1.0);
/// }
/// ```
#[derive(Clone, PartialEq, Debug, Serialize, SelfSameGeometry)]
pub struct BsplineCurve<P> {
    knot_vec: KnotVector,   // the knot vector
    control_points: Vec<P>, // the indices of control points
}

/// B-spline surface
/// # Examples
/// ```
/// use truck_geometry::prelude::*;
/// const N : usize = 100; // sample size in test
///
/// // the knot vectors
/// let knot_vec0 = KnotVector::bezier_knot(3);
/// let knot_vec1 = KnotVector::from(
///     vec![0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 1.0, 1.0, 1.0]
/// );
/// let knot_vecs = (knot_vec0, knot_vec1);
///
/// // the control points
/// let mut v = vec![vec![Vector4::zero(); 7]; 4];
/// v[0][0] = Vector4::new(0.0, 0.0, 1.0, 1.0);
/// v[0][1] = &v[0][0] / 3.0;
/// v[0][2] = v[0][1].clone();
/// v[0][3] = v[0][0].clone();
/// v[0][4] = v[0][1].clone();
/// v[0][5] = v[0][1].clone();
/// v[0][6] = v[0][0].clone();
/// v[1][0] = Vector4::new(2.0, 0.0, 1.0, 1.0) / 3.0;
/// v[1][1] = Vector4::new(2.0, 4.0, 1.0, 1.0) / 9.0;
/// v[1][2] = Vector4::new(-2.0, 4.0, 1.0, 1.0) / 9.0;
/// v[1][3] = Vector4::new(-2.0, 0.0, 1.0, 1.0) / 3.0;
/// v[1][4] = Vector4::new(-2.0, -4.0, 1.0, 1.0) / 9.0;
/// v[1][5] = Vector4::new(2.0, -4.0, 1.0, 1.0) / 9.0;
/// v[1][6] = Vector4::new(2.0, 0.0, 1.0, 1.0) / 3.0;
/// v[2][0] = Vector4::new(2.0, 0.0, -1.0, 1.0) / 3.0;
/// v[2][1] = Vector4::new(2.0, 4.0, -1.0, 1.0) / 9.0;
/// v[2][2] = Vector4::new(-2.0, 4.0, -1.0, 1.0) / 9.0;
/// v[2][3] = Vector4::new(-2.0, 0.0, -1.0, 1.0) / 3.0;
/// v[2][4] = Vector4::new(-2.0, -4.0, -1.0, 1.0) / 9.0;
/// v[2][5] = Vector4::new(2.0, -4.0, -1.0, 1.0) / 9.0;
/// v[2][6] = Vector4::new(2.0, 0.0, -1.0, 1.0) / 3.0;
/// v[3][0] = Vector4::new(0.0, 0.0, -1.0, 1.0);
/// v[3][1] = &v[3][0] / 3.0;
/// v[3][2] = v[3][1].clone();
/// v[3][3] = v[3][0].clone();
/// v[3][4] = v[3][1].clone();
/// v[3][5] = v[3][1].clone();
/// v[3][6] = v[3][0].clone();
///
/// // cunstruct the B-spline curve
/// let bspline = BsplineSurface::new(knot_vecs, v);
///
/// // This B-spline curve is a nurbs representation of the unit sphere.
/// for i in 0..N {
///     for j in 0..N {
///         let u = 1.0 / (N as f64) * (i as f64);
///         let v = 1.0 / (N as f64) * (j as f64);
///         let v = bspline.subs(u, v); // We can use the instances as a function.
///         let c = (v[0] / v[3]).powi(2) + (v[1] / v[3]).powi(2) + (v[2] / v[3]).powi(2);
///         assert_near2!(c, 1.0);
///     }
/// }
/// ```
#[derive(Clone, PartialEq, Debug, Serialize, SelfSameGeometry)]
pub struct BsplineSurface<P> {
    knot_vecs: (KnotVector, KnotVector),
    control_points: Vec<Vec<P>>,
}

/// Rational B-spline curve — a B-spline with weighted control points.
///
/// The generic parameter `V` is typically `Vector4`: the first three components
/// are `w·x, w·y, w·z` and the fourth is the weight `w`. This homogeneous
/// representation allows exact circles, ellipses, and other conics.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, SelfSameGeometry)]
pub struct NurbsCurve<V>(BsplineCurve<V>);

/// Rational B-spline surface — a tensor-product B-spline with weighted control points.
///
/// The generic parameter `V` is typically `Vector4` (homogeneous coordinates).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, SelfSameGeometry)]
pub struct NurbsSurface<V>(BsplineSurface<V>);

/// Renamed to [`BsplineCurve`] per RFC 430 (C-CASE).
#[deprecated(note = "renamed to BsplineCurve per RFC 430 (C-CASE)")]
pub type BSplineCurve<P> = BsplineCurve<P>;

/// Renamed to [`BsplineSurface`] per RFC 430 (C-CASE).
#[deprecated(note = "renamed to BsplineSurface per RFC 430 (C-CASE)")]
pub type BSplineSurface<P> = BsplineSurface<P>;

/// Renamed to [`KnotVector`] for clarity.
#[deprecated(note = "renamed to KnotVector for clarity")]
pub type KnotVec = KnotVector;

mod bspline_curve;
mod bspline_surface;
mod knot_vector;
mod nurbs_curve;
mod nurbs_surface;

#[doc(hidden)]
#[inline(always)]
pub const fn inv_or_zero(delta: f64) -> f64 {
    if delta.abs() <= TOLERANCE {
        0.0
    } else {
        1.0 / delta
    }
}

// This code is modified version of https://the-algorithms.com/algorithm/gaussian-elimination?lang=rust
mod gaussian_elimination {
    use truck_base::cgmath64::cgmath::BaseFloat;

    // Gaussian Elimination of Quadratic Matrices
    // Takes an augmented matrix as input, returns vector of results
    // Wikipedia reference: augmented matrix: https://en.wikipedia.org/wiki/Augmented_matrix
    // Wikipedia reference: algorithm: https://en.wikipedia.org/wiki/Gaussian_elimination

    pub fn gaussian_elimination<S: BaseFloat>(matrix: &mut [Vec<S>]) -> Option<Vec<S>> {
        let size = matrix.len();
        if size != matrix[0].len() - 1 {
            return None;
        }

        for i in 0..size - 1 {
            for j in i..size - 1 {
                echelon(matrix, i, j);
            }
        }

        for i in (1..size).rev() {
            eliminate(matrix, i);
        }

        // Disable cargo clippy warnings about needless range loops.
        // Checking the diagonal like this is simpler than any alternative.
        #[allow(clippy::needless_range_loop)]
        for i in 0..size {
            if matrix[i][i].is_zero() {
                return None;
            }
        }

        Some((0..size).map(|i| matrix[i][size] / matrix[i][i]).collect())
    }

    fn echelon<S: BaseFloat>(matrix: &mut [Vec<S>], i: usize, j: usize) {
        let size = matrix.len();
        if matrix[i][i] != S::zero() {
            let factor = matrix[j + 1][i] / matrix[i][i];
            (i..size + 1).for_each(|k| {
                matrix[j + 1][k] = matrix[j + 1][k] - factor * matrix[i][k];
            });
        }
    }

    fn eliminate<S: BaseFloat>(matrix: &mut [Vec<S>], i: usize) {
        let size = matrix.len();
        if matrix[i][i] != S::zero() {
            for j in (1..i + 1).rev() {
                let factor = matrix[j - 1][i] / matrix[i][i];
                for k in (0..size + 1).rev() {
                    matrix[j - 1][k] = matrix[j - 1][k] - factor * matrix[i][k];
                }
            }
        }
    }
}
