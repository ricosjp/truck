#![allow(missing_docs)]

use ruststep::{
    ast::{DataSection, EntityInstance, Name, Parameter, SubSuperRecord},
    error::Result,
    primitive::Logical,
    tables::{EntityTable, IntoOwned, PlaceHolder},
    Holder,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use truck_geometry::*;
use truck_topology::compress::*;

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

    // surface
    pub plane: HashMap<u64, PlaneHolder>,
    pub b_spline_surface_with_knots: HashMap<u64, BSplineSurfaceWithKnotsHolder>,

    // topology
    pub vertex_point: HashMap<u64, VertexPointHolder>,
    pub edge_curve: HashMap<u64, EdgeCurveHolder>,
    pub oriented_edge: HashMap<u64, OrientedEdgeHolder>,
    pub edge_loop: HashMap<u64, EdgeLoopHolder>,
    pub face_bound: HashMap<u64, FaceBoundHolder>,
    pub face_surface: HashMap<u64, FaceSurfaceHolder>,
    pub oriented_face: HashMap<u64, OrientedFaceHolder>,
    pub shell: HashMap<u64, ShellHolder>,
    pub oriented_shell: HashMap<u64, OrientedShellHolder>,
}

impl Table {
    pub fn push_instance(&mut self, instance: &EntityInstance) -> Result<()> {
        match instance {
            EntityInstance::Simple { id, record } => match record.name.as_str() {
                "CARTESIAN_POINT" => {
                    self.cartesian_point
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "DIRECTION" => {
                    self.direction
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "VECTOR" => {
                    self.vector.insert(*id, Deserialize::deserialize(record)?);
                }
                "PLACEMENT" => {
                    self.placement
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "AXIS1_PLACEMENT" => {
                    if let Parameter::List(params) = &record.parameter {
                        if params.len() != 3 {
                            Axis1PlacementHolder::deserialize(record)?;
                        }
                        self.axis1_placement.insert(
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
                        self.axis2_placement_2d.insert(
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
                        self.axis2_placement_3d.insert(
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
                    self.line
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "POLYLINE" => {
                    self.polyline.insert(*id, Deserialize::deserialize(record)?);
                }
                "B_SPLINE_CURVE_WITH_KNOTS" => {
                    if let Parameter::List(params) = &record.parameter {
                        if params.len() != 9 {
                            BSplineCurveWithKnotsHolder::deserialize(record)?;
                        }
                        self.b_spline_curve_with_knots.insert(
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
                        self.bezier_curve.insert(
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
                        self.quasi_uniform_curve.insert(
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
                        self.uniform_curve.insert(
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
                "CIRCLE" => {
                    self.circle.insert(*id, CircleHolder::deserialize(record)?);
                }
                "PLANE" => {
                    self.plane.insert(*id, PlaneHolder::deserialize(record)?);
                }
                "B_SPLINE_SURFACE_WITH_KNOTS" => {
                    if let Parameter::List(params) = &record.parameter {
                        if params.len() == 13 {
                            self.b_spline_surface_with_knots.insert(
                                *id,
                                BSplineSurfaceWithKnotsHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    u_degree: Deserialize::deserialize(&params[1])?,
                                    v_degree: Deserialize::deserialize(&params[2])?,
                                    control_points_list: Deserialize::deserialize(&params[3])?,
                                    surface_form: Deserialize::deserialize(&params[4])?,
                                    u_closed: deserialize_logical(&params[5])?,
                                    v_closed: deserialize_logical(&params[6])?,
                                    self_intersect: deserialize_logical(&params[7])?,
                                    u_multiplicities: Deserialize::deserialize(&params[8])?,
                                    v_multiplicities: Deserialize::deserialize(&params[9])?,
                                    u_knots: Deserialize::deserialize(&params[10])?,
                                    v_knots: Deserialize::deserialize(&params[11])?,
                                    knot_spec: Deserialize::deserialize(&params[12])?,
                                },
                            );
                        }
                    }
                }

                "VERTEX_POINT" => {
                    self.vertex_point
                        .insert(*id, VertexPointHolder::deserialize(record)?);
                }
                "EDGE_CURVE" => {
                    if let Parameter::List(params) = &record.parameter {
                        if params.len() == 5 {
                            self.edge_curve.insert(
                                *id,
                                EdgeCurveHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    edge_start: Deserialize::deserialize(&params[1])?,
                                    edge_end: Deserialize::deserialize(&params[2])?,
                                    edge_geometry: Deserialize::deserialize(&params[3])?,
                                    same_sense: deserialize_bool(&params[4])?,
                                },
                            );
                        }
                    }
                }
                "ORIENTED_EDGE" => {
                    if let Parameter::List(params) = &record.parameter {
                        if params.len() == 5 {
                            self.oriented_edge.insert(
                                *id,
                                OrientedEdgeHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    edge_element: Deserialize::deserialize(&params[3])?,
                                    orientation: deserialize_bool(&params[4])?,
                                },
                            );
                        }
                    }
                }
                "EDGE_LOOP" => {
                    self.edge_loop
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "FACE_BOUND" => {
                    if let Parameter::List(params) = &record.parameter {
                        if params.len() == 3 {
                            self.face_bound.insert(
                                *id,
                                FaceBoundHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    bound: Deserialize::deserialize(&params[1])?,
                                    orientation: deserialize_bool(&params[2])?,
                                },
                            );
                        }
                    }
                }
                "FACE_OUTER_BOUND" => {
                    if let Parameter::List(params) = &record.parameter {
                        if params.len() == 3 {
                            self.face_bound.insert(
                                *id,
                                FaceBoundHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    bound: Deserialize::deserialize(&params[1])?,
                                    orientation: deserialize_bool(&params[2])?,
                                },
                            );
                        }
                    }
                }
                "FACE_SURFACE" => {
                    if let Parameter::List(params) = &record.parameter {
                        if params.len() == 4 {
                            self.face_surface.insert(
                                *id,
                                FaceSurfaceHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    bounds: Deserialize::deserialize(&params[1])?,
                                    face_geometry: Deserialize::deserialize(&params[2])?,
                                    same_sense: deserialize_bool(&params[3])?,
                                },
                            );
                        }
                    }
                }
                "ADVANCED_FACE" => {
                    if let Parameter::List(params) = &record.parameter {
                        if params.len() == 4 {
                            self.face_surface.insert(
                                *id,
                                FaceSurfaceHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    bounds: Deserialize::deserialize(&params[1])?,
                                    face_geometry: Deserialize::deserialize(&params[2])?,
                                    same_sense: deserialize_bool(&params[3])?,
                                },
                            );
                        }
                    }
                }
                "ORIENTED_FACE" => {
                    if let Parameter::List(params) = &record.parameter {
                        if params.len() == 4 {
                            self.oriented_face.insert(
                                *id,
                                OrientedFaceHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    face_element: Deserialize::deserialize(&params[2])?,
                                    orientation: deserialize_bool(&params[3])?,
                                },
                            );
                        }
                    }
                }
                "OPEN_SHELL" => {
                    self.shell
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "CLOSED_SHELL" => {
                    self.shell
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "ORIENTED_OPEN_SHELL" => {
                    if let Parameter::List(params) = &record.parameter {
                        if params.len() == 4 {
                            self.oriented_shell.insert(
                                *id,
                                OrientedShellHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    shell_element: Deserialize::deserialize(&params[2])?,
                                    orientation: deserialize_bool(&params[3])?,
                                },
                            );
                        }
                    }
                }
                "ORIENTED_CLOSED_SHELL" => {
                    if let Parameter::List(params) = &record.parameter {
                        if params.len() == 4 {
                            self.oriented_shell.insert(
                                *id,
                                OrientedShellHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    shell_element: Deserialize::deserialize(&params[2])?,
                                    orientation: deserialize_bool(&params[3])?,
                                },
                            );
                        }
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
                        &records[0].parameter,
                        records[1].name.as_str(),
                        &records[1].parameter,
                        records[2].name.as_str(),
                        &records[2].parameter,
                        records[3].name.as_str(),
                        &records[3].parameter,
                        records[4].name.as_str(),
                        &records[4].parameter,
                        records[5].name.as_str(),
                        &records[5].parameter,
                        records[6].name.as_str(),
                        &records[6].parameter,
                    ) {
                        (
                            "BOUNDED_CURVE",
                            _,
                            "B_SPLINE_CURVE",
                            Parameter::List(bsp_params),
                            "B_SPLINE_CURVE_WITH_KNOTS",
                            Parameter::List(knots_params),
                            "CURVE",
                            _,
                            "GEOMETRIC_REPRESENTATION_ITEM",
                            _,
                            "RATIONAL_B_SPLINE_CURVE",
                            Parameter::List(weights),
                            "REPRESENTATION_ITEM",
                            Parameter::List(label),
                        ) => {
                            self.rational_b_spline_curve.insert(
                                *id,
                                RationalBSplineCurveHolder {
                                    non_rational_b_spline_curve: PlaceHolder::Owned(
                                        NRBC::BSplineCurveWithKnots(BSplineCurveWithKnotsHolder {
                                            label: Deserialize::deserialize(&label[0])?,
                                            degree: Deserialize::deserialize(&bsp_params[0])?,
                                            control_points_list: Deserialize::deserialize(
                                                &bsp_params[1],
                                            )?,
                                            curve_form: Deserialize::deserialize(&bsp_params[2])?,
                                            closed_curve: deserialize_logical(&bsp_params[3])?,
                                            self_intersect: deserialize_logical(&bsp_params[4])?,
                                            knot_multiplicities: Deserialize::deserialize(
                                                &knots_params[0],
                                            )?,
                                            knots: Deserialize::deserialize(&knots_params[1])?,
                                            knot_spec: Deserialize::deserialize(&knots_params[2])?,
                                        }),
                                    ),
                                    weights_data: Deserialize::deserialize(&weights[0])?,
                                },
                            );
                        }
                        (
                            "BEZIER_CURVE",
                            _,
                            "BOUNDED_CURVE",
                            _,
                            "B_SPLINE_CURVE",
                            Parameter::List(bsp_params),
                            "CURVE",
                            _,
                            "GEOMETRIC_REPRESENTATION_ITEM",
                            _,
                            "RATIONAL_B_SPLINE_CURVE",
                            Parameter::List(weights),
                            "REPRESENTATION_ITEM",
                            Parameter::List(label),
                        ) => {
                            let mut params = vec![label[0].clone()];
                            params.extend(bsp_params.iter().cloned());
                            self.rational_b_spline_curve.insert(
                                *id,
                                RationalBSplineCurveHolder {
                                    non_rational_b_spline_curve: PlaceHolder::Owned(
                                        NRBC::BezierCurve(BezierCurveHolder {
                                            label: Deserialize::deserialize(&params[0])?,
                                            degree: Deserialize::deserialize(&params[1])?,
                                            control_points_list: Deserialize::deserialize(
                                                &params[2],
                                            )?,
                                            curve_form: Deserialize::deserialize(&params[3])?,
                                            closed_curve: deserialize_logical(&params[4])?,
                                            self_intersect: deserialize_logical(&params[5])?,
                                        }),
                                    ),
                                    weights_data: Deserialize::deserialize(&weights[0])?,
                                },
                            );
                        }
                        (
                            "BOUNDED_CURVE",
                            _,
                            "B_SPLINE_CURVE",
                            Parameter::List(bsp_params),
                            "CURVE",
                            _,
                            "GEOMETRIC_REPRESENTATION_ITEM",
                            _,
                            "QUASI_UNIFORM_CURVE",
                            _,
                            "RATIONAL_B_SPLINE_CURVE",
                            Parameter::List(weights),
                            "REPRESENTATION_ITEM",
                            Parameter::List(label),
                        ) => {
                            let mut params = vec![label[0].clone()];
                            params.extend(bsp_params.iter().cloned());
                            self.rational_b_spline_curve.insert(
                                *id,
                                RationalBSplineCurveHolder {
                                    non_rational_b_spline_curve: PlaceHolder::Owned(
                                        NRBC::QuasiUniformCurve(QuasiUniformCurveHolder {
                                            label: Deserialize::deserialize(&params[0])?,
                                            degree: Deserialize::deserialize(&params[1])?,
                                            control_points_list: Deserialize::deserialize(
                                                &params[2],
                                            )?,
                                            curve_form: Deserialize::deserialize(&params[3])?,
                                            closed_curve: deserialize_logical(&params[4])?,
                                            self_intersect: deserialize_logical(&params[5])?,
                                        }),
                                    ),
                                    weights_data: Deserialize::deserialize(&weights[0])?,
                                },
                            );
                        }
                        (
                            "BOUNDED_CURVE",
                            _,
                            "B_SPLINE_CURVE",
                            Parameter::List(bsp_params),
                            "CURVE",
                            _,
                            "GEOMETRIC_REPRESENTATION_ITEM",
                            _,
                            "RATIONAL_B_SPLINE_CURVE",
                            Parameter::List(weights),
                            "REPRESENTATION_ITEM",
                            Parameter::List(label),
                            "UNIFORM_CURVE",
                            _,
                        ) => {
                            let mut params = vec![label[0].clone()];
                            params.extend(bsp_params.iter().cloned());
                            self.rational_b_spline_curve.insert(
                                *id,
                                RationalBSplineCurveHolder {
                                    non_rational_b_spline_curve: PlaceHolder::Owned(
                                        NRBC::UniformCurve(UniformCurveHolder {
                                            label: Deserialize::deserialize(&params[0])?,
                                            degree: Deserialize::deserialize(&params[1])?,
                                            control_points_list: Deserialize::deserialize(
                                                &params[2],
                                            )?,
                                            curve_form: Deserialize::deserialize(&params[3])?,
                                            closed_curve: deserialize_logical(&params[4])?,
                                            self_intersect: deserialize_logical(&params[5])?,
                                        }),
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
        Ok(())
    }
    #[inline(always)]
    pub fn from_data_section(data_section: &DataSection) -> Table {
        Table::from_iter(&data_section.entities)
    }
}

impl<'a> FromIterator<&'a EntityInstance> for Table {
    fn from_iter<I: IntoIterator<Item = &'a EntityInstance>>(iter: I) -> Table {
        let mut res = Table::default();
        iter.into_iter().for_each(|instance| {
            res.push_instance(instance)
                .unwrap_or_else(|e| eprintln!("{e}"))
        });
        res
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

fn deserialize_bool(parameter: &Parameter) -> Result<bool> {
    #[derive(Deserialize)]
    enum CharBool {
        F,
        T,
    }
    impl From<CharBool> for bool {
        fn from(x: CharBool) -> bool { matches!(x, CharBool::T) }
    }

    CharBool::deserialize(parameter).map(Into::into)
}

/// `cartesian_point`
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Holder)]
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
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Holder)]
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
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Holder)]
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
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Holder)]
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

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Holder)]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum Axis2Placement {
    #[holder(use_place_holder)]
    Axis2Placement2d(Axis2Placement2d),
    #[holder(use_place_holder)]
    Axis2Placement3d(Axis2Placement3d),
}

impl TryFrom<&Axis2Placement> for Matrix3 {
    type Error = ExpressParseError;
    fn try_from(axis: &Axis2Placement) -> std::result::Result<Self, ExpressParseError> {
        use Axis2Placement::*;
        match axis {
            Axis2Placement2d(axis) => Ok(Matrix3::from(axis)),
            Axis2Placement3d(_) => Err("This is not a 2D axis placement.".to_string()),
        }
    }
}
impl TryFrom<&Axis2Placement> for Matrix4 {
    type Error = ExpressParseError;
    fn try_from(axis: &Axis2Placement) -> std::result::Result<Self, ExpressParseError> {
        use Axis2Placement::*;
        match axis {
            Axis2Placement2d(_) => Err("This is not a 3D axis placement.".to_string()),
            Axis2Placement3d(axis) => Ok(Matrix4::from(axis)),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Holder)]
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

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Holder)]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum CurveAny {
    #[holder(use_place_holder)]
    #[holder(field = line)]
    Line(Line),
    #[holder(use_place_holder)]
    #[holder(field = polyline)]
    Polyline(Polyline),
    #[holder(use_place_holder)]
    BSplineCurve(BSplineCurveAny),
    #[holder(use_place_holder)]
    #[holder(field = circle)]
    Circle(Circle),
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Holder)]
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

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Holder)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BSplineCurveForm {
    PolylineForm,
    CircularArc,
    EllipticArc,
    ParabolicArc,
    HyperbolicArc,
    Unspecified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnotType {
    UniformKnots,
    Unspecified,
    QuasiUniformKnots,
    PiecewiseBezierKnots,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
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
        let knots = curve.knots.clone();
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
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
        let division = num_ctrl - degree;
        let mut knots = KnotVec::uniform_knot(degree, division);
        knots.transform(division as f64, 0.0);
        let ctrpts = curve.control_points_list.iter().map(Into::into).collect();
        Self::try_new(knots, ctrpts).map_err(|x| x.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum BSplineCurveAny {
    #[holder(use_place_holder)]
    #[holder(field = b_spline_curve_with_knots)]
    BSplineCurveWithKnots(BSplineCurveWithKnots),
    #[holder(use_place_holder)]
    #[holder(field = bezier_curve)]
    BezierCurve(BezierCurve),
    #[holder(use_place_holder)]
    #[holder(field = quasi_uniform_curve)]
    QuasiUniformCurve(QuasiUniformCurve),
    #[holder(use_place_holder)]
    #[holder(field = uniform_curve)]
    UniformCurve(UniformCurve),
    #[holder(use_place_holder)]
    #[holder(field = rational_b_spline_curve)]
    RationalBSplineCurve(RationalBSplineCurve),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = circle)]
#[holder(generate_deserialize)]
pub struct Circle {
    pub label: String,
    #[holder(use_place_holder)]
    pub position: Axis2Placement,
    pub radius: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum SurfaceAny {
    #[holder(use_place_holder)]
    #[holder(field = plane)]
    Plane(Plane),
    #[holder(use_place_holder)]
    #[holder(field = b_spline_surface_with_knots)]
    BSplineSurfaceWithKnots(BSplineSurfaceWithKnots),
}

impl TryFrom<&SurfaceAny> for Surface {
    type Error = ExpressParseError;
    fn try_from(x: &SurfaceAny) -> std::result::Result<Self, Self::Error> {
        use SurfaceAny::*;
        match x {
            Plane(plane) => Ok(Self::ElementarySurface(ElementarySurface::Plane(
                plane.into(),
            ))),
            BSplineSurfaceWithKnots(bsp) => Ok(Self::BSplineSurface(bsp.try_into()?)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = plane)]
#[holder(generate_deserialize)]
pub struct Plane {
    label: String,
    #[holder(use_place_holder)]
    position: Axis2Placement3d,
}

impl From<&Plane> for truck_geometry::Plane {
    fn from(plane: &Plane) -> Self {
        let mat = Matrix4::from(&plane.position);
        let o = Point3::from_homogeneous(mat[3]);
        let p = o + mat[0].truncate();
        let q = o + mat[1].truncate();
        Self::new(o, p, q)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BSplineSurfaceForm {
    PlaneSurf,
    CylindricalSurf,
    ConicalSurf,
    SphericalSurf,
    ToroidalSurf,
    SurfOfRevolution,
    RuledSurf,
    GeneralisedCone,
    QuadricSurf,
    SurfOfLinearExtrusion,
    Unspecified,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = b_spline_surface_with_knots)]
#[holder(generate_deserialize)]
pub struct BSplineSurfaceWithKnots {
    label: String,
    u_degree: i64,
    v_degree: i64,
    #[holder(use_place_holder)]
    control_points_list: Vec<Vec<CartesianPoint>>,
    surface_form: BSplineSurfaceForm,
    u_closed: Logical,
    v_closed: Logical,
    self_intersect: Logical,
    u_multiplicities: Vec<i64>,
    v_multiplicities: Vec<i64>,
    u_knots: Vec<f64>,
    v_knots: Vec<f64>,
    knot_spec: KnotType,
}

impl TryFrom<&BSplineSurfaceWithKnots> for BSplineSurface<Point3> {
    type Error = ExpressParseError;
    fn try_from(surface: &BSplineSurfaceWithKnots) -> std::result::Result<Self, ExpressParseError> {
        let uknots = surface.u_knots.to_vec();
        let umulti = surface
            .u_multiplicities
            .iter()
            .map(|n| *n as usize)
            .collect();
        let uknots = KnotVec::from_single_multi(uknots, umulti).unwrap();
        let vknots = surface.v_knots.to_vec();
        let vmulti = surface
            .v_multiplicities
            .iter()
            .map(|n| *n as usize)
            .collect();
        let vknots = KnotVec::from_single_multi(vknots, vmulti).unwrap();
        let ctrls = surface
            .control_points_list
            .iter()
            .map(|vec| vec.iter().map(Point3::from).collect())
            .collect();
        Self::try_new((uknots, vknots), ctrls).map_err(|x| x.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = vertex_point)]
#[holder(generate_deserialize)]
pub struct VertexPoint {
    pub label: String,
    #[holder(use_place_holder)]
    pub vertex_geometry: CartesianPoint,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum EdgeAny {
    #[holder(use_place_holder)]
    EdgeCurve(EdgeCurve),
    #[holder(use_place_holder)]
    OrientedEdge(OrientedEdge),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = edge_curve)]
#[holder(generate_deserialize)]
pub struct EdgeCurve {
    pub label: String,
    #[holder(use_place_holder)]
    pub edge_start: VertexPoint,
    #[holder(use_place_holder)]
    pub edge_end: VertexPoint,
    #[holder(use_place_holder)]
    pub edge_geometry: CurveAny,
    pub same_sense: bool,
}

impl EdgeCurve {
    pub fn parse_curve2d(&self) -> std::result::Result<Curve2D, ExpressParseError> {
        use CurveAny::*;
        let p = Point2::from(&self.edge_start.vertex_geometry);
        let q = Point2::from(&self.edge_end.vertex_geometry);
        let (p, q) = match self.same_sense {
            true => (p, q),
            false => (q, p),
        };
        let mut curve = match &self.edge_geometry {
            Line(_) => Curve2D::Line(truck_geometry::Line(p, q)),
            Polyline(poly) => Curve2D::Polyline(PolylineCurve::from(poly)),
            BSplineCurve(bspcurve) => match bspcurve {
                BSplineCurveAny::BSplineCurveWithKnots(bsp) => {
                    Curve2D::BSplineCurve(truck_geometry::BSplineCurve::try_from(bsp)?)
                }
                BSplineCurveAny::BezierCurve(bsp) => {
                    Curve2D::BSplineCurve(truck_geometry::BSplineCurve::try_from(bsp)?)
                }
                BSplineCurveAny::QuasiUniformCurve(bsp) => {
                    Curve2D::BSplineCurve(truck_geometry::BSplineCurve::try_from(bsp)?)
                }
                BSplineCurveAny::UniformCurve(bsp) => {
                    Curve2D::BSplineCurve(truck_geometry::BSplineCurve::try_from(bsp)?)
                }
                BSplineCurveAny::RationalBSplineCurve(bsp) => {
                    Curve2D::NURBSCurve(truck_geometry::NURBSCurve::try_from(bsp)?)
                }
            },
            Circle(circle) => {
                let mat = Matrix3::try_from(&circle.position)?;
                let inv_mat = mat.invert().ok_or("Failed to convert Circle".to_string())?;
                let (p, q) = (inv_mat.transform_point(p), inv_mat.transform_point(q));
                let (u, v) = (
                    UnitCircle::<Point2>::new()
                        .search_parameter(p, None, 0)
                        .ok_or("the point is not on circle".to_string())?,
                    UnitCircle::<Point2>::new()
                        .search_parameter(q, None, 0)
                        .ok_or("the point is not on circle".to_string())?,
                );
                let circle = TrimmedCurve::new(UnitCircle::<Point2>::new(), (u, v));
                let mut ellipse = Processor::new(circle);
                ellipse.transform_by(mat);
                Curve2D::Conic(Conic2D::Ellipse(ellipse))
            }
        };
        if !self.same_sense {
            curve.invert();
        }
        Ok(curve)
    }
    pub fn parse_curve3d(&self) -> std::result::Result<Curve3D, ExpressParseError> {
        use CurveAny::*;
        let p = Point3::from(&self.edge_start.vertex_geometry);
        let q = Point3::from(&self.edge_end.vertex_geometry);
        let (p, q) = match self.same_sense {
            true => (p, q),
            false => (q, p),
        };
        let mut curve = match &self.edge_geometry {
            Line(_) => Curve3D::Line(truck_geometry::Line(p, q)),
            Polyline(poly) => Curve3D::Polyline(PolylineCurve::from(poly)),
            BSplineCurve(bspcurve) => match bspcurve {
                BSplineCurveAny::BSplineCurveWithKnots(bsp) => {
                    Curve3D::BSplineCurve(truck_geometry::BSplineCurve::try_from(bsp)?)
                }
                BSplineCurveAny::BezierCurve(bsp) => {
                    Curve3D::BSplineCurve(truck_geometry::BSplineCurve::try_from(bsp)?)
                }
                BSplineCurveAny::QuasiUniformCurve(bsp) => {
                    Curve3D::BSplineCurve(truck_geometry::BSplineCurve::try_from(bsp)?)
                }
                BSplineCurveAny::UniformCurve(bsp) => {
                    Curve3D::BSplineCurve(truck_geometry::BSplineCurve::try_from(bsp)?)
                }
                BSplineCurveAny::RationalBSplineCurve(bsp) => {
                    Curve3D::NURBSCurve(truck_geometry::NURBSCurve::try_from(bsp)?)
                }
            },
            Circle(circle) => {
                let mat = Matrix4::try_from(&circle.position)?;
                let inv_mat = mat.invert().ok_or("Failed to convert Circle".to_string())?;
                let (p, q) = (inv_mat.transform_point(p), inv_mat.transform_point(q));
                let (u, v) = (
                    UnitCircle::<Point3>::new()
                        .search_parameter(p, None, 0)
                        .ok_or("the point is not on circle".to_string())?,
                    UnitCircle::<Point3>::new()
                        .search_parameter(q, None, 0)
                        .ok_or("the point is not on circle".to_string())?,
                );
                let circle = TrimmedCurve::new(UnitCircle::<Point3>::new(), (u, v));
                let mut ellipse = Processor::new(circle);
                ellipse.transform_by(mat);
                Curve3D::Conic(Conic3D::Ellipse(ellipse))
            }
        };
        if !self.same_sense {
            curve.invert();
        }
        Ok(curve)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = oriented_edge)]
#[holder(generate_deserialize)]
/// `ORIENTED_EDGE` has duplicated information.
/// These are not included here because they are essentially omitted.
pub struct OrientedEdge {
    pub label: String,
    #[holder(use_place_holder)]
    pub edge_element: EdgeCurve,
    pub orientation: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = edge_loop)]
#[holder(generate_deserialize)]
pub struct EdgeLoop {
    pub label: String,
    #[holder(use_place_holder)]
    pub edge_list: Vec<EdgeAny>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = face_bound)]
#[holder(generate_deserialize)]
/// `FACE_OUTER_BOUNDS` is also parsed to this struct.
pub struct FaceBound {
    pub label: String,
    // For now, we are going with the policy of accepting nothing but edgeloop.
    #[holder(use_place_holder)]
    pub bound: EdgeLoop,
    pub orientation: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum FaceAny {
    #[holder(use_place_holder)]
    FaceSurface(FaceSurface),
    #[holder(use_place_holder)]
    OrientedFace(OrientedFace),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = face_surface)]
#[holder(generate_deserialize)]
/// `ADVANCED_FACE` is also parsed to this struct.
pub struct FaceSurface {
    pub label: String,
    #[holder(use_place_holder)]
    pub bounds: Vec<FaceBound>,
    #[holder(use_place_holder)]
    pub face_geometry: SurfaceAny,
    pub same_sense: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = oriented_face)]
#[holder(generate_deserialize)]
/// `ORIENTED_EDGE` has duplicated information.
/// These are not included here because they are essentially omitted.
pub struct OrientedFace {
    pub label: String,
    #[holder(use_place_holder)]
    pub face_element: FaceSurface,
    pub orientation: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = shell)]
#[holder(generate_deserialize)]
/// Includes `OPEN_SHELL` and `CLOSED_SHELL`.
/// Since these differences are only informal propositions, the data structure does not distinguish between the two.
pub struct Shell {
    pub label: String,
    #[holder(use_place_holder)]
    pub cfs_faces: Vec<FaceAny>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = oriented_shell)]
#[holder(generate_deserialize)]
/// Includes `ORIENTED_OPEN_SHELL` and `ORIENTED_CLOSED_SHELL`.
/// Since these differences are only informal propositions, the data structure does not distinguish between the two.
pub struct OrientedShell {
    pub label: String,
    #[holder(use_place_holder)]
    pub shell_element: Shell,
    pub orientation: bool,
}

impl Table {
    pub fn to_compressed_shell(
        &self,
        shell: &ShellHolder,
    ) -> std::result::Result<CompressedShell<Point3, Curve3D, Surface>, ExpressParseError> {
        let mut vidx_map = HashMap::<u64, usize>::new();
        let mut eidx_map = HashMap::<u64, usize>::new();

        let vertices: Vec<Point3> = shell
            .cfs_faces
            .iter()
            .filter_map(|face| {
                if let PlaceHolder::Ref(Name::Entity(idx)) = face {
                    if let Some(face) = self.face_surface.get(&idx) {
                        Some(face)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .flat_map(|face| &face.bounds)
            .filter_map(|bound| {
                if let PlaceHolder::Ref(Name::Entity(idx)) = bound {
                    self.face_bound.get(&idx)
                } else {
                    None
                }
            })
            .filter_map(|bound| {
                if let PlaceHolder::Ref(Name::Entity(idx)) = bound.bound {
                    self.edge_loop.get(&idx)
                } else {
                    None
                }
            })
            .flat_map(|edge_loop| &edge_loop.edge_list)
            .filter_map(|edge| {
                if let PlaceHolder::Ref(Name::Entity(idx)) = edge {
                    let idx = if let Some(oriented_edge) = self.oriented_edge.get(&idx) {
                        if let PlaceHolder::Ref(Name::Entity(idx)) = oriented_edge.edge_element {
                            idx
                        } else {
                            *idx
                        }
                    } else {
                        *idx
                    };
                    self.edge_curve.get(&idx)
                } else {
                    None
                }
            })
            .flat_map(|edge| vec![&edge.edge_start, &edge.edge_end])
            .filter_map(|v| {
                if let PlaceHolder::Ref(Name::Entity(idx)) = v {
                    if vidx_map.get(&idx).is_none() {
                        let len = vidx_map.len();
                        vidx_map.insert(*idx, len);
                        let p = EntityTable::<VertexPointHolder>::get_owned(self, *idx).ok()?;
                        Some(Point3::from(&p.vertex_geometry))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        let edges: Vec<CompressedEdge<Curve3D>> = shell
            .cfs_faces
            .iter()
            .filter_map(|face| {
                if let PlaceHolder::Ref(Name::Entity(idx)) = face {
                    if let Some(face) = self.face_surface.get(&idx) {
                        Some(face)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .flat_map(|face| &face.bounds)
            .filter_map(|bound| {
                if let PlaceHolder::Ref(Name::Entity(idx)) = bound {
                    self.face_bound.get(&idx)
                } else {
                    None
                }
            })
            .filter_map(|bound| {
                if let PlaceHolder::Ref(Name::Entity(idx)) = bound.bound {
                    self.edge_loop.get(&idx)
                } else {
                    None
                }
            })
            .flat_map(|edge_loop| &edge_loop.edge_list)
            .filter_map(|edge| {
                if let PlaceHolder::Ref(Name::Entity(idx)) = edge {
                    let idx = if let Some(oriented_edge) = self.oriented_edge.get(&idx) {
                        if let PlaceHolder::Ref(Name::Entity(idx)) = oriented_edge.edge_element {
                            idx
                        } else {
                            *idx
                        }
                    } else {
                        *idx
                    };
                    Some((idx, self.edge_curve.get(&idx)?))
                } else {
                    None
                }
            })
            .filter_map(|(idx, edge)| {
                if eidx_map.get(&idx).is_some() {
                    return None;
                }
                let len = eidx_map.len();
                eidx_map.insert(idx, len);
                let edge_curve = edge.clone().into_owned(self).ok()?;
                let curve = edge_curve.parse_curve3d().ok()?;
                let front = if let PlaceHolder::Ref(Name::Entity(idx)) = edge.edge_start {
                    *vidx_map.get(&idx)?
                } else {
                    return None;
                };
                let back = if let PlaceHolder::Ref(Name::Entity(idx)) = edge.edge_end {
                    *vidx_map.get(&idx)?
                } else {
                    return None;
                };
                Some(CompressedEdge {
                    vertices: (front, back),
                    curve,
                })
            })
            .collect();

        let faces = shell
            .cfs_faces
            .iter()
            .filter_map(|face| {
                if let PlaceHolder::Ref(Name::Entity(idx)) = face {
                    let (flag, idx) = if let Some(face) = self.oriented_face.get(&idx) {
                        if let PlaceHolder::Ref(Name::Entity(idx)) = face.face_element {
                            (face.orientation, idx)
                        } else {
                            (true, *idx)
                        }
                    } else {
                        (true, *idx)
                    };
                    if let Some(face) = self.face_surface.get(&idx) {
                        let mut face = face.clone();
                        face.same_sense = face.same_sense == flag;
                        Some(face)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .filter_map(|face| {
                let step_surface: SurfaceAny = face.face_geometry.clone().into_owned(self).ok()?;
                let mut surface = Surface::try_from(&step_surface).ok()?;
                if !face.same_sense {
                    surface.invert()
                }
                let boundaries: Vec<_> = face
                    .bounds
                    .iter()
                    .filter_map(|bound| {
                        if let PlaceHolder::Ref(Name::Entity(idx)) = bound {
                            if let Some(bound) = self.face_bound.get(&idx) {
                                let ori = bound.orientation;
                                if let PlaceHolder::Ref(Name::Entity(idx)) = bound.bound {
                                    if let Some(edge_list) = self.edge_loop.get(&idx) {
                                        let edges = edge_list.edge_list.iter().filter_map(|edge| {
                                            if let PlaceHolder::Ref(Name::Entity(idx)) = edge {
                                                if let Some(oriented_edge) =
                                                    self.oriented_edge.get(&idx)
                                                {
                                                    if let PlaceHolder::Ref(Name::Entity(idx)) =
                                                        oriented_edge.edge_element
                                                    {
                                                        let edge_idx = *eidx_map.get(&idx)?;
                                                        return Some(CompressedEdgeIndex {
                                                            index: edge_idx,
                                                            orientation: oriented_edge.orientation,
                                                        });
                                                    } else {
                                                        return None;
                                                    }
                                                }
                                                let edge_idx = *vidx_map.get(&idx)?;
                                                Some(CompressedEdgeIndex {
                                                    index: edge_idx,
                                                    orientation: true,
                                                })
                                            } else {
                                                None
                                            }
                                        });
                                        Some((ori, edges.collect::<Vec<_>>()))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();
                let boundaries = boundaries
                    .into_iter()
                    .map(|(ori, mut boundary)| {
                        if !ori {
                            boundary.reverse();
                            boundary.iter_mut().for_each(|edge| {
                                edge.orientation = !edge.orientation;
                            });
                        }
                        boundary
                    })
                    .collect();
                Some(CompressedFace {
                    surface,
                    boundaries,
                    orientation: true,
                })
            })
            .collect();
        Ok(CompressedShell {
            vertices,
            edges,
            faces,
        })
    }
}
