use crate::{errors::Error, *};
use std::f64::consts::PI;
use truck_base::newton::{self, CalcOutput};

pub fn circle_arc_by_three_points(
    point0: Point2,
    point1: Point2,
    transit: Point2,
) -> Processor<TrimmedCurve<UnitCircle<Point2>>, Matrix3> {
    let origin = circum_center(point0, point1, transit);
    let Rad(circum_angle) = (point1 - transit).angle(point0 - transit);
    let direction = circum_angle.signum();
    let angle = 2.0 * (PI - circum_angle.abs());
    circle_arc(point0, origin, angle, direction)
}

pub fn circle_arc_by_tangent0(
    point0: Point2,
    point1: Point2,
    tangent0: Vector2,
) -> Processor<TrimmedCurve<UnitCircle<Point2>>, Matrix3> {
    let chord = point1 - point0;
    let tangent0 = tangent0.normalize();
    let to_origin = Vector2::new(-tangent0.y, tangent0.x);
    let denom = 2.0 * chord.dot(to_origin);
    assert!(
        !denom.so_small(),
        "cannot construct a circle arc when the tangent is parallel to the chord"
    );
    let radius = chord.magnitude2() / denom;
    let origin = point0 + radius * to_origin;
    let Rad(tc_angle) = tangent0.angle(chord);
    circle_arc(point0, origin, 2.0 * tc_angle.abs(), tc_angle.signum())
}

fn circum_center(point0: Point2, point1: Point2, point2: Point2) -> Point2 {
    let vec0 = point1 - point0;
    let vec1 = point2 - point0;
    let det = vec0.perp_dot(vec1);
    assert!(
        !det.so_small(),
        "cannot construct a circle arc from collinear points"
    );
    let a2 = vec0.magnitude2();
    let b2 = vec1.magnitude2();
    let u = Vector2::new(
        (vec1.y * a2 - vec0.y * b2) / (2.0 * det),
        (vec0.x * b2 - vec1.x * a2) / (2.0 * det),
    );
    point0 + u
}

fn circle_arc(
    point: Point2,
    origin: Point2,
    angle: f64,
    direction: f64,
) -> Processor<TrimmedCurve<UnitCircle<Point2>>, Matrix3> {
    let x_axis = point - origin;
    let y_axis = direction * Vector2::new(-x_axis.y, x_axis.x);
    let transform = Matrix3::from_cols(
        x_axis.extend(0.0),
        y_axis.extend(0.0),
        Vector3::new(origin.x, origin.y, 1.0),
    );
    let unit_arc = TrimmedCurve::new(UnitCircle::new(), (0.0, angle));
    Processor::with_transform(unit_arc, transform)
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct FilletCandidate {
    pub(crate) center: Point2,
    pub(crate) parameter0: f64,
    pub(crate) parameter1: f64,
}

pub fn fillet_candidate<C>(
    curve0: C,
    curve1: C,
    t0: f64,
    t1: f64,
    radius: f64,
) -> Result<FilletCandidate, Error>
where
    C: ParametricCurve2D,
{
    if radius <= 0.0 {
        return Err(Error::NonPositiveFilletRadius);
    }
    let der0 = curve0.der(t0);
    let der1 = curve1.der(t1);
    if der0.so_small() || der1.so_small() {
        return Err(Error::DegenerateTangent);
    }
    let point = curve0.subs(t0).midpoint(curve1.subs(t1));
    let seed_direction = match (der1 - der0).so_small() {
        true => Vector2::new(-der0.y, der0.x).normalize(),
        false => (der1 - der0).normalize(),
    };
    let hint = Vector4::new(
        point.x + radius * seed_direction.x,
        point.y + radius * seed_direction.y,
        t0,
        t1,
    );
    let function = |Vector4 {
                        x: ox,
                        y: oy,
                        z: parameter0,
                        w: parameter1,
                    }: Vector4| {
        let center = Point2::new(ox, oy);

        let point0 = curve0.subs(parameter0);
        let der0 = curve0.der(parameter0);
        let der20 = curve0.der2(parameter0);
        let diff0 = center - point0;
        let perp0 = diff0.dot(der0);
        let rad0 = diff0.magnitude2() - radius * radius;

        let point1 = curve1.subs(parameter1);
        let der1 = curve1.der(parameter1);
        let der21 = curve1.der2(parameter1);
        let diff1 = center - point1;
        let perp1 = diff1.dot(der1);
        let rad1 = diff1.magnitude2() - radius * radius;

        CalcOutput {
            value: Vector4::new(perp0, rad0, perp1, rad1),
            derivation: Matrix4::from_cols(
                Vector4::new(der0.x, 2.0 * diff0.x, der1.x, 2.0 * diff1.x),
                Vector4::new(der0.y, 2.0 * diff0.y, der1.y, 2.0 * diff1.y),
                Vector4::new(
                    -der0.magnitude2() + diff0.dot(der20),
                    -2.0 * diff0.dot(der0),
                    0.0,
                    0.0,
                ),
                Vector4::new(
                    0.0,
                    0.0,
                    -der1.magnitude2() + diff1.dot(der21),
                    -2.0 * diff1.dot(der1),
                ),
            ),
        }
    };
    let solution = newton::solve(function, hint, 100).map_err(|log| match log.degenerate() {
        true => Error::DegenerateFilletJacobian(log.to_string()),
        false => Error::FilletNewtonNotConverged(log.to_string()),
    })?;
    Ok(FilletCandidate {
        center: Point2::new(solution.x, solution.y),
        parameter0: solution.z,
        parameter1: solution.w,
    })
}

pub fn parameter_at_curve_length<C>(
    curve: &C,
    origin: f64,
    signed_length: f64,
) -> Result<f64, Error>
where
    C: ParametricCurve2D + BoundedCurve,
{
    if signed_length.so_small() {
        return Ok(origin);
    }
    let (front, back) = curve.range_tuple();
    if origin < front - TOLERANCE || origin > back + TOLERANCE {
        return Err(Error::CurveLengthOutOfRange);
    }

    let direction = signed_length.signum();
    let length = signed_length.abs();
    let steps = usize::max(8, (length / 0.01).ceil() as usize);
    let ds = length / steps as f64;
    let mut parameter = origin;
    for _ in 0..steps {
        let k1 = curve_length_parameter_derivative(curve, parameter, direction)?;
        let k2 = curve_length_parameter_derivative(curve, parameter + 0.5 * ds * k1, direction)?;
        let k3 = curve_length_parameter_derivative(curve, parameter + 0.5 * ds * k2, direction)?;
        let k4 = curve_length_parameter_derivative(curve, parameter + ds * k3, direction)?;
        parameter += ds * (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0;
        if parameter < front - TOLERANCE || parameter > back + TOLERANCE {
            return Err(Error::CurveLengthOutOfRange);
        }
    }
    Ok(f64::clamp(parameter, front, back))
}

fn curve_length_parameter_derivative<C>(
    curve: &C,
    parameter: f64,
    direction: f64,
) -> Result<f64, Error>
where
    C: ParametricCurve2D + BoundedCurve,
{
    let (front, back) = curve.range_tuple();
    if parameter < front - TOLERANCE || parameter > back + TOLERANCE {
        return Err(Error::CurveLengthOutOfRange);
    }
    let der = curve.der(f64::clamp(parameter, front, back));
    match der.so_small() {
        true => Err(Error::DegenerateTangent),
        false => Ok(direction / der.magnitude()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::{prelude::*, property_test};

    #[property_test]
    fn test_circle_arc(
        #[strategy = prop::array::uniform2(-10.0f64..=10.0f64)] origin: [f64; 2],
        #[strategy = prop::array::uniform2(-10.0f64..=10.0f64)] direction0: [f64; 2],
        #[strategy = prop::array::uniform2(-10.0f64..=10.0f64)] direction1: [f64; 2],
        #[strategy = 0.5f64..=10.0f64] radius: f64,
        #[strategy = 0.05f64..=0.95f64] sample_ratio: f64,
    ) {
        let origin = Point2::from(origin);
        let direction0 = Vector2::from(direction0);
        let direction1 = Vector2::from(direction1);
        prop_assume!(!direction0.so_small());
        prop_assume!(!direction1.so_small());

        let point0 = origin + radius * direction0.normalize();
        let point1 = origin + radius * direction1.normalize();
        let Rad(sweep_angle) = (point0 - origin).angle(point1 - origin);

        let reference = origin + Matrix2::from_angle(Rad(0.5 * sweep_angle)) * (point0 - origin);
        let curve = circle_arc(point0, origin, sweep_angle.abs(), sweep_angle.signum());
        let (t0, t1) = curve.range_tuple();
        let sample = curve.subs(t0 + sample_ratio * (t1 - t0));

        prop_assert_near!(curve.subs(t0), point0);
        prop_assert_near!(curve.subs(t1), point1);

        let angle0 = (reference - point0).angle(reference - point1);
        let angle1 = (sample - point0).angle(sample - point1);
        prop_assert_near!(angle0, angle1);
    }

    #[property_test]
    fn test_circle_arc_by_three_points(
        #[strategy = prop::array::uniform2(-10.0f64..=10.0f64)] origin: [f64; 2],
        #[strategy = prop::array::uniform2(-10.0f64..=10.0f64)] direction0: [f64; 2],
        #[strategy = prop::array::uniform2(-10.0f64..=10.0f64)] direction1: [f64; 2],
        #[strategy = prop::array::uniform2(-10.0f64..=10.0f64)] transit_direction: [f64; 2],
        #[strategy = 0.5f64..=10.0f64] radius: f64,
        #[strategy = 0.05f64..=0.95f64] sample_ratio: f64,
    ) {
        let origin = Point2::from(origin);
        let direction0 = Vector2::from(direction0);
        let direction1 = Vector2::from(direction1);
        let transit_direction = Vector2::from(transit_direction);
        prop_assume!(!direction0.so_small());
        prop_assume!(!direction1.so_small());
        prop_assume!(!transit_direction.so_small());

        let point0 = origin + radius * direction0.normalize();
        let point1 = origin + radius * direction1.normalize();
        let transit = origin + radius * transit_direction.normalize();
        prop_assume!((point1 - point0).magnitude() > 0.05);
        prop_assume!((transit - point0).magnitude() > 0.05);
        prop_assume!((transit - point1).magnitude() > 0.05);
        prop_assume!((point1 - point0).perp_dot(transit - point0).abs() > 0.05);

        let curve = circle_arc_by_three_points(point0, point1, transit);
        let (t0, t1) = curve.range_tuple();
        let sample = curve.subs(t0 + sample_ratio * (t1 - t0));

        prop_assert_near!(curve.subs(t0), point0);
        prop_assert_near!(curve.subs(t1), point1);

        let angle0 = (transit - point0).angle(transit - point1);
        let angle1 = (sample - point0).angle(sample - point1);
        prop_assert_near!(angle0, angle1);
    }

    #[property_test]
    fn circle_arc_by_tangent0_has_no_excess_or_shortage(
        #[strategy = prop::array::uniform2(-10.0f64..=10.0f64)] point0: [f64; 2],
        #[strategy = prop::array::uniform2(-10.0f64..=10.0f64)] point1: [f64; 2],
        #[strategy = 0f64..2.0 * PI] tangent_angle: f64,
        #[strategy = 0.05f64..=0.95f64] sample_ratio: f64,
    ) {
        let point0 = Point2::from(point0);
        let point1 = Point2::from(point1);
        let tangent0 = Vector2::new(tangent_angle.cos(), tangent_angle.sin());
        prop_assume!(!(point0 - point1).perp_dot(tangent0).so_small());

        let curve = circle_arc_by_tangent0(point0, point1, tangent0);
        let (t0, t1) = curve.range_tuple();
        let sample = curve.subs(t0 + sample_ratio * (t1 - t0));

        prop_assert_near!(curve.subs(t0), point0);
        prop_assert_near!(curve.subs(t1), point1);
        prop_assert_near!(curve.der(t0).normalize(), tangent0);

        let angle0 = tangent0.angle(sample - point0);
        let angle1 = (point0 - point1).angle(sample - point1);
        assert_near!(angle0, angle1);
    }

    #[test]
    fn test_circle_arc_by_three_points_specific() {
        let curve = circle_arc_by_three_points(
            Point2::new(1.0, 0.0),
            Point2::new(-1.0, 0.0),
            Point2::new(0.0, 1.0),
        );
        let (t0, t1) = curve.range_tuple();
        assert_near!(curve.subs(t0), Point2::new(1.0, 0.0));
        assert_near!(curve.subs((t0 + t1) * 0.5), Point2::new(0.0, 1.0));
        assert_near!(curve.subs(t1), Point2::new(-1.0, 0.0));
    }

    #[test]
    fn test_circle_arc_tangent0_specific() {
        let tangent = Vector2::new(0.0, 1.0);
        let curve = circle_arc_by_tangent0(Point2::new(1.0, 0.0), Point2::new(0.0, 1.0), tangent);
        let (t0, t1) = curve.range_tuple();
        assert_near!(curve.subs(t0), Point2::new(1.0, 0.0));
        assert_near!(curve.subs(t1), Point2::new(0.0, 1.0));
        assert_near!(curve.der(t0).normalize(), tangent.normalize());
    }

    #[test]
    fn fillet_candidate_for_two_lines() {
        let curve0 = Line(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
        let curve1 = Line(Point2::new(1.0, 0.0), Point2::new(1.0, 1.0));
        let candidate = fillet_candidate(curve0, curve1, 1.0, 0.0, 0.2).unwrap();
        assert_near!(candidate.center, Point2::new(0.8, 0.2));
        assert_near2!(candidate.parameter0, 0.8);
        assert_near2!(candidate.parameter1, 0.2);
    }

    #[test]
    fn fillet_candidate_for_two_quadratic_bspline_curves() {
        let curve0 = BSplineCurve::new(
            KnotVec::bezier_knot(2),
            vec![
                Point2::new(0.0, 0.0),
                Point2::new(0.45, -0.25),
                Point2::new(1.0, 0.0),
            ],
        );
        let curve1 = BSplineCurve::new(
            KnotVec::bezier_knot(2),
            vec![
                Point2::new(1.0, 0.0),
                Point2::new(1.25, 0.55),
                Point2::new(0.95, 1.2),
            ],
        );
        let radius = 0.12;
        let candidate = fillet_candidate(curve0.clone(), curve1.clone(), 1.0, 0.0, radius).unwrap();

        assert!(0.0 < candidate.parameter0 && candidate.parameter0 < 1.0);
        assert!(0.0 < candidate.parameter1 && candidate.parameter1 < 1.0);
        assert!(!curve0.der2(candidate.parameter0).so_small());
        assert!(!curve1.der2(candidate.parameter1).so_small());

        let contact0 = curve0.subs(candidate.parameter0);
        let contact1 = curve1.subs(candidate.parameter1);
        let radius0 = candidate.center - contact0;
        let radius1 = candidate.center - contact1;
        assert_near2!(radius0.magnitude(), radius);
        assert_near2!(radius1.magnitude(), radius);
        assert_near2!(radius0.dot(curve0.der(candidate.parameter0)), 0.0);
        assert_near2!(radius1.dot(curve1.der(candidate.parameter1)), 0.0);
    }

    #[test]
    fn parameter_at_curve_length_for_line() {
        let curve = Line(Point2::new(0.0, 0.0), Point2::new(2.0, 0.0));
        assert_near2!(parameter_at_curve_length(&curve, 1.0, -0.5).unwrap(), 0.75);
        assert_near2!(parameter_at_curve_length(&curve, 0.0, 0.5).unwrap(), 0.25);
    }

    #[test]
    fn parameter_at_curve_length_for_quadratic_bezier() {
        let curve = BSplineCurve::new(
            KnotVec::bezier_knot(2),
            vec![
                Point2::new(0.0, 0.0),
                Point2::new(0.5, 0.0),
                Point2::new(1.0, 1.0),
            ],
        );
        let distance = 0.35;
        let parameter = parameter_at_curve_length(&curve, 0.0, distance).unwrap();
        assert!(0.0 < parameter && parameter < 1.0);
        assert_near!(quadratic_bezier_arc_length(parameter), distance);
    }

    fn quadratic_bezier_arc_length(parameter: f64) -> f64 {
        let root = (1.0 + 4.0 * parameter * parameter).sqrt();
        0.25 * (2.0 * parameter * root + (2.0 * parameter + root).ln())
    }
}
