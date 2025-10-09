use super::*;
use array_macro::array;
use proptest::prelude::*;
use truck_polymesh::algo::DefaultSplitParams;
use std::f64::consts::PI;
use truck_modeling::*;

/// create uniform unit vector from [0.0f64..1.0f64; 2]
fn dir_from_array(arr: [f64; 2]) -> Vector3 {
    let z = 2.0 * arr[1] - 1.0;
    let theta = 2.0 * PI * arr[0];
    let r = f64::sqrt(f64::max(1.0 - z * z, 0.0));
    Vector3::new(r * f64::cos(theta), r * f64::sin(theta), z)
}

proptest! {
    #[test]
    fn triangle_prism(
        p in prop::array::uniform3(prop::array::uniform2(-100.0f64..100.0f64)),
        h in 0.1f64..100.0f64,
        dir_array in prop::array::uniform2(0.0f64..1.0f64),
        angle in 0.0..PI * 2.0,
        vec in prop::array::uniform3(-100.0f64..100.0f64),
    ) {
        let p = array![i => Point3::new(p[i][0], p[i][1], -h / 2.0); 3];
        let (a, b) = (p[1] - p[0], p[2] - p[0]);
        let volume = (a.x * b.y - a.y * b.x) * h * 0.5;
        let mut grav = p[0] + a / 3.0 + b / 3.0;
        grav.z = 0.0;

        prop_assume!(!volume.is_zero());

        let v = Vertex::news(p);
        let edge = array![i => builder::line(&v[i], &v[(i + 1) % 3]); 3];
        let face = builder::try_attach_plane(&[edge.to_vec().into()]).unwrap();
        let base_solid: Solid = builder::tsweep(&face, h * Vector3::unit_z());
        let axis = dir_from_array(dir_array);
        let trans = Matrix4::from_translation(vec.into()) * Matrix4::from_axis_angle(axis, Rad(angle));
        let solid = builder::transformed(&base_solid, trans);
        let msolid = solid.triangulation(DefaultSplitParams::new(0.05)).collect_option().unwrap();

        prop_assert_near!(msolid.volume(), volume);
        prop_assert_near!(msolid.center_of_gravity().to_point(), trans.transform_point(grav));
    }
}
