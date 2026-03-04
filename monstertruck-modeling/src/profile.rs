//! Planar profile normalization: loop orientation detection, outer/hole
//! classification, and winding normalization for complex profiles with holes.

use crate::{Result, errors::Error};
use itertools::Itertools;
use monstertruck_core::cgmath64::*;
use monstertruck_core::hash::HashGen;
use monstertruck_geometry::prelude::*;

type Wire<C> = monstertruck_topology::Wire<Point3, C>;
type Face<C, S> = monstertruck_topology::Face<Point3, C, S>;

/// Projects a 3D wire onto 2D by sampling vertex and curve midpoints.
/// Uses the same sampling strategy as [`builder::try_attach_plane`]:
/// vertex point + curve midpoint per edge.
fn project_wire_to_2d<C>(wire: &Wire<C>, u_axis: Vector3, v_axis: Vector3) -> Vec<Point2>
where C: ParametricCurve3D + BoundedCurve + Clone {
    wire.edge_iter()
        .flat_map(|edge| {
            let p0 = edge.front().point();
            let curve = edge.curve();
            let (t0, t1) = curve.range_tuple();
            let p1 = curve.subs((t0 + t1) / 2.0);
            [p0, p1]
        })
        .map(|p| Point2::new(p.to_vec().dot(u_axis), p.to_vec().dot(v_axis)))
        .collect()
}

/// Signed area of a 2D polyline treated as a closed polygon (shoelace formula).
/// Positive means counter-clockwise, negative means clockwise.
fn signed_area_2d(pts: &[Point2]) -> f64 {
    pts.iter()
        .circular_tuple_windows()
        .map(|(p, q)| (q.x + p.x) * (q.y - p.y))
        .sum::<f64>()
        / 2.0
}

/// Centroid of 2D sample points.
fn centroid_2d(pts: &[Point2]) -> Point2 {
    let n = pts.len() as f64;
    let sum = pts.iter().fold(Vector2::zero(), |acc, p| acc + p.to_vec());
    Point2::from_vec(sum / n)
}

/// Point-in-polygon test via ray casting (winding number).
fn point_in_polygon(pts: &[Point2], c: Point2) -> bool {
    let t = 2.0 * std::f64::consts::PI * HashGen::hash1(c);
    let r = Vector2::new(f64::cos(t), f64::sin(t));
    pts.iter()
        .circular_tuple_windows()
        .try_fold(0_i32, |counter, (p0, p1)| {
            if (*p0 - c).so_small() {
                return None;
            }
            let a = p0 - c;
            let b = p1 - c;
            let s0 = r.x * a.y - r.y * a.x;
            let s1 = r.x * b.y - r.y * b.x;
            let s2 = a.x * b.y - a.y * b.x;
            let x = s2 / (s1 - s0);
            if x.so_small() && s0 * s1 < 0.0 {
                None
            } else if x > 0.0 && s0 <= 0.0 && s1 > 0.0 {
                Some(counter + 1)
            } else if x > 0.0 && s0 >= 0.0 && s1 < 0.0 {
                Some(counter - 1)
            } else {
                Some(counter)
            }
        })
        .map(|counter| counter > 0)
        .unwrap_or(true)
}

/// Computes the average normal of a set of wires by accumulating cross products
/// of consecutive sample points around a common centroid.
fn compute_plane_normal<C>(wires: &[Wire<C>]) -> Option<Vector3>
where C: ParametricCurve3D + BoundedCurve + Clone {
    let all_pts: Vec<Point3> = wires
        .iter()
        .flat_map(|w| {
            w.edge_iter().flat_map(|edge| {
                let p0 = edge.front().point();
                let curve = edge.curve();
                let (t0, t1) = curve.range_tuple();
                let p1 = curve.subs((t0 + t1) / 2.0);
                [p0, p1]
            })
        })
        .collect();

    if all_pts.is_empty() {
        return None;
    }

    let center =
        all_pts.iter().fold(Point3::origin(), |s, p| s + p.to_vec()) / all_pts.len() as f64;

    let normal = all_pts
        .iter()
        .circular_tuple_windows()
        .fold(Vector3::zero(), |sum, (p0, p1)| {
            sum + (p0 - center).cross(p1 - center)
        });

    if normal.so_small() {
        None
    } else {
        Some(normal.normalize())
    }
}

/// Derives orthonormal `u` and `v` axes from a plane normal.
fn axes_from_normal(n: Vector3) -> (Vector3, Vector3) {
    let a = n.map(f64::abs);
    let u = if a.x > a.z || a.y > a.z {
        Vector3::new(-n.y, n.x, 0.0).normalize()
    } else {
        Vector3::new(-n.z, 0.0, n.x).normalize()
    };
    (u, n.cross(u))
}

/// Classifies and normalizes a set of wires for planar face construction.
///
/// Returns `(outer_index, wires)` where `wires` have been reoriented so that:
/// - The outer loop (largest positive signed area) is CCW.
/// - All hole loops are CW.
///
/// The returned `Vec` has the outer wire first, followed by hole wires.
fn classify_and_normalize<C>(mut wires: Vec<Wire<C>>) -> Result<Vec<Wire<C>>>
where C: ParametricCurve3D + BoundedCurve + Clone + Invertible {
    if wires.is_empty() {
        return Err(Error::FromTopology(
            monstertruck_topology::errors::Error::EmptyWire,
        ));
    }

    // Validate all wires are closed.
    if wires.iter().any(|w| !w.is_closed()) {
        return Err(Error::OpenWire);
    }

    // Single wire: just ensure it is CCW and return.
    if wires.len() == 1 {
        let normal = compute_plane_normal(&wires).ok_or(Error::WireNotInOnePlane)?;
        let (u, v) = axes_from_normal(normal);
        let pts = project_wire_to_2d(&wires[0], u, v);
        if signed_area_2d(&pts) < 0.0 {
            wires[0] = wires[0].inverse();
        }
        return Ok(wires);
    }

    let normal = compute_plane_normal(&wires).ok_or(Error::WireNotInOnePlane)?;
    let (u, v) = axes_from_normal(normal);

    // Project all wires and compute signed areas.
    let projections: Vec<Vec<Point2>> = wires.iter().map(|w| project_wire_to_2d(w, u, v)).collect();

    let areas: Vec<f64> = projections.iter().map(|pts| signed_area_2d(pts)).collect();

    // Find the outer loop: the one with the largest absolute area.
    let (outer_idx, _) = areas
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.abs().partial_cmp(&b.abs()).unwrap())
        .ok_or(Error::NoOuterLoop)?;

    // Ensure the outer wire is CCW (positive area).
    if areas[outer_idx] < 0.0 {
        wires[outer_idx] = wires[outer_idx].inverse();
    }

    // Recompute projection of outer after potential inversion.
    let outer_pts = project_wire_to_2d(&wires[outer_idx], u, v);

    // Validate holes: each hole's centroid must be inside the outer loop.
    for (i, proj) in projections.iter().enumerate() {
        if i == outer_idx {
            continue;
        }
        let c = centroid_2d(proj);
        if !point_in_polygon(&outer_pts, c) {
            return Err(Error::AmbiguousNesting);
        }
        // Ensure holes are CW (negative area).
        if areas[i] > 0.0 {
            wires[i] = wires[i].inverse();
        }
    }

    // Reorder: outer first, then holes.
    let mut result = Vec::with_capacity(wires.len());
    result.push(wires.swap_remove(outer_idx));
    result.extend(wires);
    Ok(result)
}

/// Attaches a plane to a set of wires with automatic loop orientation
/// normalization.
///
/// Unlike [`builder::try_attach_plane`], this function does not require the
/// caller to ensure correct winding order. It:
///
/// 1. Detects the common plane of all wires.
/// 2. Classifies loops as outer or holes via signed area and containment.
/// 3. Normalizes winding directions (outer = CCW, holes = CW).
/// 4. Delegates to [`builder::try_attach_plane`] with the normalized wires.
///
/// # Errors
///
/// - [`Error::WireNotInOnePlane`] if the wires are not coplanar.
/// - [`Error::OpenWire`] if any wire is not closed.
/// - [`Error::AmbiguousNesting`] if hole containment is ambiguous.
/// - [`Error::NoOuterLoop`] if no outer loop can be identified.
pub fn attach_plane_normalized<C, S>(wires: Vec<Wire<C>>) -> Result<Face<C, S>>
where
    C: ParametricCurve3D + BoundedCurve + Clone + Invertible,
    Plane: IncludeCurve<C> + ToSameGeometry<S>, {
    let normalized = classify_and_normalize(wires)?;
    crate::builder::try_attach_plane(normalized)
}

/// Constructs a [`Solid`] by extruding a planar profile along a direction
/// vector.
///
/// Takes a set of wires (possibly with holes), normalizes orientation,
/// attaches a plane, and extrudes into a solid.
///
/// # Errors
///
/// Returns errors from [`attach_plane_normalized`] if the profile is invalid.
pub fn solid_from_planar_profile<C, S>(
    wires: Vec<Wire<C>>,
    dir: Vector3,
) -> Result<monstertruck_topology::Solid<Point3, C, S>>
where
    C: ParametricCurve3D + BoundedCurve + Clone + Invertible + Cut + Transformed<Matrix4>,
    S: Invertible + Transformed<Matrix4>,
    Plane: IncludeCurve<C> + ToSameGeometry<S>,
    Line<Point3>: ToSameGeometry<C>,
    ExtrudedCurve<C, Vector3>: ToSameGeometry<S>,
{
    let face = attach_plane_normalized(wires)?;
    let solid: monstertruck_topology::Solid<Point3, C, S> = crate::builder::extrude(&face, dir);
    Ok(solid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Curve, Surface, builder};

    type Wire = monstertruck_topology::Wire<Point3, Curve>;
    type Face = monstertruck_topology::Face<Point3, Curve, Surface>;

    /// Helper: builds a rectangular wire in the XY plane.
    fn rect_wire(x0: f64, y0: f64, x1: f64, y1: f64) -> Wire {
        let v0 = builder::vertex(Point3::new(x0, y0, 0.0));
        let v1 = builder::vertex(Point3::new(x1, y0, 0.0));
        let v2 = builder::vertex(Point3::new(x1, y1, 0.0));
        let v3 = builder::vertex(Point3::new(x0, y1, 0.0));
        vec![
            builder::line(&v0, &v1),
            builder::line(&v1, &v2),
            builder::line(&v2, &v3),
            builder::line(&v3, &v0),
        ]
        .into()
    }

    /// Helper: builds a CW (clockwise) rectangular wire in the XY plane.
    fn rect_wire_cw(x0: f64, y0: f64, x1: f64, y1: f64) -> Wire {
        rect_wire(x0, y0, x1, y1).inverse()
    }

    #[test]
    fn single_wire_ccw() {
        let wire = rect_wire(-1.0, -1.0, 1.0, 1.0);
        let face: Face = attach_plane_normalized(vec![wire]).unwrap();
        assert_eq!(face.boundaries().len(), 1);
    }

    #[test]
    fn single_wire_cw_gets_normalized() {
        // CW wire should be automatically flipped to CCW.
        let wire = rect_wire_cw(-1.0, -1.0, 1.0, 1.0);
        let face: Face = attach_plane_normalized(vec![wire]).unwrap();
        assert_eq!(face.boundaries().len(), 1);
    }

    #[test]
    fn outer_with_hole() {
        let outer = rect_wire(-2.0, -2.0, 2.0, 2.0);
        let hole = rect_wire(-0.5, -0.5, 0.5, 0.5);
        let face: Face = attach_plane_normalized(vec![outer, hole]).unwrap();
        assert_eq!(face.boundaries().len(), 2);
    }

    #[test]
    fn outer_with_hole_both_ccw_gets_normalized() {
        // Both wires are CCW; the hole must be auto-flipped to CW.
        let outer = rect_wire(-2.0, -2.0, 2.0, 2.0);
        let hole = rect_wire(-0.5, -0.5, 0.5, 0.5);
        let face: Face = attach_plane_normalized(vec![outer, hole]).unwrap();
        assert_eq!(face.boundaries().len(), 2);
    }

    #[test]
    fn outer_with_hole_reversed_order() {
        // Hole given first, outer second - should still work.
        let outer = rect_wire(-2.0, -2.0, 2.0, 2.0);
        let hole = rect_wire_cw(-0.5, -0.5, 0.5, 0.5);
        let face: Face = attach_plane_normalized(vec![hole, outer]).unwrap();
        assert_eq!(face.boundaries().len(), 2);
    }

    #[test]
    fn multiple_holes() {
        let outer = rect_wire(-5.0, -5.0, 5.0, 5.0);
        let hole1 = rect_wire(-4.0, -4.0, -2.0, -2.0);
        let hole2 = rect_wire(1.0, 1.0, 3.0, 3.0);
        let face: Face = attach_plane_normalized(vec![outer, hole1, hole2]).unwrap();
        assert_eq!(face.boundaries().len(), 3);
    }

    #[test]
    fn mixed_winding_multiple_holes() {
        // All wires given as CCW; normalization should flip the holes.
        let outer = rect_wire(-5.0, -5.0, 5.0, 5.0);
        let hole1 = rect_wire(-4.0, -4.0, -2.0, -2.0);
        let hole2 = rect_wire(1.0, 1.0, 3.0, 3.0);
        let face: Face = attach_plane_normalized(vec![hole1, outer, hole2]).unwrap();
        assert_eq!(face.boundaries().len(), 3);
    }

    #[test]
    fn open_wire_rejected() {
        let v0 = builder::vertex(Point3::new(0.0, 0.0, 0.0));
        let v1 = builder::vertex(Point3::new(1.0, 0.0, 0.0));
        let v2 = builder::vertex(Point3::new(1.0, 1.0, 0.0));
        let wire: Wire = vec![builder::line(&v0, &v1), builder::line(&v1, &v2)].into();
        let result = attach_plane_normalized::<Curve, Surface>(vec![wire]);
        assert!(matches!(result, Err(Error::OpenWire)));
    }

    #[test]
    fn solid_from_profile_simple() {
        let outer = rect_wire(-1.0, -1.0, 1.0, 1.0);
        let solid =
            solid_from_planar_profile::<Curve, Surface>(vec![outer], Vector3::new(0.0, 0.0, 1.0))
                .unwrap();
        // A box: 6 faces.
        assert_eq!(solid.boundaries()[0].len(), 6);
    }

    #[test]
    fn solid_from_profile_with_hole() {
        let outer = rect_wire(-2.0, -2.0, 2.0, 2.0);
        let hole = rect_wire(-0.5, -0.5, 0.5, 0.5);
        let solid = solid_from_planar_profile::<Curve, Surface>(
            vec![outer, hole],
            Vector3::new(0.0, 0.0, 1.0),
        )
        .unwrap();
        let shell = &solid.boundaries()[0];
        // Bottom face + top face + 4 outer sides + 4 inner sides = 10 faces.
        assert_eq!(shell.len(), 10);
    }

    #[test]
    fn near_degenerate_tiny_hole() {
        let outer = rect_wire(-10.0, -10.0, 10.0, 10.0);
        // Very tiny hole.
        let hole = rect_wire(-0.001, -0.001, 0.001, 0.001);
        let face: Face = attach_plane_normalized(vec![outer, hole]).unwrap();
        assert_eq!(face.boundaries().len(), 2);
    }
}
