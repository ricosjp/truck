use super::*;
use std::f64::consts::PI;

impl<C, S0, S1, R> RbfSurface<C, S0, S1, R> {
    /// constructor
    #[inline]
    pub const fn new(edge_curve: C, surface0: S0, surface1: S1, radius: R) -> Self {
        Self {
            edge_curve,
            surface0,
            surface1,
            radius,
        }
    }

    /// returns edge curve
    #[inline]
    pub const fn edge_curve(&self) -> &C { &self.edge_curve }
    /// returns first surface
    #[inline]
    pub const fn surface0(&self) -> &S0 { &self.surface0 }
    /// returns second surface
    #[inline]
    pub const fn surface1(&self) -> &S1 { &self.surface1 }
    /// returns radius function
    #[inline]
    pub const fn radius(&self) -> &R { &self.radius }

    /// returns the orbit curve of contact point with `surface0`.
    #[inline]
    pub fn contact_curve0(&self) -> RbfContactCurve<C, S0, S1, R>
    where Self: Clone {
        RbfContactCurve {
            surface: self.clone(),
            index: 0,
        }
    }
    /// returns the orbit curve of contact point with `surface1`.
    #[inline]
    pub fn contact_curve1(&self) -> RbfContactCurve<C, S0, S1, R>
    where Self: Clone {
        RbfContactCurve {
            surface: self.clone(),
            index: 1,
        }
    }
}

/// trait for radius function
pub trait RadiusFunction: Clone {
    /// Returns the `n`th order derivation.
    fn der_n(&self, n: usize, t: f64) -> f64;
    /// Substitutes the parameter `t`.
    #[inline]
    fn subs(&self, t: f64) -> f64 { self.der_n(0, t) }
    /// Returns the derivation.
    #[inline]
    fn der(&self, t: f64) -> f64 { self.der_n(1, t) }
    /// Returns the 2nd-order derivation.
    #[inline]
    fn der2(&self, t: f64) -> f64 { self.der_n(2, t) }
    /// Substitutes the higher-order derivations to `out`.
    #[inline]
    fn ders(&self, t: f64, out: &mut [f64]) {
        out.iter_mut()
            .enumerate()
            .for_each(|(i, o)| *o = self.der_n(i, t))
    }
}

impl RadiusFunction for f64 {
    #[inline]
    fn der_n(&self, n: usize, _: f64) -> f64 {
        match n {
            0 => *self,
            _ => 0.0,
        }
    }
}

macro_rules! impl_radius_1dim {
    ($ty: ty) => {
        impl RadiusFunction for $ty {
            #[inline]
            fn der_n(&self, n: usize, t: f64) -> f64 { ParametricCurve::der_n(self, n, t).x }
        }
    };
}
impl_radius_1dim!(BSplineCurve<Point1>);
impl_radius_1dim!(NurbsCurve<Vector2>);

/// Contact point of the rolling ball and surface
#[derive(Clone, Copy, Debug)]
pub struct ContactPoint {
    /// the 3d-coordinate of contact point
    pub point: Point3,
    /// the parameter on the surface
    pub uv: Point2,
}

impl From<(Point3, Point2)> for ContactPoint {
    #[inline]
    fn from((point, uv): (Point3, Point2)) -> Self { Self { point, uv } }
}

impl From<ContactPoint> for (Point3, (f64, f64)) {
    #[inline]
    fn from(cp: ContactPoint) -> Self { (cp.point, (cp.uv.x, cp.uv.y)) }
}

/// Contact circle for rolling ball fillet.
#[derive(Clone, Copy, Debug)]
pub struct ContactCircle {
    center: Point3,
    axis: Vector3,
    angle: Rad<f64>,
    t: f64,
    contact_point0: ContactPoint,
    contact_point1: ContactPoint,
}

mod algo;
mod contact_circle;

impl<C, S0, S1, R> ParametricSurface for RbfSurface<C, S0, S1, R>
where
    C: ParametricCurve3D,
    S0: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    R: RadiusFunction,
{
    type Point = Point3;
    type Vector = Vector3;
    fn subs(&self, u: f64, v: f64) -> Point3 { self.contact_circle(v).unwrap().subs(u) }
    fn uder(&self, u: f64, v: f64) -> Vector3 { self.contact_circle(v).unwrap().der(u) }
    fn vder(&self, u: f64, v: f64) -> Vector3 { self.vder(u, self.contact_circle(v).unwrap()) }
    fn uuder(&self, u: f64, v: f64) -> Self::Vector { self.contact_circle(v).unwrap().der2(u) }
    fn uvder(&self, _u: f64, _v: f64) -> Self::Vector { unimplemented!() }
    fn vvder(&self, _u: f64, _v: f64) -> Self::Vector { unimplemented!() }
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) {
        use std::ops::Bound::*;
        (
            (Included(0.0), Included(1.0)),
            self.edge_curve.parameter_range(),
        )
    }
    fn v_period(&self) -> Option<f64> { self.edge_curve.period() }
}

impl<C, S0, S1, R> ParametricSurface3D for RbfSurface<C, S0, S1, R>
where
    C: ParametricCurve3D,
    S0: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    R: RadiusFunction,
{
}

impl<C, S0, S1, R> BoundedSurface for RbfSurface<C, S0, S1, R>
where
    C: ParametricCurve3D + BoundedCurve,
    S0: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    R: RadiusFunction,
{
}

impl<C, S0, S1, R> ParameterDivision2D for RbfSurface<C, S0, S1, R>
where
    C: ParametricCurve3D,
    S0: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    R: RadiusFunction,
{
    fn parameter_division(
        &self,
        range: ((f64, f64), (f64, f64)),
        tol: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        nonpositive_tolerance!(tol);
        let udiv = self.u_parameter_division(range, tol).unwrap();
        let mut vdiv = vec![range.1 .0, range.1 .1];
        algo::v_parameter_division_for_fillet(self, &udiv, &mut vdiv, tol);
        (udiv, vdiv)
    }
}

impl<C, S0, S1, R> SearchParameter<D2> for RbfSurface<C, S0, S1, R>
where
    C: ParametricCurve3D + SearchNearestParameter<D1, Point = Point3>,
    S0: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    R: RadiusFunction,
{
    type Point = Point3;
    fn search_parameter<H: Into<SPHint2D>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        let curve_hint = match hint.into() {
            SPHint2D::Parameter(_, v) => SPHint1D::Parameter(v),
            SPHint2D::Range(_, (v0, v1)) => SPHint1D::Range(v0, v1),
            SPHint2D::None => SPHint1D::None,
        };
        let edge_curve = &self.edge_curve;
        let v = edge_curve.search_nearest_parameter(point, curve_hint, trials)?;
        let cc = self.contact_circle(v)?;

        let cp0 = cc.contact_point0.point - cc.center;
        let cp = point - cc.center;
        let u = cp.angle(cp0).0 / cc.angle.0;
        match cp.magnitude2().near(&cp0.magnitude2()) {
            true => Some((u, v)),
            false => None,
        }
    }
}

impl<C, S0, S1, R> RbfContactCurve<C, S0, S1, R> {
    /// original fillet surface
    #[inline]
    pub const fn fillet_surface(&self) -> &RbfSurface<C, S0, S1, R> { &self.surface }
    /// curve index: curve on `surface0` => 0, curve on `surface1` => 1.
    #[inline]
    pub const fn index(&self) -> usize { self.index }
}

impl<C, S0, S1, R> ParametricCurve for RbfContactCurve<C, S0, S1, R>
where
    C: ParametricCurve3D,
    S0: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    R: RadiusFunction,
{
    type Point = Point3;
    type Vector = Vector3;
    fn subs(&self, t: f64) -> Self::Point {
        let cc = self.surface.contact_circle(t).unwrap();
        match self.index {
            0 => cc.contact_point0.point,
            _ => cc.contact_point1.point,
        }
    }
    fn der(&self, t: f64) -> Self::Vector {
        let cc = self.surface.contact_circle(t).unwrap();
        match self.index {
            0 => self.surface.contact_point0_der(cc),
            _ => self.surface.contact_point1_der(cc),
        }
    }
    fn der2(&self, _t: f64) -> Self::Vector { unimplemented!() }
    #[inline]
    fn parameter_range(&self) -> ParameterRange { self.surface.edge_curve.parameter_range() }
    #[inline]
    fn period(&self) -> Option<f64> { self.surface.edge_curve.period() }
}

impl<C, S0, S1, R> BoundedCurve for RbfContactCurve<C, S0, S1, R>
where
    C: ParametricCurve3D + BoundedCurve,
    S0: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    R: RadiusFunction,
{
}

impl<C, S0, S1, R> ParameterDivision1D for RbfContactCurve<C, S0, S1, R>
where
    C: ParametricCurve3D,
    S0: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    R: RadiusFunction,
{
    type Point = Point3;
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Self::Point>) {
        truck_geotrait::algo::curve::parameter_division(self, range, tol)
    }
}

impl<C, S0, S1, R> SearchParameter<D1> for RbfContactCurve<C, S0, S1, R>
where
    C: ParametricCurve3D + SearchNearestParameter<D1, Point = Point3>,
    S0: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    R: RadiusFunction,
{
    type Point = Point3;
    fn search_parameter<H: Into<SPHint1D>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        let edge_curve = &self.surface.edge_curve;
        let t = edge_curve.search_nearest_parameter(point, hint, trials)?;
        let cc = self.surface.contact_circle(t)?;
        let q = match self.index {
            0 => cc.contact_point0.point,
            _ => cc.contact_point1.point,
        };
        match point.near(&q) {
            true => Some(t),
            false => None,
        }
    }
}
