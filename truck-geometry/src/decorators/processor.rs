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
    pub fn entity(&self) -> &E { &self.entity }
    #[inline(always)]
    fn sign(&self) -> f64 {
        match self.orientation {
            true => 1.0,
            false => -1.0,
        }
    }
    #[inline(always)]
    fn get_surface_parameter(&self, u: f64, v: f64) -> (f64, f64) {
        match self.orientation {
            true => (u, v),
            false => (v, u),
        }
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

impl<C: ParametricCurve, T> Processor<C, T> {
    #[inline(always)]
    fn get_curve_parameter(&self, t: f64) -> f64 {
        let (t0, t1) = self.parameter_range();
        match self.orientation {
            true => t,
            false => t0 + t1 - t,
        }
    }
}

impl<C, T> ParametricCurve for Processor<C, T>
where
    C: ParametricCurve,
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
    fn parameter_range(&self) -> (f64, f64) { self.entity.parameter_range() }
}

impl<S, T> ParametricSurface for Processor<S, T>
where
    S: ParametricSurface<Point = Point3, Vector = Vector3>,
    T: Transform<S::Point> + Clone,
{
    type Point = Point3;
    type Vector = Vector3;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Self::Point {
        let (u, v) = self.get_surface_parameter(u, v);
        self.transform.transform_point(self.entity.subs(u, v))
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
    fn normal(&self, u: f64, v: f64) -> Self::Vector {
        let (u, v) = self.get_surface_parameter(u, v);
        let n = self.entity.normal(u, v);
        let (a, b) = get_axis(n);
        let a = self.transform.transform_vector(a);
        let b = self.transform.transform_vector(b);
        let normal = a.cross(b).normalize();
        match self.orientation {
            true => normal,
            false => -normal,
        }
    }
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
where T: Mul<T, Output = T> + Copy
{
    #[inline(always)]
    fn transform_by(&mut self, trans: T) { self.transform = trans * self.transform; }
    #[inline(always)]
    fn transformed(self, trans: T) -> Self {
        Self {
            entity: self.entity,
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

fn get_axis(n: Vector3) -> (Vector3, Vector3) {
    let min = if n[0].abs() < n[1].abs() { 0 } else { 1 };
    let min = if n[min].abs() < n[2].abs() { min } else { 2 };
    let mut a = Vector3::zero();
    a[(min + 1) % 3] = -n[(min + 2) % 3];
    a[(min + 2) % 3] = n[(min + 1) % 3];
    let a = a.normalize();
    (a, n.cross(a))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exec_get_axis_test() {
        let n = Vector3::new(
            2.0 * rand::random::<f64>() - 1.0,
            2.0 * rand::random::<f64>() - 1.0,
            2.0 * rand::random::<f64>() - 1.0,
        );
        if n.so_small() {
            println!("ommited: {:?}", n);
            return;
        }
        let n = n.normalize();
        let (a, b) = get_axis(n);
        assert_near2!(a.magnitude2(), 1.0);
        assert_near2!(b.magnitude2(), 1.0);
        assert_near!(a.cross(b), n)
    }

    #[test]
    fn get_axis_test() { (0..100).for_each(|_| exec_get_axis_test()) }

    fn exec_compatible_with_bspcurve() {
        const DEGREE: usize = 3;
        const DIVISION: usize = 4;
        let knot_vec = KnotVec::uniform_knot(DEGREE, DIVISION);
        let control_points: Vec<Vector3> = (0..DEGREE + DIVISION)
            .map(|i| Vector3::new(i as f64, 20.0 * rand::random::<f64>() - 10.0, 0.0))
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
            println!("ommited: {:?}", mat);
            return;
        }
        curve = mat * curve;
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
    fn compatible_with_bspcurve() { (0..100).for_each(|_| exec_compatible_with_bspcurve()) }

    fn exec_compatible_with_bspsurface() {
        const DEGREE: usize = 3;
        const DIVISION: usize = 4;
        let knot_vec = KnotVec::uniform_knot(DEGREE, DIVISION);
        let knot_vecs = (knot_vec.clone(), knot_vec);
        let control_points: Vec<Vec<Vector3>> = (0..DEGREE + DIVISION)
            .map(|i| {
                (0..DEGREE + DIVISION)
                    .map(|j| Vector3::new(i as f64, j as f64, 20.0 * rand::random::<f64>() - 10.0))
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
            println!("ommited: {:?}", mat);
            return;
        }
        surface = mat * surface;
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
                let n0 = surface.normal(u, v);
                let n1 = processor.normal(u, v);
                assert_near!(n0, n1);
            }
        }
    }

    #[test]
    fn compatible_with_bspsurface() { (0..10).for_each(|_| exec_compatible_with_bspsurface()) }
}
