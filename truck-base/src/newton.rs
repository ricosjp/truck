//! Implementation of Newton method
//!
//! # Examples
//! ```
//! use truck_base::{newton::newton1, assert_near2};
//! 
//! let value = |x: f64| x * x - 2.0;
//! let der = |x: f64| 2.0 * x;
//! let sqrt2 = newton1(value, der, 1.0, 10).unwrap();
//! assert_near2!(sqrt2, f64::sqrt(2.0));
//! ```

use crate::{cgmath64::*, tolerance::*};

macro_rules! define_newton {
    ($method_name: ident, $matrix: ty, $vector: ty, ($($ders: ident),*), $doc: expr) => {
        #[doc = $doc]
        pub fn $method_name(
            value: impl Fn($vector) -> $vector,
            $($ders: impl Fn($vector) -> $vector,)*
            mut hint: $vector,
            trials: usize,
        ) -> Result<$vector, NewtonLog<$vector>> {
            let mut log = NewtonLog::new(cfg!(debug_assertions), trials);
            for _ in 0..=trials {
                log.push(hint);
                let mat = <$matrix>::from_cols($($ders (hint),)*);
                let Some(inv) = mat.invert() else {
                    log.set_degenerate(true);
                    return Err(log);
                };
                let next = hint - inv * value(hint);
                if next.near2(&hint) {
                    return Ok(next);
                }
                hint = next;
            }
            Err(log)
        }
    };
}

define_newton!(
    newton1,
    f64,
    f64,
    (der),
    "Newton method in 1-dimension."
);
define_newton!(
    newton2,
    Matrix2,
    Vector2,
    (der0, der1),
    "Newton method in 2-dimension."
);
define_newton!(
    newton3,
    Matrix3,
    Vector3,
    (der0, der1, der2),
    "Newton method in 3-dimension."
);
define_newton!(
    newton4,
    Matrix4,
    Vector4,
    (der0, der1, der2, der3),
    "Newton method in 4-dimension."
);

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
        pub(super) fn push(&mut self, log: T) { self.log.as_mut().map(|vec| vec.push(log)); }
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

trait InvertFloat: Sized {
    fn from_cols(self) -> Self { self }
    fn invert(self) -> Option<Self>;
}

impl InvertFloat for f64 {
    #[inline(always)]
    fn invert(self) -> Option<Self> {
        match self.is_zero() {
            true => None,
            false => Some(1.0 / self),
        }
    }
}
