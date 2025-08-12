#![cfg(feature = "derive")]
#![allow(dead_code)]

use polynomial::{PolynomialCurve, PolynomialSurface};
use truck_base::{cgmath64::*, hash::HashGen};
use truck_geotrait::*;

#[test]
fn derive_build_test_is_running() {}

#[derive(Clone, Debug, ParametricCurve, BoundedCurve, ParameterDivision1D)]
enum DerivedCurve<P>
where
    P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>,
    P::Diff: std::fmt::Debug, {
    CurveA(PolynomialCurve<P>),
    CurveB { polycurve: PolynomialCurve<P> },
}

#[derive(Clone, Debug, ParametricSurface, BoundedSurface, ParameterDivision2D)]
enum DeriveSurface<P>
where
    P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>,
    P::Diff: std::fmt::Debug, {
    SurfaceA(PolynomialSurface<P>),
    SurfaceB { polysurface: PolynomialSurface<P> },
}

#[derive(Clone, Debug, ParametricCurve, BoundedCurve, ParameterDivision1D)]
struct TupledCurve<P>(PolynomialCurve<P>)
where
    P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>,
    P::Diff: std::fmt::Debug;

#[derive(Clone, Debug, ParametricSurface, BoundedSurface, ParameterDivision2D)]
struct TupledSurface<P>(PolynomialSurface<P>)
where
    P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>,
    P::Diff: std::fmt::Debug;
