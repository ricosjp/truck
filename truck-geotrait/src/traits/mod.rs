use truck_base::cgmath64::*;

mod curve;
pub use curve::*;
mod surface;
pub use surface::*;

/// Search parameter `t` such that `self.subs(t)` is near point.
pub trait SearchParameter {
    /// point
    type Point;
    /// curve => `f64`, surface => `(f64, f64)`
    type Parameter;
    /// Search parameter `t` such that `self.subs(t)` is near point.  
    /// Returns `None` if could not find such parameter.
    fn search_parameter(
        &self,
        point: Self::Point,
        hint: Option<Self::Parameter>,
        trial: usize,
    ) -> Option<Self::Parameter>;
}

impl<'a, T: SearchParameter> SearchParameter for &'a T {
    type Point = T::Point;
    type Parameter = T::Parameter;
    fn search_parameter(
        &self,
        point: Self::Point,
        hint: Option<Self::Parameter>,
        trial: usize,
    ) -> Option<Self::Parameter> {
        T::search_parameter(&*self, point, hint, trial)
    }
}

/// Search parameter `t` such that `self.subs(t)` is nearest point.
pub trait SearchNearestParameter {
    /// point
    type Point;
    /// curve => `f64`, surface => `(f64, f64)`
    type Parameter;
    /// Search nearest parameter `t` such that `self.subs(t)` is nearest point.  
    /// Returns `None` if could not find such parameter.
    fn search_nearest_parameter(
        &self,
        point: Self::Point,
        hint: Option<Self::Parameter>,
        trial: usize,
    ) -> Option<Self::Parameter>;
}

impl<'a, T: SearchNearestParameter> SearchNearestParameter for &'a T {
    type Point = T::Point;
    type Parameter = T::Parameter;
    fn search_nearest_parameter(
        &self,
        point: Self::Point,
        hint: Option<Self::Parameter>,
        trial: usize,
    ) -> Option<Self::Parameter> {
        T::search_nearest_parameter(&*self, point, hint, trial)
    }
}

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
