use super::*;
use monstertruck_core::{
    assert_near,
    cgmath64::{Point2, Point3, Vector2, Vector3},
    tolerance::Tolerance,
};
use std::fmt::Debug;
use thiserror::Error;

/// A curve defined by a mapping `P(t)` from a scalar parameter `t` to a point.
///
/// As `t` varies over the parameter range, the returned points trace out the curve.
/// Derivatives give the tangent (`derivative`), curvature direction (`derivative_2`), and
/// higher-order information (`derivative_n`) at any parameter value.
/// New code should prefer `evaluate` and `derivative*` methods.
/// Legacy `subs` and `der*` methods are kept for compatibility.
pub trait ParametricCurve: Clone {
    /// The point type the curve maps into (e.g. `Point2`, `Point3`).
    type Point;
    /// The derivative vector type (e.g. `Vector2`, `Vector3`).
    type Vector: Zero + Copy;
    /// Evaluates the curve at parameter `t`, returning the point `P(t)`.
    fn evaluate(&self, t: f64) -> Self::Point;
    /// Returns `dP/dt` -- the first derivative (tangent) at `t`.
    fn derivative(&self, t: f64) -> Self::Vector;
    /// Returns `d²P/dt²` -- the second derivative at `t`.
    fn derivative_2(&self, t: f64) -> Self::Vector;
    /// Returns the `n`-th derivative `dⁿP/dtⁿ` at `t`.
    fn derivative_n(&self, n: usize, t: f64) -> Self::Vector;
    /// Returns all derivatives at parameter `t` with order `0..=n`.
    fn derivatives(&self, n: usize, t: f64) -> CurveDerivatives<Self::Vector> {
        (0..=n).map(|i| self.derivative_n(i, t)).collect()
    }
    /// Deprecated: use [`evaluate`](ParametricCurve::evaluate).
    #[inline(always)]
    fn subs(&self, t: f64) -> Self::Point { self.evaluate(t) }
    /// Deprecated: use [`derivative`](ParametricCurve::derivative).
    #[inline(always)]
    fn der(&self, t: f64) -> Self::Vector { self.derivative(t) }
    /// Deprecated: use [`derivative_2`](ParametricCurve::derivative_2).
    #[inline(always)]
    fn der2(&self, t: f64) -> Self::Vector { self.derivative_2(t) }
    /// Deprecated: use [`derivative_n`](ParametricCurve::derivative_n).
    #[inline(always)]
    fn der_n(&self, n: usize, t: f64) -> Self::Vector { self.derivative_n(n, t) }
    /// Deprecated: use [`derivatives`](ParametricCurve::derivatives).
    #[inline(always)]
    fn ders(&self, n: usize, t: f64) -> CurveDerivatives<Self::Vector> { self.derivatives(n, t) }
    /// Returns default parameter range
    #[inline(always)]
    fn parameter_range(&self) -> ParameterRange { (Bound::Unbounded, Bound::Unbounded) }
    /// Return the ends of `parameter_range` by tuple.
    /// If the range is unbounded, return `None`.
    #[inline(always)]
    fn try_range_tuple(&self) -> Option<(f64, f64)> {
        let (x, y) = self.parameter_range();
        bound2opt(x).and_then(move |x| bound2opt(y).map(move |y| (x, y)))
    }
    /// `None` in default implementation; `Some(period)` if periodic.
    #[inline(always)]
    fn period(&self) -> Option<f64> { None }
}

/// bounded parametric curves i.e. it is guaranteed that the return value of `parameter_range` is not `Bound::Unbounded`.
pub trait BoundedCurve: ParametricCurve {
    /// Return the ends of `parameter_range` by tuple.
    #[inline(always)]
    fn range_tuple(&self) -> (f64, f64) { self.try_range_tuple().expect(UNBOUNDED_ERROR) }
    /// The front end point of the curve.
    #[inline(always)]
    fn front(&self) -> Self::Point {
        let (x, _) = self.parameter_range();
        self.evaluate(bound2opt(x).expect(UNBOUNDED_ERROR))
    }
    /// The back end point of the curve.
    #[inline(always)]
    fn back(&self) -> Self::Point {
        let (_, y) = self.parameter_range();
        self.evaluate(bound2opt(y).expect(UNBOUNDED_ERROR))
    }
}

/// Implementation for the test of topological methods.
impl ParametricCurve for (usize, usize) {
    type Point = usize;
    type Vector = usize;
    fn derivative_n(&self, _: usize, _: f64) -> Self::Vector { self.1 - self.0 }
    fn evaluate(&self, t: f64) -> Self::Point {
        match t < 0.5 {
            true => self.0,
            false => self.1,
        }
    }
    fn derivative(&self, _: f64) -> Self::Vector { self.1 - self.0 }
    fn derivative_2(&self, _: f64) -> Self::Vector { self.1 - self.0 }
    fn parameter_range(&self) -> ParameterRange { (Bound::Included(0.0), Bound::Included(1.0)) }
}

/// Implementation for the test of topological methods.
impl BoundedCurve for (usize, usize) {}

impl<C: ParametricCurve> ParametricCurve for &C {
    type Point = C::Point;
    type Vector = C::Vector;
    fn evaluate(&self, t: f64) -> Self::Point { (*self).evaluate(t) }
    #[inline(always)]
    fn derivative(&self, t: f64) -> Self::Vector { (*self).derivative(t) }
    #[inline(always)]
    fn derivative_2(&self, t: f64) -> Self::Vector { (*self).derivative_2(t) }
    #[inline(always)]
    fn derivative_n(&self, n: usize, t: f64) -> Self::Vector { (*self).derivative_n(n, t) }
    #[inline(always)]
    fn derivatives(&self, n: usize, t: f64) -> CurveDerivatives<Self::Vector> {
        (*self).derivatives(n, t)
    }
    #[inline(always)]
    fn parameter_range(&self) -> ParameterRange { (*self).parameter_range() }
    #[inline(always)]
    fn period(&self) -> Option<f64> { (*self).period() }
}

impl<C: BoundedCurve> BoundedCurve for &C {
    #[inline(always)]
    fn front(&self) -> Self::Point { (*self).front() }
    #[inline(always)]
    fn back(&self) -> Self::Point { (*self).back() }
}

impl<C: ParametricCurve> ParametricCurve for Box<C> {
    type Point = C::Point;
    type Vector = C::Vector;
    fn evaluate(&self, t: f64) -> Self::Point { (**self).evaluate(t) }
    #[inline(always)]
    fn derivative(&self, t: f64) -> Self::Vector { (**self).derivative(t) }
    #[inline(always)]
    fn derivative_2(&self, t: f64) -> Self::Vector { (**self).derivative_2(t) }
    #[inline(always)]
    fn derivative_n(&self, n: usize, t: f64) -> Self::Vector { (**self).derivative_n(n, t) }
    #[inline(always)]
    fn derivatives(&self, n: usize, t: f64) -> CurveDerivatives<Self::Vector> {
        (**self).derivatives(n, t)
    }
    #[inline(always)]
    fn parameter_range(&self) -> ParameterRange { (**self).parameter_range() }
    #[inline(always)]
    fn period(&self) -> Option<f64> { (**self).period() }
}

impl<C: BoundedCurve> BoundedCurve for Box<C> {
    #[inline(always)]
    fn front(&self) -> Self::Point { (**self).front() }
    #[inline(always)]
    fn back(&self) -> Self::Point { (**self).back() }
}

impl<C: Cut> Cut for Box<C> {
    #[inline(always)]
    fn cut(&mut self, t: f64) -> Self { Box::new((**self).cut(t)) }
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
    /// Creates the curve division (parameters, corresponding points).
    ///
    /// # Panics
    ///
    /// `tol` must be greater than or equal to `TOLERANCE`.
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Self::Point>);
}

impl<C: ParameterDivision1D> ParameterDivision1D for &C {
    type Point = C::Point;
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Self::Point>) {
        (*self).parameter_division(range, tol)
    }
}

impl<C: ParameterDivision1D> ParameterDivision1D for Box<C> {
    type Point = C::Point;
    fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<Self::Point>) {
        (**self).parameter_division(range, tol)
    }
}

/// parameter range move by affine transformation
pub trait ParameterTransform: BoundedCurve {
    /// parameter range move by affine transformation
    /// # Panics
    /// Panic occurs if `scalar` is not positive.
    fn parameter_transform(&mut self, scalar: f64, r#move: f64) -> &mut Self;
    /// parameter range move by affine transformation
    /// # Examples
    /// ```ignore
    /// let curve0 = ... // implemented ParameterTransform
    /// assert_eq!(curve0.range_tuple(), (0.0, 1.0));
    /// let curve1 = curve0.parameter_transformed(1.0, 2.0);
    /// assert_eq!(curve1.evaluate(0.5), curve0.evaluate(2.5));
    /// ```
    fn parameter_transformed(&self, scalar: f64, r#move: f64) -> Self {
        let mut curve = self.clone();
        curve.parameter_transform(scalar, r#move);
        curve
    }
    /// Makes the parameter range `(0.0, 1.0)`.
    fn parameter_normalization(&mut self) -> &mut Self {
        let (t0, t1) = self.range_tuple();
        let a = 1.0 / (t1 - t0);
        let b = -t0 * a;
        self.parameter_transform(a, b)
    }
}

impl<C: ParameterTransform> ParameterTransform for Box<C> {
    #[inline(always)]
    fn parameter_transform(&mut self, scalar: f64, r#move: f64) -> &mut Self {
        (**self).parameter_transform(scalar, r#move);
        self
    }
    #[inline(always)]
    fn parameter_transformed(&self, scalar: f64, r#move: f64) -> Self {
        Box::new((**self).parameter_transformed(scalar, r#move))
    }
    #[inline(always)]
    fn parameter_normalization(&mut self) -> &mut Self {
        (**self).parameter_normalization();
        self
    }
}

/// Concats two curves
pub trait Concat<Rhs: BoundedCurve<Point = Self::Point, Vector = Self::Vector>>:
    BoundedCurve
where Self::Point: Debug {
    /// The result of concat two curves
    type Output: BoundedCurve<Point = Self::Point, Vector = Self::Vector>;
    /// Try concat two curves.
    /// # Failure
    /// Returns `None` if `self.range_tuple().1 != rhs.range_tuple().0`.
    fn try_concat(&self, rhs: &Rhs) -> Result<Self::Output, ConcatError<Self::Point>>;
    /// Try concat two curves.
    /// # Panic
    /// Panic occurs if `self.range_tuple().1 != rhs.range_tuple().0`.
    fn concat(&self, rhs: &Rhs) -> Self::Output {
        self.try_concat(rhs).unwrap_or_else(|err| panic!("{}", err))
    }
}

impl<Rhs, C> Concat<Rhs> for Box<C>
where
    Rhs: BoundedCurve<Point = C::Point, Vector = C::Vector>,
    C: Concat<Rhs>,
    C::Point: Debug,
{
    type Output = C::Output;
    fn try_concat(&self, rhs: &Rhs) -> Result<Self::Output, ConcatError<C::Point>> {
        (**self).try_concat(rhs)
    }
    fn concat(&self, rhs: &Rhs) -> Self::Output { (**self).concat(rhs) }
}

/// Error for concat curves
#[derive(Clone, Copy, PartialEq, Debug, Error)]
pub enum ConcatError<Point: Debug> {
    /// Failed to concat curves since the end parameter of the first curve is different form the start parameter of the second curve.
    #[error(
        "The end parameter {0} of the first curve is different from the start parameter {1} of the second curve."
    )]
    DisconnectedParameters(f64, f64),
    /// Failed to concat curves since the end point of the first curve is different from the start point of the second curve.
    #[error(
        "The end point {0:?} of the first curve is different from the start point {1:?} of the second curve."
    )]
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
        Rhs: BoundedCurve<Point = C::Point, Vector = C::Vector> + Into<C>, {
        match self {
            CurveCollector::Singleton => {
                *self = CurveCollector::Curve(curve.clone().into());
            }
            CurveCollector::Curve(curve0) => {
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
        Rhs: BoundedCurve<Point = C::Point, Vector = C::Vector> + Into<C>, {
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
pub trait Cut: BoundedCurve {
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

    let (t0, t1) = curve.range_tuple();
    assert_near!(transformed.range_tuple().0, t0 * a + b);
    assert_near!(transformed.range_tuple().1, t1 * a + b);
    let p = rand::random::<f64>();
    let t = (1.0 - p) * t0 + p * t1;
    assert_near!(transformed.evaluate(t * a + b), curve.evaluate(t));
    assert_near!(transformed.derivative(t * a + b) * a, curve.derivative(t));
    assert_near!(
        transformed.derivative_2(t * a + b) * a * a,
        curve.derivative_2(t)
    );
    assert_near!(transformed.front(), curve.front());
    assert_near!(transformed.back(), curve.back());
}

/// positive test implementation for `Concat`.
pub fn concat_random_test<C0, C1>(curve0: &C0, curve1: &C1, trials: usize)
where
    C0: Concat<C1>,
    C0::Point: Debug + Tolerance,
    C0::Vector: Debug + Tolerance,
    C0::Output: BoundedCurve<Point = C0::Point, Vector = C0::Vector> + Debug,
    C1: BoundedCurve<Point = C0::Point, Vector = C0::Vector>, {
    (0..trials).for_each(move |_| exec_concat_random_test(curve0, curve1))
}

fn exec_concat_random_test<C0, C1>(curve0: &C0, curve1: &C1)
where
    C0: Concat<C1>,
    C0::Point: Debug + Tolerance,
    C0::Vector: Debug + Tolerance,
    C0::Output: BoundedCurve<Point = C0::Point, Vector = C0::Vector> + Debug,
    C1: BoundedCurve<Point = C0::Point, Vector = C0::Vector>, {
    let concatted = curve0.try_concat(curve1).unwrap();
    let (t0, t1) = curve0.range_tuple();
    let (_, t2) = curve1.range_tuple();
    assert_near!(concatted.range_tuple().0, t0);
    assert_near!(concatted.range_tuple().1, t2);

    let p = rand::random::<f64>();
    let t = t0 * (1.0 - p) + t1 * p;
    assert_near!(concatted.evaluate(t), curve0.evaluate(t));
    assert_near!(concatted.derivative(t), curve0.derivative(t));
    assert_near!(concatted.derivative_2(t), curve0.derivative_2(t));
    assert_near!(concatted.front(), curve0.front());

    let p = rand::random::<f64>();
    let t = t1 * (1.0 - p) + t2 * p;
    assert_near!(concatted.evaluate(t), curve1.evaluate(t));
    assert_near!(concatted.derivative(t), curve1.derivative(t));
    assert_near!(concatted.derivative_2(t), curve1.derivative_2(t));
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
    let (t0, t1) = curve.range_tuple();
    let p = rand::random::<f64>();
    let t = t0 * (1.0 - p) + t1 * p;
    let part1 = part0.cut(t);
    assert_near!(part0.range_tuple().0, t0);
    assert_near!(part0.range_tuple().1, t);
    assert_near!(part1.range_tuple().0, t);
    assert_near!(part1.range_tuple().1, t1);

    let p = rand::random::<f64>();
    let s = t0 * (1.0 - p) + t * p;
    assert_near!(part0.evaluate(s), curve.evaluate(s));
    assert_near!(part0.front(), curve.front());
    assert_near!(part0.back(), curve.evaluate(t));

    let p = rand::random::<f64>();
    let s = t * (1.0 - p) + t1 * p;
    assert_near!(part1.evaluate(s), curve.evaluate(s));
    assert_near!(part1.front(), curve.evaluate(t));
    assert_near!(part1.back(), curve.back());
}
