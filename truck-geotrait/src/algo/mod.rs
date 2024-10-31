#![allow(clippy::many_single_char_names)]

use crate::traits::*;
use truck_base::{cgmath64::*, hash::HashGen, tolerance::*};

/// A structure that stores logs for debugging.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct NewtonLog<T>(
    #[cfg(all(test, debug_assertions))] Vec<T>,
    std::marker::PhantomData<T>,
);

impl<T: std::fmt::Debug> NewtonLog<T> {
    #[inline(always)]
    fn new(_trials: usize) -> Self {
        Self(
            #[cfg(all(test, debug_assertions))]
            Vec::with_capacity(_trials),
            std::marker::PhantomData::<T>,
        )
    }
    #[inline(always)]
    fn push(&mut self, _x: T) {
        #[cfg(all(test, debug_assertions))]
        self.0.push(_x)
    }
    #[inline(always)]
    fn print_error(self) {
        #[cfg(all(test, debug_assertions))]
        {
            eprintln!("Newton method is not converges");
            self.0.into_iter().for_each(|t| eprintln!("{:?}", t));
        }
    }
}

/// curve algorithms
pub mod curve;
/// surface algorithms
pub mod surface;
