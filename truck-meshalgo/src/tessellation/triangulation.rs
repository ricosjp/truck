use super::*;
use std::collections::HashMap;

pub fn delaunay_2d(positions: &Vec<Vector2>, polyline: &Vec<[usize; 2]>) -> Vec<[usize; 3]> {
    let mut triangulation = ConstrainedDelaunayTriangulation::<[f64; 2], FloatKernel>::new();
    let mut tri2poly = HashMap::new();
    let poly2tri: Vec<usize> = positions
        .iter()
        .enumerate()
        .map(|(i, pt)| {
            let idx = triangulation.insert((*pt).into());
            tri2poly.insert(idx, i);
            idx
        })
        .collect();
    polyline.iter().for_each(|a| {
        println!("constraint: {} {}", a[0], a[1]);
        triangulation.add_constraint(poly2tri[a[0]], poly2tri[a[1]]);
    });
    triangulation
        .triangles()
        .map(|tri| {
            let tri = tri.as_triangle();
            [
                tri2poly[&tri[0].fix()],
                tri2poly[&tri[1].fix()],
                tri2poly[&tri[2].fix()],
            ]
        })
        .filter(|tri| {
            let c = (positions[tri[0]] + positions[tri[1]] + positions[tri[2]]) / 3.0;
            let counter: i32 = polyline
                .iter()
                .map(|edge| {
                    let a = positions[edge[0]] - c;
                    let b = positions[edge[1]] - c;
                    let x = (a[0] * b[1] - a[1] * b[0]) * (b[1] - a[1]);
                    if x > 0.0 && a[1] <= 0.0 && b[1] > 0.0 {
                        1
                    } else if x > 0.0 && a[1] >= 0.0 && b[1] < 0.0 {
                        -1
                    } else {
                        0
                    }
                })
                .sum();
            counter > 0
        })
        .collect()
}

pub fn create_polymesh(
    surface: &impl ParametricSurface<Point = Point3, Vector = Vector3>,
    uv_coords: &Vec<Vector2>,
    indices: &Vec<[usize; 3]>,
) -> PolygonMesh {
    let uv_coords: Vec<Vector2> = uv_coords.clone();
    let positions: Vec<Point3> = uv_coords.iter().map(|a| surface.subs(a[0], a[1])).collect();
    let normals: Vec<Vector3> = uv_coords
        .iter()
        .map(|a| surface.normal(a[0], a[1]))
        .collect();
    let tri_faces: Vec<[Vertex; 3]> = indices
        .iter()
        .map(|a| [
            [a[0], a[0], a[0]].into(),
            [a[1], a[1], a[1]].into(),
            [a[2], a[2], a[2]].into(),
        ])
        .collect();
    let faces = Faces::from_tri_and_quad_faces(tri_faces, Vec::new());
    PolygonMesh::debug_new(positions, uv_coords, normals, faces)
}
