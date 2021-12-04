use serde::Deserialize;
use ruststep::primitive::Logical;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum BSplineCurveForm {
    EllipticArc,
    PolylineForm,
    ParabolicArc,
    CircularArc,
    Unspecified,
    HyperbolicArc,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BSplineCurve<P> {
    pub degree: i64,
    pub control_points_list: Vec<P>,
    pub curve_form: BSplineCurveForm,
    pub closed_curve: Logical,
    pub self_intersect: Logical,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum BSplineCurveAny<P> {
    BSplineCurveWithKnots(Box<BSplineCurveWithKnots<P>>),
    BezierCurve(Box<BezierCurve<P>>),
    QuasiUniformCurve(Box<QuasiUniformCurve<P>>),
    RationalBSplineCurve(Box<RationalBSplineCurve<P>>),
    UniformCurve(Box<UniformCurve<P>>),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum KnotType {
    UniformKnots,
    QuasiUniformKnots,
    PiecewiseBezierKnots,
    Unspecified,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BSplineCurveWithKnots<P> {
    pub b_spline_curve: BSplineCurve<P>,
    pub knot_multiplicities: Vec<usize>,
    pub knots: Vec<f64>,
    pub knot_spec: KnotType,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BezierCurve<P> {
    pub b_spline_curve: BSplineCurve<P>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct QuasiUniformCurve<P> {
    pub b_spline_curve: BSplineCurve<P>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RationalBSplineCurve<P> {
    pub b_spline_curve: BSplineCurve<P>,
    pub weights_data: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct UniformCurve<P> {
    pub b_spline_curve: BSplineCurve<P>,
}


