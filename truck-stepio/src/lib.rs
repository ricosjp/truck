pub mod alias;
mod impl_curve;
mod impl_surface;
mod parse_primitives;
mod toporep;

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
