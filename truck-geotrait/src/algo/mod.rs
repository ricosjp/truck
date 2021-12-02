#![allow(clippy::many_single_char_names)]

use crate::traits::*;
use truck_base::{cgmath64::*, tolerance::*};

#[cfg(all(test, debug_assertions))]
macro_rules! newton_log_error {
	($log: expr) => {
        eprintln!("Newton method is not converges");
        $log.into_iter().for_each(|t| eprintln!("{:?}", t));
	};
}

/// curve algorithms
pub mod curve;
/// surface algorithms
pub mod surface;
