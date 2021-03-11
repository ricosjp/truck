use crate::cgmath64::*;
use cgmath::AbsDiffEq;
use std::fmt::Debug;

/// general tolerance
pub const TOLERANCE: f64 = 1.0e-7;

/// general tolerance of square order
pub const TOLERANCE2: f64 = TOLERANCE * TOLERANCE;

/// Defines a tolerance in the whole package
pub trait Tolerance: AbsDiffEq<Epsilon = f64> + Debug {
    /// The "distance" is less than `TOLERANCE`.
    fn near(&self, other: &Self) -> bool { self.abs_diff_eq(other, TOLERANCE) }

    /// The "distance" is less than `TOLERANCR2`.
    fn near2(&self, other: &Self) -> bool { self.abs_diff_eq(other, TOLERANCE2) }
}

/// assert near
#[macro_export]
macro_rules! assert_near {
    ($left: expr, $right: expr $(,)?) => {
        assert!($left.near(&$right), "assertion failed: `left` is near `right`
left: {:?},
right: {:?}", $left, $right)
    };
    ($left: expr, $right: expr, $($arg: tt)+) => {
        assert!($left.near(&$right), "assertion failed: `left` is near `right`
left: {:?},
right: {:?}: {}", $left, $right, format_args!($($arg)+))
    };
}

#[test]
#[should_panic]
fn assert_near_without_msg() {
    assert_near!(1.0, 2.0);
}

#[test]
#[should_panic]
fn assert_near_with_msg() {
    assert_near!(1.0, 2.0, "{}", "test OK");
}

/// assert_near2
#[macro_export]
macro_rules! assert_near2 {
    ($left: expr, $right: expr $(,)?) => {
        assert!($left.near2(&$right), "assertion failed: `left` is near `right`
left: {:?},
right: {:?}", $left, $right)
    };
    ($left: expr, $right: expr, $($arg: tt)+) => {
        assert!($left.near2(&$right), "assertion failed: `left` is near `right`
left: {:?},
right: {:?}: {}", $left, $right, format_args!($($arg)+))
    };
}

#[test]
#[should_panic]
fn assert_near2_without_msg() {
    assert_near2!(1.0, 2.0);
}

#[test]
#[should_panic]
fn assert_near2_with_msg() {
    assert_near2!(1.0, 2.0, "{}", "test OK");
}

/// The structs defined the origin. `f64`, `Vector`, and so on.
pub trait Origin: Tolerance + Zero {
    /// near origin
    #[inline(always)]
    fn so_small(&self) -> bool { self.near(&Self::zero()) }

    /// near origin in square order
    #[inline(always)]
    fn so_small2(&self) -> bool { self.near2(&Self::zero()) }
}

/// Rounds the value by tolerance
/// # Example
/// ```
/// use truck_base::{cgmath64::*, tolerance::*};
/// assert_eq!(1.23456789_f64.round_by_tolerance(), &1.2345678);
/// ```
pub trait RoundByTolerance: Tolerance {
    /// Rounds the value by tolerance
    fn round_by_tolerance(&mut self) -> &mut Self;
}

impl Tolerance for f64 {}
impl Tolerance for [f64] {}
impl Origin for f64 {}

impl RoundByTolerance for f64 {
    fn round_by_tolerance(&mut self) -> &mut f64 {
        *self = (*self / TOLERANCE).floor() * TOLERANCE;
        self
    }
}

macro_rules! impl_tolerance {
    ($typename: ty, $($num: expr),+) => {
        impl Tolerance for $typename {}
        impl Origin for $typename {}
        impl RoundByTolerance for $typename {
            #[inline(always)]
            fn round_by_tolerance(&mut self) -> &mut Self {
                $(self[$num].round_by_tolerance();)+
                self
            }
        }
    };
}

impl_tolerance!(Vector1, 0);
impl_tolerance!(Vector2, 0, 1);
impl_tolerance!(Vector3, 0, 1, 2);
impl_tolerance!(Vector4, 0, 1, 2, 3);
impl_tolerance!(Matrix2, 0, 1, 2, 3);
impl_tolerance!(Matrix3, 0, 1, 2, 3, 4, 5, 6, 7, 8);
impl_tolerance!(Matrix4, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);

impl Tolerance for Point1 {}
impl Tolerance for Point2 {}
impl Tolerance for Point3 {}
