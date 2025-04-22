use super::*;
use algo::surface::SsnpVector;

impl<E, T: One> Processor<E, T> {
    /// Creates new processor
    #[inline(always)]
    pub fn new(entity: E) -> Self {
        Self {
            entity,
            transform: T::one(),
            orientation: true,
        }
    }

    /// Creates new transformed processor
    #[inline(always)]
    pub const fn with_transform(entity: E, transform: T) -> Self {
        Self {
            entity,
            transform,
            orientation: true,
        }
    }

    /// Returns the reference of entity
    #[inline(always)]
    pub const fn entity(&self) -> &E { &self.entity }

    /// Returns the reference of transform
    #[inline(always)]
    pub const fn transform(&self) -> &T { &self.transform }

    /// Returns the orientation of surface
    #[inline(always)]
    pub const fn orientation(&self) -> bool { self.orientation }

    #[inline(always)]
    fn sign(&self) -> f64 {
        match self.orientation {
            true => 1.0,
            false => -1.0,
        }
    }

    /// apply the function to the entity geometry
    #[inline(always)]
    pub fn map<G, F: FnOnce(E) -> G>(self, f: F) -> Processor<G, T> {
        Processor {
            entity: f(self.entity),
            transform: self.transform,
            orientation: self.orientation,
        }
    }

    /// apply the function to the entity geometry
    #[inline(always)]
    pub fn map_ref<G, F: FnOnce(&E) -> G>(&self, f: F) -> Processor<G, T>
    where T: Copy {
        Processor {
            entity: f(&self.entity),
            transform: self.transform,
            orientation: self.orientation,
        }
    }

    /// apply the transform and inverse
    pub fn contract(self) -> E
    where E: Transformed<T> + Invertible {
        let mut res = self.entity;
        res.transform_by(self.transform);
        if !self.orientation {
            res.invert();
        }
        res
    }
}

impl<E: Clone, T: Clone> Invertible for Processor<E, T> {
    #[inline(always)]
    fn invert(&mut self) { self.orientation = !self.orientation; }
    #[inline(always)]
    fn inverse(&self) -> Self {
        Processor {
            entity: self.entity.clone(),
            transform: self.transform.clone(),
            orientation: !self.orientation,
        }
    }
}

impl<C: BoundedCurve, T> Processor<C, T> {
    #[inline(always)]
    fn get_curve_parameter(&self, t: f64) -> f64 {
        let (t0, t1) = self.range_tuple();
        match self.orientation {
            true => t,
            false => t0 + t1 - t,
        }
    }
}

impl<C, T> ParametricCurve for Processor<C, T>
where
    C: BoundedCurve,
    C::Point: EuclideanSpace<Diff = C::Vector>,
    C::Vector: VectorSpace<Scalar = f64>,
    T: Transform<C::Point> + Clone,
{
    type Point = C::Point;
    type Vector = C::Vector;
    #[inline(always)]
    fn der_n(&self, t: f64, n: usize) -> Self::Vector {
        if n == 0 {
            self.subs(t).to_vec()
        } else {
            let t = self.get_curve_parameter(t);
            self.transform.transform_vector(self.entity.der_n(t, n))
        }
    }
    #[inline(always)]
    fn subs(&self, t: f64) -> C::Point {
        let t = self.get_curve_parameter(t);
        self.transform.transform_point(self.entity.subs(t))
    }
    #[inline(always)]
    fn der(&self, t: f64) -> Self::Vector {
        let t = self.get_curve_parameter(t);
        self.transform.transform_vector(self.entity.der(t)) * self.sign()
    }
    #[inline(always)]
    fn der2(&self, t: f64) -> Self::Vector {
        let t = self.get_curve_parameter(t);
        self.transform.transform_vector(self.entity.der2(t))
    }
    #[inline(always)]
    fn parameter_range(&self) -> ParameterRange { self.entity.parameter_range() }
    #[inline(always)]
    fn period(&self) -> Option<f64> { self.entity.period() }
}

impl<C, T> BoundedCurve for Processor<C, T>
where
    C: BoundedCurve,
    C::Point: EuclideanSpace<Diff = C::Vector>,
    C::Vector: VectorSpace<Scalar = f64>,
    T: Transform<C::Point> + Clone,
{
}

impl<C, T> Cut for Processor<C, T>
where
    C: BoundedCurve + Cut,
    C::Point: EuclideanSpace<Diff = C::Vector>,
    C::Vector: VectorSpace<Scalar = f64>,
    T: Transform<C::Point> + Clone,
{
    fn cut(&mut self, t: f64) -> Self {
        let t = self.get_curve_parameter(t);
        let mut entity = self.entity.cut(t);
        if !self.orientation {
            std::mem::swap(&mut entity, &mut self.entity);
        }
        Self {
            entity,
            transform: self.transform.clone(),
            orientation: self.orientation,
        }
    }
}

impl<S, T> ParametricSurface for Processor<S, T>
where
    S: ParametricSurface,
    S::Point: EuclideanSpace<Scalar = f64, Diff = S::Vector>,
    T: Transform<S::Point> + SquareMatrix<Scalar = f64> + Clone,
{
    type Point = S::Point;
    type Vector = S::Vector;
    #[inline(always)]
    fn der_mn(&self, u: f64, v: f64, m: usize, n: usize) -> Self::Vector {
        if (m, n) == (0, 0) {
            self.subs(u, v).to_vec()
        } else {
            match self.orientation {
                true => self
                    .transform
                    .transform_vector(self.entity.der_mn(u, v, m, n)),
                false => self
                    .transform
                    .transform_vector(self.entity.der_mn(v, u, n, m)),
            }
        }
    }
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Self::Point {
        match self.orientation {
            true => self.transform.transform_point(self.entity.subs(u, v)),
            false => self.transform.transform_point(self.entity.subs(v, u)),
        }
    }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Self::Vector {
        match self.orientation {
            true => self.transform.transform_vector(self.entity.uder(u, v)),
            false => self.transform.transform_vector(self.entity.vder(v, u)),
        }
    }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Self::Vector {
        match self.orientation {
            true => self.transform.transform_vector(self.entity.vder(u, v)),
            false => self.transform.transform_vector(self.entity.uder(v, u)),
        }
    }
    #[inline(always)]
    fn uuder(&self, u: f64, v: f64) -> Self::Vector {
        match self.orientation {
            true => self.transform.transform_vector(self.entity.uuder(u, v)),
            false => self.transform.transform_vector(self.entity.vvder(v, u)),
        }
    }
    #[inline(always)]
    fn uvder(&self, u: f64, v: f64) -> Self::Vector {
        match self.orientation {
            true => self.transform.transform_vector(self.entity.uvder(u, v)),
            false => self.transform.transform_vector(self.entity.uvder(v, u)),
        }
    }
    #[inline(always)]
    fn vvder(&self, u: f64, v: f64) -> Self::Vector {
        match self.orientation {
            true => self.transform.transform_vector(self.entity.vvder(u, v)),
            false => self.transform.transform_vector(self.entity.uuder(v, u)),
        }
    }
    #[inline(always)]
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) {
        let (urange, vrange) = self.entity.parameter_range();
        match self.orientation {
            true => (urange, vrange),
            false => (vrange, urange),
        }
    }
    #[inline(always)]
    fn u_period(&self) -> Option<f64> {
        match self.orientation {
            true => self.entity.u_period(),
            false => self.entity.v_period(),
        }
    }
    #[inline(always)]
    fn v_period(&self) -> Option<f64> {
        match self.orientation {
            true => self.entity.v_period(),
            false => self.entity.u_period(),
        }
    }
}

impl<S, T> ParametricSurface3D for Processor<S, T>
where
    S: ParametricSurface3D,
    T: Transform<Point3> + SquareMatrix<Scalar = f64> + Clone,
{
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Self::Vector {
        let transtrans = self.transform.transpose();
        let n = match self.orientation {
            true => self.entity.normal(u, v),
            false => -self.entity.normal(v, u),
        };
        let n = transtrans
            .inverse_transform_vector(n)
            .expect("invalid transform");
        (n / self.transform.determinant()).normalize()
    }
}

impl<S, T> BoundedSurface for Processor<S, T>
where
    S: BoundedSurface<Point = Point3, Vector = Vector3>,
    T: Transform<S::Point> + SquareMatrix<Scalar = f64> + Clone,
{
}

impl<E, T> Deref for Processor<E, T> {
    type Target = E;
    #[inline(always)]
    fn deref(&self) -> &E { &self.entity }
}

impl<E, T> DerefMut for Processor<E, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut E { &mut self.entity }
}

impl<E, T> Transformed<T> for Processor<E, T>
where
    T: Mul<T, Output = T> + Copy,
    E: Clone,
{
    #[inline(always)]
    fn transform_by(&mut self, trans: T) { self.transform = trans * self.transform; }
    #[inline(always)]
    fn transformed(&self, trans: T) -> Self {
        Self {
            entity: self.entity.clone(),
            transform: trans * self.transform,
            orientation: self.orientation,
        }
    }
}

impl<E, T, C> IncludeCurve<C> for Processor<E, T>
where
    C: ParametricCurve + Transformed<T> + Clone,
    C::Point: EuclideanSpace,
    E: IncludeCurve<C>,
    T: Transform<C::Point>,
{
    fn include(&self, curve: &C) -> bool {
        let inv = self
            .transform
            .inverse_transform()
            .expect("irregular transform");
        let curve = curve.clone().transformed(inv);
        self.entity.include(&curve)
    }
}

impl<C> ParameterDivision1D for Processor<C, Matrix3>
where C: ParameterDivision1D<Point = Point2> + BoundedCurve<Point = Point2>
{
    type Point = Point2;
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Self::Point>) {
        let a = self.transform;
        let range = match self.orientation {
            true => range,
            false => (
                self.get_curve_parameter(range.1),
                self.get_curve_parameter(range.0),
            ),
        };
        let (_, k, _) = a
            .iwasawa_decomposition()
            .expect("transform matrix must be invertible!");
        let n = f64::abs(k[0][0])
            .max(f64::abs(k[1][1]))
            .max(f64::abs(k[2][2]));
        let (mut params, mut points) = self.entity.parameter_division(range, tol / n);
        points
            .iter_mut()
            .for_each(|pt| *pt = a.transform_point(*pt));
        if !self.orientation {
            params
                .iter_mut()
                .for_each(|t| *t = self.get_curve_parameter(*t));
            points.reverse();
        }
        (params, points)
    }
}

impl<C> ParameterDivision1D for Processor<C, Matrix4>
where C: ParameterDivision1D<Point = Point3> + BoundedCurve<Point = Point3>
{
    type Point = Point3;
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Self::Point>) {
        let a = self.transform;
        let range = match self.orientation {
            true => range,
            false => (
                self.get_curve_parameter(range.1),
                self.get_curve_parameter(range.0),
            ),
        };
        let (_, k, _) = a
            .iwasawa_decomposition()
            .expect("transform matrix must be invertible!");
        let n = f64::abs(k[0][0])
            .max(f64::abs(k[1][1]))
            .max(f64::abs(k[2][2]))
            / f64::abs(k[3][3]);
        let (mut params, mut points) = self.entity.parameter_division(range, tol / n);
        points
            .iter_mut()
            .for_each(|pt| *pt = a.transform_point(*pt));
        if !self.orientation {
            params
                .iter_mut()
                .for_each(|t| *t = self.get_curve_parameter(*t));
            points.reverse();
        }
        (params, points)
    }
}

impl<S: ParameterDivision2D> ParameterDivision2D for Processor<S, Matrix3> {
    fn parameter_division(
        &self,
        range: ((f64, f64), (f64, f64)),
        tol: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        let a = self.transform;
        let range = match self.orientation {
            true => range,
            false => (range.1, range.0),
        };
        let (_, k, _) = a
            .iwasawa_decomposition()
            .expect("transform matrix must be invertible!");
        let n = f64::abs(k[0][0])
            .max(f64::abs(k[1][1]))
            .max(f64::abs(k[2][2]));
        let (udiv, vdiv) = self.entity.parameter_division(range, tol / n);
        match self.orientation {
            true => (udiv, vdiv),
            false => (vdiv, udiv),
        }
    }
}

impl<S: ParameterDivision2D> ParameterDivision2D for Processor<S, Matrix4> {
    fn parameter_division(
        &self,
        range: ((f64, f64), (f64, f64)),
        tol: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        let a = self.transform;
        let range = match self.orientation {
            true => range,
            false => (range.1, range.0),
        };
        let (_, k, _) = a
            .iwasawa_decomposition()
            .expect("transform matrix must be invertible!");
        let n = f64::abs(k[0][0])
            .max(f64::abs(k[1][1]))
            .max(f64::abs(k[2][2]))
            / f64::abs(k[3][3]);
        let (udiv, vdiv) = self.entity.parameter_division(range, tol / n);
        match self.orientation {
            true => (udiv, vdiv),
            false => (vdiv, udiv),
        }
    }
}

impl<E, T> SearchParameter<D1> for Processor<E, T>
where
    E: SearchParameter<D1> + BoundedCurve,
    <E as SearchParameter<D1>>::Point: EuclideanSpace,
    T: Transform<<E as SearchParameter<D1>>::Point>,
{
    type Point = <E as SearchParameter<D1>>::Point;
    fn search_parameter<H: Into<SPHint1D>>(
        &self,
        point: <E as SearchParameter<D1>>::Point,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        let inv = self.transform.inverse_transform().unwrap();
        let t = self
            .entity
            .search_parameter(inv.transform_point(point), hint, trials)?;
        Some(self.get_curve_parameter(t))
    }
}

impl<E, T> SearchParameter<D2> for Processor<E, T>
where
    E: SearchParameter<D2>,
    E::Point: EuclideanSpace,
    T: Transform<E::Point>,
{
    type Point = E::Point;
    fn search_parameter<H: Into<SPHint2D>>(
        &self,
        point: E::Point,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        let inv = self.transform.inverse_transform().unwrap();
        let (u, v) = self
            .entity
            .search_parameter(inv.transform_point(point), hint, trials)?;
        match self.orientation {
            true => Some((u, v)),
            false => Some((v, u)),
        }
    }
}

impl<P, E, T> SearchNearestParameter<D1> for Processor<E, T>
where
    E: BoundedCurve<Point = P> + SearchNearestParameter<D1, Point = P>,
    P: EuclideanSpace<Scalar = f64, Diff = E::Vector>,
    E::Vector: InnerSpace<Scalar = f64> + Tolerance,
    T: Transform<P> + Clone,
{
    type Point = P;
    fn search_nearest_parameter<H: Into<SPHint1D>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        let inv = self.transform.inverse_transform().unwrap();
        let hint =
            self.entity
                .search_nearest_parameter(inv.transform_point(point), hint, trials)?;
        let hint = self.get_curve_parameter(hint);
        algo::curve::search_nearest_parameter(self, point, hint, trials)
    }
}

impl<P, E, T> SearchNearestParameter<D2> for Processor<E, T>
where
    E: ParametricSurface<Point = P> + SearchNearestParameter<D2, Point = P>,
    P: EuclideanSpace<Scalar = f64, Diff = E::Vector> + MetricSpace<Metric = f64> + Tolerance,
    E::Vector: SsnpVector<Point = P>,
    T: Transform<P> + SquareMatrix<Scalar = f64> + Clone,
{
    type Point = P;
    fn search_nearest_parameter<H: Into<SPHint2D>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        let inv = self.transform.inverse_transform().unwrap();
        let hint =
            self.entity
                .search_nearest_parameter(inv.transform_point(point), hint, trials)?;
        let hint = match self.orientation {
            true => hint,
            false => (hint.1, hint.0),
        };
        algo::surface::search_nearest_parameter(self, point, hint, trials)
    }
}

impl<E, T, U> ToSameGeometry<U> for Processor<E, T>
where
    E: ToSameGeometry<U>,
    T: Copy,
    U: Transformed<T> + Invertible,
{
    fn to_same_geometry(&self) -> U {
        let Self {
            entity,
            transform,
            orientation,
        } = self;
        let mut u = entity.to_same_geometry();
        u.transform_by(*transform);
        if !orientation {
            u.invert();
        }
        u
    }
}
