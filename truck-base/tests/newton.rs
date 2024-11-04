use proptest::prelude::*;
use truck_base::{assert_near, cgmath64::*, newton::*, tolerance::*};
use std::f64::consts::PI;

proptest! {
    #[test]
    fn test_newton1(
        a in prop::array::uniform4(-10.0f64..=10.0f64),
        x0 in -10.0f64..=10.0f64,
        delta in -0.5f64..=0.5f64,
    ) {
        let poly = |x: f64| a[0] + a[1] * x + a[2] * x * x + a[3] * x * x * x;
        let value = |x: f64| poly(x) - poly(x0);
        let der = |x: f64| a[1] + 2.0 * a[2] * x + 3.0 * a[3] * x * x;
        match newton1(value, der, x0 + delta, 100) {
            Ok(res) => assert_near!(value(res), 0.0),
            Err(log) => assert!(log.degenerate(), "{log}"),
        }
    }

    #[test]
    fn test_newton2(
        n in prop::array::uniform2(-10.0f64..=10.0f64),
        delta in prop::array::uniform2(-0.5f64..=0.5f64),
    ) {
        let n = Vector2::from(n);
        if n.so_small() {
            return Ok(());
        }
        let n = n.normalize();
        let value = |vec: Vector2| Vector2::new(vec.magnitude2() - 1.0, vec.dot(n));
        let der0 = |Vector2 { x, .. }| Vector2::new(2.0 * x, n.x);
        let der1 = |Vector2 { y, .. }| Vector2::new(2.0 * y, n.y);
        let hint = Matrix2::from_angle(-Rad(PI / 2.0)) * n + Vector2::from(delta);
        match newton2(value, der0, der1, hint, 10) {
            Ok(res) => assert_near!(value(res), Vector2::zero()),
            Err(log) => assert!(log.degenerate(), "{log}"),
        }
    }
}
