use super::*;

impl<E, T: One> Processor<E, T> {
    /// Creates new processor
    #[inline(always)]
    pub fn new(entity: E) -> Processor<E, T> {
        Processor {
            entity,
            transform: T::one(),
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
    pub fn constract(self) -> E
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
    fn parameter_range(&self) -> (Bound<f64>, Bound<f64>) { self.entity.parameter_range() }
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

impl<S, T> ParametricSurface for Processor<S, T>
where
    S: ParametricSurface,
    S::Point: EuclideanSpace<Scalar = f64, Diff = S::Vector>,
    T: Transform<S::Point> + SquareMatrix<Scalar = f64> + Clone,
{
    type Point = S::Point;
    type Vector = S::Vector;
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
    #[inline(always)]
    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) { self.entity.parameter_range() }
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
        let n = a[0][0] * a[0][0]
            + a[0][1] * a[0][1]
            + a[0][2] * a[0][2]
            + a[1][0] * a[1][0]
            + a[1][1] * a[1][1]
            + a[1][2] * a[1][2]
            + a[2][0] * a[2][0]
            + a[2][1] * a[2][1]
            + a[2][2] * a[2][2];
        let (mut params, mut points) = self.entity.parameter_division(range, tol / n.sqrt());
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
        let n = a[0][0] * a[0][0]
            + a[0][1] * a[0][1]
            + a[0][2] * a[0][2]
            + a[0][3] * a[0][3]
            + a[1][0] * a[1][0]
            + a[1][1] * a[1][1]
            + a[1][2] * a[1][2]
            + a[1][3] * a[1][3]
            + a[2][0] * a[2][0]
            + a[2][1] * a[2][1]
            + a[2][2] * a[2][2]
            + a[2][3] * a[2][3]
            + a[3][0] * a[3][0]
            + a[3][1] * a[3][1]
            + a[3][2] * a[3][2]
            + a[3][3] * a[3][3];
        let (mut params, mut points) = self.entity.parameter_division(range, tol / n.sqrt());
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
        let n = a[0][0] * a[0][0]
            + a[0][1] * a[0][1]
            + a[0][2] * a[0][2]
            + a[1][0] * a[1][0]
            + a[1][1] * a[1][1]
            + a[1][2] * a[1][2]
            + a[2][0] * a[2][0]
            + a[2][1] * a[2][1]
            + a[2][2] * a[2][2];
        let (udiv, vdiv) = self.entity.parameter_division(range, tol / n.sqrt());
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
        let n = a[0][0] * a[0][0]
            + a[0][1] * a[0][1]
            + a[0][2] * a[0][2]
            + a[1][0] * a[1][0]
            + a[1][1] * a[1][1]
            + a[1][2] * a[1][2]
            + a[2][0] * a[2][0]
            + a[2][1] * a[2][1]
            + a[2][2] * a[2][2];
        let (udiv, vdiv) = self.entity.parameter_division(range, tol / n.sqrt());
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

#[cfg(test)]
mod tests {
    use super::*;

    fn exec_compatible_with_bspcurve() {
        const DEGREE: usize = 3;
        const DIVISION: usize = 4;
        let knot_vec = KnotVec::uniform_knot(DEGREE, DIVISION);
        let control_points: Vec<Point3> = (0..DEGREE + DIVISION)
            .map(|i| Point3::new(i as f64, 20.0 * rand::random::<f64>() - 10.0, 0.0))
            .collect();
        let mut curve = BSplineCurve::new(knot_vec, control_points);
        let mut processor = Processor::new(curve.clone());
        let mat = Matrix3::new(
            4.0 * rand::random::<f64>() - 2.0,
            4.0 * rand::random::<f64>() - 2.0,
            4.0 * rand::random::<f64>() - 2.0,
            4.0 * rand::random::<f64>() - 2.0,
            4.0 * rand::random::<f64>() - 2.0,
            4.0 * rand::random::<f64>() - 2.0,
            4.0 * rand::random::<f64>() - 2.0,
            4.0 * rand::random::<f64>() - 2.0,
            4.0 * rand::random::<f64>() - 2.0,
        );
        if mat.determinant().so_small() {
            println!("omitted: {mat:?}");
            return;
        }
        curve.transform_by(mat);
        processor.transform_by(mat);
        assert_eq!(curve.parameter_range(), processor.parameter_range());

        const N: usize = 100;
        for i in 0..=N {
            let t = i as f64 / N as f64;
            assert_near!(ParametricCurve::subs(&curve, t), processor.subs(t));
            assert_near!(ParametricCurve::der(&curve, t), processor.der(t));
            assert_near!(ParametricCurve::der2(&curve, t), processor.der2(t));
        }

        curve.invert();
        processor.invert();
        assert_eq!(curve.parameter_range(), processor.parameter_range());
        for i in 0..=N {
            let t = i as f64 / N as f64;
            assert_near!(ParametricCurve::subs(&curve, t), processor.subs(t));
            assert_near!(ParametricCurve::der(&curve, t), processor.der(t));
            assert_near!(ParametricCurve::der2(&curve, t), processor.der2(t));
        }
    }

    #[test]
    fn compatible_with_bspcurve() { (0..10).for_each(|_| exec_compatible_with_bspcurve()) }

    fn exec_compatible_with_bspsurface() {
        const DEGREE: usize = 3;
        const DIVISION: usize = 4;
        let knot_vec = KnotVec::uniform_knot(DEGREE, DIVISION);
        let knot_vecs = (knot_vec.clone(), knot_vec);
        let control_points: Vec<Vec<Point3>> = (0..DEGREE + DIVISION)
            .map(|i| {
                (0..DEGREE + DIVISION)
                    .map(|j| Point3::new(i as f64, j as f64, 20.0 * rand::random::<f64>() - 10.0))
                    .collect()
            })
            .collect();
        let mut surface = BSplineSurface::new(knot_vecs, control_points);
        let mut processor = Processor::new(surface.clone());
        let mat = Matrix3::new(
            4.0 * rand::random::<f64>() - 2.0,
            4.0 * rand::random::<f64>() - 2.0,
            4.0 * rand::random::<f64>() - 2.0,
            4.0 * rand::random::<f64>() - 2.0,
            4.0 * rand::random::<f64>() - 2.0,
            4.0 * rand::random::<f64>() - 2.0,
            4.0 * rand::random::<f64>() - 2.0,
            4.0 * rand::random::<f64>() - 2.0,
            4.0 * rand::random::<f64>() - 2.0,
        );
        if mat.determinant().so_small() {
            println!("omitted: {mat:?}");
            return;
        }
        surface.transform_by(mat);
        processor.transform_by(mat);
        assert_eq!(surface.parameter_range(), processor.parameter_range());

        const N: usize = 30;
        for i in 0..=N {
            for j in 0..=N {
                let u = i as f64 / N as f64;
                let v = j as f64 / N as f64;
                let pt0 = ParametricSurface::subs(&surface, u, v);
                let pt1 = processor.subs(u, v);
                assert_near!(pt0, pt1);
                let uder0 = surface.uder(u, v);
                let uder1 = processor.uder(u, v);
                assert_near!(uder0, uder1);
                let vder0 = surface.vder(u, v);
                let vder1 = processor.vder(u, v);
                assert_near!(vder0, vder1);
                let uuder0 = surface.uuder(u, v);
                let uuder1 = processor.uuder(u, v);
                assert_near!(uuder0, uuder1);
                let uvder0 = surface.uvder(u, v);
                let uvder1 = processor.uvder(u, v);
                assert_near!(uvder0, uvder1);
                let vvder0 = surface.vvder(u, v);
                let vvder1 = processor.vvder(u, v);
                assert_near!(vvder0, vvder1);
                let n0 = surface.normal(u, v);
                let n1 = processor.normal(u, v);
                assert_near!(n0, n1);
            }
        }

        surface.swap_axes();
        processor.invert();
        assert_eq!(surface.parameter_range(), processor.parameter_range());
        for i in 0..=N {
            for j in 0..=N {
                let u = i as f64 / N as f64;
                let v = j as f64 / N as f64;
                let pt0 = ParametricSurface::subs(&surface, u, v);
                let pt1 = processor.subs(u, v);
                assert_near!(pt0, pt1);
                let uder0 = surface.uder(u, v);
                let uder1 = processor.uder(u, v);
                assert_near!(uder0, uder1);
                let vder0 = surface.vder(u, v);
                let vder1 = processor.vder(u, v);
                assert_near!(vder0, vder1);
                let uuder0 = surface.uuder(u, v);
                let uuder1 = processor.uuder(u, v);
                assert_near!(uuder0, uuder1);
                let uvder0 = surface.uvder(u, v);
                let uvder1 = processor.uvder(u, v);
                assert_near!(uvder0, uvder1);
                let vvder0 = surface.vvder(u, v);
                let vvder1 = processor.vvder(u, v);
                assert_near!(vvder0, vvder1);
                let n0 = surface.normal(u, v);
                let n1 = processor.normal(u, v);
                assert_near!(n0, n1);
            }
        }
    }

    #[test]
    fn compatible_with_bspsurface() { (0..3).for_each(|_| exec_compatible_with_bspsurface()) }
}
