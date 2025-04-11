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
    FilletSide(RbfContactCurve<Box<Curve>, Box<Surface>, Box<Surface>, f64>),
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

    let (face0, face1, fillet) = simple_fillet(&face0, &face1, shared_edge_id, 0.3).unwrap();

    let shell: Shell = [face0, face1, fillet].into();
    let pshell = shell.triangulation(0.005);
    assert!(pshell.iter().all(|face| face.surface().is_some()));
    let poly = pshell.to_polygon();
    let file = std::fs::File::create("fillet-shell.obj").unwrap();
    obj::write(&poly, file).unwrap();
}
