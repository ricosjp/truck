//! Test boolean operations with STEP geometry types
//! This test reproduces and fixes issue #91
//!
//! Issue #91: Boolean operations with STEP-loaded solids failed because Curve3D
//! didn't implement the required From<IntersectionCurve<...>> trait.

use truck_stepio::r#in::step_geometry::{Curve3D, Surface};

/// This test verifies that the trait bound `Curve3D: ShapeOpsCurve<Surface>` is satisfied.
/// Before the fix, this would fail to compile with:
/// ```text
/// error[E0277]: the trait bound `Curve3D: ShapeOpsCurve<Surface>` is not satisfied
/// = help: the trait `From<IntersectionCurve<BSplineCurve<Point3>, Surface, Surface>>`
///         is not implemented for `Curve3D`
/// ```
#[test]
fn test_curve3d_implements_shapeops_trait() {
    // This test ensures that the trait bounds are satisfied.
    // We don't actually need to run a full boolean operation - just checking
    // that the code compiles proves that Curve3D implements ShapeOpsCurve<Surface>.

    // The key trait requirement is:
    // From<IntersectionCurve<BSplineCurve<Point3>, Surface, Surface>> for Curve3D

    // We can verify this by creating a simple type check:
    fn _type_check_shapeops_curve<
        C: truck_shapeops::ShapeOpsCurve<Surface>,
        S: truck_shapeops::ShapeOpsSurface,
    >() {
    }

    // If Curve3D didn't implement the trait, this would fail to compile:
    _type_check_shapeops_curve::<Curve3D, Surface>();
}

/// Verify the From implementation works correctly
#[test]
fn test_intersection_curve_to_curve3d_conversion() {
    use truck_modeling::{BSplineCurve, IntersectionCurve, Plane, Point3};
    use truck_stepio::r#in::step_geometry::ElementarySurface;

    // Create a simple B-spline curve
    let knot_vec = truck_modeling::KnotVec::from(vec![0.0, 0.0, 1.0, 1.0]);
    let control_points = vec![Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 0.0)];
    let bspline = BSplineCurve::new(knot_vec, control_points);

    // Create two planes
    let plane1 = Plane::new(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    );
    let plane2 = Plane::new(
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
    );

    // Create surfaces from planes
    let surf1 = Surface::ElementarySurface(ElementarySurface::Plane(plane1));
    let surf2 = Surface::ElementarySurface(ElementarySurface::Plane(plane2));

    // Create an intersection curve
    let intersection = IntersectionCurve::new(surf1, surf2, bspline);

    // Convert to Curve3D - this is what the From implementation does
    let curve3d: Curve3D = intersection.into();

    // Verify it's a BSplineCurve variant
    match curve3d {
        Curve3D::BSplineCurve(_) => {
            // Success! The conversion worked correctly
        }
        _ => panic!("Expected BSplineCurve variant"),
    }
}
