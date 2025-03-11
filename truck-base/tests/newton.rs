use proptest::prelude::*;
use std::f64::consts::PI;
use truck_base::{cgmath64::*, newton::*, prop_assert_near, tolerance::*};

proptest! {
    #[test]
    fn test_newton1(
        a in prop::array::uniform4(-10.0f64..=10.0f64),
        x0 in -10.0f64..=10.0f64,
        delta in -0.5f64..=0.5f64,
    ) {
        let poly = |x: f64| a[0] + a[1] * x + a[2] * x * x + a[3] * x * x * x;
        let function = |x: f64| CalcOutput {
            value: poly(x) - poly(x0),
            derivation: a[1] + 2.0 * a[2] * x + 3.0 * a[3] * x * x,
        };
        match solve(function, x0 + delta, 100) {
            Ok(res) => prop_assert_near!(function(res).value, 0.0),
            Err(log) => prop_assert!(log.degenerate(), "{log}"),
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
        let function = |vec: Vector2| CalcOutput {
            value: Vector2::new(vec.magnitude2() - 1.0, vec.dot(n)),
            derivation: Matrix2::new(2.0 * vec.x, n.x, 2.0 * vec.y, n.y),
        };
        let hint = Matrix2::from_angle(-Rad(PI / 2.0)) * n + Vector2::from(delta);
        match solve(function, hint, 10) {
            Ok(res) => prop_assert_near!(function(res).value, Vector2::zero()),
            Err(log) => prop_assert!(log.degenerate(), "{log}"),
        }
    }
}
