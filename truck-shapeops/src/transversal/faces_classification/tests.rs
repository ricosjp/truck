use super::{
    super::{divide_face, loops_store},
    *,
};
use truck_geometry::prelude::*;
use truck_meshalgo::prelude::*;
use truck_topology::{shell::ShellCondition, Vertex};
const TOL: f64 = 0.05;

type AlternativeIntersection = crate::alternative::Alternative<
    NurbsCurve<Vector4>,
    IntersectionCurve<PolylineCurve<Point3>, AlternativeSurface, AlternativeSurface>,
>;
type AlternativeSurface = crate::alternative::Alternative<BSplineSurface<Point3>, Plane>;

fn parabola_surfaces() -> (AlternativeSurface, AlternativeSurface) {
    // define surfaces
    #[rustfmt::skip]
	let ctrl0 = vec![
		vec![Point3::new(-1.0, -1.0, 3.0), Point3::new(-1.0, 0.0, -1.0), Point3::new(-1.0, 1.0, 3.0)],
		vec![Point3::new(0.0, -1.0, -1.0), Point3::new(0.0, 0.0, -5.0), Point3::new(0.0, 1.0, -1.0)],
		vec![Point3::new(1.0, -1.0, 3.0), Point3::new(1.0, 0.0, -1.0), Point3::new(1.0, 1.0, 3.0)],
	];
    #[rustfmt::skip]
	let ctrl1 = vec![
		vec![Point3::new(-1.0, -1.0, -3.0), Point3::new(-1.0, 0.0, 1.0), Point3::new(-1.0, 1.0, -3.0)],
		vec![Point3::new(0.0, -1.0, 1.0), Point3::new(0.0, 0.0, 5.0), Point3::new(0.0, 1.0, 1.0)],
		vec![Point3::new(1.0, -1.0, -3.0), Point3::new(1.0, 0.0, 1.0), Point3::new(1.0, 1.0, -3.0)],
	];
    (
        BSplineSurface::new((KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)), ctrl0).into(),
        BSplineSurface::new((KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)), ctrl1).into(),
    )
}

#[test]
fn independent_intersection() {
    // prepare geoetries
    let arc00: AlternativeIntersection = NurbsCurve::new(BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Vector4::new(1.0, 0.0, 1.0, 1.0),
            Vector4::new(0.0, 1.0, 0.0, 0.0),
            Vector4::new(-1.0, 0.0, 1.0, 1.0),
        ],
    ))
    .into();
    let arc01: AlternativeIntersection = NurbsCurve::new(BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Vector4::new(-1.0, 0.0, 1.0, 1.0),
            Vector4::new(0.0, -1.0, 0.0, 0.0),
            Vector4::new(1.0, 0.0, 1.0, 1.0),
        ],
    ))
    .into();
    let arc10: AlternativeIntersection = NurbsCurve::new(BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Vector4::new(1.0, 0.0, -1.0, 1.0),
            Vector4::new(0.0, 1.0, 0.0, 0.0),
            Vector4::new(-1.0, 0.0, -1.0, 1.0),
        ],
    ))
    .into();
    let arc11: AlternativeIntersection = NurbsCurve::new(BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Vector4::new(-1.0, 0.0, -1.0, 1.0),
            Vector4::new(0.0, -1.0, 0.0, 0.0),
            Vector4::new(1.0, 0.0, -1.0, 1.0),
        ],
    ))
    .into();
    let (surface0, surface1) = parabola_surfaces();
    let plane0: AlternativeSurface = Plane::new(
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
    )
    .into();
    let plane1: AlternativeSurface = Plane::new(
        Point3::new(0.0, 0.0, -1.0),
        Point3::new(1.0, 0.0, -1.0),
        Point3::new(0.0, 1.0, -1.0),
    )
    .into();

    // prepare topologies
    let v00 = Vertex::new(Point3::new(1.0, 0.0, 1.0));
    let v01 = Vertex::new(Point3::new(-1.0, 0.0, 1.0));
    let v10 = Vertex::new(Point3::new(1.0, 0.0, -1.0));
    let v11 = Vertex::new(Point3::new(-1.0, 0.0, -1.0));
    let wire0: Wire<_, _> = vec![Edge::new(&v00, &v01, arc00), Edge::new(&v01, &v00, arc01)].into();
    let wire1: Wire<_, _> = vec![Edge::new(&v10, &v11, arc10), Edge::new(&v11, &v10, arc11)].into();
    let shell0: Shell<_, _, _> = vec![
        Face::new(vec![wire0.clone()], plane0),
        Face::new(vec![wire0], surface0).inverse(),
    ]
    .into();
    assert_eq!(shell0.shell_condition(), ShellCondition::Closed);
    let shell1: Shell<_, _, _> = vec![
        Face::new(vec![wire1.clone()], plane1).inverse(),
        Face::new(vec![wire1], surface1),
    ]
    .into();
    assert_eq!(shell1.shell_condition(), ShellCondition::Closed);
    let poly_shell0 = shell0.triangulation(TOL);
    let poly_shell1 = shell1.triangulation(TOL);

    let loops_store::LoopsStoreQuadruple {
        geom_loops_store0: loops_store0,
        geom_loops_store1: loops_store1,
        ..
    } = loops_store::create_loops_stores(&shell0, &poly_shell0, &shell1, &poly_shell1).unwrap();
    let mut cls0 = divide_face::divide_faces(&shell0, &loops_store0, TOL).unwrap();
    cls0.integrate_by_component();
    let mut cls1 = divide_face::divide_faces(&shell1, &loops_store1, TOL).unwrap();
    cls1.integrate_by_component();

    let [mut and, mut or, _] = cls0.and_or_unknown();
    let [and1, or1, _] = cls1.and_or_unknown();
    and.extend(and1);
    or.extend(or1);

    assert_eq!(and.len(), 2);
    assert_eq!(or.len(), 4);
    assert_eq!(and.shell_condition(), ShellCondition::Closed);
    assert_eq!(or.shell_condition(), ShellCondition::Closed);
}
