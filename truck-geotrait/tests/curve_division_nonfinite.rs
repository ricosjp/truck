use std::{cell::Cell, rc::Rc};
use truck_base::cgmath64::*;
use truck_geotrait::{algo::curve::parameter_division, ParametricCurve};

#[derive(Clone)]
struct Curve {
    mode: u8,
    calls: Rc<Cell<usize>>,
}

impl Curve {
    fn new(mode: u8) -> Self {
        Self {
            mode,
            calls: Rc::new(Cell::new(0)),
        }
    }
}

impl ParametricCurve for Curve {
    type Point = Point2;
    type Vector = Vector2;
    fn subs(&self, t: f64) -> Point2 {
        let calls = self.calls.get() + 1;
        self.calls.set(calls);
        // Keep this regression bounded even against the unfixed implementation.
        assert!(calls <= 64, "test guard: excessive subdivision");
        if self.mode == 1 && t == 0. || self.mode == 2 && t > 0. && t < 1. {
            Point2::new(f64::NAN, 0.)
        } else if self.mode == 3 {
            Point2::new(f64::INFINITY, 0.)
        } else {
            Point2::new(t, t * t)
        }
    }
    fn der(&self, t: f64) -> Vector2 {
        Vector2::new(1., 2. * t)
    }
    fn der2(&self, _: f64) -> Vector2 {
        Vector2::new(0., 2.)
    }
    fn der_n(&self, n: usize, t: f64) -> Vector2 {
        match n {
            0 => self.subs(t).to_vec(),
            1 => self.der(t),
            2 => self.der2(t),
            _ => Vector2::zero(),
        }
    }
}

#[test]
#[should_panic(expected = "curve subdivision encountered a nonfinite sample")]
fn nan_endpoint_is_rejected_before_recursion() {
    parameter_division(&Curve::new(1), (0., 1.), 0.01);
}

#[test]
#[should_panic(expected = "curve subdivision encountered a nonfinite sample")]
fn nan_interior_is_rejected_before_recursion() {
    parameter_division(&Curve::new(2), (0., 1.), 0.01);
}

#[test]
#[should_panic(expected = "curve subdivision encountered a nonfinite sample")]
fn infinite_sample_is_rejected_before_recursion() {
    parameter_division(&Curve::new(3), (0., 1.), 0.01);
}

#[test]
#[should_panic(expected = "curve subdivision requires a finite parameter range")]
fn nonfinite_range_is_rejected() {
    parameter_division(&Curve::new(0), (0., f64::INFINITY), 0.01);
}

#[test]
fn finite_curve_preserves_endpoints_and_tolerance() {
    let curve = Curve::new(0);
    let (params, points) = parameter_division(&curve, (0., 1.), 0.01);
    assert_eq!(params.first(), Some(&0.));
    assert_eq!(params.last(), Some(&1.));
    assert_eq!(params.len(), points.len());
    assert!(params.len() > 2);
    for pair in params.windows(2) {
        let a = curve.subs(pair[0]);
        let b = curve.subs(pair[1]);
        assert!(curve.subs((pair[0] + pair[1]) / 2.).distance(a.midpoint(b)) < 0.02);
    }
}

#[test]
fn reversed_finite_range_remains_supported() {
    let (params, points) = parameter_division(&Curve::new(0), (1., 0.), 0.01);
    assert_eq!(params.first(), Some(&1.));
    assert_eq!(params.last(), Some(&0.));
    assert_eq!(points.first(), Some(&Point2::new(1., 1.)));
    assert_eq!(points.last(), Some(&Point2::new(0., 0.)));
}
