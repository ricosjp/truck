#![allow(clippy::many_single_char_names)]

use crate::traits::*;
use truck_base::{
    cgmath64::*,
    hash::HashGen,
    newton::{self, CalcOutput},
    tolerance::*,
};

/// curve algorithms
pub mod curve;
/// surface algorithms
pub mod surface;
