use ruststep::{
    ast::{DataSection, Name},
    tables::PlaceHolder,
};
use std::{collections::HashMap, str::FromStr};
use truck_stepio::r#in::*;

#[test]
fn read() {
    let data_section = DataSection::from_str(
        "DATA;
#1 = CARTESIAN_POINT('Point', (0.1, 0.2, 0.3));
#2 = DIRECTION('Dir', (1.0, 2.0, 3.0));
#3 = VECTOR('Vector', #2, 2.0);
#4 = PLACEMENT('Placement', #1);
#5 = AXIS1_PLACEMENT('Axis1Placement_0', #1, $);
#6 = AXIS1_PLACEMENT('Axis1Placement_1', #1, #2);
#7 = AXIS2_PLACEMENT_2D('Axis2Placement2d_0', #1, $);
#8 = AXIS2_PLACEMENT_2D('Axis2Placement2d_1', #1, #2);
#9 = AXIS2_PLACEMENT_3D('Axis2Placement3d_0', #1, $, $);
#10 = AXIS2_PLACEMENT_3D('Axis2Placement3d_1', #1, #2, $);
#11 = AXIS2_PLACEMENT_3D('Axis2Placement3d_2', #1, $, #2);
#12 = AXIS2_PLACEMENT_3D('Axis2Placement3d_3', #1, #2, #2);
ENDSEC;
",
    )
    .unwrap();
    let table = Table::from_data_section(&data_section).unwrap();
    let ans_table = Table {
        cartesian_point: HashMap::from_iter(vec![(
            1,
            CartesianPointHolder {
                label: "Point".to_string(),
                coordinates: vec![0.1, 0.2, 0.3],
            },
        )]),
        direction: HashMap::from_iter(vec![(
            2,
            DirectionHolder {
                label: "Dir".to_string(),
                direction_ratios: vec![1.0, 2.0, 3.0],
            },
        )]),
        vector: HashMap::from_iter(vec![(
            3,
            VectorHolder {
                label: "Vector".to_string(),
                orientation: PlaceHolder::Ref(Name::Entity(2)),
                magnitude: 2.0,
            },
        )]),
        placement: HashMap::from_iter(vec![(
            4,
            PlacementHolder {
                label: "Placement".to_string(),
                location: PlaceHolder::Ref(Name::Entity(1)),
            },
        )]),
        axis1_placement: HashMap::from_iter(vec![
            (
                5,
                Axis1PlacementHolder {
                    label: "Axis1Placement_0".to_string(),
                    location: PlaceHolder::Ref(Name::Entity(1)),
                    direction: None,
                },
            ),
            (
                6,
                Axis1PlacementHolder {
                    label: "Axis1Placement_1".to_string(),
                    location: PlaceHolder::Ref(Name::Entity(1)),
                    direction: Some(PlaceHolder::Ref(Name::Entity(2))),
                },
            ),
        ]),
        axis2_placement_2d: HashMap::from_iter(vec![
            (
                7,
                Axis2Placement2dHolder {
                    label: "Axis2Placement2d_0".to_string(),
                    location: PlaceHolder::Ref(Name::Entity(1)),
                    ref_direction: None,
                },
            ),
            (
                8,
                Axis2Placement2dHolder {
                    label: "Axis2Placement2d_1".to_string(),
                    location: PlaceHolder::Ref(Name::Entity(1)),
                    ref_direction: Some(PlaceHolder::Ref(Name::Entity(2))),
                },
            ),
        ]),
        axis2_placement_3d: HashMap::from_iter(vec![
            (
                9,
                Axis2Placement3dHolder {
                    label: "Axis2Placement3d_0".to_string(),
                    location: PlaceHolder::Ref(Name::Entity(1)),
                    axis: None,
                    ref_direction: None,
                },
            ),
            (
                10,
                Axis2Placement3dHolder {
                    label: "Axis2Placement3d_1".to_string(),
                    location: PlaceHolder::Ref(Name::Entity(1)),
                    axis: Some(PlaceHolder::Ref(Name::Entity(2))),
                    ref_direction: None,
                },
            ),
            (
                11,
                Axis2Placement3dHolder {
                    label: "Axis2Placement3d_2".to_string(),
                    location: PlaceHolder::Ref(Name::Entity(1)),
                    axis: None,
                    ref_direction: Some(PlaceHolder::Ref(Name::Entity(2))),
                },
            ),
            (
                12,
                Axis2Placement3dHolder {
                    label: "Axis2Placement3d_3".to_string(),
                    location: PlaceHolder::Ref(Name::Entity(1)),
                    axis: Some(PlaceHolder::Ref(Name::Entity(2))),
                    ref_direction: Some(PlaceHolder::Ref(Name::Entity(2))),
                },
            ),
        ]),
        ..Default::default()
    };
    assert_eq!(table, ans_table);
}
