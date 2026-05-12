use crate::{errors::Error, *};
use std::{f64::consts::PI, ops::RangeBounds};
use truck_base::newton::{self, CalcOutput};

type CircleArc = Processor<TrimmedCurve<UnitCircle<Point2>>, Matrix3>;

pub fn circle_arc_by_three_points(
    point0: Point2,
    point1: Point2,
    transit: Point2,
) -> Result<CircleArc, Error> {
    let origin = circum_center(point0, point1, transit)?;
    let Rad(circum_angle) = (point1 - transit).angle(point0 - transit);
    let direction = circum_angle.signum();
    let angle = 2.0 * (PI - circum_angle.abs());
    Ok(circle_arc(point0, origin, angle, direction))
}

pub fn circle_arc_by_tangent0(
    point0: Point2,
    point1: Point2,
    tangent0: Vector2,
) -> Result<CircleArc, Error> {
    let chord = point1 - point0;
    if tangent0.so_small() {
        return Err(Error::DegenerateTangent);
    }
    let to_origin = rot_4(tangent0.normalize());
    let denom = 2.0 * chord.dot(to_origin);
    if denom.so_small() {
        return Err(Error::ParallelArcTangent);
    }
    let radius = chord.magnitude2() / denom;
    let origin = point0 + radius * to_origin;
    let Rad(tc_angle) = tangent0.angle(chord);
    Ok(circle_arc(
        point0,
        origin,
        2.0 * tc_angle.abs(),
        tc_angle.signum(),
    ))
}

fn circum_center(point0: Point2, point1: Point2, point2: Point2) -> Result<Point2, Error> {
    let vec0 = point1 - point0;
    let vec1 = point2 - point0;
    let det = vec0.perp_dot(vec1);
    if det.so_small() {
        return Err(Error::CollinearArcPoints);
    }
    let a2 = vec0.magnitude2();
    let b2 = vec1.magnitude2();
    let u = Vector2::new(
        (vec1.y * a2 - vec0.y * b2) / (2.0 * det),
        (vec0.x * b2 - vec1.x * a2) / (2.0 * det),
    );
    Ok(point0 + u)
}

fn circle_arc(point: Point2, origin: Point2, angle: f64, direction: f64) -> CircleArc {
    let x_axis = point - origin;
    let y_axis = direction * rot_4(x_axis);
    let transform = Matrix3::from_cols(
        x_axis.extend(0.0),
        y_axis.extend(0.0),
        Vector3::new(origin.x, origin.y, 1.0),
    );
    let unit_arc = TrimmedCurve::new(UnitCircle::new(), (0.0, angle));
    Processor::with_transform(unit_arc, transform)
}

pub fn lines_crossing_point(
    point0: Point2,
    point1: Point2,
    direction0: Vector2,
    direction1: Vector2,
) -> Result<Point2, Error> {
    let matrix = Matrix2::from_cols(direction0, -direction1);
    if matrix.determinant().so_small() {
        panic!("directions are parallel");
    }
    let params = matrix.invert().unwrap() * (point1 - point0);
    Ok((point0 + params.x * direction0).midpoint(point1 + params.y * direction1))
}

pub fn arc_arc_transit(
    point0: Point2,
    point1: Point2,
    tangent0: Vector2,
    radius0: f64,
    tangent1: Vector2,
) -> Result<Point2, Error> {
    if radius0 <= 0.0 {
        return Err(Error::NonPositiveRadius);
    }
    if tangent0.so_small() || tangent1.so_small() {
        return Err(Error::DegenerateTangent);
    }

    let normal0 = rot_4(tangent0.normalize());
    let normal1 = rot_4(tangent1.normalize());
    let delta = point1 - point0;

    let delta2 = delta.magnitude2();
    let n0del = normal0.dot(delta);
    let n1del = normal1.dot(delta);
    let n01 = normal0.dot(normal1);
    let signs = [-1.0, 1.0];
    itertools::iproduct!(signs, signs, signs)
        .filter_map(|(s0, s1, s2)| {
            let numerator = delta2 / 2.0 - s0 * radius0 * n0del;
            let denominator = s2 * radius0 - s1 * n1del + s0 * s1 * radius0 * n01;
            if numerator * denominator < TOLERANCE {
                return None;
            }
            let radius1 = numerator / denominator;

            let center0 = point0 + s0 * radius0 * normal0;
            let center1 = point1 + s1 * radius1 * normal1;

            if center0.near(&center1) {
                return by_one_arc(point0, point1, tangent0, radius0);
            }

            let k = radius0 / (radius0 + s2 * radius1);
            let transit = center0 + k * (center1 - center0);

            let transit_tangent0 = s0 * rot_4(transit - center0);
            let transit_tangent1 = s1 * rot_4(transit - center1);

            if transit_tangent0.dot(transit_tangent1) < 0.0 {
                return None;
            }

            let angle0 = 2.0 * tangent0.angle(transit - point0).0.abs();
            let angle1 = 2.0 * (-tangent1).angle(transit - point1).0.abs();

            if (radius0 * angle0).so_small() || (radius1 * angle1).so_small() {
                return by_one_arc(point0, point1, tangent0, radius0);
            }

            Some((transit, radius0 * angle0 + radius1 * angle1))
        })
        .min_by(|(_, l0), (_, l1)| l0.partial_cmp(&l1).unwrap())
        .map(|(point, _)| point)
        .ok_or_else(|| panic!("there is no circle connection"))
}

fn by_one_arc(
    point0: Point2,
    point1: Point2,
    tangent0: Vector2,
    radius0: f64,
) -> Option<(Point2, f64)> {
    let arc = circle_arc_by_tangent0(point0, point1, tangent0).ok()?;
    let (t0, t1) = arc.range_tuple();
    let transit = arc.subs((t0 + t1) / 2.0);
    Some((transit, radius0 * (t1 - t0)))
}

pub fn line_arc_line_transit(
    point0: Point2,
    point1: Point2,
    direction0: Vector2,
    radius: f64,
    direction1: Vector2,
) -> Result<(Point2, Point2), Error> {
    if radius <= 0.0 {
        return Err(Error::NonPositiveRadius);
    }
    if direction0.so_small() || direction1.so_small() {
        return Err(Error::DegenerateTangent);
    }
    let crossing = lines_crossing_point(point0, point1, direction0, direction1)?;
    if point0.near(&crossing) || point1.near(&crossing) {
        panic!("crossing point is near vertices");
    }
    let direction0 = (point0 - crossing).normalize();
    let direction1 = (point1 - crossing).normalize();
    let cos = direction0.dot(direction1);
    let tan_2 = ((1.0 - cos) / (1.0 + cos)).sqrt();
    let length = radius / tan_2;
    Ok((
        crossing + length * direction0,
        crossing + length * direction1,
    ))
}

pub fn arc_line_arc_transit(
    point0: Point2,
    point1: Point2,
    tangent0: Vector2,
    tangent1: Vector2,
    radius0: f64,
    radius1: f64,
) -> Result<(Point2, Point2), Error> {
    if radius0 <= 0.0 || radius1 <= 0.0 {
        return Err(Error::NonPositiveRadius);
    }
    if tangent0.so_small() || tangent1.so_small() {
        return Err(Error::DegenerateTangent);
    }

    let normal0 = rot_4(tangent0.normalize());
    let normal1 = rot_4(tangent1.normalize());

    let signs = [-1.0, 1.0];
    itertools::iproduct!(signs, signs, signs, signs)
        .filter_map(|(s0, s1, s2, s3)| {
            let center0 = point0 + s0 * radius0 * normal0;
            let center1 = point1 + s1 * radius1 * normal1;

            if center0.near(&center1) {
                return None;
            }

            let delta = center1 - center0;
            let delta_length = delta.magnitude();
            let radius_sum = radius0 + s2 * radius1;

            if radius_sum.abs() > delta_length + TOLERANCE {
                return None;
            }

            let x_axis = delta.normalize();
            let cos = f64::clamp(radius_sum / delta_length, -1.0, 1.0);
            let sin = s3 * (1.0 - cos * cos).sqrt();
            let rotation = Matrix2::new(cos, sin, -sin, cos);

            let transit0 = center0 + rotation * x_axis * radius0;
            let transit_tangent0 = s0 * rot_4(transit0 - center0);
            let transit1 = transit0
                + ((center1 - transit0).dot(transit_tangent0) / transit_tangent0.magnitude2())
                    * transit_tangent0;
            let transit_tangent1 = s1 * rot_4(transit1 - center1);

            if transit_tangent0.dot(transit1 - transit0) < 0.0
                || transit_tangent1.dot(transit1 - transit0) < 0.0
            {
                return None;
            }

            let angle0 = 2.0 * tangent0.angle(transit0 - point0).0.abs();
            let angle1 = 2.0 * (-tangent1).angle(transit1 - point1).0.abs();
            let length = radius0 * angle0 + (transit0 - transit1).magnitude() + radius1 * angle1;

            Some(((transit0, transit1), length))
        })
        .min_by(|(_, length0), (_, length1)| length0.partial_cmp(&length1).unwrap())
        .map(|(pair, _)| pair)
        .ok_or_else(|| panic!("there is no connection"))
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
    C: ParametricCurve2D + SearchNearestParameter<D1, Point = Point2>,
{
    if radius <= 0.0 {
        return Err(Error::NonPositiveRadius);
    }
    let der0 = curve0.der(t0);
    let der1 = curve1.der(t1);
    if der0.so_small() || der1.so_small() {
        return Err(Error::DegenerateTangent);
    }
    let point = curve0.subs(t0).midpoint(curve1.subs(t1));
    let (tan0, tan1) = (der0.normalize(), der1.normalize());
    let seed_direction = match tan0.near(&tan1) {
        true => return Err(Error::DegenerateCorner),
        false => (tan1 - tan0).normalize(),
    };

    let param_center = point + radius * seed_direction;
    let t0_hint = curve0
        .search_nearest_parameter(param_center, t0, 100)
        .filter(|t0| curve0.parameter_range().contains(t0))
        .unwrap_or(t0);
    let t1_hint = curve1
        .search_nearest_parameter(param_center, t1, 100)
        .filter(|t1| curve1.parameter_range().contains(t1))
        .unwrap_or(t1);

    let sgn = -curve0.der(t0_hint).perp_dot(curve1.der(t1_hint)).signum();
    let hint = Vector4::new(param_center.x, param_center.y, t0_hint, t1_hint);
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
        let dmag0 = der0.magnitude();
        let rad0 = sgn * der0.perp_dot(diff0) + dmag0 * radius;

        let point1 = curve1.subs(parameter1);
        let der1 = curve1.der(parameter1);
        let der21 = curve1.der2(parameter1);
        let diff1 = center - point1;
        let perp1 = diff1.dot(der1);
        let dmag1 = der1.magnitude();
        let rad1 = sgn * der1.perp_dot(diff1) + dmag1 * radius;

        CalcOutput {
            value: Vector4::new(perp0, rad0, perp1, rad1),
            derivation: Matrix4::from_cols(
                Vector4::new(der0.x, -sgn * der0.y, der1.x, -sgn * der1.y),
                Vector4::new(der0.y, sgn * der0.x, der1.y, sgn * der1.x),
                Vector4::new(
                    -der0.magnitude2() + diff0.dot(der20),
                    sgn * der20.perp_dot(diff0) + der0.dot(der20) / dmag0 * radius,
                    0.0,
                    0.0,
                ),
                Vector4::new(
                    0.0,
                    0.0,
                    -der1.magnitude2() + diff1.dot(der21),
                    sgn * der21.perp_dot(diff1) + der1.dot(der21) / dmag1 * radius,
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

#[inline]
fn rot_4(vec: Vector2) -> Vector2 { Vector2::new(-vec.y, vec.x) }

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::{prelude::*, property_test};

    #[property_test]
    fn test_circle_arc(
        #[strategy = prop::array::uniform2(-10.0..=10.0)] origin: [f64; 2],
        #[strategy = prop::array::uniform2(-10.0..=10.0)] direction0: [f64; 2],
        #[strategy = prop::array::uniform2(-10.0..=10.0)] direction1: [f64; 2],
        #[strategy = 0.5f64..=10.0] radius: f64,
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
        #[strategy = prop::array::uniform2(-10.0..=10.0)] origin: [f64; 2],
        #[strategy = prop::array::uniform2(-10.0..=10.0)] direction0: [f64; 2],
        #[strategy = prop::array::uniform2(-10.0..=10.0)] direction1: [f64; 2],
        #[strategy = prop::array::uniform2(-10.0..=10.0)] transit_direction: [f64; 2],
        #[strategy = 0.5f64..=10.0] radius: f64,
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

        let curve = circle_arc_by_three_points(point0, point1, transit).unwrap();
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
        #[strategy = prop::array::uniform2(-10.0..=10.0)] point0: [f64; 2],
        #[strategy = prop::array::uniform2(-10.0..=10.0)] point1: [f64; 2],
        #[strategy = 0.0..2.0 * PI] tangent_angle: f64,
        #[strategy = 0.05f64..=0.95f64] sample_ratio: f64,
    ) {
        let point0 = Point2::from(point0);
        let point1 = Point2::from(point1);
        let tangent0 = Vector2::new(tangent_angle.cos(), tangent_angle.sin());
        prop_assume!(!(point0 - point1).perp_dot(tangent0).so_small());

        let curve = circle_arc_by_tangent0(point0, point1, tangent0).unwrap();
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
        )
        .unwrap();
        let (t0, t1) = curve.range_tuple();
        assert_near!(curve.subs(t0), Point2::new(1.0, 0.0));
        assert_near!(curve.subs((t0 + t1) * 0.5), Point2::new(0.0, 1.0));
        assert_near!(curve.subs(t1), Point2::new(-1.0, 0.0));
    }

    #[test]
    fn circle_arc_by_three_points_rejects_collinear_points() {
        let error = circle_arc_by_three_points(
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(0.5, 0.0),
        )
        .unwrap_err();
        assert_eq!(error, Error::CollinearArcPoints);
    }

    #[test]
    fn test_circle_arc_tangent0_specific() {
        let tangent = Vector2::new(0.0, 1.0);
        let curve =
            circle_arc_by_tangent0(Point2::new(1.0, 0.0), Point2::new(0.0, 1.0), tangent).unwrap();
        let (t0, t1) = curve.range_tuple();
        assert_near!(curve.subs(t0), Point2::new(1.0, 0.0));
        assert_near!(curve.subs(t1), Point2::new(0.0, 1.0));
        assert_near!(curve.der(t0).normalize(), tangent.normalize());
    }

    #[test]
    fn circle_arc_by_tangent0_rejects_parallel_tangent() {
        let error = circle_arc_by_tangent0(
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Vector2::unit_x(),
        )
        .unwrap_err();
        assert_eq!(error, Error::ParallelArcTangent);
    }

    #[property_test]
    fn arc_arc_transit_test(
        #[strategy = prop::array::uniform2(-10.0..=10.0)] point0: [f64; 2],
        #[strategy = prop::array::uniform2(-10.0..=10.0)] point1: [f64; 2],
        #[strategy = 0.0..2.0 * PI] tangent_angle0: f64,
        #[strategy = 1.0..8.0] radius0: f64,
        #[strategy = 0.0..2.0 * PI] tangent_angle1: f64,
    ) {
        let point0 = Point2::from(point0);
        let point1 = Point2::from(point1);
        let tangent0 = Vector2::new(tangent_angle0.cos(), tangent_angle0.sin());
        let tangent1 = Vector2::new(tangent_angle1.cos(), tangent_angle1.sin());

        let opt = arc_arc_transit(point0, point1, tangent0, radius0, tangent1);
        prop_assume!(opt.is_ok());
        let transit = opt.unwrap();

        let arc0 = circle_arc_by_tangent0(point0, transit, tangent0).unwrap();
        let arc1 = circle_arc_by_tangent0(point1, transit, -tangent1).unwrap();

        let (_, t0) = arc0.range_tuple();
        let (_, t1) = arc1.range_tuple();
        let der0 = arc0.der(t0);
        let der1 = -arc1.der(t1);
        prop_assert!(der0.perp_dot(der1).so_small());
        prop_assert!(der0.dot(der1) > 0.0);
    }

    #[test]
    fn arc_arc_transit_specific0() {
        let point0 = Point2::new(1.0, 0.0);
        let tangent0 = Vector2::new(2.0, 0.0);
        let radius0 = 1.0;
        let point1 = Point2::new(0.0, 3.0);
        let tangent1 = Vector2::new(-2.0, 0.0);

        let transit = arc_arc_transit(point0, point1, tangent0, radius0, tangent1).unwrap();
        assert_near!(transit, Point2::new(2.0, 1.0));
    }
    #[test]
    fn arc_arc_transit_specific1() {
        let point0 = Point2::new(0.0, 0.0);
        let tangent0 = Vector2::new(1.5, 0.0);
        let radius0 = 1.0;
        let point1 = Point2::new(2.0, 2.0);
        let tangent1 = Vector2::new(1.23, 0.0);

        let transit = arc_arc_transit(point0, point1, tangent0, radius0, tangent1).unwrap();
        assert_near!(transit, Point2::new(1.0, 1.0));
    }
    #[test]
    fn arc_arc_transit_one_arc_case() {
        let point0 = Point2::new(0.0, -1.0);
        let tangent0 = Vector2::new(1.0, 0.0);
        let radius0 = 1.0;
        let point1 = Point2::new(-1.0, 0.0);
        let tangent1 = Vector2::new(0.0, -1.0);

        let transit = arc_arc_transit(point0, point1, tangent0, radius0, tangent1).unwrap();
        assert_near!(transit, Point2::new(f64::sqrt(0.5), f64::sqrt(0.5)));
    }

    #[property_test]
    fn line_arc_line_transit_test(
        #[strategy = prop::array::uniform2(-10.0..=10.0)] point0: [f64; 2],
        #[strategy = prop::array::uniform2(-10.0..=10.0)] point1: [f64; 2],
        #[strategy = 0.0..2.0 * PI] tangent_angle0: f64,
        #[strategy = 1.0..8.0] radius: f64,
        #[strategy = 0.0..2.0 * PI] tangent_angle1: f64,
    ) {
        let point0 = Point2::from(point0);
        let point1 = Point2::from(point1);
        let tangent0 = Vector2::new(tangent_angle0.cos(), tangent_angle0.sin());
        let tangent1 = Vector2::new(tangent_angle1.cos(), tangent_angle1.sin());

        let opt = line_arc_line_transit(point0, point1, tangent0, radius, tangent1);
        prop_assume!(opt.is_ok());
        let (transit0, transit1) = opt.unwrap();

        prop_assert!(tangent0.perp_dot(point0 - transit0).so_small());
        prop_assert!(tangent1.perp_dot(point1 - transit1).so_small());

        let arc = circle_arc_by_tangent0(transit0, transit1, transit0 - point0).unwrap();
        let matrix = arc.transform();
        let center = Point2::new(matrix[2][0] / matrix[2][2], matrix[2][1] / matrix[2][2]);
        prop_assert_near!(transit0.distance(center), radius);
        prop_assert_near!(transit1.distance(center), radius);
        let (_, t1) = arc.range_tuple();
        prop_assert!(tangent1.perp_dot(arc.der(t1)).so_small());
    }

    #[property_test]
    fn arc_line_arc_transit_test(
        #[strategy = prop::array::uniform2(-10.0..=10.0)] point0: [f64; 2],
        #[strategy = prop::array::uniform2(-10.0..=10.0)] point1: [f64; 2],
        #[strategy = 0.0..2.0 * PI] direction_angle0: f64,
        #[strategy = 0.0..2.0 * PI] direction_angle1: f64,
        #[strategy = 1.0..8.0] radius0: f64,
        #[strategy = 1.0..8.0] radius1: f64,
    ) {
        let point0 = Point2::from(point0);
        let point1 = Point2::from(point1);
        let tangent0 = Vector2::new(direction_angle0.cos(), direction_angle0.sin());
        let tangent1 = Vector2::new(direction_angle1.cos(), direction_angle1.sin());

        let opt = arc_line_arc_transit(point0, point1, tangent0, tangent1, radius0, radius1);
        prop_assume!(opt.is_ok());
        let (transit0, transit1) = opt.unwrap();

        let arc0 = circle_arc_by_tangent0(point0, transit0, tangent0).unwrap();
        let arc1 = circle_arc_by_tangent0(point1, transit1, -tangent1).unwrap();

        let matrix0 = arc0.transform();
        let center0 = Point2::new(matrix0[2][0] / matrix0[2][2], matrix0[2][1] / matrix0[2][2]);
        let matrix1 = arc1.transform();
        let center1 = Point2::new(matrix1[2][0] / matrix1[2][2], matrix1[2][1] / matrix1[2][2]);

        prop_assert_near!(transit0.distance(center0), radius0);
        prop_assert_near!(transit1.distance(center1), radius1);

        let (_, t0) = arc0.range_tuple();
        let (_, t1) = arc1.range_tuple();

        let delta = transit1 - transit0;
        prop_assert!(delta.perp_dot(arc0.der(t0)).so_small());
        prop_assert!(delta.dot(arc0.der(t0)) > 0.0);
        prop_assert!(delta.perp_dot(arc1.der(t1)).so_small());
        prop_assert!(delta.dot(arc1.der(t1)) < 0.0);
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
    fn fillet_candidate_for_two_lines_counter_direction() {
        let curve0 = Line(Point2::new(1.0, 1.0), Point2::new(1.0, 0.0));
        let curve1 = Line(Point2::new(1.0, 0.0), Point2::new(0.0, 0.0));
        let candidate = fillet_candidate(curve0, curve1, 1.0, 0.0, 0.2).unwrap();
        assert_near!(candidate.center, Point2::new(0.8, 0.2));
        assert_near2!(candidate.parameter0, 0.8);
        assert_near2!(candidate.parameter1, 0.2);
    }

    #[test]
    fn fillet_candidate_rejects_degenerate_corner() {
        let curve0 = Line(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
        let curve1 = Line(Point2::new(1.0, 0.0), Point2::new(2.0, 0.0));
        let error = fillet_candidate(curve0, curve1, 1.0, 0.0, 0.2).unwrap_err();
        assert_eq!(error, Error::DegenerateCorner);
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
    fn fillet_cndidate_for_two_arcs() {
        let curve0 = circle_arc_by_tangent0(
            Point2::new(1.0, -1.0),
            Point2::new(0.0, 0.0),
            Vector2::unit_y(),
        )
        .unwrap();
        let curve1 = circle_arc_by_tangent0(
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 1.0),
            Vector2::unit_x(),
        )
        .unwrap();
        let (_, t0) = curve0.range_tuple();
        let (t1, _) = curve1.range_tuple();
        let FilletCandidate {
            center,
            parameter0,
            parameter1,
        } = fillet_candidate(curve0, curve1, t0, t1, 1.0).unwrap();
        assert_near2!(center, Point2::new(f64::sqrt(3.0), 0.0));
        assert!(
            (center - curve0.subs(parameter0))
                .dot(curve0.der(parameter0))
                .so_small()
        );
        assert!(
            (center - curve1.subs(parameter1))
                .dot(curve1.der(parameter1))
                .so_small()
        );
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
