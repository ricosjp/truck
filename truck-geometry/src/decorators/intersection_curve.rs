use super::*;

#[doc(hidden)]
pub fn double_projection<S>(
    surface0: &S,
    hint0: Option<(f64, f64)>,
    surface1: &S,
    hint1: Option<(f64, f64)>,
    mut point: Point3,
    normal: Vector3,
    trials: usize,
) -> Option<(Point3, Point2, Point2)>
where
    S: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    #[cfg(all(test, debug_assertions))]
    let mut log = Vec::new();
    let mut uv0 = surface0.search_nearest_parameter(point, hint0, 10)?;
    let mut uv1 = surface1.search_nearest_parameter(point, hint1, 10)?;
    for _ in 0..trials {
        #[cfg(all(test, debug_assertions))]
        log.push((point, uv0, uv1));
        uv0 = surface0.search_nearest_parameter(point, Some(uv0), 10)?;
        let pt0 = surface0.subs(uv0.0, uv0.1);
        uv1 = surface1.search_nearest_parameter(point, Some(uv1), 10)?;
        let pt1 = surface1.subs(uv1.0, uv1.1);
        if point.near(&pt0) && point.near(&pt1) && pt0.near(&pt1) {
            return Some((point, Point2::from(uv0), Point2::from(uv1)));
        } else {
            let n0 = surface0.normal(uv0.0, uv0.1);
            let n1 = surface1.normal(uv1.0, uv1.1);
            let mat = Matrix3::from_cols(n0, n1, normal).transpose();
            let inv = mat.invert()?;
            let pt = inv * Vector3::new(pt0.dot(n0), pt1.dot(n1), point.dot(normal));
            point = Point3::from_vec(pt);
        }
    }
    #[cfg(all(test, debug_assertions))]
    {
        eprintln!("Newton method is not converges");
        log.into_iter().for_each(|t| eprintln!("{t:?}"));
    }
    None
}

/// Mutable editor for `IntersectionCurve`.
#[doc(hidden)]
#[derive(Debug)]
pub struct IntersectionCurveEditor<'a, C, S> {
    pub surface0: &'a mut S,
    pub surface1: &'a mut S,
    pub leader: &'a mut C,
    pub tol: &'a mut f64,
}

impl<C, S> IntersectionCurve<C, S> {
    /// This curve is a part of intersection of `self.surface0()` and `self.surface1()`.
    #[inline(always)]
    pub fn surface0(&self) -> &S { &self.surface0 }
    /// This curve is a part of intersection of `self.surface0()` and `self.surface1()`.
    #[inline(always)]
    pub fn surface1(&self) -> &S { &self.surface1 }
    /// Returns the polyline leading this curve.
    #[inline(always)]
    pub fn leader(&self) -> &C { &self.leader }
    /// Returns editor for `IntersectionCurve`. This method is only for developers, do not use.
    #[doc(hidden)]
    #[inline(always)]
    pub fn editor(&mut self) -> IntersectionCurveEditor<'_, C, S> {
        IntersectionCurveEditor {
            surface0: &mut self.surface0,
            surface1: &mut self.surface1,
            leader: &mut self.leader,
            tol: &mut self.tol,
        }
    }
    /// Change leader.
    #[doc(hidden)]
    #[inline(always)]
    pub fn change_leader<D>(self, f: impl FnOnce(C) -> D) -> IntersectionCurve<D, S> {
        IntersectionCurve {
            surface0: self.surface0,
            surface1: self.surface1,
            leader: f(self.leader),
            tol: self.tol,
        }
    }
    /// The tolerance for generating this intersection curve.
    #[inline(always)]
    pub fn tolerance(&self) -> f64 { self.tol }
    /// Creates intersection curve with unchecked bound. This method is only for developer of `truck`, deplicated for users.
    #[inline(always)]
    pub fn new_unchecked(surface0: Box<S>, surface1: Box<S>, leader: C, tol: f64) -> Self {
        Self {
            surface0,
            surface1,
            leader,
            tol,
        }
    }
}

impl<C, S> IntersectionCurve<C, S>
where
    C: ParametricCurve3D,
    S: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    /// Search triple value of the point corresponding to the parameter `t`.
    /// - the coordinate on 3D space
    /// - the uv coordinate on `self.surface0()`
    /// - the uv coordinate on `self.surface1()`
    #[inline(always)]
    pub fn search_triple(&self, t: f64) -> Option<(Point3, Point2, Point2)> {
        double_projection(
            self.surface0(),
            None,
            self.surface1(),
            None,
            self.leader.subs(t),
            self.leader.der(t),
            100,
        )
    }
}

impl<C, S> ParametricCurve for IntersectionCurve<C, S>
where
    C: ParametricCurve3D,
    S: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    type Point = Point3;
    type Vector = Vector3;
    fn subs(&self, t: f64) -> Point3 { self.search_triple(t).unwrap().0 }
    fn der(&self, t: f64) -> Vector3 {
        let n = self.leader.der(t);
        let (_, p0, p1) = self.search_triple(t).unwrap();
        let d = self
            .surface0
            .normal(p0.x, p0.y)
            .cross(self.surface1.normal(p1.x, p1.y))
            .normalize();
        d * (n.dot(n) / d.dot(n))
    }
    /// This method is unimplemented! Should panic!!
    #[inline(always)]
    fn der2(&self, _: f64) -> Vector3 {
        unimplemented!();
    }
}

impl<C, S> BoundedCurve for IntersectionCurve<C, S>
where
    C: ParametricCurve3D + BoundedCurve,
    S: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    #[inline(always)]
    fn parameter_range(&self) -> (f64, f64) { self.leader.parameter_range() }
}

impl<C, S> ParameterDivision1D for IntersectionCurve<C, S>
where
    C: ParametricCurve3D,
    S: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    type Point = Point3;
    #[inline(always)]
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Point3>) {
        algo::curve::parameter_division(self, range, tol)
    }
}

impl<C, S> Cut for IntersectionCurve<C, S>
where
    C: Cut<Point = Point3, Vector = Vector3>,
    S: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    #[inline(always)]
    fn cut(&mut self, t: f64) -> Self {
        Self {
            surface0: self.surface0.clone(),
            surface1: self.surface1.clone(),
            leader: self.leader.cut(t),
            tol: self.tol,
        }
    }
}

impl<C: Invertible, S: Clone> Invertible for IntersectionCurve<C, S> {
    fn invert(&mut self) { self.leader.invert(); }
}

impl<C, S> SearchParameter<D1> for IntersectionCurve<C, S>
where
    C: ParametricCurve3D + SearchNearestParameter<D1, Point = Point3>,
    S: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    type Point = Point3;
    fn search_parameter<H: Into<SPHint1D>>(
        &self,
        point: Point3,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        let t = self
            .leader()
            .search_nearest_parameter(point, hint, trials)
            .unwrap();
        let pt = self.subs(t);
        match pt.near(&point) {
            true => Some(t),
            false => None,
        }
    }
}

/// Only derive from leading curve. Not precise.
impl<C, S> SearchNearestParameter<D1> for IntersectionCurve<C, S>
where
    C: ParametricCurve3D + SearchNearestParameter<D1, Point = Point3>,
    S: ParametricSurface3D + SearchNearestParameter<D2, Point = Point3>,
{
    type Point = Point3;
    fn search_nearest_parameter<H: Into<SPHint1D>>(
        &self,
        point: Point3,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        self.leader().search_nearest_parameter(point, hint, trials)
    }
}

impl<C, S> Transformed<Matrix4> for IntersectionCurve<C, S>
where
    C: Transformed<Matrix4>,
    S: Transformed<Matrix4>,
{
    fn transform_by(&mut self, trans: Matrix4) {
        self.surface0.transform_by(trans);
        self.surface1.transform_by(trans);
        self.leader.transform_by(trans);
        self.tol *= trans.norm_l2();
    }
}

impl<C: BoundedCurve> IntersectionCurve<C, Plane> {
    /// Optimizes intersection curve of [`Plane`] into [`Line`].
    #[inline]
    pub fn optimize(&self) -> Line<C::Point> {
        let (s, t) = self.leader.parameter_range();
        Line(self.leader.subs(s), self.leader.subs(t))
    }
}
