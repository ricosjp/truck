use std::fmt::Debug;
use thiserror::Error;
use truck_base::{
    assert_near,
    cgmath64::{Point2, Point3, Vector2, Vector3},
    tolerance::Tolerance,
};

/// Parametric curves
pub trait ParametricCurve: Clone {
    /// The curve is in the space of `Self::Point`.
    type Point;
    /// The derivation vector of the curve.
    type Vector;
    /// Substitutes the parameter `t`.
    fn subs(&self, t: f64) -> Self::Point;
    /// Returns the derivation.
    fn der(&self, t: f64) -> Self::Vector;
    /// Returns the 2nd-order derivation.
    fn der2(&self, t: f64) -> Self::Vector;
    /// The range of the parameter of the curve.
    fn parameter_range(&self) -> (f64, f64);
    /// The front end point of the curve.
    fn front(&self) -> Self::Point {
        let (t, _) = self.parameter_range();
        self.subs(t)
    }
    /// The back end point of the curve.
    fn back(&self) -> Self::Point {
        let (_, t) = self.parameter_range();
        self.subs(t)
    }
}

/// Implementation for the test of topological methods.
impl ParametricCurve for () {
    type Point = ();
    type Vector = ();
    fn subs(&self, _: f64) -> Self::Point {}
    fn der(&self, _: f64) -> Self::Vector {}
    fn der2(&self, _: f64) -> Self::Vector {}
    fn parameter_range(&self) -> (f64, f64) { (0.0, 1.0) }
}

/// Implementation for the test of topological methods.
impl ParametricCurve for (usize, usize) {
    type Point = usize;
    type Vector = usize;
    fn subs(&self, t: f64) -> Self::Point {
        match t < 0.5 {
            true => self.0,
            false => self.1,
        }
    }
    fn der(&self, _: f64) -> Self::Vector { self.1 - self.0 }
    fn der2(&self, _: f64) -> Self::Vector { self.1 - self.0 }
    fn parameter_range(&self) -> (f64, f64) { (0.0, 1.0) }
}

impl<'a, C: ParametricCurve> ParametricCurve for &'a C {
    type Point = C::Point;
    type Vector = C::Vector;
    fn subs(&self, t: f64) -> Self::Point { (*self).subs(t) }
    #[inline(always)]
    fn der(&self, t: f64) -> Self::Vector { (*self).der(t) }
    #[inline(always)]
    fn der2(&self, t: f64) -> Self::Vector { (*self).der2(t) }
    #[inline(always)]
    fn parameter_range(&self) -> (f64, f64) { (*self).parameter_range() }
}

/// 2D parametric curve
pub trait ParametricCurve2D: ParametricCurve<Point = Point2, Vector = Vector2> {}
impl<C: ParametricCurve<Point = Point2, Vector = Vector2>> ParametricCurve2D for C {}
/// 3D parametric curve
pub trait ParametricCurve3D: ParametricCurve<Point = Point3, Vector = Vector3> {}
impl<C: ParametricCurve<Point = Point3, Vector = Vector3>> ParametricCurve3D for C {}

/// Dividable curve
pub trait ParameterDivision1D {
    /// The curve is in the space of `Self::Point`.
    type Point;
    /// Creates the curve division (prameters, corresponding points).
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Self::Point>);
}

impl<'a, C: ParameterDivision1D> ParameterDivision1D for &'a C {
    type Point = C::Point;
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Self::Point>) {
        (*self).parameter_division(range, tol)
    }
}

/// parameter range move by affine transformation
pub trait ParameterTransform: ParametricCurve {
    /// parameter range move by affine transformation
    /// # Panics
    /// Panic occurs if `scalar` is not positive.
    fn parameter_transform(&mut self, scalar: f64, r#move: f64) -> &mut Self;
    /// parameter range move by affine transformation
    /// # Examples
    /// ```ignore
    /// let curve0 = ... // implemented ParameterTransform
    /// assert_eq!(curve0.parameter_range(), (0.0, 1.0));
    /// let curve1 = curve0.parameter_transformed(1.0, 2.0);
    /// assert_eq!(curve1.subs(0.5), curve0.subs(2.5));
    /// ```
    fn parameter_transformed(&self, scalar: f64, r#move: f64) -> Self {
        let mut curve = self.clone();
        curve.parameter_transform(scalar, r#move);
        curve
    }
    /// Makes the parameter range `(0.0, 1.0)`.
    fn parameter_normalization(&mut self) -> &mut Self {
        let (t0, t1) = self.parameter_range();
        let a = 1.0 / (t1 - t0);
        let b = -t0 * a;
        self.parameter_transform(a, b)
    }
}

/// Concats two curves
pub trait Concat<Rhs: ParametricCurve<Point = Self::Point, Vector = Self::Vector>>:
    ParametricCurve
where Self::Point: Debug {
    /// The result of concat two curves
    type Output: ParametricCurve<Point = Self::Point, Vector = Self::Vector>;
    /// Try concat two curves.
    /// # Failure
    /// Returns `None` if `self.parameter_range().1 != rhs.parameter_range().0`.
    fn try_concat(&self, rhs: &Rhs) -> Result<Self::Output, ConcatError<Self::Point>>;
    /// Try concat two curves.
    /// # Panic
    /// Panic occurs if `self.parameter_range().1 != rhs.parameter_range().0`.
    fn concat(&self, rhs: &Rhs) -> Self::Output {
        self.try_concat(rhs).unwrap_or_else(|err| panic!("{}", err))
    }
}

/// Error for concat curves
#[derive(Clone, Copy, PartialEq, Debug, Error)]
pub enum ConcatError<Point: Debug> {
    /// Failed to concat curves since the end parameter of the first curve is different form the start parameter of the second curve.
    #[error("The end parameter {0} of the first curve is different from the start parameter {1} of the second curve.")]
    DisconnectedParameters(f64, f64),
    /// Failed to concat curves since the end point of the first curve is different from the start point of the second curve.
    #[error("The end point {0:?} of the first curve is different from the start point {1:?} of the second curve.")]
    DisconnectedPoints(Point, Point),
}

impl<T: Debug> ConcatError<T> {
    /// into the
    #[inline(always)]
    pub fn point_map<U: Debug, F>(self, f: F) -> ConcatError<U>
    where F: Fn(T) -> U {
        match self {
            ConcatError::DisconnectedParameters(a, b) => ConcatError::DisconnectedParameters(a, b),
            ConcatError::DisconnectedPoints(p, q) => ConcatError::DisconnectedPoints(f(p), f(q)),
        }
    }
}

/// Curve for the recursive concatting.
#[derive(Clone, Debug)]
pub enum CurveCollector<C> {
    /// the empty curve
    Singleton,
    /// a non-empty curve
    Curve(C),
}

impl<C> CurveCollector<C> {
    /// Concats two B-spline curves.
    #[inline(always)]
    pub fn try_concat<Rhs>(&mut self, curve: &Rhs) -> Result<&mut Self, ConcatError<C::Point>>
    where
        C: Concat<Rhs, Output = C>,
        C::Point: Debug,
        Rhs: ParametricCurve<Point = C::Point, Vector = C::Vector> + Into<C>, {
        match self {
            CurveCollector::Singleton => {
                *self = CurveCollector::Curve(curve.clone().into());
            }
            CurveCollector::Curve(ref mut curve0) => {
                *curve0 = curve0.try_concat(curve)?;
            }
        }
        Ok(self)
    }
    /// Concats two B-spline curves.
    #[inline(always)]
    pub fn concat<Rhs>(&mut self, curve: &Rhs) -> &mut Self
    where
        C: Concat<Rhs, Output = C>,
        C::Point: Debug,
        Rhs: ParametricCurve<Point = C::Point, Vector = C::Vector> + Into<C>, {
        self.try_concat(curve)
            .unwrap_or_else(|error| panic!("{}", error))
    }

    /// Whether `self` is the singleton or not.
    #[inline(always)]
    pub fn is_singleton(&self) -> bool {
        match self {
            CurveCollector::Singleton => true,
            CurveCollector::Curve(_) => false,
        }
    }
    /// Returns the entity curve.
    /// # Panics
    /// If `self` is `Singleton`, then panics occurs.
    #[inline(always)]
    pub fn unwrap(self) -> C {
        match self {
            CurveCollector::Curve(curve) => curve,
            CurveCollector::Singleton => panic!("This curve collector is singleton."),
        }
    }
}

impl<C> From<CurveCollector<C>> for Option<C> {
    #[inline(always)]
    fn from(collector: CurveCollector<C>) -> Option<C> {
        match collector {
            CurveCollector::Singleton => None,
            CurveCollector::Curve(curve) => Some(curve),
        }
    }
}

/// Cuts one curve into two curves.
pub trait Cut: ParametricCurve {
    /// Cuts one curve into two curves. Assigns the former curve to `self` and returns the later curve.
    fn cut(&mut self, t: f64) -> Self;
}

/// positive test implementation for `ParameterTransform` by random transformation
pub fn parameter_transform_random_test<C>(curve: &C, trials: usize)
where
    C: ParameterTransform,
    C::Point: Debug + Tolerance,
    C::Vector: Debug + Tolerance + std::ops::Mul<f64, Output = C::Vector>, {
    (0..trials).for_each(move |_| exec_parameter_transform_random_test(curve))
}

fn exec_parameter_transform_random_test<C>(curve: &C)
where
    C: ParameterTransform,
    C::Point: Debug + Tolerance,
    C::Vector: Debug + Tolerance + std::ops::Mul<f64, Output = C::Vector>, {
    let a = rand::random::<f64>() + 0.5;
    let b = rand::random::<f64>() * 2.0;
    let transformed = curve.parameter_transformed(a, b);

    let (t0, t1) = curve.parameter_range();
    assert_near!(transformed.parameter_range().0, t0 * a + b);
    assert_near!(transformed.parameter_range().1, t1 * a + b);
    let p = rand::random::<f64>();
    let t = (1.0 - p) * t0 + p * t1;
    assert_near!(transformed.subs(t * a + b), curve.subs(t));
    assert_near!(transformed.der(t * a + b) * a, curve.der(t));
    assert_near!(transformed.der2(t * a + b) * a * a, curve.der2(t));
    assert_near!(transformed.front(), curve.front());
    assert_near!(transformed.back(), curve.back());
}

/// positive test implementation for `Concat`.
pub fn concat_random_test<C0, C1>(curve0: &C0, curve1: &C1, trials: usize)
where
    C0: Concat<C1>,
    C0::Point: Debug + Tolerance,
    C0::Vector: Debug + Tolerance,
    C0::Output: ParametricCurve<Point = C0::Point, Vector = C0::Vector> + Debug,
    C1: ParametricCurve<Point = C0::Point, Vector = C0::Vector>, {
    (0..trials).for_each(move |_| exec_concat_random_test(curve0, curve1))
}

fn exec_concat_random_test<C0, C1>(curve0: &C0, curve1: &C1)
where
    C0: Concat<C1>,
    C0::Point: Debug + Tolerance,
    C0::Vector: Debug + Tolerance,
    C0::Output: ParametricCurve<Point = C0::Point, Vector = C0::Vector> + Debug,
    C1: ParametricCurve<Point = C0::Point, Vector = C0::Vector>, {
    let concatted = curve0.try_concat(&curve1).unwrap();
    let (t0, t1) = curve0.parameter_range();
    let (_, t2) = curve1.parameter_range();
    assert_near!(concatted.parameter_range().0, t0);
    assert_near!(concatted.parameter_range().1, t2);

    let p = rand::random::<f64>();
    let t = t0 * (1.0 - p) + t1 * p;
    assert_near!(concatted.subs(t), curve0.subs(t));
    assert_near!(concatted.der(t), curve0.der(t));
    assert_near!(concatted.der2(t), curve0.der2(t));
    assert_near!(concatted.front(), curve0.front());

    let p = rand::random::<f64>();
    let t = t1 * (1.0 - p) + t2 * p;
    assert_near!(concatted.subs(t), curve1.subs(t));
    assert_near!(concatted.der(t), curve1.der(t));
    assert_near!(concatted.der2(t), curve1.der2(t));
    assert_near!(concatted.back(), curve1.back());
}

/// positive test implementation for `Cut` by random transformation
pub fn cut_random_test<C>(curve: &C, trials: usize)
where
    C: Cut,
    C::Point: Debug + Tolerance,
    C::Vector: Debug + Tolerance, {
    (0..trials).for_each(move |_| exec_cut_random_test(curve))
}

fn exec_cut_random_test<C>(curve: &C)
where
    C: Cut,
    C::Point: Debug + Tolerance,
    C::Vector: Debug + Tolerance, {
    let mut part0 = curve.clone();
    let (t0, t1) = curve.parameter_range();
    let p = rand::random::<f64>();
    let t = t0 * (1.0 - p) + t1 * p;
    let part1 = part0.cut(t);
    assert_near!(part0.parameter_range().0, t0);
    assert_near!(part0.parameter_range().1, t);
    assert_near!(part1.parameter_range().0, t);
    assert_near!(part1.parameter_range().1, t1);

    let p = rand::random::<f64>();
    let s = t0 * (1.0 - p) + t * p;
    assert_near!(part0.subs(s), curve.subs(s));
    assert_near!(part0.der(s), curve.der(s));
    assert_near!(part0.der2(s), curve.der2(s));
    assert_near!(part0.front(), curve.front());
    assert_near!(part0.back(), curve.subs(t));

    let p = rand::random::<f64>();
    let s = t * (1.0 - p) + t1 * p;
    assert_near!(part1.subs(s), curve.subs(s));
    assert_near!(part1.der(s), curve.der(s));
    assert_near!(part1.der2(s), curve.der2(s));
    assert_near!(part1.front(), curve.subs(t));
    assert_near!(part1.back(), curve.back());
}
