//! Reads/writes STEP files from/to truck.

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

#[doc(hidden)]
pub mod alias;
mod impl_curve;
mod impl_surface;
/// STEP output module
pub mod out;
mod parse_primitives;

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
