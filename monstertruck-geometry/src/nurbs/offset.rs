//! Curve and surface offset operations.
//!
//! Offsets of B-spline curves and surfaces are generally not representable
//! exactly as B-splines. The functions here use a sample-and-refit strategy:
//! evaluate the original geometry and its normals at many points, offset each
//! sample, and fit a new [`BsplineCurve`] or [`BsplineSurface`] through the
//! offset points.

use super::*;
use errors::Error;
use std::result::Result as StdResult;

type Result<T> = StdResult<T, Error>;

/// Offsets a planar [`BsplineCurve`] in the XY plane by `distance`.
///
/// Positive `distance` offsets to the left of the curve direction (toward the
/// +90 degree rotation of the tangent). The result is a cubic B-spline with
/// `n_samples` control-point-equivalent interpolation points.
///
/// # Errors
///
/// Returns an error if the interpolation system is singular (e.g., degenerate
/// curve with coincident sample points).
///
/// # Examples
///
/// ```
/// use monstertruck_geometry::prelude::*;
/// use monstertruck_geometry::nurbs::offset;
///
/// // A straight line along x from (0,0) to (4,0).
/// let line = BsplineCurve::new(
///     KnotVector::bezier_knot(1),
///     vec![Point2::new(0.0, 0.0), Point2::new(4.0, 0.0)],
/// );
/// let offset = offset::curve_offset_2d(&line, 1.0, 20).unwrap();
/// // The offset should be approximately at y=1.
/// let mid = offset.subs(0.5);
/// assert!((mid.y - 1.0).abs() < 0.05, "mid.y = {}", mid.y);
/// assert!((mid.x - 2.0).abs() < 0.05, "mid.x = {}", mid.x);
/// ```
pub fn curve_offset_2d(
    curve: &BsplineCurve<Point2>,
    distance: f64,
    n_samples: usize,
) -> Result<BsplineCurve<Point2>> {
    assert!(
        n_samples >= 4,
        "curve_offset_2d requires at least 4 samples",
    );

    let (t_start, t_end) = curve.range_tuple();

    // Sample offset points.
    let param_points: Vec<(f64, Point2)> = (0..n_samples)
        .map(|i| {
            let u = i as f64 / (n_samples - 1) as f64;
            let t = t_start + (t_end - t_start) * u;
            let pt = curve.subs(t);
            let tangent = curve.der(t);

            // Normal: rotate tangent 90 degrees CCW in 2D.
            let normal_len = tangent.magnitude();
            let normal = if normal_len.so_small() {
                Vector2::new(0.0, 0.0)
            } else {
                Vector2::new(-tangent.y, tangent.x) / normal_len
            };

            let offset_pt = pt + normal * distance;
            // Map sample parameter to [0, 1] range for interpolation.
            (u, offset_pt)
        })
        .collect();

    // Fit a cubic B-spline through the offset points.
    let degree = 3.min(n_samples - 1);
    let knot = KnotVector::uniform_knot(degree, n_samples - degree);
    BsplineCurve::try_interpolate(knot, param_points)
}

/// Offsets a 3D [`BsplineCurve`] by `distance` in the direction of the
/// given `normal` plane.
///
/// At each sample point, the offset direction is computed as the component
/// of `normal` that is perpendicular to the tangent, normalized. This works
/// well for curves that lie approximately in a plane with the given normal.
///
/// # Errors
///
/// Returns an error if the interpolation system is singular.
///
/// # Examples
///
/// ```
/// use monstertruck_geometry::prelude::*;
/// use monstertruck_geometry::nurbs::offset;
///
/// // A straight line along x at z=0.
/// let line = BsplineCurve::new(
///     KnotVector::bezier_knot(1),
///     vec![Point3::new(0.0, 0.0, 0.0), Point3::new(4.0, 0.0, 0.0)],
/// );
/// // Offset upward (in z) by 1.0 using z-axis as the reference normal.
/// let offset = offset::curve_offset_3d(&line, 1.0, Vector3::unit_z(), 20).unwrap();
/// let mid = offset.subs(0.5);
/// // The tangent is along x, so the perpendicular component of z is z itself.
/// assert!((mid.z - 1.0).abs() < 0.05, "mid.z = {}", mid.z);
/// ```
pub fn curve_offset_3d(
    curve: &BsplineCurve<Point3>,
    distance: f64,
    normal: Vector3,
    n_samples: usize,
) -> Result<BsplineCurve<Point3>> {
    assert!(
        n_samples >= 4,
        "curve_offset_3d requires at least 4 samples",
    );

    let (t_start, t_end) = curve.range_tuple();

    let param_points: Vec<(f64, Point3)> = (0..n_samples)
        .map(|i| {
            let u = i as f64 / (n_samples - 1) as f64;
            let t = t_start + (t_end - t_start) * u;
            let pt = curve.subs(t);
            let tangent = curve.der(t);
            let tangent_len = tangent.magnitude();

            // Compute offset direction: component of `normal` perpendicular to tangent.
            let offset_dir = if tangent_len.so_small() {
                normal.normalize()
            } else {
                let t_hat = tangent / tangent_len;
                let perp = normal - t_hat * normal.dot(t_hat);
                let perp_len = perp.magnitude();
                if perp_len.so_small() {
                    // Normal is parallel to tangent — use cross product fallback.
                    let alt = if t_hat.x.abs() < 0.9 {
                        Vector3::unit_x()
                    } else {
                        Vector3::unit_y()
                    };
                    t_hat.cross(alt).normalize()
                } else {
                    perp / perp_len
                }
            };

            (u, pt + offset_dir * distance)
        })
        .collect();

    let degree = 3.min(n_samples - 1);
    let knot = KnotVector::uniform_knot(degree, n_samples - degree);
    BsplineCurve::try_interpolate(knot, param_points)
}

/// Offsets a [`BsplineSurface`] by `distance` along its surface normals.
///
/// At each sample point on a grid of `(n_u, n_v)` samples, the surface normal
/// is computed via the cross product of the partial derivatives, and the point
/// is displaced by `distance * normal`. A new surface is then fitted through
/// the offset grid.
///
/// **Caveat**: The offset of a NURBS surface is not generally a NURBS surface.
/// This approximation works well for smooth surfaces with moderate curvature
/// relative to the offset distance. Self-intersections in the offset are not
/// detected or resolved.
///
/// # Errors
///
/// Returns an error if the interpolation fails for any row of the surface.
///
/// # Examples
///
/// ```
/// use monstertruck_geometry::prelude::*;
/// use monstertruck_geometry::nurbs::offset;
///
/// // Flat surface at z=0 spanning [0,2] x [0,3].
/// let surface = BsplineSurface::new(
///     (KnotVector::bezier_knot(1), KnotVector::bezier_knot(1)),
///     vec![
///         vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 3.0, 0.0)],
///         vec![Point3::new(2.0, 0.0, 0.0), Point3::new(2.0, 3.0, 0.0)],
///     ],
/// );
/// let offset_surf = offset::surface_offset(&surface, 1.0, (10, 10)).unwrap();
/// let mid = offset_surf.subs(0.5, 0.5);
/// // Normal of a flat XY surface is +z, so offset should be at z=1.
/// assert!((mid.z - 1.0).abs() < 0.1, "mid.z = {}", mid.z);
/// ```
pub fn surface_offset(
    surface: &BsplineSurface<Point3>,
    distance: f64,
    (n_u, n_v): (usize, usize),
) -> Result<BsplineSurface<Point3>> {
    assert!(n_u >= 4, "surface_offset requires at least 4 u-samples");
    assert!(n_v >= 4, "surface_offset requires at least 4 v-samples");

    let ((u_start, u_end), (v_start, v_end)) = surface.range_tuple();

    // Build offset grid row by row (u-direction).
    // Each row is a B-spline in v fitted through offset points.
    let degree_v = 3.min(n_v - 1);
    let knot_v = KnotVector::uniform_knot(degree_v, n_v - degree_v);

    let rows: Vec<BsplineCurve<Point3>> = (0..n_u)
        .map(|i| {
            let u_frac = i as f64 / (n_u - 1) as f64;
            let u = u_start + (u_end - u_start) * u_frac;

            let param_points: Vec<(f64, Point3)> = (0..n_v)
                .map(|j| {
                    let v_frac = j as f64 / (n_v - 1) as f64;
                    let v = v_start + (v_end - v_start) * v_frac;
                    let pt = surface.subs(u, v);
                    let du = surface.uder(u, v);
                    let dv = surface.vder(u, v);
                    let cross = du.cross(dv);
                    let cross_len = cross.magnitude();

                    let normal = if cross_len.so_small() {
                        // Degenerate point — use z fallback.
                        Vector3::unit_z()
                    } else {
                        cross / cross_len
                    };

                    (v_frac, pt + normal * distance)
                })
                .collect();

            BsplineCurve::try_interpolate(knot_v.clone(), param_points)
        })
        .collect::<Result<Vec<_>>>()?;

    // Now skin the rows into a surface.
    Ok(BsplineSurface::skin(rows))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_2d_straight_line() {
        let line = BsplineCurve::new(
            KnotVector::bezier_knot(1),
            vec![Point2::new(0.0, 0.0), Point2::new(4.0, 0.0)],
        );
        let offset = curve_offset_2d(&line, 1.0, 20).unwrap();
        // Check several points along the offset.
        for i in 0..=10 {
            let u = i as f64 / 10.0;
            let pt = offset.subs(u);
            assert!(
                (pt.y - 1.0).abs() < 0.05,
                "at u={u}: expected y~1.0, got y={}",
                pt.y,
            );
            let expected_x = 4.0 * u;
            assert!(
                (pt.x - expected_x).abs() < 0.1,
                "at u={u}: expected x~{expected_x}, got x={}",
                pt.x,
            );
        }
    }

    #[test]
    fn offset_3d_straight_line_z_normal() {
        let line = BsplineCurve::new(
            KnotVector::bezier_knot(1),
            vec![Point3::new(0.0, 0.0, 0.0), Point3::new(4.0, 0.0, 0.0)],
        );
        let offset = curve_offset_3d(&line, 2.0, Vector3::unit_z(), 20).unwrap();
        for i in 0..=10 {
            let u = i as f64 / 10.0;
            let pt = offset.subs(u);
            assert!(
                (pt.z - 2.0).abs() < 0.05,
                "at u={u}: expected z~2.0, got z={}",
                pt.z,
            );
        }
    }

    #[test]
    fn surface_offset_flat_plane() {
        let surface = BsplineSurface::new(
            (KnotVector::bezier_knot(1), KnotVector::bezier_knot(1)),
            vec![
                vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 3.0, 0.0)],
                vec![Point3::new(2.0, 0.0, 0.0), Point3::new(2.0, 3.0, 0.0)],
            ],
        );
        let offset_surf = surface_offset(&surface, 1.5, (10, 10)).unwrap();
        // Check grid of points.
        for i in 0..=5 {
            for j in 0..=5 {
                let u = i as f64 / 5.0;
                let v = j as f64 / 5.0;
                let pt = offset_surf.subs(u, v);
                assert!(
                    (pt.z - 1.5).abs() < 0.15,
                    "at ({u},{v}): expected z~1.5, got z={}",
                    pt.z,
                );
            }
        }
    }
}
