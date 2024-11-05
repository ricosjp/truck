use crate::cgmath64::*;
use cgmath::AbsDiffEq;
use std::fmt::Debug;

/// general tolerance
pub const TOLERANCE: f64 = 1.0e-6;

/// general tolerance of square order
pub const TOLERANCE2: f64 = TOLERANCE * TOLERANCE;

/// Defines a tolerance in the whole package
pub trait Tolerance: AbsDiffEq<Epsilon = f64> + Debug {
    /// The "distance" is less than `TOLERANCE`.
    fn near(&self, other: &Self) -> bool { self.abs_diff_eq(other, TOLERANCE) }

    /// The "distance" is less than `TOLERANCR2`.
    fn near2(&self, other: &Self) -> bool { self.abs_diff_eq(other, TOLERANCE2) }
}

impl<T: AbsDiffEq<Epsilon = f64> + Debug> Tolerance for T {}

/// assert near
#[macro_export]
macro_rules! assert_near {
    ($left: expr, $right: expr $(,)?) => {
        assert!($crate::tolerance::Tolerance::near(&$left, &$right), "assertion failed: `left` is near `right`
left: {:?},
right: {:?}", $left, $right)
    };
    ($left: expr, $right: expr, $($arg: tt)+) => {
        assert!($crate::tolerance::Tolerance::near(&$left, &$right), "assertion failed: `left` is near `right`
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
        assert!($crate::tolerance::Tolerance::near2(&$left, &$right), "assertion failed: `left` is near `right`
left: {:?},
right: {:?}", $left, $right)
    };
    ($left: expr, $right: expr, $($arg: tt)+) => {
        assert!($crate::tolerance::Tolerance::near2(&$left, &$right), "assertion failed: `left` is near `right`
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

impl<T: Tolerance + Zero> Origin for T {}
