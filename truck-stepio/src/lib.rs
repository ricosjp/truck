//! Reads/writes STEP files from/to truck.
//!
//! # Current Status
//!
//! It is possible to output data modeled by truck-modeling.
//! Shapes created by set operations cannot be output yet.
//! Input will come further down the road.

#![cfg_attr(not(debug_assertions), deny(warnings))]
#![deny(clippy::all, rust_2018_idioms)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

/// STEP input module
/// # Example
/// ```
/// use truck_stepio::r#in::{*, step_geometry::*};
/// use ruststep::tables::EntityTable;
/// // read file
/// let step_string = include_str!(concat!(
///     env!("CARGO_MANIFEST_DIR"),
///     "/../resources/step/occt-cube.step",
/// ));
/// // parse step file
/// let exchange = ruststep::parser::parse(&step_string).unwrap();
/// // convert the parsing results to a Rust struct
/// let table = Table::from_data_section(&exchange.data[0]);
/// // get `CartesianPoint` registered in #102
/// let step_point = EntityTable::<CartesianPointHolder>::get_owned(&table, 102).unwrap();
/// // convert `CartesianPoint` in STEP to `Point3` in cgmath
/// let cgmath_point = Point3::from(&step_point);
/// // check parse result
/// assert_eq!(cgmath_point, Point3::new(0.0, 10.0, 0.0));
/// ```
#[cfg(feature = "in")]
pub mod r#in;
/// STEP output module
pub mod out;

/// common structures
pub mod common;
