#![cfg(feature = "derive")]

use truck_base::{cgmath64::*, hash::HashGen};
use truck_geotrait::*;
mod polynomial;
use polynomial::{PolyCurve, PolySurface};

#[test]
fn derive_build_test_is_running() {}

#[allow(dead_code)]
#[derive(Clone, Debug, ParametricCurve, BoundedCurve, ParameterDivision1D)]
enum DerivedCurve<P>
where
    P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>,
    P::Diff: std::fmt::Debug, {
    CurveA(PolyCurve<P>),
    CurveB { polycurve: PolyCurve<P> },
}

#[allow(dead_code)]
#[derive(Clone, Debug, ParametricSurface, BoundedSurface, ParameterDivision2D)]
enum DeriveSurface {
    SurfaceA(PolySurface),
    SurfaceB { polysurface: PolySurface },
}

#[allow(dead_code)]
#[derive(Clone, Debug, ParametricCurve, BoundedCurve, ParameterDivision1D)]
struct TupledCurve<P>(PolyCurve<P>)
where
    P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>,
    P::Diff: std::fmt::Debug;

#[allow(dead_code)]
#[derive(Clone, Debug, ParametricSurface, BoundedSurface, ParameterDivision2D)]
struct TupledSurface(PolySurface);
