use proptest::{prelude::*, property_test};
use std::f64::consts::{FRAC_1_SQRT_2, TAU};
use truck_modeling::*;

// Keep transforms invertible without discarding generated cases. Reflection also
// exercises normal orientation; unequal scales exercise homogeneous coordinates.
fn affine_strategy() -> impl Strategy<Value = Matrix4> {
    (
        prop::array::uniform3(-5.0f64..5.0),
        prop::array::uniform3(-3.0f64..3.0),
        prop::array::uniform3(0.5f64..2.0),
        any::<bool>(),
    )
        .prop_map(|(translation, angles, scale, reflected)| {
            Matrix4::from_translation(Vector3::from(translation))
                * Matrix4::from_angle_x(Rad(angles[0]))
                * Matrix4::from_angle_y(Rad(angles[1]))
                * Matrix4::from_angle_z(Rad(angles[2]))
                * Matrix4::from_nonuniform_scale(
                    scale[0] * if reflected { -1.0 } else { 1.0 },
                    scale[1],
                    scale[2],
                )
        })
}

fn weighted_curve(points: Vec<Point3>, weights: &[f64]) -> NurbsCurve<Vector4> {
    NurbsCurve::new(BSplineCurve::new(
        KnotVec::bezier_knot(points.len() - 1),
        points
            .into_iter()
            .zip(weights)
            .map(|(point, weight)| point.to_homogeneous() * *weight)
            .collect(),
    ))
}

fn line_variants(line: Line<Point3>, weight: f64) -> [Curve; 3] {
    [
        line.into(),
        BSplineCurve::from(line).into(),
        weighted_curve(vec![line.0, line.1], &[1.0, weight]).into(),
    ]
}

fn plane_xy() -> Plane {
    Plane::new(
        Point3::origin(),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    )
}

fn intersection() -> IntersectionCurve<BSplineCurve<Point3>, Surface, Surface> {
    IntersectionCurve::new(
        plane_xy().into(),
        Plane::new(
            Point3::origin(),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
        )
        .into(),
        BSplineCurve::from(Line(Point3::origin(), Point3::new(1.0, 0.0, 0.0))),
    )
}

fn curves(bend: f64, weight: f64) -> [Curve; 4] {
    let points = vec![
        Point3::origin(),
        Point3::new(0.5, bend, 0.25),
        Point3::new(1.0, 0.0, 0.0),
    ];
    [
        Line(points[0], points[2]).into(),
        BSplineCurve::new(KnotVec::bezier_knot(2), points.clone()).into(),
        weighted_curve(points, &[1.0, weight, 1.5]).into(),
        intersection().into(),
    ]
}

fn planar_surfaces(weight: f64) -> [Surface; 3] {
    let bsp = BSplineSurface::new(
        (KnotVec::bezier_knot(1), KnotVec::bezier_knot(1)),
        vec![
            vec![Point3::origin(), Point3::new(0.0, 1.0, 0.0)],
            vec![Point3::new(1.0, 0.0, 0.0), Point3::new(1.0, 1.0, 0.0)],
        ],
    );
    let nurbs = NurbsSurface::new(BSplineSurface::new(
        (KnotVec::bezier_knot(1), KnotVec::bezier_knot(1)),
        vec![
            vec![
                Vector4::new(0.0, 0.0, 0.0, 1.0),
                Vector4::new(0.0, weight, 0.0, weight),
            ],
            vec![
                Vector4::new(1.0, 0.0, 0.0, 1.0),
                Vector4::new(weight, weight, 0.0, weight),
            ],
        ],
    ));
    [plane_xy().into(), bsp.into(), nurbs.into()]
}

fn cylinder(generatrix: Curve) -> Surface {
    RevolutedCurve::by_revolution(generatrix, Point3::origin(), Vector3::unit_z())
        .to_same_geometry()
}

fn surfaces(bend: f64, weight: f64) -> [Surface; 4] {
    let bsp = BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)),
        (0..3)
            .map(|i| {
                (0..3)
                    .map(|j| {
                        Point3::new(
                            i as f64 / 2.0,
                            j as f64 / 2.0,
                            if i == 1 && j == 1 { bend } else { 0.0 },
                        )
                    })
                    .collect()
            })
            .collect(),
    );
    let nurbs = NurbsSurface::new(BSplineSurface::new(
        bsp.knot_vecs().clone(),
        bsp.control_points()
            .iter()
            .enumerate()
            .map(|(i, row)| {
                row.iter()
                    .enumerate()
                    .map(|(j, p)| p.to_homogeneous() * if i == 1 && j == 1 { weight } else { 1.0 })
                    .collect()
            })
            .collect(),
    ));
    [
        plane_xy().into(),
        bsp.into(),
        nurbs.into(),
        cylinder(Line(Point3::new(1.0, 0.0, 0.0), Point3::new(1.0, 0.0, 1.0)).into()),
    ]
}

#[property_test]
fn curve_transform(
    #[strategy = affine_strategy()] transform: Matrix4,
    #[strategy = -1.0f64..1.0] bend: f64,
    #[strategy = 0.5f64..2.0] weight: f64,
    #[strategy = 0.0f64..=1.0] t: f64,
) {
    for curve in curves(bend, weight) {
        let transformed = curve.transformed(transform);
        let mut in_place = curve.clone();
        in_place.transform_by(transform);
        prop_assert_eq!(curve.parameter_range(), transformed.parameter_range());
        for t in [0.0, t, 1.0] {
            let point = transform.transform_point(curve.subs(t));
            prop_assert_near!(transformed.subs(t), point);
            prop_assert_near!(in_place.subs(t), point);
            prop_assert_near!(transformed.der(t), transform.transform_vector(curve.der(t)));
            prop_assert_near!(
                transformed.der2(t),
                transform.transform_vector(curve.der2(t))
            );
            prop_assert_near!(in_place.der(t), transformed.der(t));
            prop_assert_near!(in_place.der2(t), transformed.der2(t));
            prop_assert_near!(
                transformed.transformed(transform.invert().unwrap()).subs(t),
                curve.subs(t)
            );
        }
    }
}

#[property_test]
fn curve_transform_composition(
    #[strategy = affine_strategy()] first: Matrix4,
    #[strategy = affine_strategy()] second: Matrix4,
    #[strategy = 0.0f64..=1.0] t: f64,
) {
    for curve in curves(0.7, 1.3) {
        let sequential = curve.transformed(first).transformed(second);
        let composed = curve.transformed(second * first);
        prop_assert_near!(sequential.subs(t), composed.subs(t));
        prop_assert_near!(sequential.der(t), composed.der(t));
    }
}

#[property_test]
fn curve_inverse(
    #[strategy = -1.0f64..1.0] bend: f64,
    #[strategy = 0.5f64..2.0] weight: f64,
    #[strategy = 0.0f64..=1.0] t: f64,
) {
    for curve in curves(bend, weight) {
        let inverse = curve.inverse();
        let mut in_place = curve.clone();
        in_place.invert();
        prop_assert_eq!(inverse.parameter_range(), curve.parameter_range());
        prop_assert_near!(inverse.subs(t), curve.subs(1.0 - t));
        prop_assert_near!(inverse.der(t), -curve.der(1.0 - t));
        prop_assert_near!(inverse.der2(t), curve.der2(1.0 - t));
        prop_assert_near!(in_place.subs(t), inverse.subs(t));
        prop_assert_near!(inverse.inverse().subs(t), curve.subs(t));
        prop_assert_near!(curve.front(), curve.subs(0.0));
        prop_assert_near!(curve.back(), curve.subs(1.0));
    }
}

#[property_test]
fn curve_cut(
    #[strategy = 0.1f64..0.9] cut: f64,
    #[strategy = 0.0f64..=1.0] fraction: f64,
    #[strategy = 0.5f64..2.0] weight: f64,
) {
    for curve in curves(0.7, weight) {
        let mut left = curve.clone();
        let right = left.cut(cut);
        prop_assert_near!(left.front(), curve.front());
        prop_assert_near!(left.back(), curve.subs(cut));
        prop_assert_near!(right.front(), curve.subs(cut));
        prop_assert_near!(right.back(), curve.back());
        let (a, b) = left.range_tuple();
        prop_assert_near!(
            left.subs(a + (b - a) * fraction),
            curve.subs(cut * fraction)
        );
        let (a, b) = right.range_tuple();
        prop_assert_near!(
            right.subs(a + (b - a) * fraction),
            curve.subs(cut + (1.0 - cut) * fraction)
        );
    }
}

#[property_test]
fn curve_parameter_division(#[strategy = 0.01f64..0.1] tolerance: f64) {
    for curve in curves(0.7, 1.3) {
        let (parameters, points) = curve.parameter_division((0.1, 0.9), tolerance);
        prop_assert_eq!(parameters.len(), points.len());
        prop_assert_near!(parameters[0], 0.1);
        prop_assert_near!(*parameters.last().unwrap(), 0.9);
        prop_assert!(parameters.windows(2).all(|pair| pair[0] < pair[1]));
        for (t, point) in parameters.into_iter().zip(points) {
            prop_assert_near!(curve.subs(t), point);
        }
    }
}

#[property_test]
fn curve_search_parameter(
    #[strategy = affine_strategy()] transform: Matrix4,
    #[strategy = 0.1f64..0.9] t: f64,
) {
    for curve in curves(0.7, 1.3) {
        let curve = curve.transformed(transform);
        let point = curve.subs(t);
        for hint in [
            SPHint1D::None,
            SPHint1D::Parameter(t + 0.03),
            SPHint1D::Range(0.0, 1.0),
        ] {
            let found = curve.search_parameter(point, hint, 100);
            prop_assert!(found.is_some(), "{curve:?}, {hint:?}");
            prop_assert_near!(curve.subs(found.unwrap()), point);
            let nearest = curve.search_nearest_parameter(point, hint, 100);
            prop_assert!(nearest.is_some());
            prop_assert_near!(curve.subs(nearest.unwrap()), point);
        }
        let off_curve = point + transform.transform_vector(Vector3::unit_z()) * 3.0;
        prop_assert!(curve.search_parameter(off_curve, Some(t), 100).is_none());
    }
}

#[property_test]
fn curve_lift_up(
    #[strategy = 0.5f64..2.0] weight: f64,
    #[strategy = -1.0f64..1.0] bend: f64,
    #[strategy = 0.0f64..=1.0] t: f64,
) {
    for curve in curves(bend, weight).into_iter().take(3) {
        let lifted = curve.lift_up();
        prop_assert_eq!(lifted.parameter_range(), curve.parameter_range());
        prop_assert_near!(Point3::from_homogeneous(lifted.subs(t)), curve.subs(t));
        match &curve {
            Curve::NurbsCurve(nurbs) => prop_assert_eq!(&lifted, nurbs.non_rationalized()),
            _ => prop_assert!(lifted.control_points().iter().all(|point| point.w == 1.0)),
        }
    }
}

#[property_test]
fn curve_to_same_geometry(
    #[strategy = affine_strategy()] transform: Matrix4,
    #[strategy = 0.0f64..=1.0] t: f64,
) {
    let line = Line(Point3::origin(), Point3::new(1.0, 2.0, 3.0)).transformed(transform);
    let converted: Curve = line.to_same_geometry();
    prop_assert!(matches!(converted, Curve::Line(_)));
    prop_assert_near!(converted.subs(t), line.subs(t));
    let bsp = BSplineCurve::from(line);
    let converted: Curve = bsp.to_same_geometry();
    prop_assert!(matches!(converted, Curve::BSplineCurve(_)));
    prop_assert_near!(converted.subs(t), bsp.subs(t));
}

#[property_test]
fn circle_to_same_geometry(
    #[strategy = affine_strategy()] transform: Matrix4,
    #[strategy = 0.2f64..3.0] angle: f64,
    #[strategy = 0.0f64..=1.0] fraction: f64,
) {
    let circle = Processor::with_transform(
        TrimmedCurve::new(UnitCircle::<Point3>::new(), (0.0, angle)),
        transform,
    );
    for circle in [circle, circle.inverse()] {
        let converted: Curve = circle.to_same_geometry();
        prop_assert!(matches!(converted, Curve::NurbsCurve(_)));
        prop_assert_near!(converted.front(), circle.front());
        prop_assert_near!(converted.back(), circle.back());
        let (a, b) = converted.range_tuple();
        let local = transform
            .invert()
            .unwrap()
            .transform_point(converted.subs(a + (b - a) * fraction));
        prop_assert_near!(local.x * local.x + local.y * local.y, 1.0);
        prop_assert_near!(local.z, 0.0);
    }
}

#[property_test]
fn intersection_curve_conversion(
    #[strategy = affine_strategy()] transform: Matrix4,
    #[strategy = 0.0f64..=1.0] t: f64,
) {
    let original = intersection();
    let converted: Curve = original.clone().into();
    prop_assert!(matches!(converted, Curve::IntersectionCurve(_)));
    prop_assert_near!(converted.subs(t), Point3::new(t, 0.0, 0.0));
    prop_assert_near!(converted.der(t), Vector3::unit_x());
    prop_assert_near!(converted.der2(t), Vector3::zero());
    let converted = converted.transformed(transform);
    prop_assert_near!(
        converted.subs(t),
        transform.transform_point(Point3::new(t, 0.0, 0.0))
    );
    if let Curve::IntersectionCurve(curve) = converted {
        let point = curve.subs(t);
        for surface in [curve.surface0(), curve.surface1()] {
            let uv = surface.search_parameter(point, None, 100).unwrap();
            prop_assert_near!(surface.subs(uv.0, uv.1), point);
        }
    }
}

#[property_test]
fn surface_transform(
    #[strategy = affine_strategy()] transform: Matrix4,
    #[strategy = -1.0f64..1.0] bend: f64,
    #[strategy = 0.5f64..2.0] weight: f64,
    #[strategy = 0.0f64..=1.0] u: f64,
    #[strategy = 0.0f64..=1.0] v: f64,
) {
    for surface in surfaces(bend, weight) {
        let transformed = surface.transformed(transform);
        let mut in_place = surface.clone();
        in_place.transform_by(transform);
        prop_assert_eq!(transformed.parameter_range(), surface.parameter_range());
        prop_assert_near!(
            transformed.subs(u, v),
            transform.transform_point(surface.subs(u, v))
        );
        prop_assert_near!(in_place.subs(u, v), transformed.subs(u, v));
        for (actual, expected) in [
            (transformed.uder(u, v), surface.uder(u, v)),
            (transformed.vder(u, v), surface.vder(u, v)),
            (transformed.uuder(u, v), surface.uuder(u, v)),
            (transformed.uvder(u, v), surface.uvder(u, v)),
            (transformed.vvder(u, v), surface.vvder(u, v)),
        ] {
            prop_assert_near!(actual, transform.transform_vector(expected));
        }
        let normal = transformed
            .uder(u, v)
            .cross(transformed.vder(u, v))
            .normalize();
        prop_assert_near!(transformed.normal(u, v).normalize(), normal);
        prop_assert_near!(in_place.normal(u, v), transformed.normal(u, v));
        prop_assert_near!(
            transformed
                .transformed(transform.invert().unwrap())
                .subs(u, v),
            surface.subs(u, v)
        );
    }
}

#[property_test]
fn surface_transform_composition(
    #[strategy = affine_strategy()] first: Matrix4,
    #[strategy = affine_strategy()] second: Matrix4,
    #[strategy = 0.0f64..=1.0] u: f64,
    #[strategy = 0.0f64..=1.0] v: f64,
) {
    for surface in surfaces(0.7, 1.3) {
        let sequential = surface.transformed(first).transformed(second);
        let composed = surface.transformed(second * first);
        prop_assert_near!(sequential.subs(u, v), composed.subs(u, v));
        prop_assert_near!(sequential.normal(u, v), composed.normal(u, v));
    }
}

#[property_test]
fn surface_inverse(
    #[strategy = affine_strategy()] transform: Matrix4,
    #[strategy = 0.1f64..0.9] u: f64,
    #[strategy = 0.1f64..0.9] v: f64,
) {
    for surface in surfaces(0.7, 1.3) {
        let surface = surface.transformed(transform);
        let inverse = surface.inverse();
        let mut in_place = surface.clone();
        in_place.invert();
        prop_assert_near!(inverse.subs(v, u), surface.subs(u, v));
        prop_assert_near!(inverse.uder(v, u), surface.vder(u, v));
        prop_assert_near!(inverse.vder(v, u), surface.uder(u, v));
        prop_assert_near!(
            inverse.normal(v, u).normalize(),
            -surface.normal(u, v).normalize()
        );
        prop_assert_near!(in_place.subs(v, u), inverse.subs(v, u));
        prop_assert_near!(inverse.inverse().subs(u, v), surface.subs(u, v));
    }
}

#[property_test]
fn surface_parameter_division(#[strategy = 0.01f64..0.1] tolerance: f64) {
    for surface in surfaces(0.7, 1.3) {
        let (us, vs) = surface.parameter_division(((0.1, 0.9), (0.2, 0.8)), tolerance);
        for (parameters, start, end) in [(us, 0.1, 0.9), (vs, 0.2, 0.8)] {
            prop_assert_near!(parameters[0], start);
            prop_assert_near!(*parameters.last().unwrap(), end);
            prop_assert!(parameters.windows(2).all(|pair| pair[0] < pair[1]));
        }
    }
}

#[property_test]
fn surface_search_parameter(
    #[strategy = affine_strategy()] transform: Matrix4,
    #[strategy = 0.1f64..0.9] u: f64,
    #[strategy = 0.2f64..0.8] v: f64,
) {
    for surface in surfaces(0.7, 1.3) {
        for surface in [
            surface.transformed(transform),
            surface.transformed(transform).inverse(),
        ] {
            let point = surface.subs(u, v);
            for hint in [
                SPHint2D::None,
                SPHint2D::Parameter(u + 0.03, v + 0.03),
                SPHint2D::Range((0.0, 1.0), (0.0, 1.0)),
            ] {
                let found = surface.search_parameter(point, hint, 100);
                prop_assert!(found.is_some(), "{surface:?}, {hint:?}");
                let (a, b) = found.unwrap();
                prop_assert_near!(surface.subs(a, b), point);
                let found = surface.search_nearest_parameter(point, hint, 100);
                prop_assert!(found.is_some(), "{surface:?}, {hint:?}");
                let (a, b) = found.unwrap();
                prop_assert_near!(surface.subs(a, b), point);
            }
        }
    }
}

#[property_test]
fn planar_surface_nearest_parameter(
    #[strategy = affine_strategy()] transform: Matrix4,
    #[strategy = 0.1f64..0.9] u: f64,
    #[strategy = 0.1f64..0.9] v: f64,
    #[strategy = 0.1f64..2.0] distance: f64,
) {
    for surface in planar_surfaces(1.3) {
        let surface = surface.transformed(transform);
        let foot = surface.subs(u, v);
        let point = foot + surface.normal(u, v).normalize() * distance;
        for hint in [
            SPHint2D::None,
            SPHint2D::Parameter(u, v),
            SPHint2D::Range((0.0, 1.0), (0.0, 1.0)),
        ] {
            prop_assert!(surface.search_parameter(point, hint, 100).is_none());
            let nearest = surface.search_nearest_parameter(point, hint, 100);
            prop_assert!(nearest.is_some());
            let (a, b) = nearest.unwrap();
            prop_assert_near!(surface.subs(a, b), foot);
        }
    }
}

#[property_test]
fn planar_include_curve(
    #[strategy = affine_strategy()] transform: Matrix4,
    #[strategy = 0.5f64..2.0] weight: f64,
    #[strategy = 0.1f64..0.3] offset: f64,
) {
    for surface in planar_surfaces(weight) {
        for surface in [
            surface.transformed(transform),
            surface.transformed(transform).inverse(),
        ] {
            let line = Line(Point3::new(0.2, 0.2, 0.0), Point3::new(0.8, 0.8, 0.0));
            let bsp = BSplineCurve::new(
                KnotVec::bezier_knot(2),
                vec![line.0, Point3::new(0.2, 0.8, 0.0), line.1],
            );
            let queries: [Curve; 3] = [
                line.into(),
                bsp.clone().into(),
                weighted_curve(bsp.control_points().clone(), &[1.0, weight, 1.5]).into(),
            ];
            for query in queries {
                let query = query.transformed(transform);
                prop_assert!(surface.include(&query), "{surface:?}, {query:?}");
                prop_assert!(surface.include(&query.inverse()));
                let off = query.transformed(Matrix4::from_translation(
                    transform.transform_vector(Vector3::unit_z() * offset),
                ));
                prop_assert!(!surface.include(&off));
            }
        }
    }
}

#[property_test]
fn planar_include_rejects_interior_bulge(
    #[strategy = affine_strategy()] transform: Matrix4,
    #[strategy = 0.1f64..0.5] bulge: f64,
    #[strategy = 0.5f64..2.0] weight: f64,
) {
    // Endpoints lie on the plane, but the interior does not. Testing endpoints
    // alone would incorrectly accept both polynomial and rational curves.
    let points = vec![
        Point3::new(0.2, 0.2, 0.0),
        Point3::new(0.5, 0.5, bulge),
        Point3::new(0.8, 0.8, 0.0),
    ];
    let queries: [Curve; 2] = [
        BSplineCurve::new(KnotVec::bezier_knot(2), points.clone()).into(),
        weighted_curve(points, &[1.0, weight, 1.0]).into(),
    ];
    for surface in planar_surfaces(weight) {
        let surface = surface.transformed(transform);
        for query in &queries {
            prop_assert!(!surface.include(&query.transformed(transform)));
        }
    }
}

#[property_test]
fn revolved_include_curve(
    #[strategy = affine_strategy()] transform: Matrix4,
    #[strategy = 0.5f64..2.0] radius: f64,
    #[strategy = 0.5f64..2.0] height: f64,
    #[strategy = 0.0f64..TAU] angle: f64,
    #[strategy = 0.5f64..2.0] weight: f64,
) {
    let line = Line(
        Point3::new(radius, 0.0, 0.0),
        Point3::new(radius, 0.0, height),
    );
    for generatrix in line_variants(line, weight) {
        let surface = cylinder(generatrix).transformed(transform);
        for surface in [surface.clone(), surface.inverse()] {
            for radial_scale in [0.5, 1.0, 1.5] {
                let radial = Vector3::new(angle.cos(), angle.sin(), 0.0) * radius * radial_scale;
                let query = Line(
                    Point3::from_vec(radial) + Vector3::unit_z() * height * 0.2,
                    Point3::from_vec(radial) + Vector3::unit_z() * height * 0.8,
                );
                for query in line_variants(query, weight) {
                    let query = query.transformed(transform);
                    prop_assert_eq!(
                        surface.include(&query),
                        radial_scale == 1.0,
                        "surface={:?}, query={:?}",
                        surface,
                        query
                    );
                    prop_assert_eq!(surface.include(&query.inverse()), radial_scale == 1.0);
                }
            }
        }
    }
}

#[property_test]
fn revolved_include_rejects_old_location(#[strategy = 5.0f64..10.0] translation: f64) {
    let transform = Matrix4::from_translation(Vector3::new(translation, 0.0, 0.0));
    for generatrix in line_variants(
        Line(Point3::new(1.0, 0.0, 0.0), Point3::new(1.0, 0.0, 1.0)),
        1.3,
    ) {
        let surface = cylinder(generatrix).transformed(transform);
        for query in line_variants(
            Line(Point3::new(1.0, 0.0, 0.2), Point3::new(1.0, 0.0, 0.8)),
            0.7,
        ) {
            prop_assert!(!surface.include(&query));
            prop_assert!(surface.include(&query.transformed(transform)));
        }
    }
}

#[property_test]
fn revolved_include_curved_nurbs(
    #[strategy = affine_strategy()] transform: Matrix4,
    #[strategy = 0.1f64..0.9] height: f64,
) {
    // A rational quadratic quarter circle, with genuinely nonuniform weights.
    let arc: Curve = weighted_curve(
        vec![
            Point3::new(1.0, 0.0, height),
            Point3::new(1.0, 1.0, height),
            Point3::new(0.0, 1.0, height),
        ],
        &[1.0, FRAC_1_SQRT_2, 1.0],
    )
    .into();
    for generatrix in line_variants(
        Line(Point3::new(1.0, 0.0, 0.0), Point3::new(1.0, 0.0, 1.0)),
        1.3,
    ) {
        let surface = cylinder(generatrix).transformed(transform);
        prop_assert!(surface.include(&arc.transformed(transform)));
        prop_assert!(!surface.include(
            &arc.transformed(Matrix4::from_nonuniform_scale(1.5, 1.5, 1.0))
                .transformed(transform)
        ));
    }
}

#[property_test]
fn plane_to_same_geometry(
    #[strategy = affine_strategy()] transform: Matrix4,
    #[strategy = -2.0f64..2.0] u: f64,
    #[strategy = -2.0f64..2.0] v: f64,
) {
    let plane = plane_xy().transformed(transform);
    let surface: Surface = plane.to_same_geometry();
    prop_assert!(matches!(surface, Surface::Plane(_)));
    prop_assert_near!(surface.subs(u, v), ParametricSurface::subs(&plane, u, v));
}

#[property_test]
fn revolved_to_same_geometry(
    #[strategy = 0.5f64..2.0] radius: f64,
    #[strategy = 0.0f64..=1.0] u: f64,
    #[strategy = 0.0f64..TAU] angle: f64,
) {
    for generatrix in line_variants(
        Line(Point3::new(radius, 0.0, 0.0), Point3::new(radius, 0.0, 1.0)),
        1.3,
    ) {
        let original =
            RevolutedCurve::by_revolution(generatrix, Point3::origin(), Vector3::unit_z());
        let converted: Surface = original.to_same_geometry();
        prop_assert!(matches!(converted, Surface::RevolutedCurve(_)));
        prop_assert_near!(converted.subs(u, angle), original.subs(u, angle));
        prop_assert_near!(converted.subs(u, angle + TAU), converted.subs(u, angle));
        prop_assert_near!(converted.v_period().unwrap(), TAU);
    }
}

#[property_test]
fn homotopy_to_same_geometry(
    #[strategy = 0.5f64..2.0] weight: f64,
    #[strategy = -1.0f64..1.0] bend: f64,
    #[strategy = 0.0f64..=1.0] u: f64,
    #[strategy = 0.0f64..=1.0] v: f64,
) {
    // All nine pairs, including curves with different degrees and weights.
    let first = curves(bend, weight);
    let second = curves(-bend, 1.0 / weight);
    for c0 in first.iter().take(3) {
        for c1 in second.iter().take(3) {
            let c1 = c1.transformed(Matrix4::from_translation(Vector3::unit_z() * 2.0));
            let original = HomotopySurface::new(c0.clone(), c1);
            let converted: Surface = original.to_same_geometry();
            prop_assert!(matches!(converted, Surface::NurbsSurface(_)));
            // ToSameGeometry preserves the image, not the parameterization.
            // Different homogeneous weights reparameterize each ruling.
            let start = original.curve0().subs(u);
            let end = original.curve1().subs(u);
            let ruling = end - start;
            let point = converted.subs(u, v);
            let fraction = (point - start).dot(ruling) / ruling.magnitude2();
            prop_assert!((-TOLERANCE..=1.0 + TOLERANCE).contains(&fraction));
            prop_assert_near!(point, start + ruling * fraction);
            prop_assert_near!(converted.vder(u, v).cross(ruling), Vector3::zero());
            let original_point = original.subs(u, v);
            let found = converted.search_parameter(original_point, Some((u, v)), 100);
            prop_assert!(found.is_some());
            let (a, b) = found.unwrap();
            prop_assert_near!(converted.subs(a, b), original_point);
            prop_assert_near!(converted.subs(u, 0.0), original.curve0().subs(u));
            prop_assert_near!(converted.subs(u, 1.0), original.curve1().subs(u));
        }
    }
}

#[property_test]
fn extrusion_to_same_geometry(
    #[strategy = affine_strategy()] transform: Matrix4,
    #[strategy = 0.5f64..2.0] weight: f64,
    #[strategy = 0.0f64..=1.0] u: f64,
    #[strategy = 0.0f64..=1.0] v: f64,
) {
    for (i, curve) in curves(0.7, weight).into_iter().take(3).enumerate() {
        let original = ExtrudedCurve::by_extrusion(
            curve.transformed(transform),
            transform.transform_vector(Vector3::unit_z() * 2.0),
        );
        let converted: Surface = original.to_same_geometry();
        prop_assert!(matches!(
            (i, &converted),
            (0, Surface::Plane(_))
                | (1, Surface::BSplineSurface(_))
                | (2, Surface::NurbsSurface(_))
        ));
        prop_assert_near!(converted.subs(u, v), original.subs(u, v));
        prop_assert_near!(converted.uder(u, v), original.uder(u, v));
        prop_assert_near!(converted.vder(u, v), original.vder(u, v));
        prop_assert_near!(converted.subs(u, 0.0), original.entity_curve().subs(u));
        prop_assert_near!(
            converted.subs(u, 1.0),
            original.entity_curve().subs(u) + original.extruding_vector()
        );
    }
}

// Unsupported operations do not depend on generated inputs. Keep each panic
// to a single ordinary test: property testing would repeat panic-hook output
// hundreds of times under the CI runner's --nocapture setting.
#[test]
#[should_panic(expected = "intersection curve cannot connect by homotopy")]
fn intersection_lift_up_is_unsupported() {
    let curve: Curve = intersection().into();
    curve.lift_up();
}

#[test]
#[should_panic(expected = "not implemented")]
fn intersection_extrusion_is_unsupported() {
    let curve: Curve = intersection().into();
    let _: Surface = ExtrudedCurve::by_extrusion(curve, Vector3::unit_z()).to_same_geometry();
}

#[test]
#[should_panic(expected = "intersection curve cannot connect by homotopy")]
fn intersection_homotopy_is_unsupported() {
    let curve: Curve = intersection().into();
    let _: Surface = HomotopySurface::new(curve.clone(), curve).to_same_geometry();
}

#[test]
#[should_panic(expected = "not implemented")]
fn intersection_plane_include_is_unsupported() {
    let curve: Curve = intersection().into();
    let surface = Surface::Plane(plane_xy());
    surface.include(&curve);
}

#[test]
#[should_panic(expected = "not implemented")]
fn intersection_bspline_surface_include_is_unsupported() {
    let curve: Curve = intersection().into();
    let surface = planar_surfaces(1.3)[1].clone();
    surface.include(&curve);
}

#[test]
#[should_panic(expected = "not implemented")]
fn intersection_nurbs_surface_include_is_unsupported() {
    let curve: Curve = intersection().into();
    let surface = planar_surfaces(1.3)[2].clone();
    surface.include(&curve);
}

#[test]
#[should_panic(expected = "not implemented")]
fn intersection_revolved_surface_include_is_unsupported() {
    let curve: Curve = intersection().into();
    let surface = cylinder(Line(Point3::new(1.0, 0.0, 0.0), Point3::new(1.0, 0.0, 1.0)).into());
    surface.include(&curve);
}

#[test]
#[should_panic(expected = "not implemented")]
fn intersection_revolved_generatrix_is_unsupported() {
    let surface = cylinder(intersection().into());
    let query: Curve = Line(Point3::origin(), Point3::new(1.0, 0.0, 0.0)).into();
    surface.include(&query);
}

#[property_test]
fn curve_analytic_evaluation(
    #[strategy = -1.0f64..1.0] bend: f64,
    #[strategy = 0.5f64..2.0] weight: f64,
    #[strategy = 0.0f64..=1.0] t: f64,
) {
    let [line, bsp, nurbs, _] = curves(bend, weight);
    prop_assert_near!(line.subs(t), Point3::new(t, 0.0, 0.0));
    prop_assert_near!(
        bsp.subs(t),
        Point3::new(t, 2.0 * bend * t * (1.0 - t), 0.5 * t * (1.0 - t))
    );
    prop_assert_near!(
        bsp.der(t),
        Vector3::new(1.0, 2.0 * bend * (1.0 - 2.0 * t), 0.5 * (1.0 - 2.0 * t))
    );
    prop_assert_near!(bsp.der2(t), Vector3::new(0.0, -4.0 * bend, -1.0));
    let b0 = (1.0 - t).powi(2);
    let b1 = 2.0 * t * (1.0 - t) * weight;
    let b2 = t * t * 1.5;
    let denominator = b0 + b1 + b2;
    prop_assert_near!(
        nurbs.subs(t),
        Point3::new(
            (0.5 * b1 + b2) / denominator,
            bend * b1 / denominator,
            0.25 * b1 / denominator
        )
    );
}

#[property_test]
fn planar_include_respects_patch_boundary(#[strategy = affine_strategy()] transform: Matrix4) {
    for (i, surface) in planar_surfaces(1.3).into_iter().enumerate() {
        let surface = surface.transformed(transform);
        // The plane is unbounded; the polynomial/rational patches are [0,1]^2.
        for query in line_variants(
            Line(Point3::new(0.5, 0.5, 0.0), Point3::new(2.0, 0.5, 0.0)),
            1.3,
        ) {
            prop_assert_eq!(surface.include(&query.transformed(transform)), i == 0);
        }
    }
}

#[property_test]
fn revolved_include_curved_generatrix(
    #[strategy = affine_strategy()] transform: Matrix4,
    #[strategy = 0.0f64..TAU] angle: f64,
    #[strategy = 0.5f64..2.0] weight: f64,
    #[strategy = -0.2f64..0.2] bulge: f64,
) {
    let points = vec![
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0 + bulge, 0.0, 0.5),
        Point3::new(0.8, 0.0, 1.0),
    ];
    let bsp = BSplineCurve::new(KnotVec::bezier_knot(2), points.clone());
    let nurbs = weighted_curve(points, &[1.0, weight, 1.0]);
    let bsp_queries: [Curve; 2] = [
        bsp.clone().into(),
        NurbsCurve::<Vector4>::from(bsp.clone()).into(),
    ];
    for (generatrix, queries) in [
        (Curve::BSplineCurve(bsp), bsp_queries.to_vec()),
        (
            Curve::NurbsCurve(nurbs.clone()),
            vec![Curve::NurbsCurve(nurbs)],
        ),
    ] {
        let surface = cylinder(generatrix).transformed(transform);
        for query in queries {
            let query = query.transformed(Matrix4::from_angle_z(Rad(angle)));
            prop_assert!(surface.include(&query.transformed(transform)));
            let outside = query.transformed(Matrix4::from_nonuniform_scale(3.0, 3.0, 1.0));
            prop_assert!(!surface.include(&outside.transformed(transform)));
        }
    }
}

#[property_test]
fn revolved_nearest_parameter(
    #[strategy = prop::array::uniform3(-5.0f64..5.0)] translation: [f64; 3],
    #[strategy = 0.5f64..2.0] radius: f64,
    #[strategy = 0.1f64..0.9] height: f64,
    #[strategy = 0.0f64..TAU] angle: f64,
    #[strategy = 0.1f64..1.0] offset: f64,
) {
    let transform = Matrix4::from_translation(Vector3::from(translation));
    let radial = Vector3::new(angle.cos(), angle.sin(), 0.0);
    let point = transform.transform_point(Point3::from_vec(
        radial * (radius + offset) + Vector3::unit_z() * height,
    ));
    let foot = transform.transform_point(Point3::from_vec(
        radial * radius + Vector3::unit_z() * height,
    ));
    for generatrix in line_variants(
        Line(Point3::new(radius, 0.0, 0.0), Point3::new(radius, 0.0, 1.0)),
        1.3,
    ) {
        let surface = cylinder(generatrix).transformed(transform);
        for hint in [
            SPHint2D::None,
            SPHint2D::Parameter(height, angle),
            SPHint2D::Range((0.0, 1.0), (0.0, TAU)),
        ] {
            prop_assert!(surface.search_parameter(point, hint, 100).is_none());
            let nearest = surface.search_nearest_parameter(point, hint, 100);
            prop_assert!(nearest.is_some());
            let (u, v) = nearest.unwrap();
            prop_assert_near!(surface.subs(u, v), foot);
        }
    }
}

#[property_test]
fn curve_serde_roundtrip(
    #[strategy = affine_strategy()] transform: Matrix4,
    #[strategy = 0.0f64..=1.0] t: f64,
) {
    for curve in curves(0.7, 1.3) {
        let curve = curve.transformed(transform).inverse();
        let restored: Curve =
            serde_json::from_str(&serde_json::to_string(&curve).unwrap()).unwrap();
        prop_assert_eq!(
            std::mem::discriminant(&restored),
            std::mem::discriminant(&curve)
        );
        prop_assert_near!(restored.subs(t), curve.subs(t));
        prop_assert_near!(restored.der(t), curve.der(t));
    }
}

#[property_test]
fn surface_serde_roundtrip(
    #[strategy = affine_strategy()] transform: Matrix4,
    #[strategy = 0.0f64..=1.0] u: f64,
    #[strategy = 0.0f64..=1.0] v: f64,
) {
    for surface in surfaces(0.7, 1.3) {
        let surface = surface.transformed(transform).inverse();
        let restored: Surface =
            serde_json::from_str(&serde_json::to_string(&surface).unwrap()).unwrap();
        prop_assert_eq!(
            std::mem::discriminant(&restored),
            std::mem::discriminant(&surface)
        );
        prop_assert_near!(restored.subs(u, v), surface.subs(u, v));
        prop_assert_near!(restored.normal(u, v), surface.normal(u, v));
    }
}

#[property_test]
fn curve_from_try_into(#[strategy = 0.0f64..=1.0] t: f64) {
    for curve in curves(0.7, 1.3) {
        let roundtrip = match curve.clone() {
            Curve::Line(_) => {
                let native: Line<Point3> = curve.clone().try_into().unwrap();
                Curve::from(native)
            }
            Curve::BSplineCurve(_) => {
                let native: BSplineCurve<Point3> = curve.clone().try_into().unwrap();
                Curve::from(native)
            }
            Curve::NurbsCurve(_) => {
                let native: NurbsCurve<Vector4> = curve.clone().try_into().unwrap();
                Curve::from(native)
            }
            Curve::IntersectionCurve(_) => {
                let native: IntersectionCurve<Box<Curve>, Box<Surface>, Box<Surface>> =
                    curve.clone().try_into().unwrap();
                Curve::from(native)
            }
        };
        prop_assert_eq!(
            std::mem::discriminant(&roundtrip),
            std::mem::discriminant(&curve)
        );
        prop_assert_near!(roundtrip.subs(t), curve.subs(t));
        let as_line: std::result::Result<Line<Point3>, _> = curve.clone().try_into();
        prop_assert_eq!(as_line.is_ok(), matches!(curve, Curve::Line(_)));
    }
}

#[property_test]
fn surface_from_try_into(#[strategy = 0.0f64..=1.0] u: f64, #[strategy = 0.0f64..=1.0] v: f64) {
    for surface in surfaces(0.7, 1.3) {
        let roundtrip = match surface.clone() {
            Surface::Plane(_) => {
                let native: Plane = surface.clone().try_into().unwrap();
                Surface::from(native)
            }
            Surface::BSplineSurface(_) => {
                let native: BSplineSurface<Point3> = surface.clone().try_into().unwrap();
                Surface::from(native)
            }
            Surface::NurbsSurface(_) => {
                let native: NurbsSurface<Vector4> = surface.clone().try_into().unwrap();
                Surface::from(native)
            }
            Surface::RevolutedCurve(_) => {
                let native: Processor<RevolutedCurve<Curve>, Matrix4> =
                    surface.clone().try_into().unwrap();
                Surface::from(native)
            }
        };
        prop_assert_eq!(
            std::mem::discriminant(&roundtrip),
            std::mem::discriminant(&surface)
        );
        prop_assert_near!(roundtrip.subs(u, v), surface.subs(u, v));
        let as_plane: std::result::Result<Plane, _> = surface.clone().try_into();
        prop_assert_eq!(as_plane.is_ok(), matches!(surface, Surface::Plane(_)));
    }
}
