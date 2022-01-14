use truck_geometry::*;
use truck_stepio::out::*;

#[test]
fn geometry() {
    let x = Point2::new(0.0, 1.0);
    assert_eq!(
        &StepDisplay::new(x, 1).to_string(),
        "#1 = CARTESIAN_POINT('', (0.0, 1.0));\n",
    );
    assert_eq!(x.step_length(), 1);

    let x = Point3::new(0.0, 1.0, 2.453);
    assert_eq!(
        &StepDisplay::new(x, 2).to_string(),
        "#2 = CARTESIAN_POINT('', (0.0, 1.0, 2.453));\n",
    );
    assert_eq!(x.step_length(), 1);

    let x = Vector2::new(3.0, 4.0);
    assert_eq!(
        &StepDisplay::new(x, 1).to_string(),
        "#1 = VECTOR('', #2, 5.0);\n#2 = DIRECTION('', (0.6, 0.8));\n",
    );
    assert_eq!(x.step_length(), 2);

    let x = Vector3::new(3.0, 4.0, 3.75);
    assert_eq!(
        &StepDisplay::new(x, 1).to_string(),
        "#1 = VECTOR('', #2, 6.25);\n#2 = DIRECTION('', (0.48, 0.64, 0.6));\n",
    );
    assert_eq!(x.step_length(), 2);

    let x = BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 1.0),
            Point2::new(2.0, 0.0),
        ],
    );
    assert_eq!(
		&StepDisplay::new(&x, 1).to_string(),
		"#1 = B_SPLINE_CURVE_WITH_KNOTS('', 2, (#2, #3, #4), .UNSPECIFIED., .UNKNOWN., .UNKNOWN., (3, 3), (0.0, 1.0), .UNSPECIFIED.);
#2 = CARTESIAN_POINT('', (0.0, 0.0));
#3 = CARTESIAN_POINT('', (1.0, 1.0));
#4 = CARTESIAN_POINT('', (2.0, 0.0));\n",
	);
    assert_eq!(x.step_length(), 4);

    let x = NURBSCurve::new(BSplineCurve::new(
        KnotVec::bezier_knot(2),
        vec![
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(1.0, 1.0, 2.0),
            Vector3::new(2.0, 0.0, 4.0),
        ],
    ));
    assert_eq!(
        &StepDisplay::new(&x, 1).to_string(),
        "#1 = (
    BOUNDED_CURVE()
    B_SPLINE_CURVE(2, (#2, #3, #4), .UNSPECIFIED., .UNKNOWN., .UNKNOWN.)
    B_SPLINE_CURVE_WITH_KNOTS((3, 3), (0.0, 1.0), .UNSPECIFIED.)
    CURVE()
    GEOMETRIC_REPRESENTATION_ITEM()
    RATIONAL_B_SPLINE_CURVE((1.0, 2.0, 4.0))
    REPRESENTATION_ITEM('')
);
#2 = CARTESIAN_POINT('', (0.0, 0.0));
#3 = CARTESIAN_POINT('', (0.5, 0.5));
#4 = CARTESIAN_POINT('', (0.5, 0.0));\n",
    );
    assert_eq!(x.step_length(), 4);

    let x = Plane::new(
        Point3::new(1.0, 2.0, 3.0),
        Point3::new(1.0, 2.0, 4.0),
        Point3::new(2.0, 2.0, 3.0),
    );
    assert_eq!(
        &StepDisplay::new(x, 1).to_string(),
        "#1 = PLANE('', #2);
#2 = AXIS2_PLACEMENT_3D('', #3, #4, #5);
#3 = CARTESIAN_POINT('', (1.0, 2.0, 3.0));
#4 = DIRECTION('', (0.0, 1.0, 0.0));
#5 = DIRECTION('', (0.0, 0.0, 1.0));\n"
    );
    assert_eq!(x.step_length(), 5);
}
