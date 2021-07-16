use crate::*;
use std::ops::{Deref, DerefMut};
use truck_base::cgmath64::control_point::ControlPoint;

impl<P> Deref for PolylineCurve<P> {
	type Target = Vec<P>;
	fn deref(&self) -> &Self::Target { &self.0 }
}

impl<P> DerefMut for PolylineCurve<P> {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<P: ControlPoint<f64>> ParametricCurve for PolylineCurve<P> {
	type Point = P;
	type Vector = P::Diff;
	#[inline(always)]
	fn parameter_range(&self) -> (f64, f64) { (0.0, self.len() as f64 - 1.0) }
	#[inline(always)]
	fn subs(&self, t: f64) -> P {
		if self.is_empty() {
			P::origin()
		} else if t <= 0.0 {
			self[0]
		} else if t + 1.0 >= self.len() as f64 {
			self[self.len() - 1]
		} else {
			let n = t as usize;
			let t = t - n as f64;
			self[n] + (self[n + 1] - self[n]) * t
		}
	}
	#[inline(always)]
	fn der(&self, t: f64) -> P::Diff {
		if t <= 0.0 || self.len() as f64 <= t + 1.0 {
			P::Diff::zero()
		} else {
			let n = t as usize;
			self[n + 1] - self[n]
		}
	}
	#[inline(always)]
	fn der2(&self, _: f64) -> P::Diff { P::Diff::zero() }
}
