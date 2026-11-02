use std::str::FromStr;
use truck_stepio::r#in::{
    ruststep::{ast::DataSection, tables::EntityTable},
    step_geometry::*,
    *,
};

fn table(source: &str) -> Table {
    Table::from_data_section(&DataSection::from_str(source).unwrap())
}

fn frame(axis: [f64; 3], reference: &str) -> Matrix4 {
    let source = format!(
        "DATA;
        #1=AXIS2_PLACEMENT_3D('',#2,#3,{reference});
        #2=CARTESIAN_POINT('',(10.,20.,30.));
        #3=DIRECTION('',({},{},{}));
        #4=DIRECTION('',(3.,0.,3.)); ENDSEC;",
        axis[0], axis[1], axis[2]
    );
    let value = EntityTable::<Axis2Placement3dHolder>::get_owned(&table(&source), 1).unwrap();
    Matrix4::from(&value)
}

fn assert_frame(matrix: Matrix4) {
    let axes = [
        matrix.x.truncate(),
        matrix.y.truncate(),
        matrix.z.truncate(),
    ];
    for a in axes {
        assert!((a.magnitude2() - 1.0).abs() < 1e-12, "{matrix:?}");
    }
    for (i, a) in axes.iter().enumerate() {
        for b in &axes[i + 1..] {
            assert!(a.dot(*b).abs() < 1e-12, "{matrix:?}");
        }
    }
    assert!((matrix.determinant() - 1.0).abs() < 1e-12);
    assert_eq!(matrix.w, Vector4::new(10., 20., 30., 1.));
}

#[test]
fn scaled_axis_with_reference_is_orthonormal() {
    let matrix = frame([0., 0., 2.], "#4");
    assert_frame(matrix);
    assert!((matrix.x.truncate() - Vector3::unit_x()).magnitude() < 1e-12);
}

#[test]
fn default_reference_for_positive_x_axis() {
    let matrix = frame([0.0393700787402, 0., 0.], "$");
    assert_frame(matrix);
    assert_eq!(matrix.x.truncate(), Vector3::unit_y());
    assert_eq!(matrix.z.truncate(), Vector3::unit_x());
}

#[test]
fn default_reference_for_negative_x_axis() {
    let matrix = frame([-0.0393700787402, 0., 0.], "$");
    assert_frame(matrix);
    assert_eq!(matrix.x.truncate(), Vector3::unit_y());
    assert_eq!(matrix.z.truncate(), -Vector3::unit_x());
}

#[test]
fn scaled_oblique_axis_is_orthonormal() {
    assert_frame(frame([2., 3., 4.], "$"));
}

#[test]
fn scaled_2d_reference_is_not_a_scale_transform() {
    let t=table("DATA; #1=AXIS2_PLACEMENT_2D('',#2,#3); #2=CARTESIAN_POINT('',(10.,20.)); #3=DIRECTION('',(3.,4.)); ENDSEC;");
    let axis = EntityTable::<Axis2Placement2dHolder>::get_owned(&t, 1).unwrap();
    let matrix = Matrix3::from(&axis);
    assert!((matrix.x.truncate() - Vector2::new(0.6, 0.8)).magnitude() < 1e-12);
    assert!((matrix.determinant() - 1.).abs() < 1e-12);
    assert_eq!(matrix.z, Vector3::new(10., 20., 1.));
}

#[test]
fn nearly_parallel_default_reference_keeps_the_projected_x_branch() {
    let matrix = frame([1., 1e-8, 0.], "$");
    assert_frame(matrix);
    let expected = Vector3::new(1e-8, -1., 0.).normalize();
    assert!((matrix.x.truncate() - expected).magnitude() < 1e-12);
}

#[test]
fn vector_magnitude_is_independent_of_direction_ratios() {
    let t = table("DATA; #1=VECTOR('',#2,7.); #2=DIRECTION('',(0.,3.,4.)); ENDSEC;");
    let vector = EntityTable::<VectorHolder>::get_owned(&t, 1).unwrap();
    assert!((Vector3::from(&vector) - Vector3::new(0., 4.2, 5.6)).magnitude() < 1e-12);
}

#[test]
fn vector_2d_magnitude_is_independent_of_direction_ratios() {
    let t = table("DATA; #1=VECTOR('',#2,7.); #2=DIRECTION('',(3.,4.)); ENDSEC;");
    let vector = EntityTable::<VectorHolder>::get_owned(&t, 1).unwrap();
    assert!((Vector2::from(&vector) - Vector2::new(4.2, 5.6)).magnitude() < 1e-12);
}

#[test]
fn omitted_axes_preserve_default_frame() {
    let t =
        table("DATA; #1=AXIS2_PLACEMENT_3D('',#2,$,$); #2=CARTESIAN_POINT('',(0.,0.,0.)); ENDSEC;");
    let axis = EntityTable::<Axis2Placement3dHolder>::get_owned(&t, 1).unwrap();
    assert_eq!(Matrix4::from(&axis), Matrix4::identity());
}
