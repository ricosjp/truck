use truck_meshalgo::prelude::*;

#[derive(Clone, Debug)]
pub enum Alternatives<T, U> {
	FirstType(T),
	SecondType(U),
}

#[macro_export]
macro_rules! impl_from {
	($firsttype: ty, $secondtype: ty) => {
		impl From<$firsttype> for $crate::test_util::Alternatives<$firsttype, $secondtype> {
			#[inline(always)]
			fn from(t: $firsttype) -> Self { $crate::test_util::Alternatives::FirstType(t) }
		}
		impl From<$secondtype> for $crate::test_util::Alternatives<$firsttype, $secondtype> {
			#[inline(always)]
			fn from(t: $secondtype) -> Self { $crate::test_util::Alternatives::SecondType(t) }
		}
	};
}

// test for impl_from
impl_from!((), usize);

macro_rules! derive_method {
	($method: tt, $return_type: ty, $($var: ident : $paramtype: ty),*) => {
		fn $method(&self, $($var: $paramtype),*) -> $return_type {
			match &self {
				Alternatives::FirstType(got) => got.$method($($var),*),
				Alternatives::SecondType(got) => got.$method($($var),*),
			}
		}
	};
}

impl<C0, C1> ParametricCurve for Alternatives<C0, C1>
where
	C0: ParametricCurve,
	C1: ParametricCurve<Point = C0::Point, Vector = C0::Vector>,
{
	type Point = C0::Point;
	type Vector = C0::Vector;
	derive_method!(subs, C0::Point, t: f64);
	derive_method!(der, C0::Vector, t: f64);
	derive_method!(der2, C0::Vector, t: f64);
	derive_method!(parameter_range, (f64, f64),);
}

impl<C0, C1> Cut for Alternatives<C0, C1>
where
	C0: Cut,
	C1: Cut<Point = C0::Point, Vector = C0::Vector>,
{
	fn cut(&mut self, t: f64) -> Self {
		match self {
			Self::FirstType(curve) => Self::FirstType(curve.cut(t)),
			Self::SecondType(curve) => Self::SecondType(curve.cut(t)),
		}
	}
}

impl<C0, C1> ParameterDivision1D for Alternatives<C0, C1>
where
	C0: ParameterDivision1D,
	C1: ParameterDivision1D,
{
	derive_method!(parameter_division, Vec<f64>, range: (f64, f64), tol: f64);
}

impl<T, U> SearchParameter for Alternatives<T, U>
where
	T: SearchParameter,
	U: SearchParameter<Point = T::Point, Parameter = T::Parameter>,
{
	type Point = T::Point;
	type Parameter = T::Parameter;
	derive_method!(
		search_parameter,
		Option<T::Parameter>,
		point: T::Point,
		hint: Option<T::Parameter>,
		trials: usize
	);
}

impl<T, U> SearchNearestParameter for Alternatives<T, U>
where
	T: SearchNearestParameter,
	U: SearchNearestParameter<Point = T::Point, Parameter = T::Parameter>,
{
	type Point = T::Point;
	type Parameter = T::Parameter;
	derive_method!(
		search_nearest_parameter,
		Option<T::Parameter>,
		point: T::Point,
		hint: Option<T::Parameter>,
		trials: usize
	);
}

impl<T, U> Invertible for Alternatives<T, U>
where
	T: Invertible,
	U: Invertible,
{
	#[inline(always)]
	fn invert(&mut self) {
		match self {
			Self::FirstType(entity) => entity.invert(),
			Self::SecondType(entity) => entity.invert(),
		}
	}
	#[inline(always)]
	fn inverse(&self) -> Self {
		match self {
			Self::FirstType(entity) => Self::FirstType(entity.inverse()),
			Self::SecondType(entity) => Self::SecondType(entity.inverse()),
		}
	}
}
