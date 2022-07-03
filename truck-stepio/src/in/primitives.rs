use ruststep::{
    ast::{DataSection, EntityInstance, Record, SubSuperRecord},
    Holder,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use truck_base::cgmath64::*;

enum RecordType {
    Record(Record),
    SubSuperRecord(SubSuperRecord),
}

fn records(data_section: &DataSection) -> HashMap<u64, RecordType> {
    let mut res = HashMap::default();
    for instance in &data_section.entities {
        match instance {
            EntityInstance::Simple { id, record } => {
                res.insert(*id, RecordType::Record(record.clone()));
            }
            EntityInstance::Complex { id, subsuper } => {
                res.insert(*id, RecordType::SubSuperRecord(subsuper.clone()));
            }
        }
    }
    res
}

/// table
#[derive(Clone, Debug, Default)]
pub struct Table {
    pub cartesian_point: HashMap<u64, CartesianPointHolder>,
    pub direction: HashMap<u64, DirectionHolder>,
    pub vector: HashMap<u64, VectorHolder>,
    pub placement: HashMap<u64, PlacementHolder>,
}

impl Table {
    pub fn from_data_section(data_section: &DataSection) -> ruststep::error::Result<Table> {
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
                        table.placement.insert(*id, Deserialize::deserialize(record)?);
                    }
                    _ => {}
                };
            }
        }
        Ok(table)
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

#[test]
fn read() {
    use std::str::FromStr;
    let data_section = DataSection::from_str(
        "DATA;
#1 = CARTESIAN_POINT('Point', (0.1, 0.2, 0.3));
#2 = DIRECTION('Dir', (1.0, 2.0, 3.0));
#3 = VECTOR('Vector', #2, 2.0);
#4 = PLACEMENT('Placement', #1);
ENDSEC;
",
    )
    .unwrap();
    println!("{data_section:?}");
    let table = Table::from_data_section(&data_section);
    println!("{table:?}");
}
