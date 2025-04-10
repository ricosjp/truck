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
    /// Substitutes the parameter `t`.
    fn subs(&self, t: f64) -> f64;
    /// Returns the derivation.
    fn der(&self, t: f64) -> f64;
    /// Returns the 2nd-order derivation.
    fn der2(&self, t: f64) -> f64;
}

impl RadiusFunction for f64 {
    #[inline]
    fn subs(&self, _: f64) -> f64 { *self }
    #[inline]
    fn der(&self, _: f64) -> f64 { 0.0 }
    #[inline]
    fn der2(&self, _: f64) -> f64 { 0.0 }
}

macro_rules! impl_radius_1dim {
    ($ty: ty) => {
        impl RadiusFunction for $ty {
            #[inline]
            fn subs(&self, t: f64) -> f64 { ParametricCurve::subs(self, t).x }
            #[inline]
            fn der(&self, t: f64) -> f64 { ParametricCurve::der(self, t).x }
            #[inline]
            fn der2(&self, t: f64) -> f64 { ParametricCurve::der2(self, t).x }
        }
    };
}
impl_radius_1dim!(BSplineCurve<Point1>);
impl_radius_1dim!(NurbsCurve<Vector2>);

impl<T: RadiusFunction> RadiusFunction for &T {
    #[inline(always)]
    fn subs(&self, t: f64) -> f64 { (**self).subs(t) }
    #[inline(always)]
    fn der(&self, t: f64) -> f64 { (**self).der(t) }
    #[inline(always)]
    fn der2(&self, t: f64) -> f64 { (**self).der2(t) }
}

/// Oriented and reversible
pub trait InvertibleRadiusFunction: RadiusFunction {
    /// Inverts `self`
    fn inverse(&self) -> Self;
    /// Returns the inverse.
    fn invert(&mut self);
}

impl InvertibleRadiusFunction for f64 {
    #[inline(always)]
    fn inverse(&self) -> Self { *self }
    #[inline(always)]
    fn invert(&mut self) {}
}

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
    fn uuder(&self, _u: f64, _v: f64) -> Self::Vector { unimplemented!() }
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

impl<C, S0, S1, R> From<RbfSurface<C, S0, S1, R>> for RbfSurface<Box<C>, Box<S0>, Box<S1>, R> {
    fn from(value: RbfSurface<C, S0, S1, R>) -> Self {
        Self {
            edge_curve: Box::new(value.edge_curve),
            surface0: Box::new(value.surface0),
            surface1: Box::new(value.surface1),
            radius: value.radius,
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
    fn der2(&self, t: f64) -> Self::Vector {
        (self.der(t + TOLERANCE) - self.der(t - TOLERANCE)) / (2.0 * TOLERANCE)
    }
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

impl<C, S0, S1, R> Cut for RbfContactCurve<C, S0, S1, R>
where
    C: ParametricCurve3D + Cut,
    S0: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    R: RadiusFunction,
{
    fn cut(&mut self, t: f64) -> Self {
        let edge_curve = self.surface.edge_curve.cut(t);
        Self {
            surface: RbfSurface {
                edge_curve,
                surface0: self.surface.surface0.clone(),
                surface1: self.surface.surface1.clone(),
                radius: self.surface.radius.clone(),
            },
            index: self.index,
        }
    }
}

impl<C, S, R> Invertible for RbfContactCurve<C, S, S, R>
where
    C: ParametricCurve3D + Invertible,
    S: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    R: InvertibleRadiusFunction,
{
    fn inverse(&self) -> Self {
        Self {
            surface: RbfSurface {
                edge_curve: self.surface.edge_curve.inverse(),
                surface0: self.surface.surface1.clone(),
                surface1: self.surface.surface0.clone(),
                radius: self.surface.radius.inverse(),
            },
            index: 1 - self.index,
        }
    }
    fn invert(&mut self) {
        self.surface.edge_curve.invert();
        std::mem::swap(&mut self.surface.surface0, &mut self.surface.surface1);
        self.surface.radius.invert();
        self.index = 1 - self.index;
    }
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
    S0: ParametricSurface3D
        + SearchParameter<D2, Point = Point3>
        + SearchNearestParameter<D2, Point = Point3>,
    S1: ParametricSurface3D
        + SearchParameter<D2, Point = Point3>
        + SearchNearestParameter<D2, Point = Point3>,
    R: RadiusFunction,
{
    type Point = Point3;
    fn search_parameter<H: Into<SPHint1D>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        match self.index {
            0 => self
                .surface
                .search_contact_curve0_parameter(point, hint, trials, true),
            _ => RbfSurface {
                edge_curve: &self.surface.edge_curve,
                surface0: &self.surface.surface1,
                surface1: &self.surface.surface0,
                radius: &self.surface.radius,
            }
            .search_contact_curve0_parameter(point, hint, trials, false),
        }
    }
}

impl<C, S0, S1, R> SearchNearestParameter<D1> for RbfContactCurve<C, S0, S1, R>
where
    C: ParametricCurve3D + BoundedCurve,
    S0: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    R: RadiusFunction,
{
    type Point = Point3;
    fn search_nearest_parameter<H: Into<<D1 as SPDimension>::Hint>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<<D1 as SPDimension>::Parameter> {
        use truck_geotrait::algo;
        let hint = match hint.into() {
            SPHint1D::Parameter(hint) => hint,
            SPHint1D::Range(x, y) => {
                algo::curve::presearch(self, point, (x, y), PRESEARCH_DIVISION)
            }
            SPHint1D::None => {
                algo::curve::presearch(self, point, self.range_tuple(), PRESEARCH_DIVISION)
            }
        };
        algo::curve::search_nearest_parameter(self, point, hint, trials)
    }
}

impl<C, S0, S1, R> From<RbfContactCurve<C, S0, S1, R>>
    for RbfContactCurve<Box<C>, Box<S0>, Box<S1>, R>
{
    fn from(value: RbfContactCurve<C, S0, S1, R>) -> Self {
        Self {
            surface: RbfSurface {
                edge_curve: value.surface.edge_curve.into(),
                surface0: value.surface.surface0.into(),
                surface1: value.surface.surface1.into(),
                radius: value.surface.radius,
            },
            index: value.index,
        }
    }
}
