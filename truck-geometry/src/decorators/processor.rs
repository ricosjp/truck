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
    /// Transforms the geometry entity by `transform`
    #[inline(always)]
    pub fn transformed_by(&mut self, transform: T)
    where T: Mul<T, Output = T> + Copy {
        self.transform = transform * self.transform;
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

impl<C: Curve, T> Processor<C, T> {
    #[inline(always)]
    fn get_curve_parameter(&self, t: f64) -> f64 {
        let (t0, t1) = self.parameter_range();
        match self.orientation {
            true => t,
            false => t0 + t1 - t,
        }
    }
}

impl<C, T> Curve for Processor<C, T>
where
    C: Curve,
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
    fn parameter_range(&self) -> (f64, f64) { self.entity.parameter_range() }
}

impl<S, T> ParametricSurface for Processor<S, T>
where
    S: ParametricSurface<Point = Point3, Vector = Vector3>,
    T: Transform<S::Point> + Clone,
{
    type Point = S::Point;
    type Vector = S::Vector;
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
        a.cross(b).normalize()
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

fn get_axis(n: Vector3) -> (Vector3, Vector3) {
    let min = if n[0].abs() < n[1].abs() { 0 } else { 1 };
    let min = if n[min].abs() < n[2].abs() { min } else { 2 };
    let mut a = Vector3::zero();
    a[(min + 1) % 3] = -n[(min + 2) % 3];
    a[(min + 2) % 3] = n[(min + 1) % 3];
    let a = a.normalize();
    (a, n.cross(a))
}

#[test]
fn get_axis_test() {
    const N: usize = 100;
    for _ in 0..N {
        let n = Vector3::new(
            2.0 * rand::random::<f64>() - 1.0,
            2.0 * rand::random::<f64>() - 1.0,
            2.0 * rand::random::<f64>() - 1.0,
        );
        if n.so_small() {
            continue;
        }
        let n = n.normalize();
        let (a, b) = get_axis(n);
        f64::assert_near2(&a.magnitude2(), &1.0);
        f64::assert_near2(&b.magnitude2(), &1.0);
        Vector3::assert_near(&a.cross(b), &n)
    }
}
