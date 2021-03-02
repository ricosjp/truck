use super::*;
use std::ops::{Deref, DerefMut, Mul};

impl<E, T> Transformed<E, T> {
    /// Creates new transformed geometric entity
    #[inline(always)]
    pub fn new(entity: E, transform: T) -> Self { Transformed { entity, transform } }
    /// Transforms the geometry entity by `transform`
    #[inline(always)]
    pub fn transformed_by(&mut self, transform: T)
    where T: Mul<T, Output = T> + Copy {
        self.transform = transform * self.transform;
    }
    /// Returns the reference of entity
    #[inline(always)]
    pub fn entity(&self) -> &E { &self.entity }
}

impl<C, T> Curve for Transformed<C, T>
where
    C: Curve,
    C::Point: EuclideanSpace<Diff = C::Vector>,
    T: Transform<C::Point> + Clone,
{
    type Point = C::Point;
    type Vector = C::Vector;
    #[inline(always)]
    fn subs(&self, t: f64) -> Self::Point { self.transform.transform_point(self.entity.subs(t)) }
    #[inline(always)]
    fn der(&self, t: f64) -> Self::Vector { self.transform.transform_vector(self.entity.der(t)) }
    #[inline(always)]
    fn parameter_range(&self) -> (f64, f64) { self.entity.parameter_range() }
}

impl<S, T> ParametricSurface for Transformed<S, T>
where
    S: ParametricSurface<Point = Point3, Vector = Vector3>,
    T: Transform3<Scalar = f64> + Clone,
{
    type Point = Point3;
    type Vector = Vector3;
    #[inline(always)]
    fn subs(&self, u: f64, v: f64) -> Point3 {
        self.transform.transform_point(self.entity.subs(u, v))
    }
    #[inline(always)]
    fn uder(&self, u: f64, v: f64) -> Vector3 {
        self.transform.transform_vector(self.entity.uder(u, v))
    }
    #[inline(always)]
    fn vder(&self, u: f64, v: f64) -> Vector3 {
        self.transform.transform_vector(self.entity.vder(u, v))
    }
    #[inline(always)]
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        self.uder(u, v).cross(self.vder(u, v)).normalize()
    }
}

impl<S, T> BoundedSurface for Transformed<S, T>
where
    S: BoundedSurface<Point = Point3, Vector = Vector3>,
    T: Transform3<Scalar = f64> + Clone,
{
    #[inline(always)]
    fn parameter_range(&self) -> ((f64, f64), (f64, f64)) { self.entity.parameter_range() }
}

impl<E, T> Invertible for Transformed<E, T>
where
    E: Invertible,
    T: Clone,
{
    #[inline(always)]
    fn inverse(&self) -> Self {
        Transformed {
            entity: self.entity.inverse(),
            transform: self.transform.clone(),
        }
    }
}

impl<E, T> Deref for Transformed<E, T> {
    type Target = E;
    #[inline(always)]
    fn deref(&self) -> &E { &self.entity }
}

impl<E, T> DerefMut for Transformed<E, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut E { &mut self.entity }
}
