use truck_base::cgmath64::*;
use truck_meshalgo::prelude::*;

#[derive(Debug, Clone)]
pub struct IntersectionCurve<P, S> {
	surface0: S,
	surface1: S,
	polyline: PolylineCurve<P>,
	tol: f64,
}


