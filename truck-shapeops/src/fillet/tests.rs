use super::*;
use derive_more::From;
use truck_meshalgo::prelude::*;

#[derive(
    Clone,
    Debug,
    ParametricCurve,
    BoundedCurve,
    Cut,
    SearchNearestParameterD1,
    ParameterDivision1D,
    Invertible,
    From,
)]
enum Curve {
    Line(Line<Point3>),
    Nurbs(NurbsCurve<Vector4>),
    Parametric(PCurve<BSplineCurve<Point2>, Box<Surface>>),
    FilletSide(RbfContactCurve<Box<Self>, Box<Surface>, Box<Surface>, f64>),
    Intersection(IntersectionCurve<Box<Self>, Box<Surface>, Box<Surface>>),
}

impl ToSameGeometry<Curve> for IntersectionCurve<Curve, Surface, Surface> {
    fn to_same_geometry(&self) -> Curve {
        let (surface0, surface1, leader) = self.clone().destruct();
        Curve::Intersection(IntersectionCurve::new(
            Box::new(surface0),
            Box::new(surface1),
            Box::new(leader),
        ))
    }
}

impl ToSameGeometry<Curve> for PCurve<BSplineCurve<Point2>, Surface> {
    fn to_same_geometry(&self) -> Curve {
        let (curve, surface) = self.clone().decompose();
        Curve::Parametric(PCurve::new(curve, Box::new(surface)))
    }
}

impl ToSameGeometry<Curve> for RbfContactCurve<Curve, Surface, Surface, f64> {
    fn to_same_geometry(&self) -> Curve { Curve::FilletSide(self.clone().into()) }
}

#[derive(Clone, Debug, ParametricSurface3D, SearchParameterD2, ParameterDivision2D, From)]
enum Surface {
    Nurbs(NurbsSurface<Vector4>),
    Fillet(RbfSurface<Box<Curve>, Box<Self>, Box<Self>, f64>),
    Processor(Processor<Box<Self>, Matrix4>),
}

impl SearchNearestParameter<D2> for Surface {
    type Point = Point3;
    fn search_nearest_parameter<H: Into<<D2 as SPDimension>::Hint>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<<D2 as SPDimension>::Parameter> {
        match self {
            Self::Nurbs(nurbs) => nurbs.search_nearest_parameter(point, hint, trials),
            Self::Processor(processor) => processor.search_nearest_parameter(point, hint, trials),
            _ => unimplemented!(),
        }
    }
}

impl ToSameGeometry<Surface> for RbfSurface<Curve, Surface, Surface, f64> {
    fn to_same_geometry(&self) -> Surface { Surface::Fillet(self.clone().into()) }
}

impl Invertible for Surface {
    fn invert(&mut self) {
        match self {
            Self::Nurbs(surface) => surface.invert(),
            Self::Fillet(_) => {
                let mut processor = Processor::new(Box::new(self.clone()));
                processor.invert();
                *self = Self::Processor(processor);
            }
            Self::Processor(processor) => processor.invert(),
        }
    }
}

truck_topology::prelude!(Point3, Curve, Surface);

#[test]
fn create_simple_fillet() {
    #[rustfmt::skip]
    let surface0: NurbsSurface<_> = BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)),
        vec![
            vec![Point3::new(-1.0, 0.0, 0.0), Point3::new(-1.0, 0.5, 0.0), Point3::new(-1.0, 1.0, 1.0)],
            vec![Point3::new(0.0, 0.0, 0.0),  Point3::new(0.0, 0.5, 0.0),  Point3::new(0.0, 1.0, 1.0)],
            vec![Point3::new(1.0, 0.0, 0.0),  Point3::new(1.0, 0.5, 0.0),  Point3::new(1.0, 1.0, 1.0)],
        ],
    )
    .into();
    #[rustfmt::skip]
    let surface1: NurbsSurface<_> = BSplineSurface::new(
        (KnotVec::bezier_knot(2), KnotVec::bezier_knot(2)),
        vec![
            vec![Point3::new(1.0, 0.0, 0.0),  Point3::new(1.0, 0.0, -0.5),  Point3::new(1.0, 1.0, -1.0)],
            vec![Point3::new(0.0, 0.0, 0.0),  Point3::new(0.0, 0.5, -0.5),  Point3::new(0.0, 1.0, -1.0)],
            vec![Point3::new(-1.0, 0.0, 0.0), Point3::new(-1.0, 0.0, -0.5), Point3::new(-1.0, 1.0, -1.0)],
        ],
    )
    .into();

    let v = Vertex::news([
        Point3::new(-1.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(-1.0, 1.0, 1.0),
        Point3::new(-1.0, 1.0, -1.0),
        Point3::new(1.0, 1.0, -1.0),
    ]);

    let boundary0 = surface0.splitted_boundary();
    let boundary1 = surface1.splitted_boundary();

    let wire0: Wire = [
        Edge::new(&v[0], &v[1], Line(v[0].point(), v[1].point()).into()),
        Edge::new(&v[1], &v[2], boundary0[1].clone().into()),
        Edge::new(&v[2], &v[3], boundary0[2].clone().into()),
        Edge::new(&v[3], &v[0], boundary0[3].clone().into()),
    ]
    .into();

    let wire1: Wire = [
        wire0[0].inverse(),
        Edge::new(&v[0], &v[4], boundary1[1].clone().into()),
        Edge::new(&v[4], &v[5], boundary1[2].clone().into()),
        Edge::new(&v[5], &v[1], boundary1[3].clone().into()),
    ]
    .into();

    let shared_edge_id = wire0[0].id();
    let face0 = Face::new(vec![wire0], surface0.into());
    let face1 = Face::new(vec![wire1], surface1.into());

    let shell: Shell = [face0.clone(), face1.clone()].into();
    let poly = shell.triangulation(0.001).to_polygon();
    let file = std::fs::File::create("edged-shell.obj").unwrap();
    obj::write(&poly, file).unwrap();

    let res = simple_fillet(&face0, &face1, shared_edge_id, 0.3).unwrap();

    let shell: Shell = [res.face0, res.face1, res.fillet].into();
    let pshell = shell.triangulation(0.005);
    assert!(pshell.iter().all(|face| face.surface().is_some()));
    let poly = pshell.to_polygon();
    let file = std::fs::File::create("fillet-shell.obj").unwrap();
    obj::write(&poly, file).unwrap();
}

#[test]
fn create_fillet_with_side() {
    let p = [
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 0.3, 1.0),
        Point3::new(1.0, 1.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let v = Vertex::news(p);

    let line = |i: usize, j: usize| {
        let bsp = BSplineCurve::new(KnotVec::bezier_knot(1), vec![p[i], p[j]]);
        Edge::new(&v[i], &v[j], NurbsCurve::from(bsp).into())
    };

    let edge = [
        line(0, 1),
        line(1, 2),
        line(2, 3),
        line(3, 0),
        line(0, 4),
        line(1, 5),
        line(2, 6),
        line(3, 7),
        line(4, 5),
        line(5, 6),
        line(6, 7),
        line(7, 4),
    ];

    let plane = |i: usize, j: usize, k: usize, l: usize| {
        let control_points = vec![vec![p[i], p[l]], vec![p[j], p[k]]];
        let knot_vec = KnotVec::bezier_knot(1);
        let knot_vecs = (knot_vec.clone(), knot_vec);
        let bsp: NurbsSurface<Vector4> = BSplineSurface::new(knot_vecs, control_points).into();

        let wire: Wire = [i, j, k, l]
            .into_iter()
            .circular_tuple_windows()
            .map(|(i, j)| {
                edge.iter()
                    .find_map(|edge| {
                        if edge.front() == &v[i] && edge.back() == &v[j] {
                            Some(edge.clone())
                        } else if edge.back() == &v[i] && edge.front() == &v[j] {
                            Some(edge.inverse())
                        } else {
                            None
                        }
                    })
                    .unwrap()
            })
            .collect();
        Face::new(vec![wire], bsp.into())
    };

    let face = [plane(0, 1, 2, 3), plane(0, 3, 7, 4), plane(0, 4, 5, 1)];

    /*
    #[derive(Clone, Copy, Debug)]
    struct Radius;
    impl RadiusFunction for Radius {
        fn der_n(&self, n: usize, t: f64) -> f64 {
            match n {
                0 => 0.3 + 0.3 * t,
                1 => 0.3,
                _ => 0.0,
            }
        }
    }
    */

    let FilletWithSide {
        simple_fillet:
            SimpleFillet {
                face0,
                face1,
                fillet,
            },
        side1,
        ..
    } = fillet_with_side(&face[0], &face[1], edge[3].id(), None, Some(&face[2]), 0.3).unwrap();

    let shell: Shell = vec![face0, face1, fillet, side1.unwrap()].into();

    let poly = shell.robust_triangulation(0.001).to_polygon();
    let file = std::fs::File::create("fillet-with-edge.obj").unwrap();
    obj::write(&poly, file).unwrap();
}
