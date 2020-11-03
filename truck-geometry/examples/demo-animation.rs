use cgmath::Point3;
use std::f64::consts::PI;
use truck_geometry::*;

const N: usize = 100; // the number of frames

fn shift(x: &Vector4, t: f64) -> Vector4 {
    let mut x = x.clone();
    x[2] = t;
    x
}

fn main() {
    let knot_vec0 = KnotVec::from(vec![0.0, 0.0, 0.0, 0.25, 0.5, 0.75, 1.0, 1.0, 1.0]);
    let knot_vec1 = KnotVec::from(vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0]);

    let mut control_points = Vec::new();
    control_points.push(vec![
        Point3::new(0.0, 0.0, 0.0).to_homogeneous(),
        Point3::new(0.25, 0.0, 0.0).to_homogeneous(),
        Point3::new(0.5, 0.0, 0.0).to_homogeneous(),
        Point3::new(0.75, 0.0, 0.0).to_homogeneous(),
        Point3::new(1.0, 0.0, 0.0).to_homogeneous(),
    ]);
    control_points.push(control_points[0].iter().map(|x| shift(x, 0.2)).collect());
    control_points.push(control_points[0].iter().map(|x| shift(x, 0.4)).collect());
    control_points.push(control_points[0].iter().map(|x| shift(x, 0.6)).collect());
    control_points.push(control_points[0].iter().map(|x| shift(x, 0.8)).collect());
    control_points.push(control_points[0].iter().map(|x| shift(x, 1.0)).collect());

    let mut bspline = BSplineSurface::new((knot_vec0, knot_vec1), control_points);

    std::fs::DirBuilder::new()
        .recursive(true)
        .create("frames")
        .unwrap();
    for i in 0..N {
        let t = PI * (i as f64) / (N as f64);
        bspline.control_point_mut(0, 2)[1] = t.cos();
        bspline.control_point_mut(1, 2)[1] = (t + 0.2 * PI).cos();
        bspline.control_point_mut(2, 2)[1] = (t + 0.4 * PI).cos();
        bspline.control_point_mut(3, 2)[1] = (t + 0.6 * PI).cos();
        bspline.control_point_mut(4, 2)[1] = (t + 0.8 * PI).cos();
        bspline.control_point_mut(5, 2)[1] = (t + PI).cos();
        let file = std::fs::File::create(&format!("frames/frame{}.obj", i)).unwrap();
        let mesh = truck_polymesh::StructuredMesh::from_surface(&mut bspline, 0.01);
        truck_io::obj::write(&mesh.destruct(), file).unwrap();
    }
}
