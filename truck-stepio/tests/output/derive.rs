use truck_geometry::prelude::*;
use truck_stepio::out::*;

#[derive(Clone, Debug, StepLength, DisplayByStep)]
enum LengthCheck {
    Point(Point3),
    Vector(Vector3),
}

#[test]
fn derive_out() {
    let p = Point3::new(0.0, 1.0, 2.0);
    let check = LengthCheck::Point(p);
    assert_eq!(p.step_length(), check.step_length());
    assert_eq!(
        StepDataDisplay::new(p, 0).to_string(),
        StepDataDisplay::new(check, 0).to_string(),
    );

    let v = Vector3::new(4.0, -1.0, 2.0);
    let check = LengthCheck::Vector(v);
    assert_eq!(v.step_length(), check.step_length());
    assert_eq!(
        StepDataDisplay::new(v, 0).to_string(),
        StepDataDisplay::new(check, 0).to_string(),
    );
}
