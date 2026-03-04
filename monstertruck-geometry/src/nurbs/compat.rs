//! Compatibility normalization for collections of B-spline curves and surfaces.
//!
//! Before constructing network-based surfaces (skinning, Gordon, birail),
//! all input curves must share the same degree and knot vector. This module
//! provides multi-input wrappers around the pairwise [`BsplineCurve::syncro_degree`]
//! and [`BsplineCurve::syncro_knots`] primitives.

use super::*;
use crate::errors::{Error, Result};

/// Makes all curves in the slice share the same degree and normalized knot vector.
///
/// After this call every curve in `curves` has:
/// - the same polynomial degree (the maximum among all inputs),
/// - an identical knot vector normalized to \[0, 1\] (the merged union of all input knots).
///
/// The geometric shape of each curve is preserved.
///
/// # Errors
///
/// Returns [`Error::EmptyControlPoints`] if `curves` is empty.
///
/// # Examples
///
/// ```
/// use monstertruck_geometry::prelude::*;
/// use monstertruck_geometry::nurbs::compat;
///
/// let mut c0 = BsplineCurve::new(
///     KnotVector::bezier_knot(1),
///     vec![Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0)],
/// );
/// let mut c1 = BsplineCurve::new(
///     KnotVector::bezier_knot(2),
///     vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, 1.0), Vector2::new(1.0, 0.0)],
/// );
/// assert_ne!(c0.degree(), c1.degree());
///
/// let mut curves = vec![c0, c1];
/// compat::make_curves_compatible(&mut curves).unwrap();
///
/// assert_eq!(curves[0].degree(), curves[1].degree());
/// assert_eq!(curves[0].knot_vec(), curves[1].knot_vec());
/// ```
pub fn make_curves_compatible<P>(curves: &mut [BsplineCurve<P>]) -> Result<()>
where P: ControlPoint<f64> + Tolerance {
    if curves.is_empty() {
        return Err(Error::EmptyControlPoints);
    }
    if curves.len() == 1 {
        curves[0].knot_normalize();
        return Ok(());
    }

    // Phase 1: elevate all curves to the maximum degree.
    let max_degree = curves
        .iter()
        .map(|c| c.degree())
        .max()
        // SAFETY: `curves` is non-empty, checked above.
        .unwrap();
    curves.iter_mut().for_each(|c| {
        for _ in c.degree()..max_degree {
            c.elevate_degree();
        }
    });

    // Phase 2: normalize all knot vectors to [0, 1].
    curves.iter_mut().for_each(|c| {
        c.knot_normalize();
    });

    // Phase 3: merge knot vectors by synchronizing each curve against the first.
    // After each syncro_knots call the first curve accumulates all knots seen so far,
    // so subsequent curves synchronize against the running union.
    for i in 1..curves.len() {
        // Split the slice so we can mutably borrow two elements simultaneously.
        let (left, right) = curves.split_at_mut(i);
        left[0].syncro_knots(&mut right[0]);
    }

    // Phase 4: the first curve now holds the full merged knot vector.
    // Synchronize all other curves against it to pick up any knots they missed.
    for i in 1..curves.len() {
        let (left, right) = curves.split_at_mut(i);
        left[0].syncro_knots(&mut right[0]);
    }

    Ok(())
}

/// Makes all surfaces in the slice share the same degrees and normalized knot vectors
/// in both the u and v directions.
///
/// After this call every surface in `surfaces` has:
/// - the same u-degree and v-degree (the maximum among all inputs in each direction),
/// - identical u-knot and v-knot vectors normalized to \[0, 1\].
///
/// The geometric shape of each surface is preserved.
///
/// # Errors
///
/// Returns [`Error::EmptyControlPoints`] if `surfaces` is empty.
///
/// # Examples
///
/// ```
/// use monstertruck_geometry::prelude::*;
/// use monstertruck_geometry::nurbs::compat;
///
/// let s0 = BsplineSurface::new(
///     (KnotVector::bezier_knot(1), KnotVector::bezier_knot(1)),
///     vec![
///         vec![Vector2::new(0.0, 0.0), Vector2::new(1.0, 0.0)],
///         vec![Vector2::new(0.0, 1.0), Vector2::new(1.0, 1.0)],
///     ],
/// );
/// let s1 = BsplineSurface::new(
///     (KnotVector::bezier_knot(2), KnotVector::bezier_knot(2)),
///     vec![
///         vec![Vector2::new(0.0, 0.0), Vector2::new(0.5, 0.0), Vector2::new(1.0, 0.0)],
///         vec![Vector2::new(0.0, 0.5), Vector2::new(0.5, 0.5), Vector2::new(1.0, 0.5)],
///         vec![Vector2::new(0.0, 1.0), Vector2::new(0.5, 1.0), Vector2::new(1.0, 1.0)],
///     ],
/// );
/// assert_ne!(s0.udegree(), s1.udegree());
///
/// let mut surfaces = vec![s0, s1];
/// compat::make_surfaces_compatible(&mut surfaces).unwrap();
///
/// assert_eq!(surfaces[0].udegree(), surfaces[1].udegree());
/// assert_eq!(surfaces[0].vdegree(), surfaces[1].vdegree());
/// assert_eq!(surfaces[0].knot_vecs(), surfaces[1].knot_vecs());
/// ```
pub fn make_surfaces_compatible<P>(surfaces: &mut [BsplineSurface<P>]) -> Result<()>
where P: ControlPoint<f64> + Tolerance {
    if surfaces.is_empty() {
        return Err(Error::EmptyControlPoints);
    }
    if surfaces.len() == 1 {
        surfaces[0].knot_normalize();
        return Ok(());
    }

    // Phase 1: elevate all surfaces to max u-degree and max v-degree.
    let max_udeg = surfaces
        .iter()
        .map(|s| s.udegree())
        .max()
        // SAFETY: `surfaces` is non-empty, checked above.
        .unwrap();
    let max_vdeg = surfaces
        .iter()
        .map(|s| s.vdegree())
        .max()
        // SAFETY: `surfaces` is non-empty, checked above.
        .unwrap();
    surfaces.iter_mut().for_each(|s| {
        for _ in s.udegree()..max_udeg {
            s.elevate_udegree();
        }
        for _ in s.vdegree()..max_vdeg {
            s.elevate_vdegree();
        }
    });

    // Phase 2: normalize all knot vectors to [0, 1].
    surfaces.iter_mut().for_each(|s| {
        s.knot_normalize();
    });

    // Phase 3: synchronize u-knots and v-knots across all surfaces.
    // Extract columns of curves from the control point grids to synchronize
    // knot vectors direction by direction.
    //
    // For u-knots: synchronize by extracting row curves (u-direction iso-curves).
    // For v-knots: synchronize by extracting column curves (v-direction iso-curves).
    //
    // Since `BsplineSurface` does not expose pairwise surface-to-surface knot sync,
    // we synchronize by inserting knots from one surface into the other.
    sync_surface_knots_u(surfaces);
    sync_surface_knots_u(surfaces);
    sync_surface_knots_v(surfaces);
    sync_surface_knots_v(surfaces);

    Ok(())
}

/// Synchronizes u-direction knot vectors across all surfaces against the first.
fn sync_surface_knots_u<P>(surfaces: &mut [BsplineSurface<P>])
where P: ControlPoint<f64> + Tolerance {
    for i in 1..surfaces.len() {
        // Collect knots from surface[i] that surface[0] is missing.
        let knots_to_add: Vec<f64> =
            collect_missing_knots(surfaces[0].knot_vector_u(), surfaces[i].knot_vector_u());
        for &k in &knots_to_add {
            surfaces[0].add_knot_u(k);
        }

        // Collect knots from surface[0] that surface[i] is missing.
        let knots_to_add: Vec<f64> =
            collect_missing_knots(surfaces[i].knot_vector_u(), surfaces[0].knot_vector_u());
        for &k in &knots_to_add {
            surfaces[i].add_knot_u(k);
        }
    }
}

/// Synchronizes v-direction knot vectors across all surfaces against the first.
fn sync_surface_knots_v<P>(surfaces: &mut [BsplineSurface<P>])
where P: ControlPoint<f64> + Tolerance {
    for i in 1..surfaces.len() {
        let knots_to_add: Vec<f64> =
            collect_missing_knots(surfaces[0].knot_vector_v(), surfaces[i].knot_vector_v());
        for &k in &knots_to_add {
            surfaces[0].add_knot_v(k);
        }

        let knots_to_add: Vec<f64> =
            collect_missing_knots(surfaces[i].knot_vector_v(), surfaces[0].knot_vector_v());
        for &k in &knots_to_add {
            surfaces[i].add_knot_v(k);
        }
    }
}

/// Returns knots present in `source` but missing in `target`.
/// Both knot vectors must be normalized to \[0, 1\].
fn collect_missing_knots(target: &KnotVector, source: &KnotVector) -> Vec<f64> {
    let mut result = Vec::new();
    let mut ti = 0;
    let mut si = 0;
    let t_slice = target.as_slice();
    let s_slice = source.as_slice();

    while si < s_slice.len() && ti < t_slice.len() {
        if s_slice[si] - t_slice[ti] > TOLERANCE {
            // Source has a knot that target doesn't — need to insert it.
            result.push(s_slice[si]);
            si += 1;
        } else if t_slice[ti] - s_slice[si] > TOLERANCE {
            // Target has a knot that source doesn't — skip.
            ti += 1;
        } else {
            // Matching knots.
            ti += 1;
            si += 1;
        }
    }
    // Any remaining source knots are missing from target.
    while si < s_slice.len() {
        result.push(s_slice[si]);
        si += 1;
    }
    result
}

/// Makes all [`NurbsCurve`]s in the slice share the same degree and normalized knot vector.
///
/// This is the rational (weighted) counterpart of [`make_curves_compatible`].
/// Operates on the underlying [`BsplineCurve`] in homogeneous coordinates.
///
/// # Errors
///
/// Returns [`Error::EmptyControlPoints`] if `curves` is empty.
pub fn make_nurbs_curves_compatible<V>(curves: &mut [NurbsCurve<V>]) -> Result<()>
where V: Homogeneous<Scalar = f64> + ControlPoint<f64, Diff = V> + Tolerance {
    if curves.is_empty() {
        return Err(Error::EmptyControlPoints);
    }
    if curves.len() == 1 {
        curves[0].knot_normalize();
        return Ok(());
    }

    // Elevate all to max degree.
    let max_degree = curves
        .iter()
        .map(|c| c.degree())
        .max()
        // SAFETY: `curves` is non-empty, checked above.
        .unwrap();
    curves.iter_mut().for_each(|c| {
        for _ in c.degree()..max_degree {
            c.elevate_degree();
        }
    });

    // Normalize knot vectors.
    curves.iter_mut().for_each(|c| {
        c.knot_normalize();
    });

    // Synchronize knots pairwise against the first (two passes).
    for _ in 0..2 {
        for i in 1..curves.len() {
            let (left, right) = curves.split_at_mut(i);
            left[0].syncro_knots(&mut right[0]);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_curves_error() {
        let mut curves: Vec<BsplineCurve<Vector2>> = Vec::new();
        assert!(matches!(
            make_curves_compatible(&mut curves),
            Err(Error::EmptyControlPoints)
        ));
    }

    #[test]
    fn single_curve_normalizes_knots() {
        let mut curves = vec![BsplineCurve::new(
            KnotVector::from(vec![0.0, 0.0, 2.0, 2.0]),
            vec![Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0)],
        )];
        make_curves_compatible(&mut curves).unwrap();
        assert_eq!(curves[0].knot_vec().as_slice(), &[0.0, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn two_curves_same_degree_different_knots() {
        let c0 = BsplineCurve::new(
            KnotVector::from(vec![0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0]),
            vec![
                Vector2::new(0.0, 0.0),
                Vector2::new(1.0, 1.0),
                Vector2::new(2.0, 2.0),
                Vector2::new(3.0, 3.0),
            ],
        );
        let c1 = BsplineCurve::new(
            KnotVector::from(vec![0.0, 0.0, 0.0, 0.75, 1.0, 1.0, 1.0]),
            vec![
                Vector2::new(0.0, 0.0),
                Vector2::new(1.0, 0.0),
                Vector2::new(2.0, 1.0),
                Vector2::new(3.0, 1.0),
            ],
        );
        let org0 = c0.clone();
        let org1 = c1.clone();

        let mut curves = vec![c0, c1];
        make_curves_compatible(&mut curves).unwrap();

        assert_eq!(curves[0].knot_vec(), curves[1].knot_vec());
        assert_eq!(curves[0].degree(), curves[1].degree());
        // Shape is preserved.
        assert!(curves[0].near2_as_curve(&org0));
        assert!(curves[1].near2_as_curve(&org1));
    }

    #[test]
    fn two_curves_different_degrees() {
        let c0 = BsplineCurve::new(
            KnotVector::bezier_knot(1),
            vec![Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0)],
        );
        let c1 = BsplineCurve::new(
            KnotVector::bezier_knot(3),
            vec![
                Vector2::new(0.0, 0.0),
                Vector2::new(0.33, 1.0),
                Vector2::new(0.66, 1.0),
                Vector2::new(1.0, 0.0),
            ],
        );
        let org0 = c0.clone();
        let org1 = c1.clone();

        let mut curves = vec![c0, c1];
        make_curves_compatible(&mut curves).unwrap();

        assert_eq!(curves[0].degree(), curves[1].degree());
        assert_eq!(curves[0].degree(), 3);
        assert_eq!(curves[0].knot_vec(), curves[1].knot_vec());
        assert!(curves[0].near2_as_curve(&org0));
        assert!(curves[1].near2_as_curve(&org1));
    }

    #[test]
    fn three_curves_mixed() {
        let c0 = BsplineCurve::new(
            KnotVector::bezier_knot(1),
            vec![Vector2::new(0.0, 0.0), Vector2::new(1.0, 1.0)],
        );
        let c1 = BsplineCurve::new(
            KnotVector::bezier_knot(2),
            vec![
                Vector2::new(0.0, 0.0),
                Vector2::new(0.5, 1.0),
                Vector2::new(1.0, 0.0),
            ],
        );
        let c2 = BsplineCurve::new(
            KnotVector::from(vec![0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0]),
            vec![
                Vector2::new(0.0, 1.0),
                Vector2::new(0.5, 0.0),
                Vector2::new(0.5, 1.0),
                Vector2::new(1.0, 0.0),
            ],
        );
        let org0 = c0.clone();
        let org1 = c1.clone();
        let org2 = c2.clone();

        let mut curves = vec![c0, c1, c2];
        make_curves_compatible(&mut curves).unwrap();

        // All must share the same degree and knot vector.
        assert_eq!(curves[0].degree(), curves[1].degree());
        assert_eq!(curves[1].degree(), curves[2].degree());
        assert_eq!(curves[0].knot_vec(), curves[1].knot_vec());
        assert_eq!(curves[1].knot_vec(), curves[2].knot_vec());
        // Shapes preserved.
        assert!(curves[0].near2_as_curve(&org0));
        assert!(curves[1].near2_as_curve(&org1));
        assert!(curves[2].near2_as_curve(&org2));
    }

    #[test]
    fn empty_surfaces_error() {
        let mut surfaces: Vec<BsplineSurface<Vector2>> = Vec::new();
        assert!(matches!(
            make_surfaces_compatible(&mut surfaces),
            Err(Error::EmptyControlPoints)
        ));
    }

    #[test]
    fn two_surfaces_different_degrees() {
        let s0 = BsplineSurface::new(
            (KnotVector::bezier_knot(1), KnotVector::bezier_knot(1)),
            vec![
                vec![Vector2::new(0.0, 0.0), Vector2::new(1.0, 0.0)],
                vec![Vector2::new(0.0, 1.0), Vector2::new(1.0, 1.0)],
            ],
        );
        let s1 = BsplineSurface::new(
            (KnotVector::bezier_knot(2), KnotVector::bezier_knot(2)),
            vec![
                vec![
                    Vector2::new(0.0, 0.0),
                    Vector2::new(0.5, 0.0),
                    Vector2::new(1.0, 0.0),
                ],
                vec![
                    Vector2::new(0.0, 0.5),
                    Vector2::new(0.5, 0.5),
                    Vector2::new(1.0, 0.5),
                ],
                vec![
                    Vector2::new(0.0, 1.0),
                    Vector2::new(0.5, 1.0),
                    Vector2::new(1.0, 1.0),
                ],
            ],
        );
        let org0 = s0.clone();
        let org1 = s1.clone();

        let mut surfaces = vec![s0, s1];
        make_surfaces_compatible(&mut surfaces).unwrap();

        assert_eq!(surfaces[0].udegree(), surfaces[1].udegree());
        assert_eq!(surfaces[0].vdegree(), surfaces[1].vdegree());
        assert_eq!(surfaces[0].knot_vecs(), surfaces[1].knot_vecs());
        assert!(surfaces[0].near2_as_surface(&org0));
        assert!(surfaces[1].near2_as_surface(&org1));
    }
}
