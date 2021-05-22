use super::*;
use std::collections::HashMap;

pub fn tessellation<'a, C, S>(
    faces: impl Iterator<Item = &'a Face<Point3, C, S>>,
    tol: f64,
) -> Option<PolygonMesh>
where
    C: ParametricCurve<Point = Point3, Vector = Vector3> + Invertible + ParameterDivision1D + 'a,
    S: ParametricSurface3D
        + Invertible
        + ParameterDivision2D
        + SearchParameter<Point = Point3, Parameter = (f64, f64)>
        + 'a,
{
    let mut poly = PolygonMesh::default();
    let mut poly_edges = HashMap::<EdgeID<C>, Vec<Point3>>::new();
    for face in faces {
        let surface = face.oriented_surface();
        let mut uv = Vec::<Point2>::new();
        let mut polyline = Vec::<[usize; 2]>::new();
        for wire in face.boundary_iters() {
            let mut counter = 0;
            let len = uv.len();
            for edge in wire {
                let mut poly_edge = match poly_edges.get(&edge.id()) {
                    Some(got) => got.clone(),
                    None => {
                        let curve = edge.lock_curve().unwrap().clone();
                        let poly = curve.parameter_division(curve.parameter_range(), tol)
                            .into_iter()
                            .map(|t| curve.subs(t))
                            .collect::<Vec<Point3>>();
                        poly_edges.insert(edge.id(), poly.clone());
                        poly
                    }
                };
                if !edge.orientation() {
                    poly_edge.reverse();
                }
                poly_edge.pop();
                let mut hint = None;
                for pt in poly_edge {
                    hint = surface
                        .search_parameter(pt, hint, 100)
                        .or_else(|| surface.search_parameter(pt, None, 100));
                    match hint {
                        Some(hint) => uv.push(hint.into()),
                        None => return None,
                    }
                    counter += 1;
                }
            }
            polyline.extend((0..counter).map(|i| [len + i, len + (i + 1) % counter]));
        }
        poly.merge(square_tessellation(&surface, &uv, &polyline, tol));
    }
    Some(poly)
}

pub fn square_tessellation<S>(
    surface: &S,
    uv: &Vec<Point2>,
    polyline: &Vec<[usize; 2]>,
    tol: f64,
) -> PolygonMesh
where
    S: ParametricSurface3D
        + Invertible
        + ParameterDivision2D
        + SearchParameter<Point = Point3, Parameter = (f64, f64)>,
{
    let mut triangulation = ConstrainedDelaunayTriangulation::<[f64; 2], FloatKernel>::new();
    let mut bdb = BoundingBox::new();
    let poly2tri: Vec<usize> = uv
        .iter()
        .map(|pt| {
            bdb.push(pt);
            triangulation.insert((*pt).into())
        })
        .collect();
    polyline.iter().for_each(|a| {
        triangulation.add_constraint(poly2tri[a[0]], poly2tri[a[1]]);
    });
    let range = ((bdb.min()[0], bdb.max()[0]), (bdb.min()[1], bdb.max()[1]));
    let (udiv, vdiv) = surface.parameter_division(range, tol);
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
