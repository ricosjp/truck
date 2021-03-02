use super::Invertible;
use super::*;

impl<E> Invertible<E> {
    /// Creats a new invertible element.
    #[inline(always)]
    pub fn new(entity: E) -> Self {
        Invertible {
            entity,
            orientation: true,
        }
    }
}

impl<C: Curve> Invertible<C> {
    #[inline(always)]
    fn get_curve_parameter(&self, t: f64) -> f64 {
        let (t0, t1) = self.entity.parameter_range();
        if self.orientation {
            t
        } else {
            t0 + t1 - t
        }
    }
}

impl<S: BoundedSurface> Invertible<S> {
    #[inline(always)]
    fn get_parameter(&self, v: f64) -> f64 {
        let (_, (v0, v1)) = self.entity.parameter_range();
        if self.orientation {
            v
        } else {
            v0 + v1 - v
        }
    }
}

impl<C> Curve for Invertible<C>
where
    C: Curve,
    C::Vector: VectorSpace<Scalar = f64>,
{
    type Point = C::Point;
    type Vector = C::Vector;
    #[inline(always)]
    fn subs(&self, t: f64) -> Self::Point {
        let t = self.get_curve_parameter(t);
        self.entity.subs(t)
    }
    #[inline(always)]
    fn der(&self, t: f64) -> Self::Vector {
        let t = self.get_curve_parameter(t);
        self.entity.der(t) * if self.orientation { 1.0 } else { -1.0 }
    }
    #[inline(always)]
    fn parameter_range(&self) -> (f64, f64) { self.entity.parameter_range() }
}

impl<S> ParametricSurface for Invertible<S>
where
    S: BoundedSurface,
    S::Vector: VectorSpace<Scalar = f64>,
{
    type Point = S::Point;
    type Vector = S::Vector;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Self::Point {
        let v = self.get_parameter(v);
        self.entity.subs(u, v)
    }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Self::Vector {
        let v = self.get_parameter(v);
        self.entity.uder(u, v)
    }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Self::Vector {
        let v = self.get_parameter(v);
        self.vder(u, v) * if self.orientation { 1.0 } else { -1.0 }
    }
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Self::Vector {
        let v = self.get_parameter(v);
        self.normal(u, v) * if self.orientation { 1.0 } else { -1.0 }
    }
}

impl<E: Clone> truck_base::geom_traits::Invertible for Invertible<E> {
    #[inline(always)]
    fn inverse(&self) -> Self {
        Self {
            entity: self.entity.clone(),
            orientation: !self.orientation,
        }
    }
}
