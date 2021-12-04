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
pub struct Vector {
    pub orientation: Direction,
    pub magnitude: LengthMeasure,
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
