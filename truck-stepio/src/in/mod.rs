#![allow(missing_docs)]

use ruststep::{
    ast::{DataSection, EntityInstance, Parameter},
    error::Result,
    Holder,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use truck_base::cgmath64::*;

/// table
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Table {
    pub cartesian_point: HashMap<u64, CartesianPointHolder>,
    pub direction: HashMap<u64, DirectionHolder>,
    pub vector: HashMap<u64, VectorHolder>,
    pub placement: HashMap<u64, PlacementHolder>,
    pub axis1_placement: HashMap<u64, Axis1PlacementHolder>,
    pub axis2_placement_2d: HashMap<u64, Axis2Placement2dHolder>,
    pub axis2_placement_3d: HashMap<u64, Axis2Placement3dHolder>,
}

impl Table {
    pub fn from_data_section(data_section: &DataSection) -> Result<Table> {
        let mut table = Table::default();
        for instance in &data_section.entities {
            if let EntityInstance::Simple { id, record } = instance {
                match record.name.as_str() {
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
                    _ => {}
                };
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
    fn from(vec: &Vector) -> Self { Self::from(&vec.orientation) * vec.magnitude }
}
impl From<&Vector> for Vector3 {
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
