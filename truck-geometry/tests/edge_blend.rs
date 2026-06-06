use prop::array::*;
use proptest::{prelude::*, property_test};
use truck_geometry::prelude::*;

#[test]
fn test_edge_blend_ders_by_line_and_plane() {
    let knot_vec = KnotVec::bezier_knot(3);
    let plane = Plane::xy();
    let pcurve0 = Line(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
    let pcurve1 = Line(Point2::new(0.0, 1.0), Point2::new(1.0, 1.0));
    let edge_blend = EdgeBlendSurface::new(
        PCurve::new(pcurve0, plane.clone()),
        0.6,
        PCurve::new(pcurve1, plane),
        0.3,
    );

    let expected = BSplineSurface::new(
        (knot_vec.clone(), knot_vec),
        (0..4)
            .map(|i| {
                let x = i as f64 / 3.0;
                vec![
                    Point3::new(x, 0.0, 0.0),
                    Point3::new(x, -0.2, 0.0),
                    Point3::new(x, 1.1, 0.0),
                    Point3::new(x, 1.0, 0.0),
                ]
            })
            .collect::<Vec<_>>(),
    );

    for u in [0.0, 0.23, 0.71, 1.0] {
        for v in [0.0, 0.31, 0.79, 1.0] {
            assert_near!(edge_blend.subs(u, v), expected.subs(u, v));
            assert_near!(edge_blend.ders(2, u, v), expected.ders(2, u, v));
            assert_near!(edge_blend.uder(u, v), expected.uder(u, v));
            assert_near!(edge_blend.vder(u, v), expected.vder(u, v));
            assert_near!(edge_blend.uuder(u, v), expected.uuder(u, v));
            assert_near!(edge_blend.uvder(u, v), expected.uvder(u, v));
            assert_near!(edge_blend.vvder(u, v), expected.vvder(u, v));
        }
    }
}

#[property_test]
fn test_edge_blend_ends_by_bezier_surface(
    #[strategy = uniform4(uniform4(uniform3(-10.0..=10.0)))] control_points0: [[[f64; 3]; 4]; 4],
    #[strategy = uniform4(uniform4(uniform3(-10.0..=10.0)))] control_points1: [[[f64; 3]; 4]; 4],
    #[strategy = 0.0..=1.0] t: f64,
) {
    let control_points0 = control_points0
        .into_iter()
        .map(|p| p.into_iter().map(Point3::from).collect())
        .collect();
    let surface0 = BSplineSurface::new(
        (KnotVec::bezier_knot(3), KnotVec::bezier_knot(3)),
        control_points0,
    );

    let control_points1 = control_points1
        .into_iter()
        .map(|p| p.into_iter().map(Point3::from).collect())
        .collect();
    let surface1 = BSplineSurface::new(
        (KnotVec::bezier_knot(3), KnotVec::bezier_knot(3)),
        control_points1,
    );

    let line0 = Line(Point2::new(0.0, 1.0), Point2::new(1.0, 1.0));
    let line1 = Line(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));

    let normal0 = surface0.uder(t, 1.0).cross(surface0.vder(t, 1.0));
    let normal1 = surface1.uder(t, 0.0).cross(surface1.vder(t, 0.0));
    let axis0 = surface0.uder(t, 1.0).cross(normal0);
    let axis1 = surface1.uder(t, 0.0).cross(normal1);

    prop_assume!(!normal0.so_small());
    prop_assume!(!normal1.so_small());
    prop_assume!(!axis0.so_small());
    prop_assume!(!axis1.so_small());

    let pcurve0 = PCurve::new(line0, surface0);
    let pcurve1 = PCurve::new(line1, surface1);
    let surface = EdgeBlendSurface::new(pcurve0.clone(), 0.6, pcurve1.clone(), 0.4);

    prop_assert_near!(surface.subs(t, 0.0), pcurve0.subs(t));
    prop_assert_near!(surface.uder(t, 0.0), pcurve0.der(t));
    prop_assert_near!(surface.uuder(t, 0.0), pcurve0.der2(t));
    prop_assert_near!(surface.subs(t, 1.0), pcurve1.subs(t));
    prop_assert_near!(surface.uder(t, 1.0), pcurve1.der(t));
    prop_assert_near!(surface.uuder(t, 1.0), pcurve1.der2(t));
    prop_assert_near!(surface.normal(t, 0.0), -normal0.normalize());
    prop_assert_near!(surface.normal(t, 1.0), -normal1.normalize());
}
