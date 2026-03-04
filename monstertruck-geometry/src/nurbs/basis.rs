//! Basis conversion types for importing curves from non-B-spline representations.
//!
//! Each source representation is a typed struct that implements
//! [`From`]/[`Into`] for [`BsplineCurve`], following Rust conversion idioms.
//!
//! # Supported conversions
//!
//! | Source type | Description |
//! |---|---|
//! | [`HermiteSegment`] | Cubic Hermite (endpoints + tangents) |
//! | [`CatmullRomSpline`] | Catmull-Rom through a point sequence |
//! | [`PowerBasisCurve`] | Monomial polynomial coefficients |
//! | [`PiecewiseBezier`] | Concatenated Bezier segments |
//!
//! # Examples
//!
//! ```
//! use monstertruck_geometry::prelude::*;
//! use monstertruck_geometry::nurbs::basis::HermiteSegment;
//!
//! let hermite = HermiteSegment {
//!     p0: Point3::new(0.0, 0.0, 0.0),
//!     t0: Vector3::new(3.0, 0.0, 0.0),
//!     p1: Point3::new(3.0, 0.0, 0.0),
//!     t1: Vector3::new(3.0, 0.0, 0.0),
//! };
//! let curve: BsplineCurve<Point3> = hermite.into();
//! assert_near2!(curve.subs(0.0), Point3::new(0.0, 0.0, 0.0));
//! assert_near2!(curve.subs(1.0), Point3::new(3.0, 0.0, 0.0));
//! ```

use super::*;

/// A cubic Hermite segment defined by two endpoints and their tangent vectors.
///
/// Converts to a cubic Bezier [`BsplineCurve`] parameterized over \[0, 1\]
/// using the standard Hermite-to-Bezier control point mapping:
///
/// - `c0 = p0`
/// - `c1 = p0 + t0 / 3`
/// - `c2 = p1 - t1 / 3`
/// - `c3 = p1`
///
/// # Examples
///
/// ```
/// use monstertruck_geometry::prelude::*;
/// use monstertruck_geometry::nurbs::basis::HermiteSegment;
///
/// let seg = HermiteSegment {
///     p0: Point3::new(0.0, 0.0, 0.0),
///     t0: Vector3::new(3.0, 3.0, 0.0),
///     p1: Point3::new(3.0, 0.0, 0.0),
///     t1: Vector3::new(3.0, -3.0, 0.0),
/// };
/// let curve = BsplineCurve::from(seg);
/// assert_near2!(curve.subs(0.0), Point3::new(0.0, 0.0, 0.0));
/// assert_near2!(curve.subs(1.0), Point3::new(3.0, 0.0, 0.0));
/// assert_near2!(curve.der(0.0), Vector3::new(3.0, 3.0, 0.0));
/// ```
#[derive(Clone, Debug)]
pub struct HermiteSegment<P: ControlPoint<f64>> {
    /// Start point.
    pub p0: P,
    /// Tangent at start.
    pub t0: P::Diff,
    /// End point.
    pub p1: P,
    /// Tangent at end.
    pub t1: P::Diff,
}

impl<P: ControlPoint<f64> + Tolerance> From<HermiteSegment<P>> for BsplineCurve<P> {
    fn from(h: HermiteSegment<P>) -> Self {
        let c0 = h.p0;
        let c1 = h.p0 + h.t0 / 3.0;
        let c2 = h.p1 - h.t1 / 3.0;
        let c3 = h.p1;
        BsplineCurve::new(KnotVector::bezier_knot(3), vec![c0, c1, c2, c3])
    }
}

/// A Catmull-Rom spline through a sequence of points.
///
/// Given N >= 4 points, the resulting [`BsplineCurve`] interpolates all points
/// except the first and last (which serve only to define tangents). The tangent
/// at each interior point is `(p[i+1] - p[i-1]) / 2`.
///
/// # Panics
///
/// Panics on conversion if fewer than 4 points are provided.
///
/// # Examples
///
/// ```
/// use monstertruck_geometry::prelude::*;
/// use monstertruck_geometry::nurbs::basis::CatmullRomSpline;
///
/// let spline = CatmullRomSpline(vec![
///     Point3::new(-1.0, 0.0, 0.0),
///     Point3::new(0.0, 0.0, 0.0),
///     Point3::new(1.0, 1.0, 0.0),
///     Point3::new(2.0, 0.0, 0.0),
///     Point3::new(3.0, 0.0, 0.0),
/// ]);
/// let curve = BsplineCurve::from(spline);
/// assert_near2!(curve.subs(0.0), Point3::new(0.0, 0.0, 0.0));
/// assert_near2!(curve.subs(1.0), Point3::new(2.0, 0.0, 0.0));
/// ```
#[derive(Clone, Debug)]
pub struct CatmullRomSpline<P>(pub Vec<P>);

impl<P: ControlPoint<f64> + Tolerance> From<CatmullRomSpline<P>> for BsplineCurve<P> {
    fn from(cr: CatmullRomSpline<P>) -> Self {
        let points = &cr.0;
        assert!(
            points.len() >= 4,
            "CatmullRomSpline requires at least 4 points"
        );

        let segments: Vec<BsplineCurve<P>> = (0..points.len() - 3)
            .map(|i| {
                let h = HermiteSegment {
                    p0: points[i + 1],
                    t0: (points[i + 2] - points[i]) / 2.0,
                    p1: points[i + 2],
                    t1: (points[i + 3] - points[i + 1]) / 2.0,
                };
                BsplineCurve::from(h)
            })
            .collect();

        BsplineCurve::from(PiecewiseBezier(segments))
    }
}

/// A polynomial curve in power (monomial) basis.
///
/// Coefficients `[c0, c1, …, cn]` represent `p(t) = c0 + c1·t + c2·t² + … + cn·tⁿ`.
/// Converts to a degree-n Bezier [`BsplineCurve`] over \[0, 1\] via the
/// power-to-Bernstein matrix.
///
/// # Panics
///
/// Panics on conversion if `coeffs` is empty.
///
/// # Examples
///
/// ```
/// use monstertruck_geometry::prelude::*;
/// use monstertruck_geometry::nurbs::basis::PowerBasisCurve;
///
/// // p(t) = (1,0,0) + (2,3,0)·t.
/// let poly = PowerBasisCurve(vec![
///     Point3::new(1.0, 0.0, 0.0),
///     Point3::new(2.0, 3.0, 0.0),
/// ]);
/// let curve = BsplineCurve::from(poly);
/// assert_near2!(curve.subs(0.0), Point3::new(1.0, 0.0, 0.0));
/// assert_near2!(curve.subs(1.0), Point3::new(3.0, 3.0, 0.0));
/// ```
#[derive(Clone, Debug)]
pub struct PowerBasisCurve<P>(pub Vec<P>);

impl<P: ControlPoint<f64> + Tolerance> From<PowerBasisCurve<P>> for BsplineCurve<P> {
    fn from(pbc: PowerBasisCurve<P>) -> Self {
        let coeffs = &pbc.0;
        assert!(
            !coeffs.is_empty(),
            "PowerBasisCurve requires at least one coefficient"
        );
        let n = coeffs.len() - 1;

        // Power-to-Bernstein: B_i = Σ_{j=0}^{i} C(i,j)/C(n,j) · a_j.
        let bernstein: Vec<P> = (0..=n)
            .map(|i| {
                coeffs[..=i]
                    .iter()
                    .enumerate()
                    .fold(P::origin(), |acc, (j, cj)| {
                        let w = binomial(i, j) as f64 / binomial(n, j) as f64;
                        acc + cj.to_vec() * w
                    })
            })
            .collect();

        BsplineCurve::new(KnotVector::bezier_knot(n), bernstein)
    }
}

/// A sequence of Bezier curve segments to be concatenated into a single [`BsplineCurve`].
///
/// All segments must share the same degree. The last control point of each
/// segment must coincide with the first control point of the next. Internal
/// knots give C⁰ continuity at segment boundaries.
///
/// # Panics
///
/// Panics on conversion if `segments` is empty.
///
/// # Examples
///
/// ```
/// use monstertruck_geometry::prelude::*;
/// use monstertruck_geometry::nurbs::basis::PiecewiseBezier;
///
/// let seg1 = BsplineCurve::new(
///     KnotVector::bezier_knot(3),
///     vec![
///         Point3::new(0.0, 0.0, 0.0),
///         Point3::new(1.0, 1.0, 0.0),
///         Point3::new(2.0, 1.0, 0.0),
///         Point3::new(3.0, 0.0, 0.0),
///     ],
/// );
/// let seg2 = BsplineCurve::new(
///     KnotVector::bezier_knot(3),
///     vec![
///         Point3::new(3.0, 0.0, 0.0),
///         Point3::new(4.0, -1.0, 0.0),
///         Point3::new(5.0, -1.0, 0.0),
///         Point3::new(6.0, 0.0, 0.0),
///     ],
/// );
/// let combined = BsplineCurve::from(PiecewiseBezier(vec![seg1, seg2]));
/// assert_near2!(combined.subs(0.0), Point3::new(0.0, 0.0, 0.0));
/// assert_near2!(combined.subs(1.0), Point3::new(6.0, 0.0, 0.0));
/// ```
#[derive(Clone, Debug)]
pub struct PiecewiseBezier<P>(pub Vec<BsplineCurve<P>>);

impl<P: ControlPoint<f64> + Tolerance> From<PiecewiseBezier<P>> for BsplineCurve<P> {
    fn from(pb: PiecewiseBezier<P>) -> Self {
        let segments = pb.0;
        assert!(
            !segments.is_empty(),
            "PiecewiseBezier requires at least one segment"
        );

        if segments.len() == 1 {
            // SAFETY: We just checked len() == 1.
            return segments.into_iter().next().unwrap();
        }

        let degree = segments[0].degree();
        let n_segs = segments.len();

        // Clamped knot vector with C⁰ internal knots at segment boundaries.
        let mut knots = Vec::with_capacity(n_segs * degree + degree + 2);
        knots.extend(std::iter::repeat_n(0.0, degree + 1));
        (1..n_segs).for_each(|i| knots.extend(std::iter::repeat_n(i as f64, degree)));
        knots.extend(std::iter::repeat_n(n_segs as f64, degree + 1));

        // Normalize to [0, 1].
        let max_knot = n_segs as f64;
        knots.iter_mut().for_each(|k| *k /= max_knot);

        // First segment fully, then skip shared first CP of subsequent segments.
        let mut control_points: Vec<P> = segments[0].control_points().clone();
        segments[1..].iter().for_each(|seg| {
            control_points.extend_from_slice(&seg.control_points()[1..]);
        });

        BsplineCurve::new(KnotVector::from(knots), control_points)
    }
}

/// Computes the binomial coefficient C(n, k).
fn binomial(n: usize, k: usize) -> usize {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    (0..k).fold(1, |acc, i| acc * (n - i) / (i + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hermite_endpoints_and_tangents() {
        let curve = BsplineCurve::from(HermiteSegment {
            p0: Point3::new(0.0, 0.0, 0.0),
            t0: Vector3::new(3.0, 3.0, 0.0),
            p1: Point3::new(3.0, 0.0, 0.0),
            t1: Vector3::new(3.0, -3.0, 0.0),
        });
        assert_near2!(curve.subs(0.0), Point3::new(0.0, 0.0, 0.0));
        assert_near2!(curve.subs(1.0), Point3::new(3.0, 0.0, 0.0));
        assert_near2!(curve.der(0.0), Vector3::new(3.0, 3.0, 0.0));
        assert_near2!(curve.der(1.0), Vector3::new(3.0, -3.0, 0.0));
    }

    #[test]
    fn catmull_rom_interpolates_interior() {
        let curve = BsplineCurve::from(CatmullRomSpline(vec![
            Point3::new(-1.0, 0.0, 0.0),
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 1.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(3.0, 0.0, 0.0),
        ]));
        assert_near2!(curve.subs(0.0), Point3::new(0.0, 0.0, 0.0));
        assert_near2!(curve.subs(1.0), Point3::new(2.0, 0.0, 0.0));
    }

    #[test]
    fn power_basis_linear() {
        let curve = BsplineCurve::from(PowerBasisCurve(vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(3.0, 4.0, 0.0),
        ]));
        assert_near2!(curve.subs(0.0), Point3::new(0.0, 0.0, 0.0));
        assert_near2!(curve.subs(1.0), Point3::new(3.0, 4.0, 0.0));
        assert_near2!(curve.subs(0.5), Point3::new(1.5, 2.0, 0.0));
    }

    #[test]
    fn power_basis_quadratic() {
        // p(t) = (t², 0, 0).
        let curve = BsplineCurve::from(PowerBasisCurve(vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
        ]));
        assert_near2!(curve.subs(0.0), Point3::new(0.0, 0.0, 0.0));
        assert_near2!(curve.subs(1.0), Point3::new(1.0, 0.0, 0.0));
        assert_near2!(curve.subs(0.5), Point3::new(0.25, 0.0, 0.0));
    }

    #[test]
    fn concat_two_cubic_segments() {
        let seg1 = BsplineCurve::new(
            KnotVector::bezier_knot(3),
            vec![
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.0, 2.0, 0.0),
                Point3::new(2.0, 2.0, 0.0),
                Point3::new(3.0, 0.0, 0.0),
            ],
        );
        let seg2 = BsplineCurve::new(
            KnotVector::bezier_knot(3),
            vec![
                Point3::new(3.0, 0.0, 0.0),
                Point3::new(4.0, -2.0, 0.0),
                Point3::new(5.0, -2.0, 0.0),
                Point3::new(6.0, 0.0, 0.0),
            ],
        );
        let combined = BsplineCurve::from(PiecewiseBezier(vec![seg1, seg2]));
        assert_near2!(combined.subs(0.0), Point3::new(0.0, 0.0, 0.0));
        assert_near2!(combined.subs(0.5), Point3::new(3.0, 0.0, 0.0));
        assert_near2!(combined.subs(1.0), Point3::new(6.0, 0.0, 0.0));
    }
}
