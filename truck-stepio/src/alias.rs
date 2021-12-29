pub use truck_geometry::*;
pub use truck_polymesh::*;

pub type ExpressParseError = String;

pub trait Empty {
	fn empty() -> Self;
}

pub type Ellipse<P, M> = Processor<UnitCircle<P>, M>;
pub type Hyperbola<P, M> = Processor<UnitHyperbola<P>, M>;
pub type Parabola<P, M> = Processor<UnitParabola<P>, M>;
pub type RevolutedLine = Processor<RevolutedCurve<Line<Point3>>, Matrix4>;
pub type ToroidalSurface = Processor<RevolutedCurve<Ellipse<Point3, Matrix4>>, Matrix4>;
pub type StepExtrudedCurve = ExtrudedCurve<Curve<Point3, Vector4, Matrix4>, Vector3>;
pub type StepRevolutedCurve = Processor<RevolutedCurve<Curve<Point3, Vector4, Matrix4>>, Matrix4>;

macro_rules! derive_enum {
	($attr: meta, $typename: ident { $($member: ident ($subtype: ty),)* } $derive_macro_name: ident, $dol: tt) => {
		#[$attr]
		pub enum $typename {
			$($member($subtype),)*
		}
		macro_rules! $derive_macro_name {
			(fn $method: ident (&self, $dol ($field: ident : $field_type: ty),*) -> $return_type: ty) => {
				#[inline(always)]
				fn $method (&self, $dol ($field: $field_type),*) -> $return_type {
					match self {
						$($typename::$member(x) => x.$method($dol ($field),*)),*
					}
				}
			};
		}
	};
	(
		#[$attr: meta]
		pub enum $typename: ident {
			$($member: ident ($subtype: ty),)*
		},
		$derive_macro_name: ident
	) => {
		derive_enum!($attr, $typename { $($member ($subtype),)* } $derive_macro_name, $);
	};
	($attr: meta, $typename: ident <$($gen: tt),*> { $($member: ident ($subtype: ty),)* },
	$derive_macro_name: ident, $dol: tt) => {
		#[$attr]
		pub enum $typename <$($gen),*> {
			$($member($subtype),)*
		}
		macro_rules! $derive_macro_name {
			(fn $method: ident (&self, $dol ($field: ident : $field_type: ty),*) -> $return_type: ty) => {
				#[inline(always)]
				fn $method (&self, $dol ($field: $field_type),*) -> $return_type {
					match self {
						$($typename::$member(x) => x.$method($dol ($field),*)),*
					}
				}
			};
		}
	};
	(
		#[$attr: meta]
		pub enum $typename: ident <$($gen: tt),*> {
			$($member: ident ($subtype: ty),)*
		},
		$derive_macro_name: ident
	) => {
		derive_enum!($attr, $typename <$($gen),*> { $($member ($subtype),)* }, $derive_macro_name, $);
	};
}

macro_rules! derive_curve {
	($type: ty, $macro: ident, $point: ty, $vector: ty) => {
		impl ParametricCurve for $type {
			type Point = $point;
			type Vector = $vector;
			$macro!(fn subs(&self, t: f64) -> Self::Point);
			$macro!(fn der(&self, t: f64) -> Self::Vector);
			$macro!(fn der2(&self, t: f64) -> Self::Vector);
		}
		impl ParameterDivision1D for $type {
			type Point = $point;
			$macro!(fn parameter_division(&self, range: (f64, f64), tol: f64) -> (Vec<f64>, Vec<$point>));
		}
	};
}

derive_enum!(
	#[derive(Clone, Copy, Debug)]
	pub enum Conic<P, M> {
		Ellipse(Ellipse<P, M>),
		Hyperbola(Hyperbola<P, M>),
		Parabola(Parabola<P, M>),
	},
	derive_to_conic
);

derive_curve!(Conic<Point2, Matrix3>, derive_to_conic, Point2, Vector2);
derive_curve!(Conic<Point3, Matrix4>, derive_to_conic, Point3, Vector3);

#[derive(Clone, Debug)]
pub enum Curve<P, V, M> {
	Line(Line<P>),
	Conic(Conic<P, M>),
	Phantom(std::marker::PhantomData<V>),
}

macro_rules! derive_to_curve {
	(fn $method: ident (&self, $($field: ident : $field_type: ty),*) -> $return_type: ty) => {
		#[inline(always)]
		fn $method (&self, $($field: $field_type),*) -> $return_type {
			use Curve::*;
			match self {
				Line(x) => x.$method($($field),*),
				Conic(x) => x.$method($($field),*),
				Phantom(_) => unreachable!(),
			}
		}
	};
}

derive_curve!(Curve<Point2, Vector3, Matrix3>, derive_to_curve, Point2, Vector2);
derive_curve!(Curve<Point3, Vector4, Matrix4>, derive_to_curve, Point3, Vector3);

macro_rules! derive_surface {
	($type: ty, $macro: ident) => {
		impl ParametricSurface for $type {
			type Point = Point3;
			type Vector = Vector3;
			$macro!(fn subs(&self, u: f64, v: f64) -> Self::Point);
			$macro!(fn uder(&self, u: f64, v: f64) -> Self::Vector);
			$macro!(fn vder(&self, u: f64, v: f64) -> Self::Vector);
			$macro!(fn uuder(&self, u: f64, v: f64) -> Self::Vector);
			$macro!(fn uvder(&self, u: f64, v: f64) -> Self::Vector);
			$macro!(fn vvder(&self, u: f64, v: f64) -> Self::Vector);
		}
		impl ParameterDivision2D for $type {
			$macro!(fn parameter_division(&self, range: ((f64, f64), (f64, f64)), tol: f64) -> (Vec<f64>, Vec<f64>));
		}
	};
}

derive_enum!(
	#[derive(Clone, Copy, Debug)]
	pub enum ElementarySurface {
		RevolutedLine(RevolutedLine),
		Sphere(Processor<Sphere, Matrix4>),
		ToroidalSurface(ToroidalSurface),
	},
	derive_to_elementary_surface
);
derive_surface!(ElementarySurface, derive_to_elementary_surface);

derive_enum!(
	#[derive(Clone, Debug)]
	pub enum SweptCurve {
		ExtrudedCurve(StepExtrudedCurve),
		RevolutedCurve(StepRevolutedCurve),
	},
	derive_to_swept_curve
);
derive_surface!(SweptCurve, derive_to_swept_curve);

pub trait CurveFromExpress<P>: ParametricCurve<Point = P> + ParameterDivision1D<Point = P> {}
impl<P, C: ParametricCurve<Point = P> + ParameterDivision1D<Point = P>> CurveFromExpress<P> for C {}
pub trait SurfaceFromExpress: ParametricSurface3D + ParameterDivision2D {}
impl<S: ParametricSurface3D + ParameterDivision2D> SurfaceFromExpress for S {}
