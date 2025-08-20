use super::*;

#[test]
fn test_organize_characters() {
    let v = Vertex::news(std::array::from_fn::<usize, 11, _>(|i| i));
    let edge = [
        Edge::new(&v[0], &v[1], 0),
        Edge::new(&v[1], &v[2], 1),
        Edge::new(&v[2], &v[0], 2),
        Edge::new(&v[0], &v[3], 3),
        Edge::new(&v[1], &v[5], 4),
        Edge::new(&v[2], &v[6], 5),
        Edge::new(&v[3], &v[4], 6),
        Edge::new(&v[4], &v[5], 7),
        Edge::new(&v[5], &v[6], 8),
        Edge::new(&v[6], &v[7], 9),
        Edge::new(&v[7], &v[3], 10),
        Edge::new(&v[4], &v[8], 11),
        Edge::new(&v[6], &v[9], 12),
        Edge::new(&v[7], &v[10], 13),
        Edge::new(&v[10], &v[8], 14),
        Edge::new(&v[8], &v[9], 15),
        Edge::new(&v[9], &v[10], 16),
    ];
    let shell = shell![
        Face::new(
            vec![wire![
                edge[3].clone(),
                edge[6].clone(),
                edge[7].clone(),
                edge[4].inverse(),
                edge[0].inverse()
            ]],
            0,
        ),
        Face::new(
            vec![wire![
                edge[4].clone(),
                edge[8].clone(),
                edge[5].inverse(),
                edge[1].inverse(),
            ]],
            1
        ),
        Face::new(
            vec![wire![
                edge[5].clone(),
                edge[9].clone(),
                edge[10].clone(),
                edge[3].inverse(),
                edge[2].inverse(),
            ]],
            2
        ),
        Face::new(
            vec![wire![
                edge[13].clone(),
                edge[14].clone(),
                edge[11].inverse(),
                edge[6].inverse(),
                edge[10].inverse(),
            ]],
            3
        ),
        Face::new(
            vec![wire![
                edge[11].clone(),
                edge[15].clone(),
                edge[12].inverse(),
                edge[8].inverse(),
                edge[7].inverse(),
            ]],
            4
        ),
        Face::new(
            vec![wire![
                edge[12].clone(),
                edge[16].clone(),
                edge[13].inverse(),
                edge[9].inverse(),
            ]],
            5
        ),
    ];
    let wire0 = wire![
        edge[6].clone(),
        edge[7].clone(),
        edge[8].clone(),
        edge[9].clone(),
        edge[10].clone(),
    ];
    let CharactersOrganization { ectvs, fctes } = organize_characters(&shell, &wire0).unwrap();

    let ans_ectvs = vec![
        Ectv {
            side0: Some(edge[3].clone()),
            side1: None,
        },
        Ectv {
            side0: None,
            side1: Some(edge[11].clone()),
        },
        Ectv {
            side0: Some(edge[4].clone()),
            side1: None,
        },
        Ectv {
            side0: Some(edge[5].clone()),
            side1: Some(edge[12].clone()),
        },
        Ectv {
            side0: None,
            side1: Some(edge[13].clone()),
        },
    ];
    assert_eq!(ectvs, ans_ectvs);

    let ans_fctes = vec![
        Fcte {
            side0: &shell[0],
            side1: &shell[3],
        },
        Fcte {
            side0: &shell[0],
            side1: &shell[4],
        },
        Fcte {
            side0: &shell[1],
            side1: &shell[4],
        },
        Fcte {
            side0: &shell[2],
            side1: &shell[5],
        },
        Fcte {
            side0: &shell[2],
            side1: &shell[3],
        },
    ];
    assert_eq!(fctes, ans_fctes);

    let wire1 = wire![edge[9].inverse(), edge[8].inverse(), edge[7].inverse(),];
    let CharactersOrganization { ectvs, fctes } = organize_characters(&shell, &wire1).unwrap();

    let ans_ectvs = vec![
        Ectv {
            side0: Some(edge[13].inverse()),
            side1: Some(edge[10].clone()),
        },
        Ectv {
            side0: Some(edge[12].inverse()),
            side1: Some(edge[5].inverse()),
        },
        Ectv {
            side0: None,
            side1: Some(edge[4].inverse()),
        },
        Ectv {
            side0: Some(edge[11].inverse()),
            side1: Some(edge[6].inverse()),
        },
    ];
    assert_eq!(ectvs, ans_ectvs);

    let ans_fctes = vec![
        Fcte {
            side0: &shell[5],
            side1: &shell[2],
        },
        Fcte {
            side0: &shell[4],
            side1: &shell[1],
        },
        Fcte {
            side0: &shell[4],
            side1: &shell[0],
        },
    ];
    assert_eq!(fctes, ans_fctes);
}

use derive_more::From;
#[derive(
    Clone,
    Debug,
    ParametricCurve,
    BoundedCurve,
    Cut,
    SearchNearestParameterD1,
    SearchParameterD1,
    ParameterDivision1D,
    Invertible,
    From,
)]
enum Curve {
    Line(Line<Point3>),
    Nurbs(NurbsCurve<Vector4>),
    Parametric(PCurve<BSplineCurve<Point2>, Box<Surface>>),
    Intersection(IntersectionCurve<Box<Self>, Box<Surface>, Box<Surface>>),
}

impl ToSameGeometry<Curve> for Line<Point3> {
    fn to_same_geometry(&self) -> Curve { Curve::Line(*self) }
}

impl ToSameGeometry<Curve> for NurbsCurve<Vector4> {
    fn to_same_geometry(&self) -> Curve { Curve::Nurbs(self.clone()) }
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

#[derive(
    Clone,
    Debug,
    ParametricSurface3D,
    SearchParameterD2,
    SearchNearestParameterD2,
    ParameterDivision2D,
    From,
)]
enum Surface {
    Plane(Plane),
    Nurbs(NurbsSurface<Vector4>),
    Fillet(ApproxFilletSurface<Box<Self>, Box<Self>>),
    Processor(Processor<Box<Self>, Matrix4>),
}

impl ToSameGeometry<Surface> for Plane {
    fn to_same_geometry(&self) -> Surface { Surface::Plane(*self) }
}

impl ToSameGeometry<Surface> for ApproxFilletSurface<Surface, Surface> {
    fn to_same_geometry(&self) -> Surface { Surface::Fillet(self.clone().into()) }
}

impl Invertible for Surface {
    fn invert(&mut self) {
        match self {
            Self::Plane(plane) => plane.invert(),
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

#[test]
fn fillet_cube() {
    use truck_meshalgo::prelude::*;
    use truck_modeling::primitive;
    truck_topology::prelude!(Point3, Curve, Surface);

    #[derive(Clone, Copy, Debug)]
    struct Radius;
    impl RadiusFunction for Radius {
        fn der_n(&self, n: usize, t: f64) -> f64 {
            match n {
                0 => 0.2 + 0.2 * t * t,
                1 => 0.1,
                _ => 0.0,
            }
        }
    }

    let bbox = BoundingBox::from_iter([Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0)]);
    let mut shell: Shell = primitive::cuboid(bbox).into_boundaries().pop().unwrap();

    let face_idx0 = shell
        .face_iter()
        .position(|face| {
            let plane = face.surface();
            plane
                .search_parameter((0.5, 0.0, 0.5).into(), None, 10)
                .is_some()
        })
        .unwrap();
    let face_idx1 = shell
        .face_iter()
        .position(|face| {
            let plane = face.surface();
            plane
                .search_parameter((1.0, 0.5, 0.5).into(), None, 10)
                .is_some()
        })
        .unwrap();
    let fillet_corner = shell
        .edge_iter()
        .find(|edge| {
            let line = edge.curve();
            line.search_parameter((1.0, 0.0, 0.5).into(), None, 10)
                .is_some()
        })
        .unwrap();
    let side_idx0 = shell
        .face_iter()
        .position(|face| {
            let plane = face.surface();
            plane
                .search_parameter((0.5, 0.5, 0.0).into(), None, 10)
                .is_some()
        })
        .unwrap();
    let side_idx1 = shell
        .face_iter()
        .position(|face| {
            let plane = face.surface();
            plane
                .search_parameter((0.5, 0.5, 1.0).into(), None, 10)
                .is_some()
        })
        .unwrap();
    let FilletWithSide {
        simple_fillet:
            SimpleFillet {
                fillet,
                face0,
                face1,
            },
        side0,
        side1,
    } = fillet_with_side(
        &shell[face_idx0],
        &shell[face_idx1],
        fillet_corner.id(),
        Some(&shell[side_idx0]),
        Some(&shell[side_idx1]),
        Radius,
        0.001,
    )
    .unwrap();

    shell[face_idx0] = face0;
    shell[face_idx1] = face1;
    shell[side_idx0] = side0.unwrap();
    shell[side_idx1] = side1.unwrap();
    shell.push(fillet);

    let face_idx0 = shell
        .face_iter()
        .position(|face| {
            let plane = face.surface();
            matches!(plane, Surface::Plane(_))
                && plane
                    .search_parameter((1.0, 0.5, 0.5).into(), None, 10)
                    .is_some()
        })
        .unwrap();
    let face_idx1 = shell
        .face_iter()
        .position(|face| {
            let plane = face.surface();
            matches!(plane, Surface::Plane(_))
                && plane
                    .search_parameter((0.5, 1.0, 0.5).into(), None, 10)
                    .is_some()
        })
        .unwrap();
    let fillet_corner = shell
        .edge_iter()
        .find(|edge| {
            let line = edge.curve();
            line.search_parameter((1.0, 1.0, 0.5).into(), None, 10)
                .is_some()
        })
        .unwrap();
    let side_idx0 = shell
        .face_iter()
        .position(|face| {
            let plane = face.surface();
            matches!(plane, Surface::Plane(_))
                && plane
                    .search_parameter((0.5, 0.5, 0.0).into(), None, 10)
                    .is_some()
        })
        .unwrap();
    let side_idx1 = shell
        .face_iter()
        .position(|face| {
            let plane = face.surface();
            matches!(plane, Surface::Plane(_))
                && plane
                    .search_parameter((0.5, 0.5, 1.0).into(), None, 10)
                    .is_some()
        })
        .unwrap();
    let FilletWithSide {
        simple_fillet:
            SimpleFillet {
                fillet,
                face0,
                face1,
            },
        side0,
        side1,
    } = fillet_with_side(
        &shell[face_idx0],
        &shell[face_idx1],
        fillet_corner.id(),
        Some(&shell[side_idx0]),
        Some(&shell[side_idx1]),
        Radius,
        0.001,
    )
    .unwrap();

    shell[face_idx0] = face0;
    shell[face_idx1] = face1;
    shell[side_idx0] = side0.unwrap();
    shell[side_idx1] = side1.unwrap();
    shell.push(fillet);

    let poly = shell.triangulation(0.005).to_polygon();
    obj::write(&poly, std::fs::File::create("pre-fillet-cube.obj").unwrap()).unwrap();

    let roof = shell
        .face_iter()
        .position(|face| {
            let plane = face.surface();
            matches!(plane, Surface::Plane(_))
                && plane
                    .search_parameter((0.5, 0.5, 1.0).into(), None, 10)
                    .is_some()
        })
        .unwrap();
    let mut boundary = shell[roof].boundaries().pop().unwrap();
    let remove_edge_idx = boundary
        .edge_iter()
        .position(|edge| edge.front().point().x < 0.01 && edge.back().point().x < 0.01)
        .unwrap();
    boundary.rotate_left(remove_edge_idx + 1);
    boundary.pop_back();

    let CharactersOrganization { fctes, .. } = organize_characters(&shell, &boundary).unwrap();
    let fillet_shell = fillet_faces(&boundary, &fctes, 0.1, 0.001).unwrap();
    assert!(fillet_shell
        .edge_iter()
        .all(|edge| edge.is_geometric_consistent()));

    let poly = fillet_shell.triangulation(0.005).to_polygon();
    obj::write(&poly, std::fs::File::create("cube_fillet.obj").unwrap()).unwrap();
}
