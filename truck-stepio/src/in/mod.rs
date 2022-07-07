#![allow(missing_docs)]

use ruststep::{
    ast::{DataSection, EntityInstance, Parameter, SubSuperRecord},
    error::Result,
    primitive::Logical,
    tables::PlaceHolder,
    Holder,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use truck_geometry::*;

use crate::alias::ExpressParseError;

/// type alias
pub mod alias;
use alias::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Table {
    // primitives
    pub cartesian_point: HashMap<u64, CartesianPointHolder>,
    pub direction: HashMap<u64, DirectionHolder>,
    pub vector: HashMap<u64, VectorHolder>,
    pub placement: HashMap<u64, PlacementHolder>,
    pub axis1_placement: HashMap<u64, Axis1PlacementHolder>,
    pub axis2_placement_2d: HashMap<u64, Axis2Placement2dHolder>,
    pub axis2_placement_3d: HashMap<u64, Axis2Placement3dHolder>,

    // curve
    pub line: HashMap<u64, LineHolder>,
    pub polyline: HashMap<u64, PolylineHolder>,
    pub b_spline_curve_with_knots: HashMap<u64, BSplineCurveWithKnotsHolder>,
    pub bezier_curve: HashMap<u64, BezierCurveHolder>,
    pub quasi_uniform_curve: HashMap<u64, QuasiUniformCurveHolder>,
    pub uniform_curve: HashMap<u64, UniformCurveHolder>,
    pub rational_b_spline_curve: HashMap<u64, RationalBSplineCurveHolder>,
    pub circle: HashMap<u64, CircleHolder>,
}

impl Table {
    pub fn from_data_section(data_section: &DataSection) -> Result<Table> {
        let mut table = Table::default();
        for instance in &data_section.entities {
            match instance {
                EntityInstance::Simple { id, record } => match record.name.as_str() {
                    "CARTESIAN_POINT" => {
                        table
                            .cartesian_point
                            .insert(*id, Deserialize::deserialize(record)?);
                    }
                    "DIRECTION" => {
                        table
                            .direction
                            .insert(*id, Deserialize::deserialize(record)?);
                    }
                    "VECTOR" => {
                        table.vector.insert(*id, Deserialize::deserialize(record)?);
                    }
                    "PLACEMENT" => {
                        table
                            .placement
                            .insert(*id, Deserialize::deserialize(record)?);
                    }
                    "AXIS1_PLACEMENT" => {
                        if let Parameter::List(params) = &record.parameter {
                            if params.len() != 3 {
                                Axis1PlacementHolder::deserialize(record)?;
                            }
                            table.axis1_placement.insert(
                                *id,
                                Axis1PlacementHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    location: Deserialize::deserialize(&params[1])?,
                                    direction: deserialize_option(&params[2])?,
                                },
                            );
                        } else {
                            Axis1PlacementHolder::deserialize(record)?;
                        }
                    }
                    "AXIS2_PLACEMENT_2D" => {
                        if let Parameter::List(params) = &record.parameter {
                            if params.len() != 3 {
                                Axis2Placement2dHolder::deserialize(record)?;
                            }
                            table.axis2_placement_2d.insert(
                                *id,
                                Axis2Placement2dHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    location: Deserialize::deserialize(&params[1])?,
                                    ref_direction: deserialize_option(&params[2])?,
                                },
                            );
                        } else {
                            Axis2Placement2dHolder::deserialize(record)?;
                        }
                    }
                    "AXIS2_PLACEMENT_3D" => {
                        if let Parameter::List(params) = &record.parameter {
                            if params.len() != 4 {
                                Axis2Placement2dHolder::deserialize(record)?;
                            }
                            table.axis2_placement_3d.insert(
                                *id,
                                Axis2Placement3dHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    location: Deserialize::deserialize(&params[1])?,
                                    axis: deserialize_option(&params[2])?,
                                    ref_direction: deserialize_option(&params[3])?,
                                },
                            );
                        } else {
                            Axis2Placement3dHolder::deserialize(record)?;
                        }
                    }
                    "LINE" => {
                        table.line.insert(*id, Deserialize::deserialize(record)?);
                    }
                    "POLYLINE" => {
                        table
                            .polyline
                            .insert(*id, Deserialize::deserialize(record)?);
                    }
                    "B_SPLINE_CURVE_WITH_KNOTS" => {
                        if let Parameter::List(params) = &record.parameter {
                            if params.len() != 9 {
                                BSplineCurveWithKnotsHolder::deserialize(record)?;
                            }
                            table.b_spline_curve_with_knots.insert(
                                *id,
                                BSplineCurveWithKnotsHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    degree: Deserialize::deserialize(&params[1])?,
                                    control_points_list: Deserialize::deserialize(&params[2])?,
                                    curve_form: Deserialize::deserialize(&params[3])?,
                                    closed_curve: deserialize_logical(&params[4])?,
                                    self_intersect: deserialize_logical(&params[5])?,
                                    knot_multiplicities: Deserialize::deserialize(&params[6])?,
                                    knots: Deserialize::deserialize(&params[7])?,
                                    knot_spec: Deserialize::deserialize(&params[8])?,
                                },
                            );
                        } else {
                            BSplineCurveWithKnotsHolder::deserialize(record)?;
                        }
                    }
                    "BEZIER_CURVE" => {
                        if let Parameter::List(params) = &record.parameter {
                            if params.len() != 6 {
                                BezierCurveHolder::deserialize(record)?;
                            }
                            table.bezier_curve.insert(
                                *id,
                                BezierCurveHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    degree: Deserialize::deserialize(&params[1])?,
                                    control_points_list: Deserialize::deserialize(&params[2])?,
                                    curve_form: Deserialize::deserialize(&params[3])?,
                                    closed_curve: deserialize_logical(&params[4])?,
                                    self_intersect: deserialize_logical(&params[5])?,
                                },
                            );
                        } else {
                            BezierCurveHolder::deserialize(record)?;
                        }
                    }
                    "QUASI_UNIFORM_CURVE" => {
                        if let Parameter::List(params) = &record.parameter {
                            if params.len() != 6 {
                                QuasiUniformCurveHolder::deserialize(record)?;
                            }
                            table.quasi_uniform_curve.insert(
                                *id,
                                QuasiUniformCurveHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    degree: Deserialize::deserialize(&params[1])?,
                                    control_points_list: Deserialize::deserialize(&params[2])?,
                                    curve_form: Deserialize::deserialize(&params[3])?,
                                    closed_curve: deserialize_logical(&params[4])?,
                                    self_intersect: deserialize_logical(&params[5])?,
                                },
                            );
                        } else {
                            QuasiUniformCurveHolder::deserialize(record)?;
                        }
                    }
                    "UNIFORM_CURVE" => {
                        if let Parameter::List(params) = &record.parameter {
                            if params.len() != 6 {
                                UniformCurveHolder::deserialize(record)?;
                            }
                            table.uniform_curve.insert(
                                *id,
                                UniformCurveHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    degree: Deserialize::deserialize(&params[1])?,
                                    control_points_list: Deserialize::deserialize(&params[2])?,
                                    curve_form: Deserialize::deserialize(&params[3])?,
                                    closed_curve: deserialize_logical(&params[4])?,
                                    self_intersect: deserialize_logical(&params[5])?,
                                },
                            );
                        } else {
                            UniformCurveHolder::deserialize(record)?;
                        }
                    }
                    _ => {}
                },
                EntityInstance::Complex {
                    id,
                    subsuper: SubSuperRecord(records),
                } => {
                    use NonRationalBSplineCurveHolder as NRBC;
                    if records.len() == 7 {
                        match (
                            records[0].name.as_str(),
                            records[1].name.as_str(),
                            &records[1].parameter,
                            records[2].name.as_str(),
                            &records[2].parameter,
                            records[3].name.as_str(),
                            records[4].name.as_str(),
                            records[5].name.as_str(),
                            &records[5].parameter,
                            records[6].name.as_str(),
                            &records[6].parameter,
                        ) {
                            (
                                "BOUNDED_CURVE",
                                "B_SPLINE_CURVE",
                                Parameter::List(bsp_params),
                                "B_SPLINE_CURVE_WITH_KNOTS",
                                Parameter::List(knots_params),
                                "CURVE",
                                "GEOMETRIC_REPRESENTATION_ITEM",
                                "RATIONAL_B_SPLINE_CURVE",
                                Parameter::List(weights),
                                "REPRESENTATION_ITEM",
                                Parameter::List(label),
                            ) => {
                                let mut params = vec![label[0].clone()];
                                params.extend(bsp_params.iter().cloned());
                                params.extend(knots_params.iter().cloned());
                                table.rational_b_spline_curve.insert(
                                    *id,
                                    RationalBSplineCurveHolder {
                                        non_rational_b_spline_curve: PlaceHolder::Owned(
                                            NRBC::BSplineCurveWithKnots(
                                                BSplineCurveWithKnotsHolder {
                                                    label: Deserialize::deserialize(&params[0])?,
                                                    degree: Deserialize::deserialize(&params[1])?,
                                                    control_points_list: Deserialize::deserialize(
                                                        &params[2],
                                                    )?,
                                                    curve_form: Deserialize::deserialize(
                                                        &params[3],
                                                    )?,
                                                    closed_curve: deserialize_logical(&params[4])?,
                                                    self_intersect: deserialize_logical(
                                                        &params[5],
                                                    )?,
                                                    knot_multiplicities: Deserialize::deserialize(
                                                        &params[6],
                                                    )?,
                                                    knots: Deserialize::deserialize(&params[7])?,
                                                    knot_spec: Deserialize::deserialize(
                                                        &params[8],
                                                    )?,
                                                },
                                            ),
                                        ),
                                        weights_data: Deserialize::deserialize(&weights[0])?,
                                    },
                                );
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(table)
    }
}

#[inline(always)]
fn deserialize_option<'de, T: Deserialize<'de>>(parameter: &Parameter) -> Result<Option<T>> {
    match parameter {
        Parameter::NotProvided => Ok(None),
        _ => Ok(Some(T::deserialize(parameter)?)),
    }
}

fn deserialize_logical(parameter: &Parameter) -> Result<Logical> {
    #[derive(Deserialize)]
    enum CharLogical {
        U,
        F,
        T,
    }
    impl From<CharLogical> for Logical {
        fn from(x: CharLogical) -> Logical {
            match x {
                CharLogical::T => Logical::True,
                CharLogical::F => Logical::False,
                CharLogical::U => Logical::Unknown,
            }
        }
    }

    Logical::deserialize(parameter).or_else(|_| CharLogical::deserialize(parameter).map(Into::into))
}

/// `cartesian_point`
#[derive(Clone, Debug, Default, PartialEq, Serialize, Holder)]
#[holder(table = Table)]
#[holder(field = cartesian_point)]
#[holder(generate_deserialize)]
pub struct CartesianPoint {
    pub label: String,
    pub coordinates: Vec<f64>,
}
impl From<&CartesianPoint> for Point2 {
    fn from(pt: &CartesianPoint) -> Self {
        let pt = &pt.coordinates;
        match pt.len() {
            0 => Point2::origin(),
            1 => Point2::new(pt[0], 0.0),
            _ => Point2::new(pt[0], pt[1]),
        }
    }
}
impl From<&CartesianPoint> for Point3 {
    fn from(pt: &CartesianPoint) -> Self {
        let pt = &pt.coordinates;
        match pt.len() {
            0 => Point3::origin(),
            1 => Point3::new(pt[0], 0.0, 0.0),
            2 => Point3::new(pt[0], pt[1], 0.0),
            _ => Point3::new(pt[0], pt[1], pt[2]),
        }
    }
}

/// `direction`
#[derive(Clone, Debug, Default, PartialEq, Serialize, Holder)]
#[holder(table = Table)]
#[holder(field = direction)]
#[holder(generate_deserialize)]
pub struct Direction {
    pub label: String,
    pub direction_ratios: Vec<f64>,
}
impl From<&Direction> for Vector2 {
    fn from(dir: &Direction) -> Self {
        let dir = &dir.direction_ratios;
        match dir.len() {
            0 => Vector2::zero(),
            1 => Vector2::new(dir[0], 0.0),
            _ => Vector2::new(dir[0], dir[1]),
        }
    }
}
impl From<&Direction> for Vector3 {
    fn from(dir: &Direction) -> Self {
        let dir = &dir.direction_ratios;
        match dir.len() {
            0 => Vector3::zero(),
            1 => Vector3::new(dir[0], 0.0, 0.0),
            2 => Vector3::new(dir[0], dir[1], 0.0),
            _ => Vector3::new(dir[0], dir[1], dir[2]),
        }
    }
}

/// `vector`
#[derive(Clone, Debug, Default, PartialEq, Serialize, Holder)]
#[holder(table = Table)]
#[holder(field = vector)]
#[holder(generate_deserialize)]
pub struct Vector {
    pub label: String,
    #[holder(use_place_holder)]
    pub orientation: Direction,
    pub magnitude: f64,
}
impl From<&Vector> for Vector2 {
    #[inline(always)]
    fn from(vec: &Vector) -> Self { Self::from(&vec.orientation) * vec.magnitude }
}
impl From<&Vector> for Vector3 {
    #[inline(always)]
    fn from(vec: &Vector) -> Self { Self::from(&vec.orientation) * vec.magnitude }
}

/// `placement`
#[derive(Clone, Debug, Default, PartialEq, Serialize, Holder)]
#[holder(table = Table)]
#[holder(field = placement)]
#[holder(generate_deserialize)]
pub struct Placement {
    pub label: String,
    #[holder(use_place_holder)]
    pub location: CartesianPoint,
}
impl From<&Placement> for Point2 {
    fn from(p: &Placement) -> Self { Self::from(&p.location) }
}
impl From<&Placement> for Point3 {
    fn from(p: &Placement) -> Self { Self::from(&p.location) }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Holder)]
#[holder(table = Table)]
#[holder(field = axis1_placement)]
#[holder(generate_deserialize)]
pub struct Axis1Placement {
    pub label: String,
    #[holder(use_place_holder)]
    pub location: CartesianPoint,
    #[holder(use_place_holder)]
    pub direction: Option<Direction>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Holder)]
#[holder(table = Table)]
#[holder(field = axis2_placement_2d)]
#[holder(generate_deserialize)]
pub struct Axis2Placement2d {
    pub label: String,
    #[holder(use_place_holder)]
    pub location: CartesianPoint,
    #[holder(use_place_holder)]
    pub ref_direction: Option<Direction>,
}

impl From<&Axis2Placement2d> for Matrix3 {
    fn from(axis: &Axis2Placement2d) -> Self {
        let z = Point2::from(&axis.location);
        let x = match &axis.ref_direction {
            Some(axis) => Vector2::from(axis),
            None => Vector2::unit_x(),
        };
        Matrix3::new(x.x, x.y, 0.0, -x.y, x.x, 0.0, z.x, z.y, 1.0)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Holder)]
#[holder(table = Table)]
#[holder(field = axis2_placement_3d)]
#[holder(generate_deserialize)]
pub struct Axis2Placement3d {
    pub label: String,
    #[holder(use_place_holder)]
    pub location: CartesianPoint,
    #[holder(use_place_holder)]
    pub axis: Option<Direction>,
    #[holder(use_place_holder)]
    pub ref_direction: Option<Direction>,
}
impl From<&Axis2Placement3d> for Matrix4 {
    fn from(axis: &Axis2Placement3d) -> Matrix4 {
        let w = Point3::from(&axis.location);
        let z = match &axis.axis {
            Some(axis) => Vector3::from(axis),
            None => Vector3::unit_z(),
        };
        let x = match &axis.ref_direction {
            Some(axis) => Vector3::from(axis),
            None => Vector3::unit_x(),
        };
        let x = (x - x.dot(z) * z).normalize();
        let y = z.cross(x);
        Matrix4::new(
            x.x, x.y, x.z, 0.0, y.x, y.y, y.z, 0.0, z.x, z.y, z.z, 0.0, w.x, w.y, w.z, 1.0,
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Holder)]
#[holder(table = Table)]
#[holder(field = line)]
#[holder(generate_deserialize)]
pub struct Line {
    pub label: String,
    #[holder(use_place_holder)]
    pub pnt: CartesianPoint,
    #[holder(use_place_holder)]
    pub dir: Vector,
}
impl<'a, P> From<&'a Line> for truck_geometry::Line<P>
where
    P: EuclideanSpace + From<&'a CartesianPoint>,
    P::Diff: From<&'a Vector>,
{
    fn from(line: &'a Line) -> Self {
        let p = P::from(&line.pnt);
        let q = p + P::Diff::from(&line.dir);
        Self(p, q)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Holder)]
#[holder(table = Table)]
#[holder(field = polyline)]
#[holder(generate_deserialize)]
pub struct Polyline {
    pub label: String,
    #[holder(use_place_holder)]
    pub points: Vec<CartesianPoint>,
}
impl<'a, P: From<&'a CartesianPoint>> From<&'a Polyline> for PolylineCurve<P> {
    fn from(poly: &'a Polyline) -> Self { Self(poly.points.iter().map(|pt| P::from(pt)).collect()) }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BSplineCurveForm {
    PolylineForm,
    CircularArc,
    EllipticArc,
    ParabolicArc,
    HyperbolicArc,
    Unspecified,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KnotType {
    UniformKnots,
    Unspecified,
    QuasiUniformKnots,
    PiecewiseBezierKnots,
}

#[derive(Clone, Debug, PartialEq, Serialize, Holder)]
#[holder(table = Table)]
#[holder(field = b_spline_curve_with_knots)]
#[holder(generate_deserialize)]
pub struct BSplineCurveWithKnots {
    pub label: String,
    pub degree: i64,
    #[holder(use_place_holder)]
    pub control_points_list: Vec<CartesianPoint>,
    pub curve_form: BSplineCurveForm,
    pub closed_curve: Logical,
    pub self_intersect: Logical,
    pub knot_multiplicities: Vec<i64>,
    pub knots: Vec<f64>,
    pub knot_spec: KnotType,
}
impl<P: for<'a> From<&'a CartesianPoint>> TryFrom<&BSplineCurveWithKnots> for BSplineCurve<P> {
    type Error = ExpressParseError;
    fn try_from(curve: &BSplineCurveWithKnots) -> std::result::Result<Self, ExpressParseError> {
        let knots = curve.knots.iter().map(|a| *a).collect();
        let multi = curve
            .knot_multiplicities
            .iter()
            .map(|n| *n as usize)
            .collect();
        let knots = KnotVec::from_single_multi(knots, multi).unwrap();
        let ctrpts = curve.control_points_list.iter().map(Into::into).collect();
        Self::try_new(knots, ctrpts).map_err(|x| x.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Holder)]
#[holder(table = Table)]
#[holder(field = bezier_curve)]
#[holder(generate_deserialize)]
pub struct BezierCurve {
    pub label: String,
    pub degree: i64,
    #[holder(use_place_holder)]
    pub control_points_list: Vec<CartesianPoint>,
    pub curve_form: BSplineCurveForm,
    pub closed_curve: Logical,
    pub self_intersect: Logical,
}
impl<P: for<'a> From<&'a CartesianPoint>> TryFrom<&BezierCurve> for BSplineCurve<P> {
    type Error = ExpressParseError;
    fn try_from(curve: &BezierCurve) -> std::result::Result<Self, ExpressParseError> {
        let degree = curve.degree as usize;
        let knots = KnotVec::bezier_knot(degree);
        let ctrpts = curve.control_points_list.iter().map(Into::into).collect();
        Self::try_new(knots, ctrpts).map_err(|x| x.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Holder)]
#[holder(table = Table)]
#[holder(field = quasi_uniform_curve)]
#[holder(generate_deserialize)]
pub struct QuasiUniformCurve {
    pub label: String,
    pub degree: i64,
    #[holder(use_place_holder)]
    pub control_points_list: Vec<CartesianPoint>,
    pub curve_form: BSplineCurveForm,
    pub closed_curve: Logical,
    pub self_intersect: Logical,
}
impl<P: for<'a> From<&'a CartesianPoint>> TryFrom<&QuasiUniformCurve> for BSplineCurve<P> {
    type Error = ExpressParseError;
    fn try_from(curve: &QuasiUniformCurve) -> std::result::Result<Self, ExpressParseError> {
        let num_ctrl = curve.control_points_list.len();
        let degree = curve.degree as usize;
        let division = num_ctrl + 2 - degree;
        let mut knots = KnotVec::uniform_knot(degree, division);
        knots.transform(division as f64, 0.0);
        let ctrpts = curve.control_points_list.iter().map(Into::into).collect();
        Self::try_new(knots, ctrpts).map_err(|x| x.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Holder)]
#[holder(table = Table)]
#[holder(field = uniform_curve)]
#[holder(generate_deserialize)]
pub struct UniformCurve {
    pub label: String,
    pub degree: i64,
    #[holder(use_place_holder)]
    pub control_points_list: Vec<CartesianPoint>,
    pub curve_form: BSplineCurveForm,
    pub closed_curve: Logical,
    pub self_intersect: Logical,
}
impl<P: for<'a> From<&'a CartesianPoint>> TryFrom<&UniformCurve> for BSplineCurve<P> {
    type Error = ExpressParseError;
    fn try_from(curve: &UniformCurve) -> std::result::Result<Self, ExpressParseError> {
        let num_ctrl = curve.control_points_list.len();
        let degree = curve.degree as usize;
        let knots = KnotVec::try_from(
            (0..degree + num_ctrl + 1)
                .map(|i| i as f64 - degree as f64)
                .collect::<Vec<_>>(),
        );
        let ctrpts = curve.control_points_list.iter().map(Into::into).collect();
        Self::try_new(knots.unwrap(), ctrpts).map_err(|x| x.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum NonRationalBSplineCurve {
    #[holder(use_place_holder)]
    BSplineCurveWithKnots(BSplineCurveWithKnots),
    #[holder(use_place_holder)]
    BezierCurve(BezierCurve),
    #[holder(use_place_holder)]
    QuasiUniformCurve(QuasiUniformCurve),
    #[holder(use_place_holder)]
    UniformCurve(UniformCurve),
}

impl<P: for<'a> From<&'a CartesianPoint>> TryFrom<&NonRationalBSplineCurve> for BSplineCurve<P> {
    type Error = ExpressParseError;
    fn try_from(curve: &NonRationalBSplineCurve) -> std::result::Result<Self, ExpressParseError> {
        use NonRationalBSplineCurve::*;
        match curve {
            BSplineCurveWithKnots(x) => x.try_into(),
            BezierCurve(x) => x.try_into(),
            QuasiUniformCurve(x) => x.try_into(),
            UniformCurve(x) => x.try_into(),
        }
    }
}

/// This struct is an ad hoc implementation that differs from the definition by EXPRESS:
/// in AP042, rationalized curves are defined as complex entities,
/// but here the curves before rationalization are held as internal variables.
#[derive(Clone, Debug, PartialEq, Serialize, Holder)]
#[holder(table = Table)]
#[holder(field = rational_b_spline_curve)]
#[holder(generate_deserialize)]
pub struct RationalBSplineCurve {
    #[holder(use_place_holder)]
    pub non_rational_b_spline_curve: NonRationalBSplineCurve,
    pub weights_data: Vec<f64>,
}
impl<V> TryFrom<&RationalBSplineCurve> for NURBSCurve<V>
where
    V: Homogeneous<f64>,
    V::Point: for<'a> From<&'a CartesianPoint>,
{
    type Error = ExpressParseError;
    fn try_from(curve: &RationalBSplineCurve) -> std::result::Result<Self, ExpressParseError> {
        Self::try_from_bspline_and_weights(
            BSplineCurve::try_from(&curve.non_rational_b_spline_curve)?,
            curve.weights_data.clone(),
        )
        .map_err(|x| x.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum BSplineCurveAny {
    #[holder(use_place_holder)]
    BSplineCurveWithKnots(BSplineCurveWithKnots),
    #[holder(use_place_holder)]
    BezierCurve(BezierCurve),
    #[holder(use_place_holder)]
    QuasiUniformCurve(QuasiUniformCurve),
    #[holder(use_place_holder)]
    UniformCurve(UniformCurve),
    #[holder(use_place_holder)]
    RationalBSplineCurve(RationalBSplineCurve),
}

#[derive(Clone, Debug, PartialEq, Serialize, Holder)]
#[holder(table = Table)]
#[holder(field = circle)]
#[holder(generate_deserialize)]
pub struct Circle {
    pub label: String,
    #[holder(use_place_holder)]
    pub position: Axis1Placement,
    pub radius: f64,
}
