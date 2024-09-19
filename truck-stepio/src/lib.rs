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
/// use truck_stepio::r#in::{*, alias::*};
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

#[doc(hidden)]
#[macro_export]
macro_rules! impl_from {
	($(impl From<&$refed: ty> for $converted: ty {
		$from_func: item
	})*) => {
		$(impl From<&$refed> for $converted {
			$from_func
		}
		impl From<$refed> for $converted {
			fn from(x: $refed) -> Self { Self::from(&x) }
		})*
	};
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_try_from {
	($(impl TryFrom<&$refed: ty> for $converted: ty {
		$try_from_func: item
	})*) => {
		$(impl TryFrom<&$refed> for $converted {
            type Error = ExpressParseError;
			$try_from_func
		}
		impl TryFrom<$refed> for $converted {
            type Error = ExpressParseError;
            fn try_from(x: $refed) -> Result<Self, ExpressParseError> { Self::try_from(&x) }
		})*
	};
}
