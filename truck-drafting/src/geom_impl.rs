use crate::{errors::Error, *};
use std::f64::consts::TAU;
use truck_base::newton::{self, CalcOutput};

pub fn circle_arc_by_three_points(
    point0: Point2,
    point1: Point2,
    transit: Point2,
) -> Processor<TrimmedCurve<UnitCircle<Point2>>, Matrix3> {
    let origin = circum_center(point0, point1, transit);
    let x_axis = point0 - origin;
    let transit_angle = local_angle(transit - origin, x_axis);
    let mut end_angle = local_angle(point1 - origin, x_axis);
    if end_angle < transit_angle || (end_angle.so_small() && !transit_angle.so_small()) {
        end_angle += TAU;
    }
    circle_arc(point0, origin, end_angle)
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
    let radius = chord.dot(chord) / denom;
    let origin = point0 + radius * to_origin;
    let vec0 = point0 - origin;
    let vec1 = point1 - origin;
    let mut angle = perp_dot(vec0, vec1).atan2(vec0.dot(vec1));
    if angle <= 0.0 {
        angle += TAU;
    }
    circle_arc(point0, origin, angle)
}

fn circum_center(point0: Point2, point1: Point2, point2: Point2) -> Point2 {
    let vec0 = point1 - point0;
    let vec1 = point2 - point0;
    let det = vec0.x * vec1.y - vec0.y * vec1.x;
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
) -> Processor<TrimmedCurve<UnitCircle<Point2>>, Matrix3> {
    let x_axis = point - origin;
    let y_axis = Vector2::new(-x_axis.y, x_axis.x);
    let transform = Matrix3::from_cols(
        x_axis.extend(0.0),
        y_axis.extend(0.0),
        Vector3::new(origin.x, origin.y, 1.0),
    );
    let unit_arc = TrimmedCurve::new(UnitCircle::new(), (0.0, angle));
    Processor::with_transform(unit_arc, transform)
}

fn local_angle(vector: Vector2, axis: Vector2) -> f64 {
    let radius2 = axis.magnitude2();
    let perp = Vector2::new(-axis.y, axis.x);
    let x = vector.dot(axis) / radius2;
    let y = vector.dot(perp) / radius2;
    let angle = y.atan2(x);
    if angle < 0.0 { angle + TAU } else { angle }
}

fn perp_dot(vec0: Vector2, vec1: Vector2) -> f64 { vec0.x * vec1.y - vec0.y * vec1.x }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_arc_passes_via_transit() {
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
    fn circle_arc_matches_tangent_at_start() {
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
}
