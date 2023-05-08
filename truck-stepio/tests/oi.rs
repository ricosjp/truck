use ruststep::{
    ast::DataSection,
    tables::{EntityTable, Holder},
};
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};
use truck_geometry::prelude as truck;
use truck::*;
use truck_polymesh::PolylineCurve;
use truck_stepio::{out::*, r#in::*};

fn oitest<Truck, StepHolder>(t: Truck)
where
    StepHolder: Holder<Table = Table>,
    Truck: for<'a> From<&'a StepHolder::Owned> + Debug + PartialEq,
    for<'a> StepDisplay<&'a Truck>: Display,
    Table: EntityTable<StepHolder>, {
    let step_display = StepDisplay::new(&t, 1);
    let step = format!("DATA;\n{step_display}ENDSEC;");
    println!("{step}");
    itest::<Truck, StepHolder>(t, &step);
}

fn oitest_tryfrom<Truck, StepHolder>(t: Truck)
where
    StepHolder: Holder<Table = Table>,
    Truck: for<'a> TryFrom<&'a StepHolder::Owned> + Debug + PartialEq,
    for<'a> <Truck as TryFrom<&'a StepHolder::Owned>>::Error: Debug,
    for<'a> StepDisplay<&'a Truck>: Display,
    Table: EntityTable<StepHolder>, {
    let step_display = StepDisplay::new(&t, 1);
    let step = format!("DATA;\n{step_display}ENDSEC;");
    println!("{step}");
    itest_tryfrom::<Truck, StepHolder>(t, &step);
}

fn itest<Truck, StepHolder>(t: Truck, step: &str)
where
    Truck: for<'a> From<&'a StepHolder::Owned> + Debug + PartialEq,
    StepHolder: Holder<Table = Table>,
    Table: EntityTable<StepHolder>, {
    let data_section = DataSection::from_str(step).unwrap();
    let table = Table::from_data_section(&data_section);
    let step_data: StepHolder::Owned = EntityTable::get_owned(&table, 1).unwrap();
    let got = Truck::from(&step_data);
    assert_eq!(t, got);
}

fn itest_tryfrom<Truck, StepHolder>(t: Truck, step: &str)
where
    Truck: for<'a> TryFrom<&'a StepHolder::Owned> + Debug + PartialEq,
    for<'a> <Truck as TryFrom<&'a StepHolder::Owned>>::Error: Debug,
    StepHolder: Holder<Table = Table>,
    Table: EntityTable<StepHolder>, {
    let data_section = DataSection::from_str(step).unwrap();
    let table = Table::from_data_section(&data_section);
    let step_data: StepHolder::Owned = EntityTable::get_owned(&table, 1).unwrap();
    let got = Truck::try_from(&step_data).unwrap();
    assert_eq!(t, got);
}

#[test]
fn oi() {
    oitest::<Point2, CartesianPointHolder>(Point2::new(1.0, 2.0));
    oitest::<Point3, CartesianPointHolder>(Point3::new(1.0, 2.0, 3.0));
    oitest::<Vector2, VectorHolder>(Vector2::new(1.0, 2.0));
    oitest::<Vector3, VectorHolder>(Vector3::new(1.0, 2.0, 3.0));
    oitest::<truck::Line<Point2>, LineHolder>(truck::Line(
        Point2::new(0.0, 1.0),
        Point2::new(2.0, 3.0),
    ));
    oitest::<PolylineCurve<Point2>, PolylineHolder>(PolylineCurve(vec![
        Point2::origin(),
        Point2::new(1.0, 2.0),
        Point2::new(3.0, 4.0),
    ]));
    oitest_tryfrom::<BSplineCurve<Point2>, BSplineCurveWithKnotsHolder>(BSplineCurve::new(
        KnotVec::uniform_knot(3, 4),
        vec![
            Point2::new(1.0, 2.0),
            Point2::new(3.0, 4.0),
            Point2::new(5.0, 6.0),
            Point2::new(7.0, 8.0),
            Point2::new(9.0, 10.0),
            Point2::new(11.0, 12.0),
            Point2::new(13.0, 14.0),
            Point2::new(15.0, 16.0),
        ],
    ));
    itest_tryfrom::<BSplineCurve<Point2>, BezierCurveHolder>(
        BSplineCurve::new(
            KnotVec::bezier_knot(2),
            vec![
                Point2::new(0.0, 1.0),
                Point2::new(2.0, 3.0),
                Point2::new(4.0, 5.0),
            ],
        ),
        "DATA; #1 = BEZIER_CURVE('', 2, (#2, #3, #4), .UNSPECIFIED., .U., .U.);
#2 = CARTESIAN_POINT('', (0.0, 1.0)); #3 = CARTESIAN_POINT('', (2.0, 3.0));
#4 = CARTESIAN_POINT('', (4.0, 5.0)); ENDSEC;",
    );
    itest_tryfrom::<BSplineCurve<Point2>, QuasiUniformCurveHolder>(
        BSplineCurve::new(
            KnotVec::from(vec![0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 2.0]),
            vec![
                Point2::new(0.0, 1.0),
                Point2::new(2.0, 3.0),
                Point2::new(4.0, 5.0),
                Point2::new(6.0, 7.0),
            ],
        ),
        "DATA; #1 = QUASI_UNIFORM_CURVE('', 2, (#2, #3, #4, #5), .UNSPECIFIED., .U., .U.);
#2 = CARTESIAN_POINT('', (0.0, 1.0)); #3 = CARTESIAN_POINT('', (2.0, 3.0));
#4 = CARTESIAN_POINT('', (4.0, 5.0)); #5 = CARTESIAN_POINT('', (6.0, 7.0)); ENDSEC;",
    );
    oitest_tryfrom::<NurbsCurve<Vector3>, RationalBSplineCurveHolder>(NurbsCurve::new(
        BSplineCurve::new(
            KnotVec::bezier_knot(3),
            vec![
                Vector3::new(0.0, 1.0, 2.0),
                Vector3::new(3.0, 4.0, 5.0),
                Vector3::new(6.0, 7.0, 8.0),
            ],
        ),
    ));
    oitest::<truck::Plane, PlaneHolder>(truck::Plane::new(
        Point3::new(1.0, 2.0, 3.0),
        // The ISO regulations require that the coordinate axes of the Plane must be vertical;
        // on the truck side, the axes do not have to be vertical,
        // so the results will not necessarily match.
        Point3::new(2.0, 2.0, 3.0),
        Point3::new(1.0, 3.0, 3.0),
    ));
    oitest_tryfrom::<BSplineSurface<Point3>, BSplineSurfaceWithKnotsHolder>(BSplineSurface::new(
        (KnotVec::bezier_knot(3), KnotVec::bezier_knot(2)),
        vec![
            vec![
                Point3::new(0.0, 1.0, 2.0),
                Point3::new(3.0, 4.0, 5.0),
                Point3::new(6.0, 7.0, 8.0),
            ],
            vec![
                Point3::new(0.0, 1.0, 2.0),
                Point3::new(3.0, 4.0, 5.0),
                Point3::new(6.0, 7.0, 8.0),
            ],
            vec![
                Point3::new(0.0, 1.0, 2.0),
                Point3::new(3.0, 4.0, 5.0),
                Point3::new(6.0, 7.0, 8.0),
            ],
            vec![
                Point3::new(0.0, 1.0, 2.0),
                Point3::new(3.0, 4.0, 5.0),
                Point3::new(6.0, 7.0, 8.0),
            ],
        ],
    ));
}
