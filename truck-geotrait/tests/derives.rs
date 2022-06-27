#![cfg(feature = "derive")]

use truck_base::cgmath64::*;
use truck_geotrait::*;
mod polynomial;
use polynomial::{PolyCurve, PolySurface};

#[test]
fn derive_build_test_is_running() {}

#[allow(dead_code)]
#[derive(Clone, Debug, ParametricCurve, BoundedCurve, ParameterDivision1D)]
enum DerivedCurve {
    CurveA(PolyCurve<Point2>),
    CurveB { polycurve: PolyCurve<Point2> },
}

#[allow(dead_code)]
#[derive(Clone, Debug, ParametricSurface, BoundedSurface, ParameterDivision2D)]
enum DeriveSurface {
    SurfaceA(PolySurface),
    SurfaceB { polysurface: PolySurface },
}
