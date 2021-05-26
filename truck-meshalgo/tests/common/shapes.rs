use super::*;
use std::f64::consts::PI;

pub fn sphere(center: Point3, radius: f64, udiv: usize, vdiv: usize) -> PolygonMesh {
    let positions = (0..udiv)
        .flat_map(move |i| {
            (0..vdiv).map(move |j| {
                let u = 2.0 * PI * i as f64 / udiv as f64;
                let v = PI * j as f64 / (vdiv - 1) as f64;
                center + radius * Vector3::new(u.cos() * v.sin(), u.sin() * v.sin(), v.cos())
            })
        })
        .collect::<Vec<_>>();
    let faces = Faces::from_iter((0..udiv).flat_map(move |i| {
        (0..vdiv).map(move |j| {
            [
                i * vdiv + j,
                i * vdiv + (j + 1) % vdiv,
                (i + 1) % udiv * vdiv + (j + 1) % vdiv,
                (i + 1) % udiv * vdiv + j,
            ]
        })
    })); 
    PolygonMesh::new(positions, Vec::new(), Vec::new(), faces)
}
