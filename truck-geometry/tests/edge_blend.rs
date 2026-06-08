use prop::array::*;
use proptest::{prelude::*, property_test};
use truck_geometry::prelude::*;

#[property_test]
fn test_edge_blend_ders_by_bspline_surface(
    #[strategy = uniform5(-1.0..=1.0)] z0: [f64; 5],
    #[strategy = uniform5(0.01..=1.0)] tangent0: [f64; 5],
    #[strategy = uniform5(-1.0..=1.0)] z1: [f64; 5],
    #[strategy = uniform5(0.01..=1.0)] tangent1: [f64; 5],
    #[strategy = uniform2(0.0..=1.0)] [u, v]: [f64; 2],
    #[strategy = uniform2(0usize..=3)] [m, n]: [usize; 2],
) {
    let control_points0 = (0..=4)
        .map(|i| {
            vec![
                Point3::new(-tangent0[i] / 4.0, 0.25 * i as f64, z0[i]),
                Point3::new(0.0, 0.25 * i as f64, z0[i]),
            ]
        })
        .collect();
    let surface0 = BSplineSurface::new(
        (KnotVec::bezier_knot(4), KnotVec::bezier_knot(1)),
        control_points0,
    );

    let control_points1 = (0..=4)
        .map(|i| {
            vec![
                Point3::new(1.0, 0.25 * i as f64, z1[i]),
                Point3::new(1.0 + tangent1[i] / 4.0, 0.25 * i as f64, z1[i]),
            ]
        })
        .collect();
    let surface1 = BSplineSurface::new(
        (KnotVec::bezier_knot(4), KnotVec::bezier_knot(1)),
        control_points1,
    );

    let pcurve0 = PCurve::new(Line(Point2::new(1.0, 1.0), Point2::new(0.0, 1.0)), surface0);
    let pcurve1 = PCurve::new(Line(Point2::new(1.0, 0.0), Point2::new(0.0, 0.0)), surface1);

    let tangent_controls0 = (0..=4).rev().map(|i| Vector1::new(tangent0[i])).collect();
    let tangent_curve0 = BSplineCurve::new(KnotVec::bezier_knot(4), tangent_controls0);
    let tangent_controls1 = (0..=4).rev().map(|i| Vector1::new(tangent1[i])).collect();
    let tangent_curve1 = BSplineCurve::new(KnotVec::bezier_knot(4), tangent_controls1);

    let surface = EdgeBlendSurface::new(pcurve0, tangent_curve0, pcurve1, tangent_curve1);

    let control_points = (0..=4)
        .rev()
        .map(|i| {
            vec![
                Point3::new(0.0, 0.25 * i as f64, z0[i]),
                Point3::new(tangent0[i] / 3.0, 0.25 * i as f64, z0[i]),
                Point3::new(1.0 - tangent1[i] / 3.0, 0.25 * i as f64, z1[i]),
                Point3::new(1.0, 0.25 * i as f64, z1[i]),
            ]
        })
        .collect();
    let bsp_surface = BSplineSurface::new(
        (KnotVec::bezier_knot(4), KnotVec::bezier_knot(3)),
        control_points,
    );
    prop_assert_near!(surface.der_mn(m, n, u, v), bsp_surface.der_mn(m, n, u, v));
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
