use std::ops::Bound;
use truck_base::cgmath64::*;

mod curve;
pub use curve::*;
mod surface;
pub use surface::*;
mod search_parameter;
pub use search_parameter::*;

/// Oriented and reversible
pub trait Invertible: Clone {
    /// Inverts `self`
    fn invert(&mut self);
    /// Returns the inverse.
    #[inline(always)]
    fn inverse(&self) -> Self {
        let mut res = self.clone();
        res.invert();
        res
    }
}

/// Implementation for the test of topological methods.
impl Invertible for (usize, usize) {
    fn invert(&mut self) { *self = (self.1, self.0); }
    fn inverse(&self) -> Self { (self.1, self.0) }
}

impl<P: Clone> Invertible for Vec<P> {
    #[inline(always)]
    fn invert(&mut self) { self.reverse(); }
    #[inline(always)]
    fn inverse(&self) -> Self { self.iter().rev().cloned().collect() }
}

impl<T: Invertible> Invertible for Box<T> {
    #[inline(always)]
    fn invert(&mut self) { (**self).invert() }
    #[inline(always)]
    fn inverse(&self) -> Self { Box::new((**self).inverse()) }
}

/// Transform geometry
pub trait Transformed<T>: Clone {
    /// transform by `trans`.
    fn transform_by(&mut self, trans: T);
    /// transformed geometry by `trans`.
    #[inline(always)]
    fn transformed(&self, trans: T) -> Self {
        let mut res = self.clone();
        res.transform_by(trans);
        res
    }
}

impl<S: Transformed<T>, T> Transformed<T> for Box<S> {
    #[inline(always)]
    fn transform_by(&mut self, trans: T) { (**self).transform_by(trans) }
    #[inline(always)]
    fn transformed(&self, trans: T) -> Self { Box::new((**self).transformed(trans)) }
}
