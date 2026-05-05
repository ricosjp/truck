use truck_geometry::prelude::*;

#[derive(Clone)]
struct CurveLength;

impl ScalarFunctionD1 for CurveLength {
    fn der_n(&self, n: usize, t: f64) -> f64 {
        match n {
            0 => 1.0 + t + t * t,
            1 => 1.0 + 2.0 * t,
            2 => 2.0,
            _ => 0.0,
        }
    }
}

#[test]
fn normal_field_line_with_variable_length() {
    let line = Line(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
    let offset = Offset::new(line, NormalField::new(line, CurveLength));
    let t = 0.3;
    let length = 1.0 + t + t * t;

    assert_near!(offset.subs(t), Point2::new(t, length));
    assert_near!(offset.der(t), Vector2::new(1.0, 1.0 + 2.0 * t));
    assert_near!(offset.der2(t), Vector2::new(0.0, 2.0));

    let ders = offset.ders(3, t);
    assert_near!(ders[0], Point2::new(t, length).to_vec());
    assert_near!(ders[1], Vector2::new(1.0, 1.0 + 2.0 * t));
    assert_near!(ders[2], Vector2::new(0.0, 2.0));
    assert_near!(ders[3], Vector2::zero());
}

#[test]
fn offset_curve_search_parameter() {
    let line = Line(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
    let offset = Offset::new(line, NormalField::new(line, CurveLength));
    let t = 0.3;

    let point = offset.subs(t);
    assert_near!(offset.search_parameter(point, None, 10).unwrap(), t);
    assert_near!(offset.search_parameter(point, t + 0.1, 10).unwrap(), t);

    let projected = line.subs(t);
    assert!(offset.search_parameter(projected, None, 10).is_none());
}

#[test]
fn normal_field_unit_circle_with_fixed_length() {
    let circle = UnitCircle::<Point2>::new();
    let offset = Offset::new(circle, NormalField::new(circle, -1.0));

    for i in 0..=8 {
        let t = i as f64 / 8.0 * std::f64::consts::TAU;
        assert_near!(offset.subs(t), 2.0 * circle.subs(t));
        assert_near!(offset.der(t), 2.0 * circle.der(t));
        assert_near!(offset.der2(t), 2.0 * circle.der2(t));
        assert_near!(offset.der_n(3, t), 2.0 * circle.der_n(3, t));
    }
}

#[derive(Clone)]
struct SurfaceLength;

impl ScalarFunctionD2 for SurfaceLength {
    fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> f64 {
        match (m, n) {
            (0, 0) => u * u + u * v + v * v,
            (1, 0) => 2.0 * u + v,
            (0, 1) => u + 2.0 * v,
            (2, 0) | (1, 1) | (0, 2) => 2.0,
            _ => 0.0,
        }
    }
}

#[test]
fn normal_field_surface_ders_on_plane() {
    let field = NormalField::new(Plane::xy(), SurfaceLength);
    let (u, v) = (0.2, 0.3);
    let ders = field.ders(2, u, v);

    assert_near!(ders[0][0], Vector3::new(0.0, 0.0, u * u + u * v + v * v));
    assert_near!(ders[1][0], Vector3::new(0.0, 0.0, 2.0 * u + v));
    assert_near!(ders[0][1], Vector3::new(0.0, 0.0, u + 2.0 * v));
    assert_near!(ders[2][0], Vector3::new(0.0, 0.0, 2.0));
    assert_near!(ders[1][1], Vector3::new(0.0, 0.0, 2.0));
    assert_near!(ders[0][2], Vector3::new(0.0, 0.0, 2.0));
}

#[test]
fn offset_surface_search_parameter() {
    let plane = Plane::xy();
    let offset = Offset::new(plane, NormalField::new(plane, SurfaceLength));
    let uv = Vector2::new(0.2, 0.3);

    let point = offset.subs(uv.x, uv.y);
    let hint_none_uv = offset.search_parameter(point, None, 10).unwrap().into();
    assert_near!(hint_none_uv, uv);
    let hint_some_uv = offset
        .search_parameter(point, (uv.x + 0.1, uv.y + 0.1), 10)
        .unwrap()
        .into();
    assert_near!(hint_some_uv, uv);

    let projected = plane.subs(uv.x, uv.y);
    assert!(offset.search_parameter(projected, None, 10).is_none());
}
