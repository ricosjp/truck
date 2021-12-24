mod tentative {
	use derive_more::*;
	use ruststep::primitive::Logical;

	#[derive(
		Clone,
		Debug,
		PartialEq,
		AsRef,
		Deref,
		DerefMut,
		From,
		Into,
		:: serde :: Serialize,
		:: serde :: Deserialize,
	)]
	pub struct LengthMeasure(pub f64);

	#[derive(
		Clone,
		Debug,
		PartialEq,
		AsRef,
		Deref,
		DerefMut,
		From,
		Into,
		:: serde :: Serialize,
		:: serde :: Deserialize,
	)]
	pub struct PlaneAngleMeasure(pub f64);

	#[derive(
		Clone,
		Debug,
		PartialEq,
		AsRef,
		Deref,
		DerefMut,
		:: serde :: Serialize,
		:: serde :: Deserialize,
	)]
	pub struct PositiveLengthMeasure(pub LengthMeasure);

	#[derive(
		Clone,
		Debug,
		PartialEq,
		AsRef,
		Deref,
		DerefMut,
		From,
		Into,
		:: serde :: Serialize,
		:: serde :: Deserialize,
	)]
	pub struct ParameterValue(pub f64);

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct Direction {
		pub direction_ratios: Vec<f64>,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct CartesianPoint {
		pub coordinates: Vec<LengthMeasure>,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub enum CartesianPointAny {
		CartesianPoint(Box<CartesianPoint>),
	}

	impl AsRef<CartesianPoint> for CartesianPointAny {
		fn as_ref(&self) -> &CartesianPoint {
			match self {
				CartesianPointAny::CartesianPoint(got) => got,
			}
		}
	}

	impl std::ops::Deref for CartesianPointAny {
		type Target = CartesianPoint;
		fn deref(&self) -> &CartesianPoint {
			match self {
				CartesianPointAny::CartesianPoint(got) => got,
			}
		}
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct Vector {
		pub orientation: Direction,
		pub magnitude: LengthMeasure,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct Placement {
		pub location: CartesianPointAny,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct Axis2Placement2D {
		pub placement: Placement,
		pub ref_direction: Option<Direction>,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct Axis2Placement3D {
		pub placement: Placement,
		pub axis: Option<Direction>,
		pub ref_direction: Option<Direction>,
	}
	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub enum Axis2Placement {
		Axis2Placement2D(Box<Axis2Placement2D>),
		Axis2Placement3D(Box<Axis2Placement3D>),
	}
	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct Line {
		pub pnt: CartesianPointAny,
		pub dir: Vector,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct Conic {
		pub position: Axis2Placement,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct Circle {
		pub conic: Conic,
		pub radius: PositiveLengthMeasure,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct Ellipse {
		pub conic: Conic,
		pub semi_axis_1: PositiveLengthMeasure,
		pub semi_axis_2: PositiveLengthMeasure,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct Hyperbola {
		pub conic: Conic,
		pub semi_axis: PositiveLengthMeasure,
		pub semi_imag_axis: PositiveLengthMeasure,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct Parabola {
		pub conic: Conic,
		pub focal_dist: LengthMeasure,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct Polyline {
		pub points: Vec<CartesianPointAny>,
	}
	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub enum BSplineCurveForm {
		EllipticArc,
		PolylineForm,
		ParabolicArc,
		CircularArc,
		Unspecified,
		HyperbolicArc,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct BSplineCurve {
		pub degree: i64,
		pub control_points_list: Vec<CartesianPoint>,
		pub curve_form: BSplineCurveForm,
		pub closed_curve: Logical,
		pub self_intersect: Logical,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub enum BSplineCurveAny {
		BSplineCurveWithKnots(Box<BSplineCurveWithKnots>),
		BezierCurve(Box<BezierCurve>),
		QuasiUniformCurve(Box<QuasiUniformCurve>),
		RationalBSplineCurve(Box<RationalBSplineCurve>),
		UniformCurve(Box<UniformCurve>),
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub enum KnotType {
		UniformKnots,
		QuasiUniformKnots,
		PiecewiseBezierKnots,
		Unspecified,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct BSplineCurveWithKnots {
		pub b_spline_curve: BSplineCurve,
		pub knot_multiplicities: Vec<i64>,
		pub knots: Vec<ParameterValue>,
		pub knot_spec: KnotType,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct BezierCurve {
		pub b_spline_curve: BSplineCurve,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct QuasiUniformCurve {
		pub b_spline_curve: BSplineCurve,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct RationalBSplineCurve {
		pub b_spline_curve: BSplineCurve,
		pub weights_data: Vec<f64>,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct UniformCurve {
		pub b_spline_curve: BSplineCurve,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct ElementarySurface {
		pub position: Axis2Placement3D,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct Plane {
		pub elementary_surface: ElementarySurface,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct CylindricalSurface {
		pub elementary_surface: ElementarySurface,
		pub radius: PositiveLengthMeasure,
	}

	#[derive(Debug, Clone, PartialEq, :: serde :: Deserialize)]
	pub struct ConicalSurface {
		pub elementary_surface: ElementarySurface,
		pub radius: LengthMeasure,
		pub semi_angle: PlaneAngleMeasure,
	}
}

truck_stepio::parse_primitives!(tentative, __parse_primitives);
truck_stepio::impl_curve!(tentative, __impl_curve);
truck_stepio::impl_surface!(tentative, __impl_surface);
