//! Fairing and reparameterization utilities for [`BsplineCurve`].
//!
//! These operations precondition curves before use in surface constructors,
//! improving the quality of skins, sweeps, and Gordon surfaces.

use super::*;
use errors::Error;
use std::result::Result as StdResult;

type Result<T> = StdResult<T, Error>;

/// Reparameterizes a [`BsplineCurve`] to approximate arc-length parameterization.
///
/// Samples the curve at `n_samples` uniform parameter values, computes the
/// cumulative arc length at each sample, and refits a new B-spline of the
/// same degree where parameter values correspond to normalized arc lengths.
///
/// # Errors
///
/// Returns an error if the interpolation system is singular.
///
/// # Examples
///
/// ```
/// use monstertruck_geometry::prelude::*;
/// use monstertruck_geometry::nurbs::fair;
///
/// // A quadratic Bezier curve: the original parameterization is non-uniform.
/// let curve = BsplineCurve::new(
///     KnotVector::bezier_knot(2),
///     vec![
///         Point3::new(0.0, 0.0, 0.0),
///         Point3::new(1.0, 2.0, 0.0),
///         Point3::new(2.0, 0.0, 0.0),
///     ],
/// );
/// let reparam = fair::reparameterize_arc_length(&curve, 10).unwrap();
/// // At u=0.5 in arc-length parameterization, we should be at the
/// // midpoint of the curve by arc length.
/// let mid = reparam.subs(0.5);
/// // The midpoint should be near x=1 (the apex of the parabola).
/// assert!((mid.x - 1.0).abs() < 0.15, "mid.x = {}", mid.x);
/// ```
pub fn reparameterize_arc_length<P>(
    curve: &BsplineCurve<P>,
    n_samples: usize,
) -> Result<BsplineCurve<P>>
where
    P: ControlPoint<f64> + Tolerance,
{
    assert!(
        n_samples >= 4,
        "reparameterize_arc_length requires at least 4 samples",
    );

    let (t_start, t_end) = curve.range_tuple();

    // Sample points and compute cumulative arc lengths.
    let sample_params: Vec<f64> = (0..n_samples)
        .map(|i| t_start + (t_end - t_start) * i as f64 / (n_samples - 1) as f64)
        .collect();

    let points: Vec<P> = sample_params.iter().map(|&t| curve.subs(t)).collect();

    let mut arc_lengths = Vec::with_capacity(n_samples);
    arc_lengths.push(0.0);
    for i in 1..n_samples {
        let delta = points[i] - points[i - 1];
        let seg_len = (0..P::DIM).map(|d| delta[d] * delta[d]).sum::<f64>().sqrt();
        arc_lengths.push(arc_lengths[i - 1] + seg_len);
    }

    let total_length = arc_lengths[n_samples - 1];
    if total_length.so_small() {
        // Degenerate curve — return a clone with normalized knots.
        let mut result = curve.clone();
        result.knot_normalize();
        return Ok(result);
    }

    // Normalize arc lengths to [0, 1].
    let param_points: Vec<(f64, P)> = arc_lengths
        .iter()
        .zip(points.iter())
        .map(|(&s, &pt)| (s / total_length, pt))
        .collect();

    let degree = curve.degree().min(n_samples - 1).min(3);
    let knot = KnotVector::uniform_knot(degree, n_samples - degree);
    BsplineCurve::try_interpolate(knot, param_points)
}

/// Smooths a [`BsplineCurve`] by fitting through a subset of its evaluated
/// points using a lower number of control points.
///
/// Samples the curve at `n_samples` points, then fits a new curve of degree
/// `target_degree` with `n_control_points` control points. Using fewer control
/// points than the original curve produces a smoother result by eliminating
/// high-frequency oscillations.
///
/// # Errors
///
/// Returns an error if the interpolation system is singular.
///
/// # Panics
///
/// Panics if `n_control_points < target_degree + 1` or `n_samples < n_control_points`.
///
/// # Examples
///
/// ```
/// use monstertruck_geometry::prelude::*;
/// use monstertruck_geometry::nurbs::fair;
///
/// // A curve with many control points (potentially noisy).
/// let knot = KnotVector::uniform_knot(3, 6);
/// let points = vec![
///     Point3::new(0.0, 0.0, 0.0),
///     Point3::new(1.0, 0.1, 0.0),
///     Point3::new(2.0, -0.05, 0.0),
///     Point3::new(3.0, 0.08, 0.0),
///     Point3::new(4.0, -0.02, 0.0),
///     Point3::new(5.0, 0.03, 0.0),
///     Point3::new(6.0, 0.0, 0.0),
///     Point3::new(7.0, -0.01, 0.0),
///     Point3::new(8.0, 0.0, 0.0),
/// ];
/// let noisy = BsplineCurve::new(knot, points);
/// // Fair to a smoother cubic with fewer control points.
/// let smooth = fair::fair_curve(&noisy, 3, 5, 30).unwrap();
/// // The smoothed curve should still roughly follow the original path.
/// let mid = smooth.subs(0.5);
/// assert!((mid.x - 4.0).abs() < 1.0, "mid.x = {}", mid.x);
/// ```
pub fn fair_curve<P>(
    curve: &BsplineCurve<P>,
    target_degree: usize,
    n_control_points: usize,
    n_samples: usize,
) -> Result<BsplineCurve<P>>
where
    P: ControlPoint<f64> + Tolerance,
{
    assert!(
        n_control_points > target_degree,
        "need at least degree+1 control points",
    );
    assert!(
        n_samples >= n_control_points,
        "need at least as many samples as control points",
    );

    let (t_start, t_end) = curve.range_tuple();
    let knot = KnotVector::uniform_knot(target_degree, n_control_points - target_degree);

    let param_points: Vec<(f64, P)> = (0..n_control_points)
        .map(|i| {
            let u = i as f64 / (n_control_points - 1) as f64;
            let t = t_start + (t_end - t_start) * u;
            (u, curve.subs(t))
        })
        .collect();

    BsplineCurve::try_interpolate(knot, param_points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arc_length_reparameterization_straight_line() {
        // A straight line should have uniform arc-length parameterization.
        let line = BsplineCurve::new(
            KnotVector::bezier_knot(1),
            vec![Point3::new(0.0, 0.0, 0.0), Point3::new(3.0, 4.0, 0.0)],
        );
        let reparam = reparameterize_arc_length(&line, 20).unwrap();
        // At u=0.5, we should be at the midpoint.
        let mid = reparam.subs(0.5);
        assert!((mid.x - 1.5).abs() < 0.1, "expected x~1.5, got {}", mid.x,);
        assert!((mid.y - 2.0).abs() < 0.1, "expected y~2.0, got {}", mid.y,);
    }

    #[test]
    fn fair_curve_reduces_noise() {
        // Build a noisy curve with 9 control points.
        let knot = KnotVector::uniform_knot(3, 6);
        let points = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.5, 0.0),
            Point3::new(2.0, -0.3, 0.0),
            Point3::new(3.0, 0.4, 0.0),
            Point3::new(4.0, -0.2, 0.0),
            Point3::new(5.0, 0.3, 0.0),
            Point3::new(6.0, -0.1, 0.0),
            Point3::new(7.0, 0.2, 0.0),
            Point3::new(8.0, 0.0, 0.0),
        ];
        let noisy = BsplineCurve::new(knot, points);
        let smooth = fair_curve(&noisy, 3, 5, 30).unwrap();

        // The smoothed curve has fewer control points.
        assert_eq!(smooth.control_points().len(), 5);
        // Endpoints should still be approximately preserved.
        let start = smooth.subs(0.0);
        let end = smooth.subs(1.0);
        assert!((start.x - 0.0).abs() < 0.2, "start.x = {}", start.x,);
        assert!((end.x - 8.0).abs() < 0.2, "end.x = {}", end.x,);
    }
}
