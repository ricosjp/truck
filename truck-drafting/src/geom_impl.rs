use crate::*;
use std::f64::consts::TAU;

pub(super) fn circle_arc_by_three_points(
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

pub(super) fn circle_arc_by_tangent0(
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

fn perp_dot(vec0: Vector2, vec1: Vector2) -> f64 {
    vec0.x * vec1.y - vec0.y * vec1.x
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
}
