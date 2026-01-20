#![allow(missing_docs, unused_qualifications)]

/// re-export [`ruststep`](https://docs.rs/ruststep/latest/ruststep/)
pub use ruststep;

use ruststep::{
    ast::{DataSection, EntityInstance, Name, Parameter, SubSuperRecord},
    primitive::Logical,
    tables::{EntityTable, IntoOwned, PlaceHolder},
    Holder,
};
use serde::{Deserialize, Serialize};
use std::result::Result;
use std::{collections::HashMap, f64::consts::PI};
use truck_assembly::assy::*;
use truck_geometry::prelude as truck;
use truck_topology::compress::*;

pub mod convert;
/// Geometry parsed from STEP that can be handled by truck
pub mod step_geometry;
use step_geometry::*;

/// the exchange structure corresponds to a graph in STEP file
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Table {
    // representation
    pub representation: HashMap<u64, RepresentationHolder>,
    pub representation_item: HashMap<u64, RepresentationItemHolder>,
    pub representation_context: HashMap<u64, RepresentationContextHolder>,

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
    pub hyperbola: HashMap<u64, HyperbolaHolder>,
    pub parabola: HashMap<u64, ParabolaHolder>,
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
    pub shell_based_surface_model: HashMap<u64, ShellBasedSurfaceModelHolder>,
    pub manifold_solid_brep: HashMap<u64, ManifoldSolidBrepHolder>,

    // assembly
    pub application_context: HashMap<u64, ApplicationContextHolder>,
    pub product_context: HashMap<u64, ProductContextHolder>,
    pub product: HashMap<u64, ProductHolder>,
    pub product_definition_formation: HashMap<u64, ProductDefinitionFormationHolder>,
    pub product_definition_context: HashMap<u64, ProductDefinitionContextHolder>,
    pub product_definition: HashMap<u64, ProductDefinitionHolder>,
    pub product_definition_shape: HashMap<u64, ProductDefinitionShapeHolder>,
    pub shape_definition_representation: HashMap<u64, ShapeDefinitionRepresentationHolder>,
    pub shape_representation: HashMap<u64, ShapeRepresentationHolder>,
    pub context_dependent_shape_representation:
        HashMap<u64, ContextDependentShapeRepresentationHolder>,
    pub shape_representation_relationship: HashMap<u64, ShapeRepresentationRelationshipHolder>,
    pub shape_representation_relationship_with_transformation:
        HashMap<u64, ShapeRepresentationRelationshipWithTransformationHolder>,
    pub next_assembly_usage_occurrence: HashMap<u64, NextAssemblyUsageOccurrenceHolder>,
    pub item_defined_transformation: HashMap<u64, ItemDefinedTransformationHolder>,

    // others
    pub definitional_representation: HashMap<u64, DefinitionalRepresentationHolder>,

    // dummy
    pub dummy: HashMap<u64, DummyHolder>,
}

impl Table {
    pub fn push_instance(&mut self, instance: &EntityInstance) -> ruststep::error::Result<()> {
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
                "HYPERBOLA" => {
                    self.hyperbola
                        .insert(*id, Deserialize::deserialize(record)?);
                }
                "PARABOLA" => {
                    self.parabola.insert(*id, Deserialize::deserialize(record)?);
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
                "SHELL_BASED_SURFACE_MODEL" => {
                    self.shell_based_surface_model
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "MANIFOLD_SOLID_BREP" => {
                    if let Parameter::List(params) = &record.parameter {
                        if params.len() == 2 {
                            self.manifold_solid_brep.insert(
                                *id,
                                ManifoldSolidBrepHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    outer: Deserialize::deserialize(&params[1])?,
                                    voids: Vec::new(),
                                },
                            );
                        }
                    }
                }
                "BREP_WITH_VOIDS" => {
                    self.manifold_solid_brep
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "DEFINITIONAL_REPRESENTATION" => {
                    if let Parameter::List(params) = &record.parameter {
                        if params.len() == 3 {
                            self.definitional_representation.insert(
                                *id,
                                DefinitionalRepresentationHolder {
                                    label: Deserialize::deserialize(&params[0])?,
                                    representation_item: Deserialize::deserialize(&params[1])?,
                                    context_of_items: match &params[2] {
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
                "APPLICATION_CONTEXT" => {
                    self.application_context
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "PRODUCT_CONTEXT" => {
                    self.product_context
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "PRODUCT" => {
                    self.product
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "PRODUCT_DEFINITION_FORMATION" => {
                    self.product_definition_formation
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "PRODUCT_DEFINITION_FORMATION_WITH_SPECIFIED_SOURCE" => {
                    if let Parameter::List(params) = &record.parameter {
                        if params.len() >= 3 {
                            self.product_definition_formation.insert(
                                *id,
                                ProductDefinitionFormationHolder {
                                    id: Deserialize::deserialize(&params[0])?,
                                    description: Deserialize::deserialize(&params[1])?,
                                    of_product: Deserialize::deserialize(&params[2])?,
                                },
                            );
                        }
                    }
                }
                "PRODUCT_DEFINITION_CONTEXT" => {
                    self.product_definition_context
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "PRODUCT_DEFINITION" => {
                    self.product_definition
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "PRODUCT_DEFINITION_SHAPE" => {
                    self.product_definition_shape
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "SHAPE_DEFINITION_REPRESENTATION" => {
                    self.shape_definition_representation
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "SHAPE_REPRESENTATION" => {
                    self.shape_representation
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "ADVANCED_BREP_SHAPE_REPRESENTATION" => {
                    self.shape_representation
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "CONTEXT_DEPENDENT_SHAPE_REPRESENTATION" => {
                    self.context_dependent_shape_representation
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "SHAPE_REPRESENTATION_RELATIONSHIP" => {
                    self.shape_representation_relationship
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "NEXT_ASSEMBLY_USAGE_OCCURRENCE" => {
                    self.next_assembly_usage_occurrence
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
                }
                "ITEM_DEFINED_TRANSFORMATION" => {
                    self.item_defined_transformation
                        .insert(*id, Deserialize::deserialize(&record.parameter)?);
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
                } else if records.len() == 3 {
                    match (
                        records[0].name.as_str(),
                        &records[0].parameter,
                        records[1].name.as_str(),
                        &records[1].parameter,
                        records[2].name.as_str(),
                        &records[2].parameter,
                    ) {
                        (
                            "REPRESENTATION_RELATIONSHIP",
                            Parameter::List(rr_parameter),
                            "REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION",
                            Parameter::List(transformation),
                            "SHAPE_REPRESENTATION_RELATIONSHIP",
                            _,
                        ) => {
                            let entity = ShapeRepresentationRelationshipWithTransformationHolder {
                                name: Deserialize::deserialize(&rr_parameter[0])?,
                                description: Deserialize::deserialize(&rr_parameter[1])?,
                                rep_1: Deserialize::deserialize(&rr_parameter[2])?,
                                rep_2: Deserialize::deserialize(&rr_parameter[3])?,
                                transformation_operator: Deserialize::deserialize(
                                    &transformation[0],
                                )?,
                            };
                            self.shape_representation_relationship_with_transformation
                                .insert(*id, entity);
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

/// Undefined structures are parsed into this.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = dummy)]
#[holder(generate_deserialize)]
pub struct Dummy {
    pub record: String,
    pub is_simple: bool,
}

/// Many geometric and topological elements are contained within this entity's child classes.
/// Since it is essentially an `Any` type, one must manually map the reference according to the context.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = representation_item)]
#[holder(generate_deserialize)]
pub struct RepresentationItem {
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = representation_context)]
#[holder(generate_deserialize)]
pub struct RepresentationContext {
    pub context_identifier: String,
    pub context_type: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = representation)]
#[holder(generate_deserialize)]
pub struct Representation {
    pub name: String,
    #[holder(use_place_holder)]
    pub items: Vec<RepresentationItem>,
    #[holder(use_place_holder)]
    pub context_of_items: Vec<RepresentationContext>,
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

/// `axis1_placement`
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

/// `axis2_placement`
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(axis: &Axis2Placement) -> Result<Self, StepConvertingError> {
        use Axis2Placement::*;
        match axis {
            Axis2Placement2d(axis) => Ok(Matrix3::from(axis)),
            Axis2Placement3d(_) => Err("This is not a 2D axis placement.".into()),
        }
    }
}
impl TryFrom<&Axis2Placement> for Matrix4 {
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(axis: &Axis2Placement) -> Result<Self, StepConvertingError> {
        use Axis2Placement::*;
        match axis {
            Axis2Placement2d(_) => Err("This is not a 3D axis placement.".into()),
            Axis2Placement3d(axis) => Ok(Matrix4::from(axis)),
        }
    }
}

/// `axis2_placement_2d`
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

/// `axis2_placement_3d`
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
            // may can cause a vector of length 0 later, if z is parallel to x
            None => match z.near(&Vector3::unit_x()) { 
                true => Vector3::unit_y(),
                false => Vector3::unit_x(),
            },
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

/// `curve`
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(curve: &CurveAny) -> Result<Self, Self::Error> {
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(curve: &CurveAny) -> Result<Self, Self::Error> {
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

/// `line`
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

/// `bounded_curve`
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(value: &BoundedCurveAny) -> Result<Self, Self::Error> {
        use BoundedCurveAny::*;
        Ok(match value {
            Polyline(x) => Self::Polyline(x.as_ref().into()),
            BSplineCurve(x) => x.as_ref().try_into()?,
        })
    }
}

impl TryFrom<&BoundedCurveAny> for Curve3D {
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(value: &BoundedCurveAny) -> Result<Self, Self::Error> {
        use BoundedCurveAny::*;
        Ok(match value {
            Polyline(x) => Self::Polyline(x.as_ref().into()),
            BSplineCurve(x) => x.as_ref().try_into()?,
        })
    }
}

/// `polyline`
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

/// `b_spline_curve_form`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BSplineCurveForm {
    PolylineForm,
    CircularArc,
    EllipticArc,
    ParabolicArc,
    HyperbolicArc,
    Unspecified,
}

/// `knot_type`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnotType {
    UniformKnots,
    Unspecified,
    QuasiUniformKnots,
    PiecewiseBezierKnots,
}

/// `b_spline_curve_with_knots`
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(curve: &BSplineCurveWithKnots) -> Result<Self, StepConvertingError> {
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

/// `bezier_curve`
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(curve: &BezierCurve) -> Result<Self, StepConvertingError> {
        let degree = curve.degree as usize;
        let knots = KnotVec::bezier_knot(degree);
        let ctrpts = curve.control_points_list.iter().map(Into::into).collect();
        Ok(Self::try_new(knots, ctrpts)?)
    }
}

/// `quasi_uniform_curve`
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(curve: &QuasiUniformCurve) -> Result<Self, StepConvertingError> {
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

/// `uniform_curve`
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(curve: &UniformCurve) -> Result<Self, StepConvertingError> {
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

/// Entity that does not exist in AP042.
/// Curve before rationalization of [`RationalBSplineCurve`] defined by a complex entity
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(curve: &NonRationalBSplineCurve) -> Result<Self, StepConvertingError> {
        use NonRationalBSplineCurve::*;
        match curve {
            BSplineCurveWithKnots(x) => x.try_into(),
            BezierCurve(x) => x.try_into(),
            QuasiUniformCurve(x) => x.try_into(),
            UniformCurve(x) => x.try_into(),
        }
    }
}

/// `rational_b_spline_curve` as complex entity
///
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
    V: Homogeneous<Scalar = f64>,
    V::Point: for<'a> From<&'a CartesianPoint>,
{
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(curve: &RationalBSplineCurve) -> Result<Self, StepConvertingError> {
        Ok(Self::try_from_bspline_and_weights(
            BSplineCurve::try_from(&curve.non_rational_b_spline_curve)?,
            curve.weights_data.clone(),
        )?)
    }
}

/// b_spline_curve
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(value: &BSplineCurveAny) -> Result<Self, Self::Error> {
        use BSplineCurveAny::*;
        Ok(match value {
            NonRationalBSplineCurve(bsp) => Self::BSplineCurve(bsp.as_ref().try_into()?),
            RationalBSplineCurve(bsp) => Self::NurbsCurve(bsp.as_ref().try_into()?),
        })
    }
}

impl TryFrom<&BSplineCurveAny> for Curve3D {
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(value: &BSplineCurveAny) -> Result<Self, Self::Error> {
        use BSplineCurveAny::*;
        Ok(match value {
            NonRationalBSplineCurve(bsp) => Self::BSplineCurve(bsp.as_ref().try_into()?),
            RationalBSplineCurve(bsp) => Self::NurbsCurve(bsp.as_ref().try_into()?),
        })
    }
}

/// `conic`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum Conic {
    #[holder(use_place_holder)]
    Circle(Circle),
    #[holder(use_place_holder)]
    Ellipse(Ellipse),
    #[holder(use_place_holder)]
    Hyperbola(Hyperbola),
    #[holder(use_place_holder)]
    Parabola(Parabola),
}

impl TryFrom<&Conic> for Conic2D {
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(value: &Conic) -> Result<Self, Self::Error> {
        Ok(match value {
            Conic::Circle(value) => Conic2D::Ellipse(value.try_into()?),
            Conic::Ellipse(value) => Conic2D::Ellipse(value.try_into()?),
            Conic::Hyperbola(value) => Conic2D::Hyperbola(value.try_into()?),
            Conic::Parabola(value) => Conic2D::Parabola(value.try_into()?),
        })
    }
}

impl TryFrom<&Conic> for Conic3D {
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(value: &Conic) -> Result<Self, Self::Error> {
        Ok(match value {
            Conic::Circle(value) => Conic3D::Ellipse(value.try_into()?),
            Conic::Ellipse(value) => Conic3D::Ellipse(value.try_into()?),
            Conic::Hyperbola(value) => Conic3D::Hyperbola(value.try_into()?),
            Conic::Parabola(value) => Conic3D::Parabola(value.try_into()?),
        })
    }
}

/// `circle`
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

impl TryFrom<&Circle> for step_geometry::Ellipse<Point2, Matrix3> {
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(circle: &Circle) -> Result<Self, Self::Error> {
        let transform = Matrix3::try_from(&circle.position)? * Matrix3::from_scale(circle.radius);
        Ok(
            Processor::new(truck::TrimmedCurve::new(UnitCircle::new(), (0.0, 2.0 * PI)))
                .transformed(transform),
        )
    }
}

impl TryFrom<&Circle> for step_geometry::Ellipse<Point3, Matrix4> {
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(circle: &Circle) -> Result<Self, Self::Error> {
        let transform = Matrix4::try_from(&circle.position)? * Matrix4::from_scale(circle.radius);
        Ok(
            Processor::new(truck::TrimmedCurve::new(UnitCircle::new(), (0.0, 2.0 * PI)))
                .transformed(transform),
        )
    }
}

/// `ellipse`
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

impl TryFrom<&Ellipse> for step_geometry::Ellipse<Point2, Matrix3> {
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(ellipse: &Ellipse) -> Result<Self, Self::Error> {
        let (r0, r1) = (ellipse.semi_axis_1, ellipse.semi_axis_2);
        let transform =
            Matrix3::try_from(&ellipse.position)? * Matrix3::from_nonuniform_scale(r0, r1);
        Ok(
            Processor::new(truck::TrimmedCurve::new(UnitCircle::new(), (0.0, 2.0 * PI)))
                .transformed(transform),
        )
    }
}

impl TryFrom<&Ellipse> for step_geometry::Ellipse<Point3, Matrix4> {
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(ellipse: &Ellipse) -> Result<Self, Self::Error> {
        let (r0, r1) = (ellipse.semi_axis_1, ellipse.semi_axis_2);
        let transform = Matrix4::try_from(&ellipse.position)?
            * Matrix4::from_nonuniform_scale(r0, r1, f64::min(r0, r1));
        Ok(
            Processor::new(truck::TrimmedCurve::new(UnitCircle::new(), (0.0, 2.0 * PI)))
                .transformed(transform),
        )
    }
}

/// `hyperbola`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = hyperbola)]
#[holder(generate_deserialize)]
pub struct Hyperbola {
    pub label: String,
    #[holder(use_place_holder)]
    pub position: Axis2Placement,
    pub semi_axis: f64,
    pub semi_imag_axis: f64,
}

impl TryFrom<&Hyperbola> for step_geometry::Hyperbola<Point2, Matrix3> {
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(hyperbola: &Hyperbola) -> Result<Self, Self::Error> {
        let (r0, r1) = (hyperbola.semi_axis, hyperbola.semi_imag_axis);
        let transform =
            Matrix3::try_from(&hyperbola.position)? * Matrix3::from_nonuniform_scale(r0, r1);
        Ok(
            Processor::new(truck::TrimmedCurve::new(UnitHyperbola::new(), (-1.0, 1.0)))
                .transformed(transform),
        )
    }
}

impl TryFrom<&Hyperbola> for step_geometry::Hyperbola<Point3, Matrix4> {
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(hyperbola: &Hyperbola) -> Result<Self, Self::Error> {
        let (r0, r1) = (hyperbola.semi_axis, hyperbola.semi_imag_axis);
        let transform = Matrix4::try_from(&hyperbola.position)?
            * Matrix4::from_nonuniform_scale(r0, r1, f64::min(r0, r1));
        Ok(
            Processor::new(truck::TrimmedCurve::new(UnitHyperbola::new(), (-1.0, 1.0)))
                .transformed(transform),
        )
    }
}

/// `parabola`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = parabola)]
#[holder(generate_deserialize)]
pub struct Parabola {
    pub label: String,
    #[holder(use_place_holder)]
    pub position: Axis2Placement,
    pub focal_dist: f64,
}

impl TryFrom<&Parabola> for step_geometry::Parabola<Point2, Matrix3> {
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(parabola: &Parabola) -> Result<Self, Self::Error> {
        let transform =
            Matrix3::try_from(&parabola.position)? * Matrix3::from_scale(parabola.focal_dist);
        Ok(
            Processor::new(truck::TrimmedCurve::new(UnitParabola::new(), (-1.0, 1.0)))
                .transformed(transform),
        )
    }
}

impl TryFrom<&Parabola> for step_geometry::Parabola<Point3, Matrix4> {
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(parabola: &Parabola) -> Result<Self, Self::Error> {
        let transform =
            Matrix4::try_from(&parabola.position)? * Matrix4::from_scale(parabola.focal_dist);
        Ok(
            Processor::new(truck::TrimmedCurve::new(UnitParabola::new(), (-1.0, 1.0)))
                .transformed(transform),
        )
    }
}

/// `definitional_representation`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = definitional_representation)]
#[holder(generate_deserialize)]
pub struct DefinitionalRepresentation {
    label: String,
    #[holder(use_place_holder)]
    representation_item: Vec<CurveAny>,
    #[holder(use_place_holder)]
    context_of_items: Dummy,
}

/// `pcurve`
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(value: &Pcurve) -> Result<Self, Self::Error> {
        let surface: Surface = (&value.basis_surface).try_into()?;
        let curve: Curve2D = value
            .reference_to_curve
            .representation_item
            .first()
            .ok_or("no representation item")?
            .try_into()?;
        Ok(step_geometry::PCurve::new(
            Box::new(curve),
            Box::new(surface),
        ))
    }
}

/// `pcurve_or_surface`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum PcurveOrSurface {
    #[holder(use_place_holder)]
    Pcurve(Box<Pcurve>),
    #[holder(use_place_holder)]
    Surface(Box<SurfaceAny>),
}

/// `preferred_surface_representation`
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

/// `surface_curve`
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(value: &SurfaceCurve) -> Result<Self, Self::Error> {
        use PreferredSurfaceCurveRepresentation as PSCR;
        match &value.master_representation {
            PSCR::Curve3D => Ok((&value.curve_3d).try_into()?),
            PSCR::PcurveS1 => {
                if let Some(PcurveOrSurface::Pcurve(x)) = value.associated_geometry.first() {
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

/// `surface`
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(x: &SurfaceAny) -> Result<Self, Self::Error> {
        use SurfaceAny::*;
        Ok(match x {
            ElementarySurface(x) => Self::ElementarySurface(x.as_ref().into()),
            BSplineSurface(x) => x.as_ref().try_into()?,
            SweptSurface(x) => Self::SweptCurve(x.as_ref().try_into()?),
        })
    }
}

/// `elementary_surface`
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

/// `plane`
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

/// `spherical_surface`
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

impl From<&SphericalSurface> for step_geometry::SphericalSurface {
    #[inline(always)]
    fn from(ss: &SphericalSurface) -> Self {
        let mat = Matrix4::from(&ss.position);
        let sphere = Sphere(truck::Sphere::new(Point3::origin(), ss.radius));
        Processor::new(sphere).transformed(mat)
    }
}

/// `cylindrical_surface`
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

impl From<&CylindricalSurface> for step_geometry::CylindricalSurface {
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

/// `toroidal_surface`
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

impl From<&ToroidalSurface> for step_geometry::ToroidalSurface {
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

/// `conical_surface`
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

impl From<&ConicalSurface> for step_geometry::ConicalSurface {
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

/// `b_spline_surface`
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(value: &BSplineSurfaceAny) -> Result<Self, Self::Error> {
        use BSplineSurfaceAny::*;
        Ok(match value {
            NonRationalBSplineSurface(bsp) => Surface::BSplineSurface(bsp.try_into()?),
            RationalBSplineSurface(bsp) => Surface::NurbsSurface(bsp.try_into()?),
        })
    }
}

/// `b_spline_surface_form`
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

/// `b_spline_surface_with_knots`
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(surface: &BSplineSurfaceWithKnots) -> Result<Self, StepConvertingError> {
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

/// `uniform_surface`
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(surface: &UniformSurface) -> Result<Self, StepConvertingError> {
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

/// `quasi_uniform_surface`
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(surface: &QuasiUniformSurface) -> Result<Self, StepConvertingError> {
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

/// `bezier_surface`
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

/// Entity that does not exist in AP042.
/// Surface before rationalization of [`RationalBSplineSurface`] defined by a complex entity
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(value: &NonRationalBSplineSurface) -> Result<Self, Self::Error> {
        use NonRationalBSplineSurface::*;
        match value {
            BSplineSurfaceWithKnots(x) => x.as_ref().try_into(),
            UniformSurface(x) => x.as_ref().try_into(),
            QuasiUniformSurface(x) => x.as_ref().try_into(),
            BezierSurface(x) => Ok(x.as_ref().into()),
        }
    }
}

/// `rational_b_spline_surface` as complex entity
///
/// This struct is an ad hoc implementation that differs from the definition by EXPRESS:
/// in AP042, rationalized curves are defined as complex entities,
/// but here the surfaces before rationalization are held as internal variables.
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(
        RationalBSplineSurface {
            non_rational_b_spline_surface,
            weights_data,
        }: &RationalBSplineSurface,
    ) -> Result<Self, Self::Error> {
        let surface: BSplineSurface<Point3> = non_rational_b_spline_surface.try_into()?;
        Ok(Self::try_from_bspline_and_weights(
            surface,
            weights_data.clone(),
        )?)
    }
}

/// `swept_surface`
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(value: &SweptSurfaceAny) -> Result<Self, Self::Error> {
        use SweptSurfaceAny::*;
        Ok(match value {
            SurfaceOfLinearExtrusion(x) => SweptCurve::ExtrudedCurve(x.as_ref().try_into()?),
            SurfaceOfRevolution(x) => SweptCurve::RevolutedCurve(x.as_ref().try_into()?),
        })
    }
}

/// `surface_of_linear_extrusion`
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(sr: &SurfaceOfLinearExtrusion) -> Result<Self, Self::Error> {
        let curve = Curve3D::try_from(&sr.swept_curve)?;
        let vector = Vector3::from(&sr.extrusion_axis);
        Ok(ExtrudedCurve::by_extrusion(curve, vector))
    }
}

/// `surface_of_revolution`
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
    type Error = StepConvertingError;
    #[inline(always)]
    fn try_from(sr: &SurfaceOfRevolution) -> Result<Self, Self::Error> {
        let curve = Curve3D::try_from(&sr.swept_curve)?;
        let origin = Point3::from(&sr.axis_position.location);
        let axis = sr.axis_position.direction().normalize();
        let mut rev = Processor::new(RevolutedCurve::by_revolution(curve, origin, axis));
        rev.invert();
        Ok(rev)
    }
}

/// `vertex_point`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = vertex_point)]
#[holder(generate_deserialize)]
pub struct VertexPoint {
    pub label: String,
    #[holder(use_place_holder)]
    pub vertex_geometry: CartesianPoint,
}

/// `edge`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum EdgeAny {
    #[holder(use_place_holder)]
    EdgeCurve(EdgeCurve),
    #[holder(use_place_holder)]
    OrientedEdge(OrientedEdge),
}

/// `edge_curve`
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
    pub fn parse_curve2d(&self) -> Result<Curve2D, StepConvertingError> {
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
    ) -> Result<Curve2D, StepConvertingError> {
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
                Conic::Hyperbola(hyperbola) => {
                    let mat = Matrix3::try_from(&hyperbola.position)?
                        * Matrix3::from_nonuniform_scale(
                            hyperbola.semi_axis,
                            hyperbola.semi_imag_axis,
                        );
                    let inv_mat = mat
                        .invert()
                        .ok_or_else(|| "Failed to convert Hyperbola".to_string())?;
                    let (p, q) = (inv_mat.transform_point(p), inv_mat.transform_point(q));
                    let (u, v) = (
                        UnitHyperbola::<Point2>::new()
                            .search_nearest_parameter(p, None, 0)
                            .ok_or_else(|| "the point is not on hyperbola".to_string())?,
                        UnitHyperbola::<Point2>::new()
                            .search_nearest_parameter(q, None, 0)
                            .ok_or_else(|| "the point is not on hyparbola".to_string())?,
                    );
                    let unit = TrimmedCurve::new(UnitHyperbola::<Point2>::new(), (u, v));
                    let mut hyperbola = Processor::new(unit);
                    hyperbola.transform_by(mat);
                    Curve2D::Conic(Conic2D::Hyperbola(hyperbola))
                }
                Conic::Parabola(parabola) => {
                    let mat = Matrix3::try_from(&parabola.position)?
                        * Matrix3::from_scale(parabola.focal_dist);
                    let inv_mat = mat
                        .invert()
                        .ok_or_else(|| "Failed to convert Parabola".to_string())?;
                    let (p, q) = (inv_mat.transform_point(p), inv_mat.transform_point(q));
                    let (u, v) = (
                        UnitHyperbola::<Point2>::new()
                            .search_nearest_parameter(p, None, 0)
                            .ok_or_else(|| "the point is not on parabola".to_string())?,
                        UnitHyperbola::<Point2>::new()
                            .search_nearest_parameter(q, None, 0)
                            .ok_or_else(|| "the point is not on parabola".to_string())?,
                    );
                    let unit = TrimmedCurve::new(UnitHyperbola::<Point2>::new(), (u, v));
                    let mut parabola = Processor::new(unit);
                    parabola.transform_by(mat);
                    Curve2D::Conic(Conic2D::Hyperbola(parabola))
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
    pub fn parse_curve3d(&self) -> Result<Curve3D, StepConvertingError> {
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
    ) -> Result<Curve3D, StepConvertingError> {
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
                Conic::Hyperbola(hyperbola) => {
                    let mat = Matrix4::try_from(&hyperbola.position)?
                        * Matrix4::from_nonuniform_scale(
                            hyperbola.semi_axis,
                            hyperbola.semi_imag_axis,
                            f64::min(hyperbola.semi_axis, hyperbola.semi_imag_axis),
                        );
                    let inv_mat = mat
                        .invert()
                        .ok_or_else(|| "Failed to convert Circle".to_string())?;
                    let (p, q) = (inv_mat.transform_point(p), inv_mat.transform_point(q));
                    let (u, mut v) = (
                        UnitHyperbola::<Point3>::new()
                            .search_nearest_parameter(p, None, 0)
                            .ok_or_else(|| "the point is not on circle".to_string())?,
                        UnitHyperbola::<Point3>::new()
                            .search_nearest_parameter(q, None, 0)
                            .ok_or_else(|| "the point is not on circle".to_string())?,
                    );
                    if v <= u + TOLERANCE {
                        v += 2.0 * PI;
                    }
                    let unit = TrimmedCurve::new(UnitHyperbola::<Point3>::new(), (u, v));
                    let mut hyperbola = Processor::new(unit);
                    hyperbola.transform_by(mat);
                    Curve3D::Conic(Conic3D::Hyperbola(hyperbola))
                }
                Conic::Parabola(parabola) => {
                    let mat = Matrix4::try_from(&parabola.position)?
                        * Matrix4::from_scale(parabola.focal_dist);
                    let inv_mat = mat
                        .invert()
                        .ok_or_else(|| "Failed to convert Parabola".to_string())?;
                    let (p, q) = (inv_mat.transform_point(p), inv_mat.transform_point(q));
                    let (u, v) = (
                        UnitHyperbola::<Point3>::new()
                            .search_nearest_parameter(p, None, 0)
                            .ok_or_else(|| "the point is not on parabola".to_string())?,
                        UnitHyperbola::<Point3>::new()
                            .search_nearest_parameter(q, None, 0)
                            .ok_or_else(|| "the point is not on parabola".to_string())?,
                    );
                    let unit = TrimmedCurve::new(UnitHyperbola::<Point3>::new(), (u, v));
                    let mut parabola = Processor::new(unit);
                    parabola.transform_by(mat);
                    Curve3D::Conic(Conic3D::Hyperbola(parabola))
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
                    .first()
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
                        if let Some(PcurveOrSurface::Pcurve(c)) = c.associated_geometry.first() {
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

/// `oriented_edge`
///
/// `oriented_edge` has duplicated information.
/// These are not included here because they are essentially omitted.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = oriented_edge)]
#[holder(generate_deserialize)]
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

/// `edge_loop`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = edge_loop)]
#[holder(generate_deserialize)]
pub struct EdgeLoop {
    pub label: String,
    #[holder(use_place_holder)]
    pub edge_list: Vec<EdgeAny>,
}

/// `face_bound`
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

/// `face`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum FaceAny {
    #[holder(use_place_holder)]
    FaceSurface(FaceSurface),
    #[holder(use_place_holder)]
    OrientedFace(OrientedFace),
}

/// `face_surface`
///
/// `advanced_face` is also parsed to this struct.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = face_surface)]
#[holder(generate_deserialize)]
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

/// `oriented_face`
///
/// `oriented_face` has duplicated information.
/// These are not included here because they are essentially omitted.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = oriented_face)]
#[holder(generate_deserialize)]
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

/// `shell`
///
/// Includes `open_shell` and `closed_shell`.
/// Since these differences are only informal propositions, the data structure does not distinguish between the two.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = shell)]
#[holder(generate_deserialize)]
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

/// `oriented_shell`
///
/// Includes `oriented_open_shell` and `oriented_closed_shell`.
/// Since these differences are only informal propositions, the data structure does not distinguish between the two.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = oriented_shell)]
#[holder(generate_deserialize)]
pub struct OrientedShell {
    pub label: String,
    #[holder(use_place_holder)]
    pub shell_element: Shell,
    pub orientation: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum ShellAny {
    #[holder(use_place_holder)]
    Shell(Shell),
    #[holder(use_place_holder)]
    OrientedShell(OrientedShell),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = shell_based_surface_model)]
#[holder(generate_deserialize)]
pub struct ShellBasedSurfaceModel {
    pub label: String,
    #[holder(use_place_holder)]
    pub sbsm_boundary: Vec<ShellAny>,
}

/// Also serves as `brep_with_voids`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = manifold_solid_brep)]
#[holder(generate_deserialize)]
pub struct ManifoldSolidBrep {
    pub label: String,
    #[holder(use_place_holder)]
    pub outer: ShellAny,
    #[holder(use_place_holder)]
    pub voids: Vec<OrientedShell>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = application_context)]
#[holder(generate_deserialize)]
pub struct ApplicationContext {
    pub application: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = product_context)]
#[holder(generate_deserialize)]
pub struct ProductContext {
    pub name: String,
    #[holder(use_place_holder)]
    pub frame_of_reference: ApplicationContext,
    pub discipline_type: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = product)]
#[holder(generate_deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    #[holder(use_place_holder)]
    pub frame_of_reference: Vec<ProductContext>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = product_definition_formation)]
#[holder(generate_deserialize)]
pub struct ProductDefinitionFormation {
    pub id: String,
    pub description: String,
    #[holder(use_place_holder)]
    pub of_product: Product,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = product_definition_context)]
#[holder(generate_deserialize)]
pub struct ProductDefinitionContext {
    pub name: String,
    #[holder(use_place_holder)]
    pub frame_of_reference: ApplicationContext,
    pub life_cycle_stage: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = product_definition)]
#[holder(generate_deserialize)]
pub struct ProductDefinition {
    pub id: String,
    pub description: String,
    #[holder(use_place_holder)]
    pub formation: ProductDefinitionFormation,
    #[holder(use_place_holder)]
    pub frame_of_reference: ProductDefinitionContext,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(generate_deserialize)]
pub enum CharacterizedDefinition {
    #[holder(use_place_holder)]
    ProductDefinition(Box<ProductDefinition>),
    #[holder(use_place_holder)]
    ProductDefinitionShape(Box<ProductDefinitionShape>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = product_definition_shape)]
#[holder(generate_deserialize)]
pub struct ProductDefinitionShape {
    pub name: String,
    pub description: String,
    #[holder(use_place_holder)]
    pub definition: CharacterizedDefinition,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = shape_representation)]
#[holder(generate_deserialize)]
pub struct ShapeRepresentation {
    pub name: String,
    #[holder(use_place_holder)]
    pub items: Vec<RepresentationItem>,
    #[holder(use_place_holder)]
    pub context_of_items: RepresentationContext,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = context_dependent_shape_representation)]
#[holder(generate_deserialize)]
pub struct ContextDependentShapeRepresentation {
    #[holder(use_place_holder)]
    pub representation_relation: ShapeRepresentationRelationshipWithTransformation,
    #[holder(use_place_holder)]
    pub represented_product_relation: ProductDefinitionShape,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = shape_definition_representation)]
#[holder(generate_deserialize)]
pub struct ShapeDefinitionRepresentation {
    #[holder(use_place_holder)]
    pub definition: ProductDefinitionShape,
    #[holder(use_place_holder)]
    pub used_representation: ShapeRepresentation,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = shape_representation_relationship)]
#[holder(generate_deserialize)]
pub struct ShapeRepresentationRelationship {
    pub name: String,
    pub description: String,
    #[holder(use_place_holder)]
    pub rep_1: ShapeRepresentation,
    #[holder(use_place_holder)]
    pub rep_2: ShapeRepresentation,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = shape_representation_relationship_with_transformation)]
#[holder(generate_deserialize)]
pub struct ShapeRepresentationRelationshipWithTransformation {
    pub name: String,
    pub description: String,
    #[holder(use_place_holder)]
    pub rep_1: ShapeRepresentation,
    #[holder(use_place_holder)]
    pub rep_2: ShapeRepresentation,
    #[holder(use_place_holder)]
    pub transformation_operator: ItemDefinedTransformation,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = next_assembly_usage_occurrence)]
#[holder(generate_deserialize)]
pub struct NextAssemblyUsageOccurrence {
    pub id: String,
    pub name: String,
    pub description: String,
    #[holder(use_place_holder)]
    pub relating_product_definition: ProductDefinition,
    #[holder(use_place_holder)]
    pub related_product_definition: ProductDefinition,
    pub reference_designator: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Holder)]
#[holder(table = Table)]
#[holder(field = item_defined_transformation)]
#[holder(generate_deserialize)]
pub struct ItemDefinedTransformation {
    name: String,
    description: String,
    #[holder(use_place_holder)]
    transform_item_1: Axis2Placement,
    #[holder(use_place_holder)]
    transform_item_2: Axis2Placement,
}

impl TryFrom<&ItemDefinedTransformation> for Matrix3 {
    type Error = StepConvertingError;
    fn try_from(value: &ItemDefinedTransformation) -> Result<Self, Self::Error> {
        let mat1: Self = (&value.transform_item_1).try_into()?;
        let mat2: Self = (&value.transform_item_2).try_into()?;
        Ok(mat2 * mat1.invert().unwrap())
    }
}

impl TryFrom<&ItemDefinedTransformation> for Matrix4 {
    type Error = StepConvertingError;
    fn try_from(value: &ItemDefinedTransformation) -> Result<Self, Self::Error> {
        let mat1: Self = (&value.transform_item_1).try_into()?;
        let mat2: Self = (&value.transform_item_2).try_into()?;
        Ok(mat2 * mat1.invert().unwrap())
    }
}
