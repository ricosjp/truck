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

/// Oriented and reversible
pub trait InvertibleRadiusFunction: ScalarFunctionD1 {
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

impl<C, S0, S1, R> RbfSurface<C, S0, S1, R>
where
    C: ParametricCurve3D,
    S0: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    R: ScalarFunctionD1,
{
    fn sub_der_mn(&self, m: usize, n: usize, u: f64, cc: ContactCircle) -> Vector3 {
        match (m, n) {
            (_, 0) => cc.der_n(m, u),
            (0, 1) => self.vder_info(cc, 1).vder(u),
            (1, 1) => self.vder_info(cc, 1).uvder(u),
            (0, 2) => self.vder_info(cc, 2).vvder(u),
            _ => unimplemented!("higher order derivation of RbfSurface is not implemented."),
        }
    }
}

impl<C, S0, S1, R> ParametricSurface for RbfSurface<C, S0, S1, R>
where
    C: ParametricCurve3D,
    S0: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    R: ScalarFunctionD1,
{
    type Point = Point3;
    type Vector = Vector3;
    fn ders(&self, max_order: usize, u: f64, v: f64) -> SurfaceDers<Vector3> {
        let cc = self.contact_circle(v).unwrap();
        let mut out = SurfaceDers::new(max_order);
        (0..=max_order).for_each(|i| {
            (0..=max_order - i).for_each(|j| {
                out[i][j] = self.sub_der_mn(i, j, u, cc);
            });
        });
        out
    }
    fn der_mn(&self, m: usize, n: usize, u: f64, v: f64) -> Vector3 {
        self.sub_der_mn(m, n, u, self.contact_circle(v).unwrap())
    }
    fn subs(&self, u: f64, v: f64) -> Point3 { self.contact_circle(v).unwrap().subs(u) }
    fn uder(&self, u: f64, v: f64) -> Vector3 { self.contact_circle(v).unwrap().der(u) }
    fn vder(&self, u: f64, v: f64) -> Vector3 {
        self.vder_info(self.contact_circle(v).unwrap(), 1).vder(u)
    }
    fn uuder(&self, u: f64, v: f64) -> Self::Vector { self.contact_circle(v).unwrap().der2(u) }
    fn uvder(&self, u: f64, v: f64) -> Self::Vector {
        self.vder_info(self.contact_circle(v).unwrap(), 1).uvder(u)
    }
    fn vvder(&self, u: f64, v: f64) -> Self::Vector {
        self.vder_info(self.contact_circle(v).unwrap(), 2).vvder(u)
    }
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
    R: ScalarFunctionD1,
{
}

impl<C, S0, S1, R> BoundedSurface for RbfSurface<C, S0, S1, R>
where
    C: ParametricCurve3D + BoundedCurve,
    S0: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    R: ScalarFunctionD1,
{
}

impl<C, S0, S1, R> ParameterDivision2D for RbfSurface<C, S0, S1, R>
where
    C: ParametricCurve3D,
    S0: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    R: ScalarFunctionD1,
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
    R: ScalarFunctionD1,
{
    type Point = Point3;
    type Vector = Vector3;
    fn ders(&self, n: usize, t: f64) -> CurveDers<Vector3> {
        if n == 0 {
            return CurveDers::try_from([self.subs(t).to_vec()]).unwrap();
        }
        let cc = self.surface.contact_circle(t).unwrap();
        let rders = self.surface.radius.ders(n, t);
        let cc_ders = self.surface.sub_center_contacts_ders(cc, &rders, n);
        match self.index {
            0 => cc_ders.contact0_ders,
            _ => cc_ders.contact1_ders,
        }
    }
    fn der_n(&self, n: usize, t: f64) -> Self::Vector { self.ders(n, t)[n] }
    fn subs(&self, t: f64) -> Self::Point {
        let cc = self.surface.contact_circle(t).unwrap();
        match self.index {
            0 => cc.contact_point0.point,
            _ => cc.contact_point1.point,
        }
    }

    fn der(&self, t: f64) -> Self::Vector { self.der_n(1, t) }
    fn der2(&self, t: f64) -> Self::Vector { self.der_n(2, t) }
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
    R: ScalarFunctionD1,
{
}

impl<C, S0, S1, R> Cut for RbfContactCurve<C, S0, S1, R>
where
    C: ParametricCurve3D + Cut,
    S0: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    S1: ParametricSurface3D + SearchParameter<D2, Point = Point3>,
    R: ScalarFunctionD1,
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
    R: ScalarFunctionD1,
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
    R: ScalarFunctionD1,
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
    R: ScalarFunctionD1,
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
