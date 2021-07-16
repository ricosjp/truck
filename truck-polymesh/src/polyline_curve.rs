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

impl<P> From<Vec<P>> for PolylineCurve<P> {
	fn from(v: Vec<P>) -> Self { Self(v) }
}

impl<P> From<PolylineCurve<P>> for Vec<P> {
	fn from(v: PolylineCurve<P>) -> Self { v.0 }
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

impl<P: Clone> Invertible for PolylineCurve<P> {
    #[inline(always)]
    fn invert(&mut self) { self.reverse(); }
    #[inline(always)]
    fn inverse(&self) -> Self { Self(self.iter().rev().map(|p| p.clone()).collect()) }
}

impl<P: ControlPoint<f64>> Cut for PolylineCurve<P> {
	fn cut(&mut self, t: f64) -> Self {
		if t < 0.0 {
			PolylineCurve(Vec::new())
		} else if t + 1.0 > self.len() as f64 {
			let mut v = Vec::new();
			v.append(&mut self.0);
			PolylineCurve(v)
		} else {
			let n = t as usize;
			if t.near(&(n as f64)) {
				let mut v = Vec::new();
				v.extend(&self[n..]);
				self.truncate(n + 1);
				PolylineCurve(v)
			} else {
				let p = self.subs(t);
				let mut v = vec![p];
				v.extend(&self[(n + 1)..]);
				self.truncate(n + 1);
				self.push(p);
				PolylineCurve(v)
			}
		}
	}
}

#[test]
fn polyline_test() {
	let vec = vec![
		Point3::new(0.0, 0.0, 0.0),
		Point3::new(1.0, 0.0, 0.0),
		Point3::new(0.0, 1.0, 0.0),
		Point3::new(0.0, 0.0, 1.0),
		Point3::new(0.0, 1.0, 1.0),
		Point3::new(1.0, 0.0, 1.0),
		Point3::new(1.0, 1.0, 0.0),
		Point3::new(1.0, 1.0, 1.0),
	];
	let polyline = PolylineCurve(vec);
	truck_base::assert_near!(polyline.subs(2.5), Point3::new(0.0, 0.5, 0.5));

	let mut part0 = polyline.clone();
	let part1 = part0.cut(2.5);
	assert_eq!(part0.len(), 4);
	assert_eq!(part1.len(), 6);
	truck_base::assert_near!(part0.subs(2.5), Point3::new(0.0, 0.75, 0.25));
	truck_base::assert_near!(part1.subs(0.5), Point3::new(0.0, 0.25, 0.75));
	
	let mut part0 = polyline.clone();
	let part1 = part0.cut(4.0);
	assert_eq!(part0.len(), 5);
	assert_eq!(part1.len(), 4);
	truck_base::assert_near!(part0.subs(2.5), Point3::new(0.0, 0.5, 0.5));
	truck_base::assert_near!(part1.subs(0.5), Point3::new(0.5, 0.5, 1.0));
}

