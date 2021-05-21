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
    surface: &impl ParametricSurface3D,
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
        .map(|a| {
            [
                [a[0], a[0], a[0]].into(),
                [a[1], a[1], a[1]].into(),
                [a[2], a[2], a[2]].into(),
            ]
        })
        .collect();
    let faces = Faces::from_tri_and_quad_faces(tri_faces, Vec::new());
    PolygonMesh::debug_new(positions, uv_coords, normals, faces)
}

pub fn tessellation(
    surface: &impl ParametricSurface3D,
    uv: &Vec<Point2>,
    polyline: &Vec<[usize; 2]>,
    tol: f64,
) -> PolygonMesh {
    let mut triangulation = ConstrainedDelaunayTriangulation::<[f64; 2], FloatKernel>::new();
    let poly2tri: Vec<usize> = uv
        .iter()
        .map(|pt| triangulation.insert((*pt).into()))
        .collect();
    polyline.iter().for_each(|a| {
        triangulation.add_constraint(poly2tri[a[0]], poly2tri[a[1]]);
    });
    let mut counter = 0;
    loop {
        let vmap: HashMap<usize, Vector3> = triangulation
            .vertices()
            .map(|v| (v.fix(), surface.subs(v[0], v[1]).to_vec()))
            .collect();
        let mut flag = true;
        let cloned = triangulation.clone();
        cloned.triangles().for_each(|face| {
            let tri = face.as_triangle();
            let u = 1.0 / 3.0 + (0.2 * rand::random::<f64>() - 0.1);
            let v = 1.0 / 3.0 + (0.2 * rand::random::<f64>() - 0.1);
            let c = [
                (1.0 - u - v) * tri[0][0] + u * tri[1][0] + v * tri[2][0],
                (1.0 - u - v) * tri[0][1] + u * tri[1][1] + v * tri[2][1],
            ];
            let sc = surface.subs(c[0], c[1]).to_vec();
            let vc = (1.0 - u - v) * vmap[&tri[0].fix()]
                + u * vmap[&tri[1].fix()]
                + v * vmap[&tri[2].fix()];
            let flag0 = sc.distance(vc) < tol;
            if !flag0 {
                let newtri = [
                    (tri[0][0] + tri[1][0] + tri[2][0]) / 3.0,
                    (tri[0][1] + tri[1][1] + tri[2][1]) / 3.0,
                ];
                triangulation.insert(newtri);
            }
            flag = flag && flag0;
        });
        if flag || counter > 10 {
            break;
        }
        counter += 1;
    }
    let mut positions = Vec::<Point3>::new();
    let mut uv_coords = Vec::<Vector2>::new();
    let mut normals = Vec::<Vector3>::new();
    let vmap: HashMap<usize, usize> = triangulation
        .vertices()
        .enumerate()
        .map(|(i, v)| {
            let uv = Vector2::from(*v);
            positions.push(surface.subs(uv[0], uv[1]));
            uv_coords.push(uv);
            normals.push(surface.normal(uv[0], uv[1]));
            (v.fix(), i)
        })
        .collect();
    let tri_faces: Vec<[Vertex; 3]> = triangulation
        .triangles()
        .map(|tri| {
            let tri = tri.as_triangle();
            let idcs = [
                vmap[&tri[0].fix()],
                vmap[&tri[1].fix()],
                vmap[&tri[2].fix()],
            ];
            [
                [idcs[0], idcs[0], idcs[0]].into(),
                [idcs[1], idcs[1], idcs[1]].into(),
                [idcs[2], idcs[2], idcs[2]].into(),
            ]
        })
        .filter(|face: &[Vertex; 3]| {
            let c = (uv_coords[face[0].uv.unwrap()]
                + uv_coords[face[1].uv.unwrap()]
                + uv_coords[face[2].uv.unwrap()])
                / 3.0;
            polyline
                .iter()
                .map(|edge| {
                    let a = uv[edge[0]] - c;
                    let b = uv[edge[1]] - c;
                    let x = (a[0] * b[1] - a[1] * b[0]) * (b[1] - a[1]);
                    if x > 0.0 && a[1] <= 0.0 && b[1] > 0.0 {
                        1
                    } else if x > 0.0 && a[1] >= 0.0 && b[1] < 0.0 {
                        -1
                    } else {
                        0
                    }
                })
                .sum::<i32>()
                > 0
        })
        .collect();
    PolygonMesh::debug_new(
        positions,
        uv_coords,
        normals,
        Faces::from_tri_and_quad_faces(tri_faces, Vec::new()),
    )
}

fn presearch(surface: &Surface, pt: Point3) -> (f64, f64) {
    match surface {
        Surface::Plane(surface) => {
            let v = surface.get_parameter(pt);
            (v[0], v[1])
        }
        Surface::BSplineSurface(surface) => {
            algo::surface::presearch(surface, pt, surface.parameter_range(), 50)
        }
        Surface::NURBSSurface(surface) => {
            algo::surface::presearch(surface, pt, surface.parameter_range(), 50)
        }
        Surface::RevolutedCurve(surface) => {
            algo::surface::presearch(surface, pt, surface.parameter_range(), 50)
        }
    }
}

pub fn tessellate_faces<'a>(
    faces: impl Iterator<Item = &'a Face>,
    tol: f64,
) -> Option<PolygonMesh> {
    let mut poly = PolygonMesh::default();
    for face in faces {
        let surface = face.oriented_surface();
        let mut uv = Vec::<Point2>::new();
        let mut polyline = Vec::<[usize; 2]>::new();
        for wire in face.boundary_iters() {
            let mut counter = 0;
            let len = uv.len();
            for edge in wire {
                let curve = edge.oriented_curve();
                let mut division = curve.parameter_division(tol);
                let _ = division.pop();
                let mut hint = presearch(&surface, curve.subs(division[0]));
                for t in division {
                    let pt = curve.subs(t);
                    hint = match surface.search_parameter(pt, hint, 100) {
                        Some(got) => got,
                        None => {
                            if surface.subs(hint.0, hint.1).near(&pt) {
                                hint
                            } else {
                                let hint0 = presearch(&surface, pt);
                                match surface.search_parameter(pt, hint0, 100) {
                                    Some(got) => got,
                                    None => return None,
                                }
                            }
                        }
                    };
                    uv.push(Point2::new(hint.0, hint.1));
                    counter += 1;
                }
            }
            polyline.extend((0..counter).map(|i| [len + i, len + (i + 1) % counter]));
        }
        poly.merge(square_tessellation(&surface, &uv, &polyline, tol));
    }
    Some(poly)
}

fn meshing_surface(surface: &Surface, tol: f64) -> (Vec<f64>, Vec<f64>) {
    match surface {
        Surface::BSplineSurface(surface) => surface.parameter_division(tol),
        Surface::NURBSSurface(surface) => surface.parameter_division(tol),
        Surface::RevolutedCurve(surface) => surface.parameter_division(tol),
        Surface::Plane(_) => (Vec::new(), Vec::new()),
    }
}

pub fn square_tessellation(
    surface: &Surface,
    uv: &Vec<Point2>,
    polyline: &Vec<[usize; 2]>,
    tol: f64,
) -> PolygonMesh {
    let mut triangulation = ConstrainedDelaunayTriangulation::<[f64; 2], FloatKernel>::new();
    let (udiv, vdiv) = meshing_surface(surface, tol);
    udiv.into_iter()
        .flat_map(|u| vdiv.iter().map(move |v| (u, *v)))
        .filter(|(u, v)| {
            let c = Vector2::new(*u, *v);
            let mut counter: i32 = 0;
            for edge in polyline {
                let a = uv[edge[0]] - c;
                let b = uv[edge[1]] - c;
                let x = (a[0] * b[1] - a[1] * b[0]) * (b[1] - a[1]);
                if f64::abs(x) < TOLERANCE && a[1] * b[1] < 0.0 {
                    return false;
                } else if x > TOLERANCE && a[1] < -TOLERANCE && b[1] > TOLERANCE {
                    counter += 1;
                } else if x > TOLERANCE && a[1] > TOLERANCE && b[1] < -TOLERANCE {
                    counter -= 1;
                }
            }
            counter > 0
        })
        .for_each(|(u, v)| {
            triangulation.insert([u, v]);
        });
    let poly2tri: Vec<usize> = uv
        .iter()
        .map(|pt| triangulation.insert((*pt).into()))
        .collect();
    polyline.iter().for_each(|a| {
        triangulation.add_constraint(poly2tri[a[0]], poly2tri[a[1]]);
    });
    let mut positions = Vec::<Point3>::new();
    let mut uv_coords = Vec::<Vector2>::new();
    let mut normals = Vec::<Vector3>::new();
    let vmap: HashMap<usize, usize> = triangulation
        .vertices()
        .enumerate()
        .map(|(i, v)| {
            let uv = Vector2::from(*v);
            positions.push(surface.subs(uv[0], uv[1]));
            uv_coords.push(uv);
            normals.push(surface.normal(uv[0], uv[1]));
            (v.fix(), i)
        })
        .collect();
    let tri_faces: Vec<[Vertex; 3]> = triangulation
        .triangles()
        .map(|tri| {
            let tri = tri.as_triangle();
            let idcs = [
                vmap[&tri[0].fix()],
                vmap[&tri[1].fix()],
                vmap[&tri[2].fix()],
            ];
            [
                [idcs[0], idcs[0], idcs[0]].into(),
                [idcs[1], idcs[1], idcs[1]].into(),
                [idcs[2], idcs[2], idcs[2]].into(),
            ]
        })
        .filter(|face: &[Vertex; 3]| {
            let c = (uv_coords[face[0].uv.unwrap()]
                + uv_coords[face[1].uv.unwrap()]
                + uv_coords[face[2].uv.unwrap()])
                / 3.0;
            polyline
                .iter()
                .map(|edge| {
                    let a = uv[edge[0]] - c;
                    let b = uv[edge[1]] - c;
                    let x = (a[0] * b[1] - a[1] * b[0]) * (b[1] - a[1]);
                    if x > 0.0 && a[1] <= 0.0 && b[1] > 0.0 {
                        1
                    } else if x > 0.0 && a[1] >= 0.0 && b[1] < 0.0 {
                        -1
                    } else {
                        0
                    }
                })
                .sum::<i32>()
                > 0
        })
        .collect();
    PolygonMesh::debug_new(
        positions,
        uv_coords,
        normals,
        Faces::from_tri_and_quad_faces(tri_faces, Vec::new()),
    )
}
