//! Planar slicing algorithms for additive manufacturing (3D printing) and contour extraction.

use std::collections::HashMap;
use rayon::prelude::*;
use truck_base::cgmath64::*;
use truck_polymesh::PolygonMesh;

/// Evaluates signed distance of a 3D point to a plane given by $ax + by + cz + d = 0$.
#[inline(always)]
fn eval_plane(plane: [f64; 4], p: Point3) -> f64 {
    plane[0] * p.x + plane[1] * p.y + plane[2] * p.z + plane[3]
}

/// Intersects a 3D triangle with a plane $ax + by + cz + d = 0$.
///
/// Returns `Some((p1, p2))` representing the directed 3D line segment of intersection
/// oriented consistently with the triangle's winding order, or `None` if the triangle
/// does not intersect the plane.
pub fn intersect_triangle_plane(tri: [Point3; 3], plane: [f64; 4]) -> Option<(Point3, Point3)> {
    let mut d0 = eval_plane(plane, tri[0]);
    let mut d1 = eval_plane(plane, tri[1]);
    let mut d2 = eval_plane(plane, tri[2]);

    // Robustness perturbation against vertices lying exactly on the plane
    const EPS: f64 = 1e-7;
    if d0.abs() < EPS {
        d0 = EPS;
    }
    if d1.abs() < EPS {
        d1 = EPS;
    }
    if d2.abs() < EPS {
        d2 = EPS;
    }

    let s0 = d0 > 0.0;
    let s1 = d1 > 0.0;
    let s2 = d2 > 0.0;

    // All vertices on the same side -> no intersection
    if s0 == s1 && s1 == s2 {
        return None;
    }

    if s0 != s1 && s0 != s2 {
        // v0 is isolated
        let t1 = d0 / (d0 - d1);
        let p1 = tri[0] + t1 * (tri[1] - tri[0]);
        let t2 = d0 / (d0 - d2);
        let p2 = tri[0] + t2 * (tri[2] - tri[0]);
        if d0 > 0.0 {
            Some((p2, p1))
        } else {
            Some((p1, p2))
        }
    } else if s1 != s0 && s1 != s2 {
        // v1 is isolated
        let t1 = d1 / (d1 - d2);
        let p1 = tri[1] + t1 * (tri[2] - tri[1]);
        let t2 = d1 / (d1 - d0);
        let p2 = tri[1] + t2 * (tri[0] - tri[1]);
        if d1 > 0.0 {
            Some((p2, p1))
        } else {
            Some((p1, p2))
        }
    } else {
        // v2 is isolated
        let t1 = d2 / (d2 - d0);
        let p1 = tri[2] + t1 * (tri[0] - tri[2]);
        let t2 = d2 / (d2 - d1);
        let p2 = tri[2] + t2 * (tri[1] - tri[2]);
        if d2 > 0.0 {
            Some((p2, p1))
        } else {
            Some((p1, p2))
        }
    }
}

/// Slices a polygon mesh against multiple planes sequentially on the CPU.
pub fn slice_mesh_planes_cpu(mesh: &PolygonMesh, planes: &[[f64; 4]]) -> Vec<Vec<(Point3, Point3)>> {
    let tri_list: Vec<[Point3; 3]> = {
        let pos = mesh.positions();
        mesh.face_iter()
            .flat_map(|face| {
                (2..face.len()).map(move |i| {
                    [pos[face[0].pos], pos[face[i - 1].pos], pos[face[i].pos]]
                })
            })
            .collect()
    };

    planes
        .iter()
        .map(|&plane| {
            tri_list
                .iter()
                .filter_map(|&tri| intersect_triangle_plane(tri, plane))
                .collect()
        })
        .collect()
}

/// Slices a polygon mesh against multiple planes in parallel using Rayon.
pub fn slice_mesh_planes_rayon(mesh: &PolygonMesh, planes: &[[f64; 4]]) -> Vec<Vec<(Point3, Point3)>> {
    let tri_list: Vec<[Point3; 3]> = {
        let pos = mesh.positions();
        mesh.face_iter()
            .flat_map(|face| {
                (2..face.len()).map(move |i| {
                    [pos[face[0].pos], pos[face[i - 1].pos], pos[face[i].pos]]
                })
            })
            .collect()
    };

    planes
        .par_iter()
        .map(|&plane| {
            tri_list
                .iter()
                .filter_map(|&tri| intersect_triangle_plane(tri, plane))
                .collect()
        })
        .collect()
}

/// Chains unordered 3D line segments from a planar slice into continuous, ordered polygon contours/loops.
///
/// Each returned polygon loop is an ordered sequence of 3D vertices forming a closed or continuous contour.
#[allow(clippy::type_complexity)]
pub fn chain_segments_into_loops(segments: &[(Point3, Point3)], tol: f64) -> Vec<Vec<Point3>> {
    if segments.is_empty() {
        return Vec::new();
    }

    let tol_val = if tol > 0.0 { tol } else { 1e-5 };
    let tol_sq = tol_val * tol_val;
    let mut used = vec![false; segments.len()];
    let mut loops = Vec::new();

    let quant = |p: Point3| -> (i64, i64, i64) {
        (
            (p.x / tol_val).round() as i64,
            (p.y / tol_val).round() as i64,
            (p.z / tol_val).round() as i64,
        )
    };

    let mut grid: HashMap<(i64, i64, i64), Vec<(usize, u8)>> = HashMap::new();
    for (idx, &(p1, p2)) in segments.iter().enumerate() {
        grid.entry(quant(p1)).or_default().push((idx, 0));
        grid.entry(quant(p2)).or_default().push((idx, 1));
    }

    let find_next = |curr_pt: Point3, used: &[bool], grid: &HashMap<(i64, i64, i64), Vec<(usize, u8)>>| -> Option<(usize, Point3)> {
        let c = quant(curr_pt);
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if let Some(list) = grid.get(&(c.0 + dx, c.1 + dy, c.2 + dz)) {
                        for &(seg_idx, endpt) in list {
                            if !used[seg_idx] {
                                let (s1, s2) = segments[seg_idx];
                                let (match_pt, next_pt) = if endpt == 0 { (s1, s2) } else { (s2, s1) };
                                if (match_pt - curr_pt).magnitude2() <= tol_sq {
                                    return Some((seg_idx, next_pt));
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    };

    for i in 0..segments.len() {
        if used[i] {
            continue;
        }
        used[i] = true;
        let mut current_loop = vec![segments[i].0, segments[i].1];
        let mut curr_tip = segments[i].1;

        // Extend forward
        while let Some((next_idx, next_pt)) = find_next(curr_tip, &used, &grid) {
            used[next_idx] = true;
            if (next_pt - current_loop[0]).magnitude2() <= tol_sq {
                break;
            } else {
                current_loop.push(next_pt);
                curr_tip = next_pt;
            }
        }

        // Backward extension if not closed
        if (curr_tip - current_loop[0]).magnitude2() > tol_sq {
            let mut curr_tail = current_loop[0];
            while let Some((prev_idx, prev_pt)) = find_next(curr_tail, &used, &grid) {
                used[prev_idx] = true;
                if (prev_pt - curr_tip).magnitude2() <= tol_sq {
                    break;
                }
                current_loop.insert(0, prev_pt);
                curr_tail = prev_pt;
            }
        }

        if current_loop.len() >= 3 {
            loops.push(current_loop);
        }
    }

    loops
}

/// Alias for [`chain_segments_into_loops`].
#[inline(always)]
pub fn chain_segments(segments: &[(Point3, Point3)], tol: f64) -> Vec<Vec<Point3>> {
    chain_segments_into_loops(segments, tol)
}

/// Trait for planar slicing on polygon meshes.
pub trait PlanarSlicing {
    /// Slices the mesh against multiple planes, returning 3D line segments for each plane.
    fn slice_mesh_planes(&self, planes: &[[f64; 4]]) -> Vec<Vec<(Point3, Point3)>>;
}

impl PlanarSlicing for PolygonMesh {
    fn slice_mesh_planes(&self, planes: &[[f64; 4]]) -> Vec<Vec<(Point3, Point3)>> {
        slice_mesh_planes_rayon(self, planes)
    }
}
