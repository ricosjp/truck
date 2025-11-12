#![allow(clippy::many_single_char_names)]

use crate::traits::*;
use truck_base::{
    cgmath64::*,
    hash::HashGen,
    newton::{self, CalcOutput},
    tolerance::*,
};

/// curve algorithms
pub mod curve;
/// surface algorithms
pub mod surface;

/// Method to split edges and surfaces to generate tesselations
pub trait TesselationSplitMethod: Clone + Copy + Send + Sync {
    /// Return whether a portion of the curve defined by `range` should be split
    fn split_curve<C: ParametricCurve>(&self, curve: &C, range: (f64, f64)) -> bool
    where
        C::Point: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>;
    /// Return whether a portion of the surface defined by `range` should be split along u and v
    fn split_surface<S: ParametricSurface>(
        &self,
        surface: &S,
        range_u: (f64, f64),
        range_v: (f64, f64),
    ) -> (bool, bool)
    where
        S::Point: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>;
    /// TODO: remove
    fn tol(self) -> f64;
    /// Scale the tolerance
    fn scale(self, f: f64) -> Self;
}

/// Simple split method based only on the distance between midpoints and the actual geometry
#[derive(Debug, Clone, Copy)]
pub struct DefaultSplitParams {
    max_dist_2: f64,
}

impl DefaultSplitParams {
    /// Create a `DefaultSplitParams` frm the maximum distance between the mid edges / mid faces and the geometry
    pub fn new(max_dist: f64) -> Self {
        nonpositive_tolerance!(max_dist);
        Self {
            max_dist_2: max_dist.powi(2),
        }
    }
}

impl TesselationSplitMethod for DefaultSplitParams {
    fn split_curve<C: ParametricCurve>(&self, curve: &C, range: (f64, f64)) -> bool
    where
        C::Point: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>,
    {
        let p0 = curve.subs(range.0);
        let p1 = curve.subs(range.1);

        let gen = p0.midpoint(p1);
        let p = 0.5 + (0.2 * HashGen::hash1(gen) - 0.1);
        let t = range.0 * (1.0 - p) + range.1 * p;
        let mid = p0 + (p1 - p0) * p;
        let new = curve.subs(t);

        let dist2 = new.distance2(mid);
        if dist2 > self.max_dist_2 {
            return true;
        }

        false
    }

    fn split_surface<S: ParametricSurface>(
        &self,
        surface: &S,
        (u0, u1): (f64, f64),
        (v0, v1): (f64, f64),
    ) -> (bool, bool)
    where
        S::Point: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>,
    {
        let (u_gen, v_gen) = ((u0 + u1) / 2.0, (v0 + v1) / 2.0);
        let gen = surface.subs(u_gen, v_gen);
        let p = 0.5 + (0.2 * HashGen::hash1(gen) - 0.1);
        let q = 0.5 + (0.2 * HashGen::hash1(gen) - 0.1);
        let u = u0 * (1.0 - p) + u1 * p;
        let v = v0 * (1.0 - q) + v1 * q;
        let p0 = surface.subs(u, v);
        let pt00 = surface.subs(u0, v0);
        let pt01 = surface.subs(u0, v1);
        let pt10 = surface.subs(u1, v0);
        let pt11 = surface.subs(u1, v1);
        let pt = S::Point::from_vec(
            pt00.to_vec() * (1.0 - p) * (1.0 - q)
                + pt01.to_vec() * (1.0 - p) * q
                + pt10.to_vec() * p * (1.0 - q)
                + pt11.to_vec() * p * q,
        );
        if p0.distance2(pt) > self.max_dist_2 {
            let delu = pt00.midpoint(pt01).distance(p0) + pt10.midpoint(pt11).distance(p0);
            let delv = pt00.midpoint(pt10).distance(p0) + pt01.midpoint(pt11).distance(p0);
            if delu > delv * 2.0 {
                return (true, false);
            } else if delv > delu * 2.0 {
                return (false, true);
            } else {
                return (true, true);
            }
        }
        (false, false)
    }

    fn tol(self) -> f64 {
        self.max_dist_2.sqrt()
    }

    fn scale(self, f: f64) -> Self {
        Self {
            max_dist_2: self.max_dist_2 * f * f,
        }
    }
}

/// Simple split method based on
/// - the distance between midpoints and the actual geometry
/// - the (approximate) angle between adjacent edges / faces
/// - a minimum egde size
#[derive(Debug, Clone, Copy)]
pub struct SplitParams {
    max_dist_2: f64,
    min_len_2: f64,
    cos_min: f64,
}

impl SplitParams {
    /// Create a `SplitParams` from:
    /// - the max distance between mid edges / faces and the geometry
    /// - the minimum edge length
    /// - the max angle (in degrees) between adjacent edges / faces
    pub fn new(max_dist: f64, min_len: f64, max_angle_degrees: f64) -> Self {
        nonpositive_tolerance!(max_dist);
        Self {
            max_dist_2: max_dist.powi(2),
            min_len_2: min_len.powi(2),
            cos_min: max_angle_degrees.to_radians().cos(),
        }
    }

    fn cos_angle<P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64>>(
        p0: P,
        p1: P,
        p: P,
    ) -> f64 {
        let a0 = p[0] - p0[0];
        let a1 = p[1] - p0[1];
        let a2 = p[2] - p0[2];
        let b0 = p1[0] - p[0];
        let b1 = p1[1] - p[1];
        let b2 = p1[2] - p[2];
        let dot = a0 * b0 + a1 * b1 + a2 * b2;
        let anrm2 = a0 * a0 + a1 * a1 + a2 * a2;
        let bnrm2 = b0 * b0 + b1 * b1 + b2 * b2;
        dot / (anrm2 * bnrm2).sqrt()
    }

    fn split<P: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64>>(
        &self,
        p0: P,
        p1: P,
        new: P,
    ) -> bool {
        if p0.distance2(p1) < self.min_len_2 {
            return false;
        }

        let mid = p0.midpoint(p1);
        let dist2 = new.distance2(mid);
        if dist2 > self.max_dist_2 {
            return true;
        }

        if Self::cos_angle(p0, p1, new) < self.cos_min {
            return true;
        }

        false
    }
}

impl TesselationSplitMethod for SplitParams {
    fn split_curve<C: ParametricCurve>(&self, curve: &C, range: (f64, f64)) -> bool
    where
        C::Point: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>,
    {
        let p0 = curve.subs(range.0);
        let p1 = curve.subs(range.1);

        let new = curve.subs(0.5 * (range.0 + range.1));

        self.split(p0, p1, new)
    }

    fn split_surface<S: ParametricSurface>(
        &self,
        surface: &S,
        (u0, u1): (f64, f64),
        (v0, v1): (f64, f64),
    ) -> (bool, bool)
    where
        S::Point: EuclideanSpace<Scalar = f64> + MetricSpace<Metric = f64> + HashGen<f64>,
    {
        let p = surface.subs(0.5 * (u0 + u1), 0.5 * (v0 + v1));

        let p0 = surface.subs(u0, 0.5 * (v0 + v1));
        let p1 = surface.subs(u1, 0.5 * (v0 + v1));
        let split_u = self.split(p0, p1, p);
        let p0 = surface.subs(0.5 * (u0 + u1), v0);
        let p1 = surface.subs(0.5 * (u0 + u1), v1);
        let split_v = self.split(p0, p1, p);
        (split_u, split_v)
    }

    fn tol(self) -> f64 {
        self.max_dist_2.sqrt()
    }

    fn scale(self, f: f64) -> Self {
        Self {
            max_dist_2: self.max_dist_2 * f * f,
            min_len_2: self.min_len_2,
            cos_min: self.cos_min,
        }
    }
}
