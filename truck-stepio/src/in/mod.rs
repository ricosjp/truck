#![allow(missing_docs)]

use ruststep::{
    ast::{DataSection, EntityInstance, Name, Parameter, SubSuperRecord},
    error::Result,
    primitive::Logical,
    tables::{EntityTable, IntoOwned, PlaceHolder},
    Holder,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, f64::consts::PI};
use truck_geometry::prelude as truck;
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
    pub ellipse: HashMap<u64, EllipseHolder>,
    pub pcurve: HashMap<u64, PcurveHolder>,
    pub surface_curve: HashMap<u64, SurfaceCurveHolder>,

    // surface
    pub plane: HashMap<u64, PlaneHolder>,
    pub spherical_surface: HashMap<u64, SphericalSurfaceHolder>,
    pub cylindrical_surface: HashMap<u64, CylindricalSurfaceHolder>,
    pub toroidal_surface: HashMap<u64, ToroidalSurfaceHolder>,
    pub conical_surface: HashMap<u64, ConicalSurfaceHolder>,
    pub b_spline_surface_with_knots: HashMap<u64, BSplineSurfaceWithKnotsHolder>,
    pub uniform_surface: HashMap<u64, UniformSurfaceHolder>,
    pub quasi_uniform_surface: HashMap<u64, QuasiUniformSurfaceHolder>,
    pub bezier_surface: HashMap<u64, BezierSurfaceHolder>,
    pub rational_b_spline_surface: HashMap<u64, RationalBSplineSurfaceHolder>,
    pub surface_of_linear_extrusion: HashMap<u64, SurfaceOfLinearExtrusionHolder>,
    pub surface_of_revolution: HashMap<u64, SurfaceOfRevolutionHolder>,

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

    // others
    pub definitional_representation: HashMap<u64, DefinitionalRepresentationHolder>,

    // dummy
    pub dummy: HashMap<u64, DummyHolder>,
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
                    self.axis1_placement
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "AXIS2_PLACEMENT_2D" => {
                    self.axis2_placement_2d
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "AXIS2_PLACEMENT_3D" => {
                    self.axis2_placement_3d
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "LINE" => {
                    self.line
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "POLYLINE" => {
                    self.polyline.insert(*id, Deserialize::deserialize(record)?);
                }
                "B_SPLINE_CURVE_WITH_KNOTS" => {
                    self.b_spline_curve_with_knots
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "BEZIER_CURVE" => {
                    self.bezier_curve
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "QUASI_UNIFORM_CURVE" => {
                    self.quasi_uniform_curve
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "UNIFORM_CURVE" => {
                    self.uniform_curve
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "CIRCLE" => {
                    self.circle.insert(*id, Deserialize::deserialize(record)?);
                }
                "ELLIPSE" => {
                    self.ellipse.insert(*id, Deserialize::deserialize(record)?);
                }
                "PCURVE" => {
                    self.pcurve.insert(*id, Deserialize::deserialize(record)?);
                }
                "SURFACE_CURVE" => {
                    self.surface_curve
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "SEAM_CURVE" => {
                    self.surface_curve
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "PLANE" => {
                    self.plane.insert(*id, Deserialize::deserialize(record)?);
                }
                "SPHERICAL_SURFACE" => {
                    self.spherical_surface
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "CYLINDRICAL_SURFACE" => {
                    self.cylindrical_surface
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "TOROIDAL_SURFACE" => {
                    self.toroidal_surface
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "CONICAL_SURFACE" => {
                    self.conical_surface
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "B_SPLINE_SURFACE_WITH_KNOTS" => {
                    self.b_spline_surface_with_knots
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "UNIFORM_SURFACE" => {
                    self.uniform_surface
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "QUASI_UNIFORM_SURFACE" => {
                    self.quasi_uniform_surface
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "BEZIER_SURFACE" => {
                    self.bezier_surface
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "SURFACE_OF_LINEAR_EXTRUSION" => {
                    self.surface_of_linear_extrusion
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "SURFACE_OF_REVOLUTION" => {
                    self.surface_of_revolution
                        .insert(*id, Deserialize::deserialize(record)?);
                }

                "VERTEX_POINT" => {
                    self.vertex_point
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "EDGE_CURVE" => {
                    self.edge_curve
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "ORIENTED_EDGE" => {
                    if let Parameter::List(params) = &record.parameter {
                        if params.len() == 5 {
                            self.oriented_edge.insert(
                                *id,
                                OrientedEdgeHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    edge_element: Deserialize::deserialize(&params[3])?,
                                    orientation: Deserialize::deserialize(&params[4])?,
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
                    self.face_bound
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "FACE_OUTER_BOUND" => {
                    self.face_bound
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "FACE_SURFACE" => {
                    self.face_surface
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "ADVANCED_FACE" => {
                    self.face_surface
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "ORIENTED_FACE" => {
                    if let Parameter::List(params) = &record.parameter {
                        if params.len() == 4 {
                            self.oriented_face.insert(
                                *id,
                                OrientedFaceHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    face_element: Deserialize::deserialize(&params[2])?,
                                    orientation: Deserialize::deserialize(&params[3])?,
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
                                    orientation: Deserialize::deserialize(&params[3])?,
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
                                    orientation: Deserialize::deserialize(&params[3])?,
                                },
                            );
                        }
                    }
                }
                "DEFINITIONAL_REPRESENTATION" => {
                    if let Parameter::List(params) = &record.parameter {
                        if params.len() == 3 {
                            self.definitional_representation.insert(
                                *id,
                                DefinitionalRepresentationHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    representation_item: Deserialize::deserialize(&params[1])?,
                                    contex_of_items: match &params[2] {
                                        Parameter::Ref(x) => PlaceHolder::Ref(x.clone()),
                                        _ => PlaceHolder::Owned(DummyHolder {
                                            record: format!("{:?}", params[2]),
                                            is_simple: true,
                                        }),
                                    },
                                },
                            );
                        }
                    }
                }
                _ => {
                    self.dummy.insert(
                        *id,
                        DummyHolder {
                            record: format!("{record:?}"),
                            is_simple: true,
                        },
                    );
                }
            },
            EntityInstance::Complex {
                id,
                subsuper: SubSuperRecord(records),
            } => {
                use NonRationalBSplineCurveHolder as NRBC;
                use NonRationalBSplineSurfaceHolder as NRBS;
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
                            let mut params = label.clone();
                            params.extend(bsp_params.clone());
                            params.extend(knots_params.clone());
                            self.rational_b_spline_curve.insert(
                                *id,
                                RationalBSplineCurveHolder {
                                    non_rational_b_spline_curve: PlaceHolder::Owned(
                                        NRBC::BSplineCurveWithKnots(Deserialize::deserialize(
                                            &Parameter::List(params),
                                        )?),
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
                            let mut params = label.clone();
                            params.extend(bsp_params.clone());
                            self.rational_b_spline_curve.insert(
                                *id,
                                RationalBSplineCurveHolder {
                                    non_rational_b_spline_curve: PlaceHolder::Owned(
                                        NRBC::BezierCurve(Deserialize::deserialize(
                                            &Parameter::List(params),
                                        )?),
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
                                        NRBC::QuasiUniformCurve(Deserialize::deserialize(
                                            &Parameter::List(params),
                                        )?),
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
                                        NRBC::UniformCurve(Deserialize::deserialize(
                                            &Parameter::List(params),
                                        )?),
                                    ),
                                    weights_data: Deserialize::deserialize(&weights[0])?,
                                },
                            );
                        }
                        (
                            "BOUNDED_SURFACE",
                            _,
                            "B_SPLINE_SURFACE",
                            Parameter::List(bsp_params),
                            "B_SPLINE_SURFACE_WITH_KNOTS",
                            Parameter::List(knots_params),
                            "GEOMETRIC_REPRESENTATION_ITEM",
                            _,
                            "RATIONAL_B_SPLINE_SURFACE",
                            Parameter::List(weights),
                            "REPRESENTATION_ITEM",
                            Parameter::List(label),
                            "SURFACE",
                            _,
                        ) => {
                            let mut params = label.clone();
                            params.extend(bsp_params.clone());
                            params.extend(knots_params.clone());
                            self.rational_b_spline_surface.insert(
                                *id,
                                RationalBSplineSurfaceHolder {
                                    non_rational_b_spline_surface: PlaceHolder::Owned(
                                        NRBS::BSplineSurfaceWithKnots(Deserialize::deserialize(
                                            &Parameter::List(params),
                                        )?),
                                    ),
                                    weights_data: Deserialize::deserialize(&weights[0])?,
                                },
                            );
                        }
                        (
                            "BEZIER_SURFACE",
                            _,
                            "BOUNDED_SURFACE",
                            _,
                            "B_SPLINE_SURFACE",
                            Parameter::List(bsp_params),
                            "GEOMETRIC_REPRESENTATION_ITEM",
                            _,
                            "RATIONAL_B_SPLINE_SURFACE",
                            Parameter::List(weights),
                            "REPRESENTATION_ITEM",
                            Parameter::List(label),
                            "SURFACE",
                            _,
                        ) => {
                            let mut params = label.clone();
                            params.extend(bsp_params.clone());
                            self.rational_b_spline_surface.insert(
                                *id,
                                RationalBSplineSurfaceHolder {
                                    non_rational_b_spline_surface: PlaceHolder::Owned(
                                        NRBS::BezierSurface(Deserialize::deserialize(
                                            &Parameter::List(params),
                                        )?),
                                    ),
                                    weights_data: Deserialize::deserialize(&weights[0])?,
                                },
                            );
                        }
                        (
                            "BOUNDED_SURFACE",
                            _,
                            "B_SPLINE_SURFACE",
                            Parameter::List(bsp_params),
                            "GEOMETRIC_REPRESENTATION_ITEM",
                            _,
                            "QUASI_UNIFORM_SURFACE",
                            _,
                            "RATIONAL_B_SPLINE_SURFACE",
                            Parameter::List(weights),
                            "REPRESENTATION_ITEM",
                            Parameter::List(label),
                            "SURFACE",
                            _,
                        ) => {
                            let mut params = label.clone();
                            params.extend(bsp_params.clone());
                            self.rational_b_spline_surface.insert(
                                *id,
                                RationalBSplineSurfaceHolder {
                                    non_rational_b_spline_surface: PlaceHolder::Owned(
                                        NRBS::QuasiUniformSurface(Deserialize::deserialize(
                                            &Parameter::List(params),
                                        )?),
                                    ),
                                    weights_data: Deserialize::deserialize(&weights[0])?,
                                },
                            );
                        }
                        (
                            "BOUNDED_SURFACE",
                            _,
                            "B_SPLINE_SURFACE",
                            Parameter::List(bsp_params),
                            "GEOMETRIC_REPRESENTATION_ITEM",
                            _,
                            "RATIONAL_B_SPLINE_SURFACE",
                            Parameter::List(weights),
                            "REPRESENTATION_ITEM",
                            Parameter::List(label),
                            "SURFACE",
                            _,
                            "UNIFORM_SURFACE",
                            _,
                        ) => {
                            let mut params = label.clone();
                            params.extend(bsp_params.clone());
                            self.rational_b_spline_surface.insert(
                                *id,
                                RationalBSplineSurfaceHolder {
                                    non_rational_b_spline_surface: PlaceHolder::Owned(
                                        NRBS::UniformSurface(Deserialize::deserialize(
                                            &Parameter::List(params),
                                        )?),
                                    ),
                                    weights_data: Deserialize::deserialize(&weights[0])?,
                                },
                            );
                        }
                        _ => {
                            self.dummy.insert(
                                *id,
                                DummyHolder {
                                    record: format!("{records:?}"),
                                    is_simple: false,
                                },
                            );
                        }
                    }
                } else {
                    self.dummy.insert(
                        *id,
                        DummyHolder {
                            record: format!("{records:?}"),
                            is_simple: false,
                        },
                    );
                }
            }
        }
        Ok(())
    }
    #[inline(always)]
    pub fn from_data_section(data_section: &DataSection) -> Table {
        Table::from_iter(&data_section.entities)
    }
    #[inline(always)]
    pub fn from_step(step_str: &str) -> Option<Table> {
        let exchange = ruststep::parser::parse(step_str).ok()?;
        Some(Table::from_data_section(&exchange.data[0]))
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = dummy)]
#[holder(generate_deserialize)]
pub struct Dummy {
    pub record: String,
    pub is_simple: bool,
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
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
    fn from(p: &Placement) -> Self { Self::from(&p.location) }
}
impl From<&Placement> for Point3 {
    #[inline(always)]
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

impl Axis1Placement {
    pub fn direction(&self) -> Vector3 {
        self.direction
            .as_ref()
            .map(Vector3::from)
            .unwrap_or_else(Vector3::unit_z)
    }
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
    #[inline(always)]
    fn try_from(axis: &Axis2Placement) -> std::result::Result<Self, ExpressParseError> {
        use Axis2Placement::*;
        match axis {
            Axis2Placement2d(axis) => Ok(Matrix3::from(axis)),
            Axis2Placement3d(_) => Err("This is not a 2D axis placement.".into()),
        }
    }
}
impl TryFrom<&Axis2Placement> for Matrix4 {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(axis: &Axis2Placement) -> std::result::Result<Self, ExpressParseError> {
        use Axis2Placement::*;
        match axis {
            Axis2Placement2d(_) => Err("This is not a 3D axis placement.".into()),
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
    #[inline(always)]
    fn from(axis: &Axis2Placement2d) -> Self {
        let z = Point2::from(&axis.location);
        let x = match &axis.ref_direction {
            Some(axis) => Vector2::from(axis),
            None => Vector2::unit_x(),
        };
        let y = Vector2::new(-x.y, x.x);
        Matrix3::from_cols(x.extend(0.0), y.extend(0.0), z.to_vec().extend(1.0))
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
    #[inline(always)]
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
        Matrix4::from_cols(
            x.extend(0.0),
            y.extend(0.0),
            z.extend(0.0),
            w.to_vec().extend(1.0),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum CurveAny {
    #[holder(use_place_holder)]
    Line(Box<Line>),
    #[holder(use_place_holder)]
    BoundedCurve(Box<BoundedCurveAny>),
    #[holder(use_place_holder)]
    Conic(Box<Conic>),
    #[holder(use_place_holder)]
    Pcurve(Box<Pcurve>),
    #[holder(use_place_holder)]
    SurfaceCurve(Box<SurfaceCurve>),
}

impl TryFrom<&CurveAny> for Curve2D {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(curve: &CurveAny) -> std::result::Result<Self, Self::Error> {
        use CurveAny::*;
        Ok(match curve {
            Line(line) => Self::Line(line.as_ref().into()),
            BoundedCurve(b) => b.as_ref().try_into()?,
            Conic(curve) => Self::Conic(curve.as_ref().try_into()?),
            Pcurve(_) => return Err("Pcurves cannot be parsed to 2D curves.".into()),
            SurfaceCurve(_) => return Err("Surface curves cannot be parsed to 2D curves.".into()),
        })
    }
}

impl TryFrom<&CurveAny> for Curve3D {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(curve: &CurveAny) -> std::result::Result<Self, Self::Error> {
        use CurveAny::*;
        Ok(match curve {
            Line(line) => Self::Line(line.as_ref().into()),
            BoundedCurve(b) => b.as_ref().try_into()?,
            Conic(curve) => Self::Conic(curve.as_ref().try_into()?),
            Pcurve(c) => Self::PCurve(c.as_ref().try_into()?),
            SurfaceCurve(c) => c.as_ref().try_into()?,
        })
    }
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
impl<'a, P> From<&'a Line> for truck::Line<P>
where
    P: EuclideanSpace + From<&'a CartesianPoint>,
    P::Diff: From<&'a Vector>,
{
    #[inline(always)]
    fn from(line: &'a Line) -> Self {
        let p = P::from(&line.pnt);
        let q = p + P::Diff::from(&line.dir);
        Self(p, q)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum BoundedCurveAny {
    #[holder(use_place_holder)]
    Polyline(Box<Polyline>),
    #[holder(use_place_holder)]
    BSplineCurve(Box<BSplineCurveAny>),
}

impl TryFrom<&BoundedCurveAny> for Curve2D {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(value: &BoundedCurveAny) -> std::result::Result<Self, Self::Error> {
        use BoundedCurveAny::*;
        Ok(match value {
            Polyline(x) => Self::Polyline(x.as_ref().try_into()?),
            BSplineCurve(x) => x.as_ref().try_into()?,
        })
    }
}

impl TryFrom<&BoundedCurveAny> for Curve3D {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(value: &BoundedCurveAny) -> std::result::Result<Self, Self::Error> {
        use BoundedCurveAny::*;
        Ok(match value {
            Polyline(x) => Self::Polyline(x.as_ref().try_into()?),
            BSplineCurve(x) => x.as_ref().try_into()?,
        })
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
    #[inline(always)]
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
    #[inline(always)]
    fn try_from(curve: &BSplineCurveWithKnots) -> std::result::Result<Self, ExpressParseError> {
        let knots = curve.knots.clone();
        let multi = curve
            .knot_multiplicities
            .iter()
            .map(|n| *n as usize)
            .collect();
        let knots = KnotVec::from_single_multi(knots, multi).unwrap();
        let ctrpts = curve.control_points_list.iter().map(Into::into).collect();
        Ok(Self::try_new(knots, ctrpts)?)
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
    #[inline(always)]
    fn try_from(curve: &BezierCurve) -> std::result::Result<Self, ExpressParseError> {
        let degree = curve.degree as usize;
        let knots = KnotVec::bezier_knot(degree);
        let ctrpts = curve.control_points_list.iter().map(Into::into).collect();
        Ok(Self::try_new(knots, ctrpts)?)
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
    #[inline(always)]
    fn try_from(curve: &QuasiUniformCurve) -> std::result::Result<Self, ExpressParseError> {
        let knots = quasi_uniform_knots(curve.control_points_list.len(), curve.degree as usize);
        let ctrpts = curve.control_points_list.iter().map(Into::into).collect();
        Ok(Self::try_new(knots, ctrpts)?)
    }
}

fn quasi_uniform_knots(num_ctrl: usize, degree: usize) -> KnotVec {
    let division = num_ctrl - degree;
    let mut knots = KnotVec::uniform_knot(degree, division);
    knots.transform(division as f64, 0.0);
    knots
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
    #[inline(always)]
    fn try_from(curve: &UniformCurve) -> std::result::Result<Self, ExpressParseError> {
        let knots = uniform_knots(curve.control_points_list.len(), curve.degree as usize)?;
        let ctrpts = curve.control_points_list.iter().map(Into::into).collect();
        Ok(Self::try_new(knots, ctrpts)?)
    }
}

fn uniform_knots(num_ctrl: usize, degree: usize) -> truck::Result<KnotVec> {
    KnotVec::try_from(
        (0..degree + num_ctrl + 1)
            .map(|i| i as f64 - degree as f64)
            .collect::<Vec<_>>(),
    )
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
    #[inline(always)]
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
impl<V> TryFrom<&RationalBSplineCurve> for NurbsCurve<V>
where
    V: Homogeneous<f64>,
    V::Point: for<'a> From<&'a CartesianPoint>,
{
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(curve: &RationalBSplineCurve) -> std::result::Result<Self, ExpressParseError> {
        Ok(Self::try_from_bspline_and_weights(
            BSplineCurve::try_from(&curve.non_rational_b_spline_curve)?,
            curve.weights_data.clone(),
        )?)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum BSplineCurveAny {
    #[holder(use_place_holder)]
    NonRationalBSplineCurve(Box<NonRationalBSplineCurve>),
    #[holder(use_place_holder)]
    RationalBSplineCurve(Box<RationalBSplineCurve>),
}

impl TryFrom<&BSplineCurveAny> for Curve2D {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(value: &BSplineCurveAny) -> std::result::Result<Self, Self::Error> {
        use BSplineCurveAny::*;
        Ok(match value {
            NonRationalBSplineCurve(bsp) => Self::BSplineCurve(bsp.as_ref().try_into()?),
            RationalBSplineCurve(bsp) => Self::NurbsCurve(bsp.as_ref().try_into()?),
        })
    }
}

impl TryFrom<&BSplineCurveAny> for Curve3D {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(value: &BSplineCurveAny) -> std::result::Result<Self, Self::Error> {
        use BSplineCurveAny::*;
        Ok(match value {
            NonRationalBSplineCurve(bsp) => Self::BSplineCurve(bsp.as_ref().try_into()?),
            RationalBSplineCurve(bsp) => Self::NurbsCurve(bsp.as_ref().try_into()?),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum Conic {
    #[holder(use_place_holder)]
    Circle(Circle),
    #[holder(use_place_holder)]
    Ellipse(Ellipse),
}

impl TryFrom<&Conic> for Conic2D {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(value: &Conic) -> std::prelude::v1::Result<Self, Self::Error> {
        Ok(match value {
            Conic::Circle(value) => Conic2D::Ellipse(value.try_into()?),
            Conic::Ellipse(value) => Conic2D::Ellipse(value.try_into()?),
        })
    }
}

impl TryFrom<&Conic> for Conic3D {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(value: &Conic) -> std::prelude::v1::Result<Self, Self::Error> {
        Ok(match value {
            Conic::Circle(value) => Conic3D::Ellipse(value.try_into()?),
            Conic::Ellipse(value) => Conic3D::Ellipse(value.try_into()?),
        })
    }
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

impl TryFrom<&Circle> for alias::Ellipse<Point2, Matrix3> {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(circle: &Circle) -> std::result::Result<Self, Self::Error> {
        let radius: f64 = circle.radius;
        let transform = Matrix3::try_from(&circle.position)? * Matrix3::from_scale(radius);
        Ok(
            Processor::new(truck::TrimmedCurve::new(UnitCircle::new(), (0.0, 2.0 * PI)))
                .transformed(transform),
        )
    }
}

impl TryFrom<&Circle> for alias::Ellipse<Point3, Matrix4> {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(circle: &Circle) -> std::result::Result<Self, Self::Error> {
        let radius: f64 = circle.radius;
        let transform = Matrix4::try_from(&circle.position)? * Matrix4::from_scale(radius);
        Ok(
            Processor::new(truck::TrimmedCurve::new(UnitCircle::new(), (0.0, 2.0 * PI)))
                .transformed(transform),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = ellipse)]
#[holder(generate_deserialize)]
pub struct Ellipse {
    pub label: String,
    #[holder(use_place_holder)]
    pub position: Axis2Placement,
    pub semi_axis_1: f64,
    pub semi_axis_2: f64,
}

impl TryFrom<&Ellipse> for alias::Ellipse<Point2, Matrix3> {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(ellipse: &Ellipse) -> std::prelude::v1::Result<Self, Self::Error> {
        let (r0, r1) = (ellipse.semi_axis_1, ellipse.semi_axis_2);
        let transform =
            Matrix3::try_from(&ellipse.position)? * Matrix3::from_nonuniform_scale(r0, r1);
        Ok(
            Processor::new(truck::TrimmedCurve::new(UnitCircle::new(), (0.0, 2.0 * PI)))
                .transformed(transform),
        )
    }
}

impl TryFrom<&Ellipse> for alias::Ellipse<Point3, Matrix4> {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(ellipse: &Ellipse) -> std::prelude::v1::Result<Self, Self::Error> {
        let (r0, r1) = (ellipse.semi_axis_1, ellipse.semi_axis_2);
        let transform = Matrix4::try_from(&ellipse.position)?
            * Matrix4::from_nonuniform_scale(r0, r1, f64::min(r0, r1));
        Ok(
            Processor::new(truck::TrimmedCurve::new(UnitCircle::new(), (0.0, 2.0 * PI)))
                .transformed(transform),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = definitional_representation)]
#[holder(generate_deserialize)]
pub struct DefinitionalRepresentation {
    label: String,
    #[holder(use_place_holder)]
    representation_item: Vec<CurveAny>,
    #[holder(use_place_holder)]
    contex_of_items: Dummy,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = pcurve)]
#[holder(generate_deserialize)]
pub struct Pcurve {
    label: String,
    #[holder(use_place_holder)]
    basis_surface: SurfaceAny,
    #[holder(use_place_holder)]
    reference_to_curve: DefinitionalRepresentation,
}

impl TryFrom<&Pcurve> for PCurve {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(value: &Pcurve) -> std::result::Result<Self, Self::Error> {
        let surface: Surface = (&value.basis_surface).try_into()?;
        let curve: Curve2D = value
            .reference_to_curve
            .representation_item
            .get(0)
            .ok_or("no representation item")?
            .try_into()?;
        Ok(alias::PCurve::new(Box::new(curve), Box::new(surface)))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum PcurveOrSurface {
    #[holder(use_place_holder)]
    Pcurve(Box<Pcurve>),
    #[holder(use_place_holder)]
    Surface(Box<SurfaceAny>),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PreferredSurfaceCurveRepresentation {
    Curve3D,
    PcurveS1,
    PcurveS2,
}

#[test]
fn deserialize_pscr() {
    let (_, p) = ruststep::parser::exchange::parameter(".PCURVE_S1.").unwrap();
    let x = PreferredSurfaceCurveRepresentation::deserialize(&p).unwrap();
    assert!(matches!(x, PreferredSurfaceCurveRepresentation::PcurveS1));
    let (_, p) = ruststep::parser::exchange::parameter(".PCURVE_S2.").unwrap();
    let x = PreferredSurfaceCurveRepresentation::deserialize(&p).unwrap();
    assert!(matches!(x, PreferredSurfaceCurveRepresentation::PcurveS2));
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = surface_curve)]
#[holder(generate_deserialize)]
pub struct SurfaceCurve {
    label: String,
    #[holder(use_place_holder)]
    curve_3d: CurveAny,
    #[holder(use_place_holder)]
    associated_geometry: Vec<PcurveOrSurface>,
    master_representation: PreferredSurfaceCurveRepresentation,
}

impl TryFrom<&SurfaceCurve> for Curve3D {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(value: &SurfaceCurve) -> std::result::Result<Self, Self::Error> {
        use PreferredSurfaceCurveRepresentation as PSCR;
        match &value.master_representation {
            PSCR::Curve3D => Ok((&value.curve_3d).try_into()?),
            PSCR::PcurveS1 => {
                if let Some(PcurveOrSurface::Pcurve(x)) = value.associated_geometry.get(0) {
                    Ok(Self::PCurve(x.as_ref().try_into()?))
                } else {
                    Err("The 0-indexed associated geometry is nothing or not PCURVE.".into())
                }
            }
            PSCR::PcurveS2 => {
                if let Some(PcurveOrSurface::Pcurve(x)) = value.associated_geometry.get(1) {
                    Ok(Self::PCurve(x.as_ref().try_into()?))
                } else {
                    Err("The 1-indexed associated geometry is nothing or not PCURVE.".into())
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum SurfaceAny {
    #[holder(use_place_holder)]
    ElementarySurface(Box<ElementarySurfaceAny>),
    #[holder(use_place_holder)]
    BSplineSurface(Box<BSplineSurfaceAny>),
    #[holder(use_place_holder)]
    SweptSurface(Box<SweptSurfaceAny>),
}

impl TryFrom<&SurfaceAny> for Surface {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(x: &SurfaceAny) -> std::result::Result<Self, Self::Error> {
        use SurfaceAny::*;
        Ok(match x {
            ElementarySurface(x) => Self::ElementarySurface(Box::new(x.as_ref().into())),
            BSplineSurface(x) => x.as_ref().try_into()?,
            SweptSurface(x) => Self::SweptCurve(Box::new(x.as_ref().try_into()?)),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum ElementarySurfaceAny {
    #[holder(use_place_holder)]
    Plane(Box<Plane>),
    #[holder(use_place_holder)]
    SphericalSurface(Box<SphericalSurface>),
    #[holder(use_place_holder)]
    CylindricalSurface(Box<CylindricalSurface>),
    #[holder(use_place_holder)]
    ToroidalSurface(Box<ToroidalSurface>),
    #[holder(use_place_holder)]
    ConicalSurface(Box<ConicalSurface>),
}

impl From<&ElementarySurfaceAny> for ElementarySurface {
    #[inline(always)]
    fn from(value: &ElementarySurfaceAny) -> Self {
        use ElementarySurfaceAny::*;
        match value {
            Plane(x) => Self::Plane(x.as_ref().into()),
            SphericalSurface(x) => Self::Sphere(x.as_ref().into()),
            CylindricalSurface(x) => Self::CylindricalSurface(x.as_ref().into()),
            ToroidalSurface(x) => Self::ToroidalSurface(x.as_ref().into()),
            ConicalSurface(x) => Self::ConicalSurface(x.as_ref().into()),
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

impl From<&Plane> for truck::Plane {
    #[inline(always)]
    fn from(plane: &Plane) -> Self {
        let mat = Matrix4::from(&plane.position);
        let o = Point3::from_homogeneous(mat[3]);
        let p = o + mat[0].truncate();
        let q = o + mat[1].truncate();
        Self::new(o, p, q)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = spherical_surface)]
#[holder(generate_deserialize)]
pub struct SphericalSurface {
    label: String,
    #[holder(use_place_holder)]
    position: Axis2Placement3d,
    radius: f64,
}

impl From<&SphericalSurface> for alias::SphericalSurface {
    #[inline(always)]
    fn from(ss: &SphericalSurface) -> Self {
        let mat = Matrix4::from(&ss.position);
        let sphere = Sphere::new(Point3::origin(), ss.radius);
        Processor::new(sphere).transformed(mat)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = cylindrical_surface)]
#[holder(generate_deserialize)]
pub struct CylindricalSurface {
    label: String,
    #[holder(use_place_holder)]
    position: Axis2Placement3d,
    radius: f64,
}

impl From<&CylindricalSurface> for alias::CylindricalSurface {
    #[inline(always)]
    fn from(cs: &CylindricalSurface) -> Self {
        let mat = Matrix4::from(&cs.position);
        let x = mat[0].truncate();
        let z = mat[2].truncate();
        let center = Point3::from_homogeneous(mat[3]);
        let radius = cs.radius;
        let p = center + x * radius;
        let mut res = Processor::new(RevolutedCurve::by_revolution(Line(p, p + z), center, z));
        res.invert();
        res
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = toroidal_surface)]
#[holder(generate_deserialize)]
pub struct ToroidalSurface {
    label: String,
    #[holder(use_place_holder)]
    position: Axis2Placement3d,
    major_radius: f64,
    minor_radius: f64,
}

impl From<&ToroidalSurface> for alias::ToroidalSurface {
    #[inline(always)]
    fn from(
        ToroidalSurface {
            position,
            major_radius,
            minor_radius,
            ..
        }: &ToroidalSurface,
    ) -> Self {
        let mat = Matrix4::from(position);
        let torus = Torus::new(Point3::origin(), *major_radius, *minor_radius);
        Processor::new(torus).transformed(mat)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = conical_surface)]
#[holder(generate_deserialize)]
pub struct ConicalSurface {
    label: String,
    #[holder(use_place_holder)]
    position: Axis2Placement3d,
    radius: f64,
    semi_angle: f64,
}

impl From<&ConicalSurface> for alias::ConicalSurface {
    fn from(
        ConicalSurface {
            position,
            radius,
            semi_angle,
            ..
        }: &ConicalSurface,
    ) -> Self {
        let mat = Matrix4::from(position);
        let p = Point3::new(*radius, 0.0, 0.0);
        let v = Vector3::new(f64::tan(*semi_angle), 0.0, 1.0);
        let rev =
            RevolutedCurve::by_revolution(Line(p, p + v), Point3::origin(), Vector3::unit_z());
        let mut processor = Processor::new(rev);
        processor.transform_by(mat);
        processor.invert();
        processor
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum BSplineSurfaceAny {
    #[holder(use_place_holder)]
    NonRationalBSplineSurface(NonRationalBSplineSurface),
    #[holder(use_place_holder)]
    RationalBSplineSurface(RationalBSplineSurface),
}

impl TryFrom<&BSplineSurfaceAny> for Surface {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(value: &BSplineSurfaceAny) -> std::result::Result<Self, Self::Error> {
        use BSplineSurfaceAny::*;
        Ok(match value {
            NonRationalBSplineSurface(bsp) => Surface::BSplineSurface(Box::new(bsp.try_into()?)),
            RationalBSplineSurface(bsp) => Surface::NurbsSurface(Box::new(bsp.try_into()?)),
        })
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
    #[inline(always)]
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
        Ok(Self::try_new((uknots, vknots), ctrls)?)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = uniform_surface)]
#[holder(generate_deserialize)]
pub struct UniformSurface {
    label: String,
    u_degree: i64,
    v_degree: i64,
    #[holder(use_place_holder)]
    control_points_list: Vec<Vec<CartesianPoint>>,
    surface_form: BSplineSurfaceForm,
    u_closed: Logical,
    v_closed: Logical,
    self_intersect: Logical,
}

impl TryFrom<&UniformSurface> for BSplineSurface<Point3> {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(surface: &UniformSurface) -> std::result::Result<Self, ExpressParseError> {
        let uknots = uniform_knots(surface.control_points_list.len(), surface.u_degree as usize)?;
        let first = surface
            .control_points_list
            .first()
            .ok_or("control points list is empty.")?;
        let vknots = uniform_knots(first.len(), surface.v_degree as usize)?;
        let ctrls = surface
            .control_points_list
            .iter()
            .map(|vec| vec.iter().map(Point3::from).collect())
            .collect();
        Ok(Self::try_new((uknots, vknots), ctrls)?)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = quasi_uniform_surface)]
#[holder(generate_deserialize)]
pub struct QuasiUniformSurface {
    label: String,
    u_degree: i64,
    v_degree: i64,
    #[holder(use_place_holder)]
    control_points_list: Vec<Vec<CartesianPoint>>,
    surface_form: BSplineSurfaceForm,
    u_closed: Logical,
    v_closed: Logical,
    self_intersect: Logical,
}

impl TryFrom<&QuasiUniformSurface> for BSplineSurface<Point3> {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(surface: &QuasiUniformSurface) -> std::result::Result<Self, ExpressParseError> {
        let uknots =
            quasi_uniform_knots(surface.control_points_list.len(), surface.u_degree as usize);
        let first = surface
            .control_points_list
            .first()
            .ok_or("control points list is empty.")?;
        let vknots = quasi_uniform_knots(first.len(), surface.v_degree as usize);
        let ctrls = surface
            .control_points_list
            .iter()
            .map(|vec| vec.iter().map(Point3::from).collect())
            .collect();
        Ok(Self::try_new((uknots, vknots), ctrls)?)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = bezier_surface)]
#[holder(generate_deserialize)]
pub struct BezierSurface {
    label: String,
    u_degree: i64,
    v_degree: i64,
    #[holder(use_place_holder)]
    control_points_list: Vec<Vec<CartesianPoint>>,
    surface_form: BSplineSurfaceForm,
    u_closed: Logical,
    v_closed: Logical,
    self_intersect: Logical,
}

impl From<&BezierSurface> for BSplineSurface<Point3> {
    #[inline(always)]
    fn from(value: &BezierSurface) -> Self {
        let uknots = KnotVec::bezier_knot(value.u_degree as usize);
        let vknots = KnotVec::bezier_knot(value.v_degree as usize);
        let ctrls = value
            .control_points_list
            .iter()
            .map(|vec| vec.iter().map(Point3::from).collect())
            .collect();
        Self::new((uknots, vknots), ctrls)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum NonRationalBSplineSurface {
    #[holder(use_place_holder)]
    BSplineSurfaceWithKnots(Box<BSplineSurfaceWithKnots>),
    #[holder(use_place_holder)]
    UniformSurface(Box<UniformSurface>),
    #[holder(use_place_holder)]
    QuasiUniformSurface(Box<QuasiUniformSurface>),
    #[holder(use_place_holder)]
    BezierSurface(Box<BezierSurface>),
}

impl TryFrom<&NonRationalBSplineSurface> for BSplineSurface<Point3> {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(value: &NonRationalBSplineSurface) -> std::result::Result<Self, Self::Error> {
        use NonRationalBSplineSurface::*;
        match value {
            BSplineSurfaceWithKnots(x) => x.as_ref().try_into(),
            UniformSurface(x) => x.as_ref().try_into(),
            QuasiUniformSurface(x) => x.as_ref().try_into(),
            BezierSurface(x) => Ok(x.as_ref().into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = rational_b_spline_surface)]
#[holder(generate_deserialize)]
pub struct RationalBSplineSurface {
    #[holder(use_place_holder)]
    non_rational_b_spline_surface: NonRationalBSplineSurface,
    weights_data: Vec<Vec<f64>>,
}

impl TryFrom<&RationalBSplineSurface> for NurbsSurface<Vector4> {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(
        RationalBSplineSurface {
            non_rational_b_spline_surface,
            weights_data,
        }: &RationalBSplineSurface,
    ) -> std::result::Result<Self, Self::Error> {
        let surface: BSplineSurface<Point3> = non_rational_b_spline_surface.try_into()?;
        Ok(Self::try_from_bspline_and_weights(
            surface,
            weights_data.clone(),
        )?)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum SweptSurfaceAny {
    #[holder(use_place_holder)]
    SurfaceOfLinearExtrusion(Box<SurfaceOfLinearExtrusion>),
    #[holder(use_place_holder)]
    SurfaceOfRevolution(Box<SurfaceOfRevolution>),
}

impl TryFrom<&SweptSurfaceAny> for SweptCurve {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(value: &SweptSurfaceAny) -> std::result::Result<Self, Self::Error> {
        use SweptSurfaceAny::*;
        Ok(match value {
            SurfaceOfLinearExtrusion(x) => SweptCurve::ExtrudedCurve(x.as_ref().try_into()?),
            SurfaceOfRevolution(x) => SweptCurve::RevolutedCurve(x.as_ref().try_into()?),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = surface_of_linear_extrusion)]
#[holder(generate_deserialize)]
pub struct SurfaceOfLinearExtrusion {
    label: String,
    #[holder(use_place_holder)]
    swept_curve: CurveAny,
    #[holder(use_place_holder)]
    extrusion_axis: Vector,
}

impl TryFrom<&SurfaceOfLinearExtrusion> for StepExtrudedCurve {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(sr: &SurfaceOfLinearExtrusion) -> std::result::Result<Self, Self::Error> {
        let curve = Curve3D::try_from(&sr.swept_curve)?;
        let vector = Vector3::from(&sr.extrusion_axis);
        Ok(ExtrudedCurve::by_extrusion(curve, vector))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = surface_of_revolution)]
#[holder(generate_deserialize)]
pub struct SurfaceOfRevolution {
    label: String,
    #[holder(use_place_holder)]
    swept_curve: CurveAny,
    #[holder(use_place_holder)]
    axis_position: Axis1Placement,
}

impl TryFrom<&SurfaceOfRevolution> for StepRevolutedCurve {
    type Error = ExpressParseError;
    #[inline(always)]
    fn try_from(sr: &SurfaceOfRevolution) -> std::result::Result<Self, Self::Error> {
        let curve = Curve3D::try_from(&sr.swept_curve)?;
        let origin = Point3::from(&sr.axis_position.location);
        let axis = sr.axis_position.direction().normalize();
        let mut rev = Processor::new(RevolutedCurve::by_revolution(curve, origin, axis));
        rev.invert();
        Ok(rev)
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
        let p = Point2::from(&self.edge_start.vertex_geometry);
        let q = Point2::from(&self.edge_end.vertex_geometry);
        let (p, q) = match self.same_sense {
            true => (p, q),
            false => (q, p),
        };
        Self::sub_parse_2d(&self.edge_geometry, p, q, self.same_sense)
    }
    fn sub_parse_2d(
        curve: &CurveAny,
        p: Point2,
        q: Point2,
        same_sense: bool,
    ) -> std::result::Result<Curve2D, ExpressParseError> {
        let mut curve = match curve {
            CurveAny::Line(line) => {
                let line = truck::Line::<Point2>::from(line.as_ref());
                let p = line.projection(p);
                let q = line.projection(q);
                Curve2D::Line(Line(p, q))
            }
            CurveAny::BoundedCurve(b) => b.as_ref().try_into()?,
            CurveAny::Conic(curve) => match curve.as_ref() {
                Conic::Circle(circle) => {
                    let mat =
                        Matrix3::try_from(&circle.position)? * Matrix3::from_scale(circle.radius);
                    let inv_mat = mat
                        .invert()
                        .ok_or_else(|| "Failed to convert Circle".to_string())?;
                    let (p, q) = (inv_mat.transform_point(p), inv_mat.transform_point(q));
                    let (u, mut v) = (
                        UnitCircle::<Point2>::new()
                            .search_nearest_parameter(p, None, 0)
                            .ok_or_else(|| "the point is not on circle".to_string())?,
                        UnitCircle::<Point2>::new()
                            .search_nearest_parameter(q, None, 0)
                            .ok_or_else(|| "the point is not on circle".to_string())?,
                    );
                    if v <= u + TOLERANCE {
                        v += 2.0 * PI;
                    }
                    let circle = TrimmedCurve::new(UnitCircle::<Point2>::new(), (u, v));
                    let mut ellipse = Processor::new(circle);
                    ellipse.transform_by(mat);
                    Curve2D::Conic(Conic2D::Ellipse(ellipse))
                }
                Conic::Ellipse(ellipse) => {
                    let mat = Matrix3::try_from(&ellipse.position)?
                        * Matrix3::from_nonuniform_scale(ellipse.semi_axis_1, ellipse.semi_axis_2);
                    let inv_mat = mat
                        .invert()
                        .ok_or_else(|| "Failed to convert Circle".to_string())?;
                    let (p, q) = (inv_mat.transform_point(p), inv_mat.transform_point(q));
                    let (u, mut v) = (
                        UnitCircle::<Point2>::new()
                            .search_nearest_parameter(p, None, 0)
                            .ok_or_else(|| "the point is not on circle".to_string())?,
                        UnitCircle::<Point2>::new()
                            .search_nearest_parameter(q, None, 0)
                            .ok_or_else(|| "the point is not on circle".to_string())?,
                    );
                    if v <= u + TOLERANCE {
                        v += 2.0 * PI;
                    }
                    let circle = TrimmedCurve::new(UnitCircle::<Point2>::new(), (u, v));
                    let mut ellipse = Processor::new(circle);
                    ellipse.transform_by(mat);
                    Curve2D::Conic(Conic2D::Ellipse(ellipse))
                }
            },
            CurveAny::Pcurve(_) => return Err("Pcurves cannot be parsed to 2D curves.".into()),
            CurveAny::SurfaceCurve(_) => {
                return Err("Surface curves cannot be parsed to 2D curves.".into())
            }
        };
        if !same_sense {
            curve.invert();
        }
        Ok(curve)
    }
    pub fn parse_curve3d(&self) -> std::result::Result<Curve3D, ExpressParseError> {
        let p = Point3::from(&self.edge_start.vertex_geometry);
        let q = Point3::from(&self.edge_end.vertex_geometry);
        let (p, q) = match self.same_sense {
            true => (p, q),
            false => (q, p),
        };
        Self::sub_parse_curve3d(&self.edge_geometry, p, q, self.same_sense)
    }
    fn sub_parse_curve3d(
        curve: &CurveAny,
        p: Point3,
        q: Point3,
        same_sense: bool,
    ) -> std::result::Result<Curve3D, ExpressParseError> {
        let mut curve = match curve {
            CurveAny::Line(_) => Curve3D::Line(Line(p, q)),
            CurveAny::BoundedCurve(b) => b.as_ref().try_into()?,
            CurveAny::Conic(curve) => match curve.as_ref() {
                Conic::Circle(circle) => {
                    let mat =
                        Matrix4::try_from(&circle.position)? * Matrix4::from_scale(circle.radius);
                    let inv_mat = mat
                        .invert()
                        .ok_or_else(|| "Failed to convert Circle".to_string())?;
                    let (p, q) = (inv_mat.transform_point(p), inv_mat.transform_point(q));
                    let (u, mut v) = (
                        UnitCircle::<Point3>::new()
                            .search_nearest_parameter(p, None, 0)
                            .ok_or_else(|| format!("the point is not on circle: {p:?}"))?,
                        UnitCircle::<Point3>::new()
                            .search_nearest_parameter(q, None, 0)
                            .ok_or_else(|| format!("the point is not on circle: {q:?}"))?,
                    );
                    if v <= u + TOLERANCE {
                        v += 2.0 * PI;
                    }
                    let circle = TrimmedCurve::new(UnitCircle::<Point3>::new(), (u, v));
                    let mut ellipse = Processor::new(circle);
                    ellipse.transform_by(mat);
                    Curve3D::Conic(Conic3D::Ellipse(ellipse))
                }
                Conic::Ellipse(ellipse) => {
                    let mat = Matrix4::try_from(&ellipse.position)?
                        * Matrix4::from_nonuniform_scale(
                            ellipse.semi_axis_1,
                            ellipse.semi_axis_2,
                            f64::min(ellipse.semi_axis_1, ellipse.semi_axis_2),
                        );
                    let inv_mat = mat
                        .invert()
                        .ok_or_else(|| "Failed to convert Circle".to_string())?;
                    let (p, q) = (inv_mat.transform_point(p), inv_mat.transform_point(q));
                    let (u, mut v) = (
                        UnitCircle::<Point3>::new()
                            .search_nearest_parameter(p, None, 0)
                            .ok_or_else(|| format!("the point is not on circle: {p:?}"))?,
                        UnitCircle::<Point3>::new()
                            .search_nearest_parameter(q, None, 0)
                            .ok_or_else(|| format!("the point is not on circle: {q:?}"))?,
                    );
                    if v <= u + TOLERANCE {
                        v += 2.0 * PI;
                    }
                    let circle = TrimmedCurve::new(UnitCircle::<Point3>::new(), (u, v));
                    let mut ellipse = Processor::new(circle);
                    ellipse.transform_by(mat);
                    Curve3D::Conic(Conic3D::Ellipse(ellipse))
                }
            },
            CurveAny::Pcurve(c) => {
                let surface: Surface = (&c.basis_surface).try_into()?;
                let u = surface
                    .search_nearest_parameter(p, None, 100)
                    .ok_or_else(|| "the point is not on surface".to_string())?;
                let v = surface
                    .search_nearest_parameter(q, None, 100)
                    .ok_or_else(|| "the point is not on surface".to_string())?;
                let curve2d = c
                    .reference_to_curve
                    .representation_item
                    .get(0)
                    .ok_or("no representation item")?;
                let curve2d = Self::sub_parse_2d(
                    curve2d,
                    Point2::new(u.0, u.1),
                    Point2::new(v.0, v.1),
                    true,
                )?;
                Curve3D::PCurve(truck::PCurve::new(Box::new(curve2d), Box::new(surface)))
            }
            CurveAny::SurfaceCurve(c) => {
                if p.near(&q) {
                    return Self::sub_parse_curve3d(&c.curve_3d, p, q, same_sense);
                }
                use PreferredSurfaceCurveRepresentation::*;
                match c.master_representation {
                    Curve3D => Self::sub_parse_curve3d(&c.curve_3d, p, q, same_sense)?,
                    PcurveS1 => {
                        if let Some(PcurveOrSurface::Pcurve(c)) = c.associated_geometry.get(0) {
                            Self::sub_parse_curve3d(&CurveAny::Pcurve(c.clone()), p, q, true)?
                        } else {
                            return Err(
                                "The 0-indexed associated geometry is nothing or not PCURVE."
                                    .into(),
                            );
                        }
                    }
                    PcurveS2 => {
                        if let Some(PcurveOrSurface::Pcurve(c)) = c.associated_geometry.get(1) {
                            Self::sub_parse_curve3d(&CurveAny::Pcurve(c.clone()), p, q, true)?
                        } else {
                            return Err(
                                "The 1-indexed associated geometry is nothing or not PCURVE."
                                    .into(),
                            );
                        }
                    }
                }
            }
        };
        if !same_sense {
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

impl OrientedEdgeHolder {
    fn edge_element_holder(&self, table: &Table) -> Option<EdgeCurveHolder> {
        match &self.edge_element {
            PlaceHolder::Owned(holder) => Some(holder.clone()),
            PlaceHolder::Ref(Name::Entity(idx)) => table.edge_curve.get(idx).cloned(),
            _ => None,
        }
    }
    fn edge_element_idx(&self) -> Option<u64> {
        if let PlaceHolder::Ref(Name::Entity(idx)) = self.edge_element {
            Some(idx)
        } else {
            None
        }
    }
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

impl FaceBoundHolder {
    fn bound_holder(&self, table: &Table) -> Option<EdgeLoopHolder> {
        match &self.bound {
            PlaceHolder::Owned(holder) => Some(holder.clone()),
            PlaceHolder::Ref(Name::Entity(ref idx)) => table.edge_loop.get(idx).cloned(),
            _ => None,
        }
    }
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

impl FaceSurfaceHolder {
    fn bounds_holder<'a>(&'a self, table: &'a Table) -> Vec<Option<FaceBoundHolder>> {
        self.bounds
            .iter()
            .map(|bound| match bound {
                PlaceHolder::Owned(bound) => Some(bound.clone()),
                PlaceHolder::Ref(Name::Entity(ref idx)) => table.face_bound.get(idx).cloned(),
                _ => None,
            })
            .collect()
    }
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

impl OrientedFaceHolder {
    fn face_element_holder(&self, table: &Table) -> Option<FaceSurfaceHolder> {
        match &self.face_element {
            PlaceHolder::Ref(Name::Entity(ref idx)) => table.face_surface.get(idx).cloned(),
            PlaceHolder::Owned(x) => Some(x.clone()),
            _ => None,
        }
    }
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

impl ShellHolder {
    fn cfs_faces_holder<'a>(
        &'a self,
        table: &'a Table,
    ) -> impl Iterator<Item = Option<FaceAnyHolder>> + 'a {
        self.cfs_faces.iter().map(|face| match face {
            PlaceHolder::Owned(holder) => Some(holder.clone()),
            PlaceHolder::Ref(Name::Entity(ref idx)) => table
                .oriented_face
                .get(idx)
                .cloned()
                .map(FaceAnyHolder::OrientedFace)
                .or_else(|| {
                    table
                        .face_surface
                        .get(idx)
                        .cloned()
                        .map(FaceAnyHolder::FaceSurface)
                }),
            _ => None,
        })
    }
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
    fn place_holder_edge_any_to_index_and_edge_curve(
        &self,
        edge: &PlaceHolder<EdgeAnyHolder>,
    ) -> Option<(u64, EdgeCurveHolder)> {
        use PlaceHolder::Ref;
        let Ref(Name::Entity(ref idx)) = edge else {
            return None;
        };
        self.oriented_edge
            .get(idx)
            .and_then(|oriented_edge| {
                Some((
                    oriented_edge.edge_element_idx()?,
                    oriented_edge.edge_element_holder(self)?,
                ))
            })
            .or_else(|| {
                self.edge_curve
                    .get(idx)
                    .map(|edge_curve| (*idx, edge_curve.clone()))
            })
    }
    fn face_any_to_orientation_and_face(
        &self,
        face: Option<FaceAnyHolder>,
    ) -> Option<(bool, FaceSurfaceHolder)> {
        match face? {
            FaceAnyHolder::FaceSurface(face) => Some((true, face)),
            FaceAnyHolder::OrientedFace(oriented_face) => {
                let face_element = oriented_face.face_element_holder(self)?;
                Some((oriented_face.orientation, face_element))
            }
        }
    }

    fn shell_vertices(&self, shell: &ShellHolder) -> (Vec<Point3>, HashMap<u64, usize>) {
        use PlaceHolder::Ref;
        let mut vidx_map = HashMap::<u64, usize>::new();
        let vertex_to_point = |v: PlaceHolder<VertexPointHolder>| {
            if let Ref(Name::Entity(ref idx)) = v {
                if vidx_map.get(idx).is_none() {
                    let len = vidx_map.len();
                    vidx_map.insert(*idx, len);
                    let p = EntityTable::<VertexPointHolder>::get_owned(self, *idx)
                        .map_err(|e| eprintln!("{e}"))
                        .ok()?;
                    return Some(Point3::from(&p.vertex_geometry));
                }
            }
            None
        };
        let vertices: Vec<Point3> = shell
            .cfs_faces_holder(self)
            .filter_map(move |face| self.face_any_to_orientation_and_face(face))
            .flat_map(move |(_, face)| face.bounds_holder(self))
            .filter_map(move |bound| bound?.bound_holder(self))
            .flat_map(move |bound| bound.edge_list)
            .filter_map(move |edge| self.place_holder_edge_any_to_index_and_edge_curve(&edge))
            .flat_map(move |(_, edge)| [edge.edge_start, edge.edge_end])
            .filter_map(vertex_to_point)
            .collect();
        (vertices, vidx_map)
    }

    fn shell_edges(
        &self,
        shell: &ShellHolder,
        vidx_map: &HashMap<u64, usize>,
    ) -> (Vec<CompressedEdge<Curve3D>>, HashMap<u64, usize>) {
        use PlaceHolder::Ref;
        let mut eidx_map = HashMap::<u64, usize>::new();
        let edge_curve_to_compressed_edge = |(idx, edge): (u64, EdgeCurveHolder)| {
            if eidx_map.get(&idx).is_some() {
                return None;
            }
            let len = eidx_map.len();
            eidx_map.insert(idx, len);
            let edge_curve = edge
                .clone()
                .into_owned(self)
                .map_err(|e| eprintln!("{e}"))
                .ok()?;
            let curve = edge_curve
                .parse_curve3d()
                .map_err(|e| eprintln!("{e}"))
                .ok()?;
            let Ref(Name::Entity(front_idx)) = edge.edge_start else {
                return None;
            };
            let Ref(Name::Entity(back_idx)) = edge.edge_end else {
                return None;
            };
            Some(CompressedEdge {
                vertices: (*vidx_map.get(&front_idx)?, *vidx_map.get(&back_idx)?),
                curve,
            })
        };
        let edges: Vec<CompressedEdge<Curve3D>> = shell
            .cfs_faces_holder(self)
            .filter_map(move |face| self.face_any_to_orientation_and_face(face))
            .flat_map(move |(_, face)| face.bounds_holder(self))
            .filter_map(move |bound| bound?.bound_holder(self))
            .flat_map(move |bound| bound.edge_list)
            .filter_map(move |edge| self.place_holder_edge_any_to_index_and_edge_curve(&edge))
            .filter_map(edge_curve_to_compressed_edge)
            .collect();
        (edges, eidx_map)
    }
    fn face_bound_to_edges(
        &self,
        bound: FaceBoundHolder,
        eidx_map: &HashMap<u64, usize>,
    ) -> Option<Vec<CompressedEdgeIndex>> {
        use PlaceHolder::Ref;
        let ori = bound.orientation;
        let bound = bound.bound_holder(self)?;
        let mut edges: Vec<CompressedEdgeIndex> = bound
            .edge_list
            .into_iter()
            .filter_map(|edge| {
                let Ref(Name::Entity(ref idx)) = edge else {
                    return None;
                };
                let edge_idx = if let Some(oriented_edge) = self.oriented_edge.get(idx) {
                    CompressedEdgeIndex {
                        index: *eidx_map.get(&oriented_edge.edge_element_idx()?)?,
                        orientation: oriented_edge.orientation == ori,
                    }
                } else {
                    CompressedEdgeIndex {
                        index: *eidx_map.get(idx)?,
                        orientation: ori,
                    }
                };
                Some(edge_idx)
            })
            .collect();
        if !ori {
            edges.reverse();
        }
        Some(edges)
    }

    fn shell_faces(
        &self,
        shell: &ShellHolder,
        eidx_map: &HashMap<u64, usize>,
    ) -> Vec<CompressedFace<Surface>> {
        shell
            .cfs_faces_holder(self)
            .filter_map(|face| self.face_any_to_orientation_and_face(face))
            .filter_map(|(orientation, face)| {
                let step_surface: SurfaceAny = face
                    .face_geometry
                    .clone()
                    .into_owned(self)
                    .map_err(|e| eprintln!("{e}"))
                    .ok()?;
                let mut surface = Surface::try_from(&step_surface)
                    .map_err(|e| eprintln!("{e}"))
                    .ok()?;
                if !face.same_sense {
                    surface.invert()
                }
                let boundaries: Vec<_> = face
                    .bounds_holder(self)
                    .into_iter()
                    .filter_map(|bound| self.face_bound_to_edges(bound?, eidx_map))
                    .collect();
                Some(CompressedFace {
                    surface,
                    boundaries,
                    orientation,
                })
            })
            .collect()
    }

    pub fn to_compressed_shell(
        &self,
        shell: &ShellHolder,
    ) -> std::result::Result<CompressedShell<Point3, Curve3D, Surface>, ExpressParseError> {
        let (vertices, vidx_map) = self.shell_vertices(shell);
        let (edges, eidx_map) = self.shell_edges(shell, &vidx_map);
        Ok(CompressedShell {
            vertices,
            edges,
            faces: self.shell_faces(shell, &eidx_map),
        })
    }
}
