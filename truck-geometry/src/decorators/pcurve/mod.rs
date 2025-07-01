use super::*;
use truck_base::cgmath64::control_point::ControlPoint;
pub(crate) mod composition;

impl<C, S> PCurve<C, S> {
    /// Creates composited
    #[inline(always)]
    pub const fn new(curve: C, surface: S) -> PCurve<C, S> { PCurve { curve, surface } }

    /// Returns the reference to the parameter curve
    #[inline(always)]
    pub const fn curve(&self) -> &C { &self.curve }

    /// Returns the reference to the surface
    #[inline(always)]
    pub const fn surface(&self) -> &S { &self.surface }

    /// Decompose the pcurve into its parameter curve and surface.
    #[inline(always)]
    pub fn decompose(self) -> (C, S) { (self.curve, self.surface) }
}

impl<C, S> PCurve<C, S>
where
    C: ParametricCurve2D,
    S: ParametricSurface,
    S::Point: ControlPoint<f64, Diff = S::Vector>,
    S::Vector: VectorSpace<Scalar = f64> + ElementWise,
{
    fn der3(&self, t: f64) -> S::Vector {
        let Point2 { x: u, y: v } = self.curve.subs(t);
        let der = self.curve.der(t);
        let der2 = self.curve.der2(t);
        let der3 = self.curve.der_n(3, t);
        let surface = self.surface();
        surface.der_mn(3, 0, u, v) * (der[0] * der[0] * der[0])
            + surface.der_mn(2, 1, u, v) * (der[0] * der[0] * der[1] * 3.0)
            + surface.der_mn(1, 2, u, v) * (der[0] * der[1] * der[1] * 3.0)
            + surface.der_mn(0, 3, u, v) * (der[1] * der[1] * der[1])
            + surface.uuder(u, v) * (der2[0] * der[0] * 3.0)
            + surface.uvder(u, v) * ((der2[0] * der[1] + der2[1] * der[0]) * 3.0)
            + surface.vvder(u, v) * (der2[1] * der[1] * 3.0)
            + surface.uder(u, v) * der3[0]
            + surface.vder(u, v) * der3[1]
    }
}
impl<C, S> ParametricCurve for PCurve<C, S>
where
    C: ParametricCurve2D,
    S: ParametricSurface,
    S::Point: ControlPoint<f64, Diff = S::Vector>,
    S::Vector: VectorSpace<Scalar = f64> + ElementWise,
{
    type Point = S::Point;
    type Vector = S::Vector;
    fn der_n(&self, n: usize, t: f64) -> Self::Vector {
        use composition::*;
        if n >= 32 {
            panic!("the order of derivation must be under 32.");
        }
        match n {
            0 => return self.subs(t).to_vec(),
            1 => return self.der(t),
            2 => return self.der2(t),
            3 => return self.der3(t),
            _ => {}
        }
        let mut cder = [Vector2::zero(); 32];
        self.curve.ders(t, &mut cder[..=n]);
        let Vector2 { x: u, y: v } = cder[0];

        let mut sder = [[S::Vector::zero(); 32]; 32];
        let mut sder_mut = sder[0..=n]
            .iter_mut()
            .enumerate()
            .map(|(i, slice)| &mut slice[0..=(n - i)])
            .collect::<Vec<_>>();
        self.surface.ders(u, v, &mut sder_mut);

        (1..=n).fold(S::Vector::zero(), |sum, len| {
            let iter = CompositionIter::<32>::try_new(n, len).unwrap();
            iter.fold(sum, |sum, idx| {
                let idx = &idx[..len];
                sum + tensor(&sder, &cder, idx) * multiplicity(idx) as f64
            })
        })
    }
    fn ders(&self, t: f64, out: &mut [Self::Vector]) {
        use composition::*;

        if out.is_empty() {
            return;
        }
        let n = out.len() - 1;
        if n >= 32 {
            panic!("the order of derivation must be under 32.");
        }
        let mut cder = [Vector2::zero(); 32];
        self.curve.ders(t, &mut cder[..=n]);
        let Vector2 { x: u, y: v } = cder[0];

        let mut sder = [[S::Vector::zero(); 32]; 32];
        let mut sder_mut = sder[0..=n]
            .iter_mut()
            .enumerate()
            .map(|(i, slice)| &mut slice[0..=(n - i)])
            .collect::<Vec<_>>();
        self.surface.ders(u, v, &mut sder_mut);
        out[0] = sder[0][0];

        (1..=n).for_each(|m| {
            out[m] = (1..=m).fold(S::Vector::zero(), |sum, len| {
                let iter = CompositionIter::<32>::try_new(m, len).unwrap();
                iter.fold(sum, |sum, idx| {
                    let idx = &idx[..len];
                    sum + tensor(&sder, &cder, idx) * multiplicity(idx) as f64
                })
            });
        });
    }
    fn ders_vec(&self, n: usize, t: f64) -> Vec<Self::Vector> {
        let mut res = vec![Self::Vector::zero(); n + 1];
        self.ders(t, &mut res);
        res
    }
    #[inline(always)]
    fn subs(&self, t: f64) -> Self::Point {
        let pt = self.curve.subs(t);
        self.surface.subs(pt[0], pt[1])
    }
    #[inline(always)]
    fn der(&self, t: f64) -> Self::Vector {
        let pt = self.curve.subs(t);
        let der = self.curve.der(t);
        self.surface.uder(pt[0], pt[1]) * der[0] + self.surface.vder(pt[0], pt[1]) * der[1]
    }
    #[inline(always)]
    fn der2(&self, t: f64) -> Self::Vector {
        let pt = self.curve.subs(t);
        let der = self.curve.der(t);
        let der2 = self.curve.der2(t);
        self.surface.uuder(pt[0], pt[1]) * (der[0] * der[0])
            + self.surface.uvder(pt[0], pt[1]) * (der[0] * der[1] * 2.0)
            + self.surface.vvder(pt[0], pt[1]) * (der[1] * der[1])
            + self.surface.uder(pt[0], pt[1]) * der2[0]
            + self.surface.vder(pt[0], pt[1]) * der2[1]
    }
    #[inline(always)]
    fn parameter_range(&self) -> ParameterRange { self.curve.parameter_range() }
    #[inline(always)]
    fn period(&self) -> Option<f64> { self.curve.period() }
}

impl<C, S> BoundedCurve for PCurve<C, S>
where
    C: BoundedCurve,
    PCurve<C, S>: ParametricCurve,
{
}

impl<C, S> Cut for PCurve<C, S>
where
    C: Cut,
    S: Clone,
    PCurve<C, S>: ParametricCurve,
{
    fn cut(&mut self, t: f64) -> Self {
        let curve = self.curve.cut(t);
        Self {
            curve,
            surface: self.surface.clone(),
        }
    }
}

impl<C, S> SearchParameter<D1> for PCurve<C, S>
where
    C: ParametricCurve2D + SearchParameter<D1, Point = Point2>,
    S: SearchParameter<D2>,
{
    type Point = <S as SearchParameter<D2>>::Point;
    fn search_parameter<H: Into<SPHint1D>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
        let hint = hint.into();
        let shint = match hint {
            SPHint1D::Parameter(hint) => {
                let p = self.curve.subs(hint);
                SPHint2D::Parameter(p.x, p.y)
            }
            SPHint1D::Range(x, y) => {
                let p = self.curve.subs(y);
                let ranges = (0..PRESEARCH_DIVISION).fold(
                    ((p.x, p.x), (p.y, p.y)),
                    |((x0, x1), (y0, y1)), i| {
                        let t = x + (y - x) * i as f64 / PRESEARCH_DIVISION as f64;
                        let p = self.curve.subs(t);
                        (
                            (f64::min(x0, p.x), f64::max(x1, p.x)),
                            (f64::min(y0, p.y), f64::max(y1, p.y)),
                        )
                    },
                );
                SPHint2D::Range(ranges.0, ranges.1)
            }
            SPHint1D::None => SPHint2D::None,
        };
        let (x, y) = self.surface.search_parameter(point, shint, trials)?;
        self.curve.search_parameter(Point2::new(x, y), hint, trials)
    }
}

impl<C, S> SearchNearestParameter<D1> for PCurve<C, S>
where
    Self: BoundedCurve,
    <Self as ParametricCurve>::Point: EuclideanSpace<Scalar = f64, Diff = <Self as ParametricCurve>::Vector>
        + MetricSpace<Metric = f64>,
    <Self as ParametricCurve>::Vector: InnerSpace<Scalar = f64> + Tolerance,
{
    type Point = <Self as ParametricCurve>::Point;
    fn search_nearest_parameter<H: Into<SPHint1D>>(
        &self,
        point: Self::Point,
        hint: H,
        trials: usize,
    ) -> Option<f64> {
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

impl<C, S> ParameterDivision1D for PCurve<C, S>
where
    C: ParametricCurve2D,
    S: ParametricSurface,
    S::Point: EuclideanSpace<Scalar = f64, Diff = S::Vector>
        + MetricSpace<Metric = f64>
        + HashGen<f64>
        + ControlPoint<f64, Diff = S::Vector>,
    S::Vector: VectorSpace<Scalar = f64> + ElementWise,
{
    type Point = S::Point;
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<S::Point>) {
        algo::curve::parameter_division(self, range, tol)
    }
}

impl<C, S> Invertible for PCurve<C, S>
where
    C: Invertible,
    S: Clone,
{
    fn invert(&mut self) { self.curve.invert() }
}

impl<C: Clone, S: Clone, T> Transformed<T> for PCurve<C, S>
where S: Transformed<T>
{
    #[inline(always)]
    fn transform_by(&mut self, trans: T) { self.surface.transform_by(trans); }
    fn transformed(&self, trans: T) -> Self {
        Self {
            curve: self.curve.clone(),
            surface: self.surface.transformed(trans),
        }
    }
}
