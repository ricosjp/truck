//! Implementation of Newton method

use std::ops::{Mul, Sub, Add};

use crate::{cgmath64::*, tolerance::*};

/// the value and jacobian corresponding to parameter
#[derive(Clone, Debug)]
pub struct CalcOutput<V, M> {
    /// the value of function
    pub value: V,
    /// the jacobian of function
    pub derivation: M,
}

/// jacobian of function
pub trait Jacobian<V>: Mul<V, Output = V> + Mul<Self, Output = Self> + Add<Self, Output = Self> + Sized {
    #[doc(hidden)]
    fn invert(self) -> Option<Self>;
    #[doc(hidden)]
    fn transpose(&self) -> Self;
    #[doc(hidden)]
    fn identity(scalar: f64) -> Self;
}

impl Jacobian<f64> for f64 {
    #[inline(always)]
    fn invert(self) -> Option<Self> {
        match self.is_zero() {
            true => None,
            false => Some(1.0 / self),
        }
    }
    #[inline(always)]
    fn transpose(&self) -> Self {
		*self
    }
    #[inline(always)]
    fn identity(scalar: f64) -> Self {
		scalar	
	}
}

macro_rules! impl_jacobian {
    ($matrix: ty, $vector: ty) => {
        impl Jacobian<$vector> for $matrix {
            #[inline(always)]
            fn invert(self) -> Option<Self> { SquareMatrix::invert(&self) }
			fn transpose(&self) -> Self { Matrix::transpose(self) }
			fn identity(scalar: f64) -> Self { SquareMatrix::from_value(scalar) }
        }
    };
}

impl_jacobian!(Matrix2, Vector2);
impl_jacobian!(Matrix3, Vector3);
impl_jacobian!(Matrix4, Vector4);

/// Solve equation by Newton's method
/// # Examples
/// ```
/// use truck_base::{newton::*, assert_near2};
///
/// let function = |x: f64| CalcOutput {
///     value: x * x - 2.0,
///     derivation: 2.0 * x,
/// };
/// let sqrt2 = solve(function, 1.0, 10).unwrap();
/// assert_near2!(sqrt2, f64::sqrt(2.0));
/// ```
pub fn solve<V, M>(
    function: impl Fn(V) -> CalcOutput<V, M>,
    mut hint: V,
    trials: usize,
) -> Result<V, NewtonLog<V>>
where
    V: Sub<Output = V> + Copy + Tolerance,
    M: Jacobian<V>,
{
    let mut log = NewtonLog::new(cfg!(debug_assertions), trials);
    for _ in 0..=trials {
        log.push(hint);
        let CalcOutput { value, derivation } = function(hint);
        let Some(inv) = derivation.invert() else {
            log.set_degenerate(true);
            return Err(log);
        };
        let next = hint - inv * value;
        if next.near2(&hint) {
            return Ok(hint);
        }
        hint = next;
    }
    Err(log)
}


pub fn gauss_newton<V, M>(
    function: impl Fn(V) -> CalcOutput<V, M>,
    mut hint: V,
    trials: usize,
) -> Result<V, NewtonLog<V>>
where
    V: Sub<Output = V> + Copy + Tolerance,
    M: Jacobian<V>,
{
    let mut log = NewtonLog::new(cfg!(debug_assertions), trials);
    for _ in 0..=trials {
        log.push(hint);
        let CalcOutput { value, derivation } = function(hint);
		let rhs=derivation.transpose() * value;
        let Some(inv) = (derivation.transpose() * derivation + M::identity(0.001)).invert() else {
            log.set_degenerate(true);
            return Err(log);
        };
        let next = hint - inv * rhs;
        if next.near2(&hint) {
            return Ok(hint);
        }
        hint = next;
    }
    Err(log)
}

mod newtonlog {
    use std::fmt::*;
    /// A structure that stores logs for debugging.
    #[derive(Clone, Debug)]
    pub struct NewtonLog<T> {
        log: Option<Vec<T>>,
        degenerate: bool,
    }

    impl<T> NewtonLog<T> {
        /// constructor
        #[inline(always)]
        pub fn new(activate: bool, trials: usize) -> Self {
            match activate {
                true => NewtonLog {
                    log: Some(Vec::with_capacity(trials)),
                    degenerate: false,
                },
                false => NewtonLog {
                    log: None,
                    degenerate: false,
                },
            }
        }
        /// Returns `true` iff the Newton method terminates due to Jacobian degeneracy.
        #[inline(always)]
        pub fn degenerate(&self) -> bool { self.degenerate }
        #[inline(always)]
        pub(super) fn push(&mut self, log: T) {
            if let Some(vec) = &mut self.log {
                vec.push(log)
            }
        }
        #[inline(always)]
        pub(super) fn set_degenerate(&mut self, degenerate: bool) { self.degenerate = degenerate }
    }

    impl<T: Debug> Display for NewtonLog<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self.degenerate {
                true => f.pad("Jacobian is dengenerate. ")?,
                false => f.pad("Newton method is not converges. ")?,
            }
            match &self.log {
                None => f.pad(
                    "If you want to see the Newton log, please re-run it with the debug build.",
                ),
                Some(vec) => {
                    f.pad("Newton Log:\n")?;
                    vec.iter()
                        .try_for_each(|log| f.write_fmt(format_args!("{log:?}\n")))
                }
            }
        }
    }
}
pub use newtonlog::NewtonLog;

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_newton() {
		let function = |x: f64| CalcOutput {
			value: x * x - 2.0,
			derivation: 2.0 * x,
		};
		let sqrt2 = solve(function, 1.0, 5).unwrap();
		assert!((sqrt2 - f64::sqrt(2.0)).abs() < 1e-10);
	}
	#[test]
	fn test_gauss_newton() {
		let function = |x: f64| CalcOutput {
			value: x * x - 2.0,// (x * x - 2.0).powi(2),
			derivation: 2.0 * x,//2.*(x*x-2.)*(2.*x),
		};
		let sqrt2 = gauss_newton(function, 1.0, 5).unwrap();
		assert!((sqrt2 - f64::sqrt(2.0)).abs() < 1e-10);
	}
}