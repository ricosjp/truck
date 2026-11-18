//! Ray-to-mesh raycasting algorithms for CAM plunge clearance, 3D printing support generation,
//! optical raytracing, and metrology probe simulation.

use rayon::prelude::*;
use truck_base::cgmath64::*;
use truck_polymesh::PolygonMesh;

/// Intersects a ray `(origin, dir)` with a 3D triangle `[v0, v1, v2]`.
/// Returns `Some((t, hit_point, normal))` if intersection exists with `t >= 0.0`.
pub fn intersect_ray_triangle(
    origin: Point3,
    dir: Vector3,
    tri: [Point3; 3],
) -> Option<(f64, Point3, Vector3)> {
    let e1 = tri[1] - tri[0];
    let e2 = tri[2] - tri[0];
    let pvec = dir.cross(e2);
    let det = e1.dot(pvec);

    if det.abs() < 1e-12 {
        return None;
    }
    let inv_det = 1.0 / det;
    let tvec = origin - tri[0];
    let u = tvec.dot(pvec) * inv_det;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let qvec = tvec.cross(e1);
    let v = dir.dot(qvec) * inv_det;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = e2.dot(qvec) * inv_det;
    if t >= 0.0 {
        let hit_pt = origin + t * dir;
        let mut normal = e1.cross(e2);
        let n_len = normal.magnitude();
        if n_len > 1e-12 {
            normal /= n_len;
        } else {
            normal = Vector3::unit_z();
        }
        Some((t, hit_pt, normal))
    } else {
        None
    }
}

/// Raycasts an array of rays against a polygon mesh sequentially on the CPU.
pub fn raycast_mesh_cpu(
    mesh: &PolygonMesh,
    rays: &[(Point3, Vector3)],
) -> Vec<Option<(f64, Point3, Vector3)>> {
    let positions = mesh.positions();
    let triangles: Vec<[Point3; 3]> = mesh
        .face_iter()
        .flat_map(|face| {
            (2..face.len()).map(move |i| {
                [
                    positions[face[0].pos],
                    positions[face[i - 1].pos],
                    positions[face[i].pos],
                ]
            })
        })
        .collect();

    rays.iter()
        .map(|&(origin, dir)| {
            let dir_norm = dir.normalize();
            let mut nearest: Option<(f64, Point3, Vector3)> = None;
            for tri in &triangles {
                if let Some(hit) = intersect_ray_triangle(origin, dir_norm, *tri) {
                    match nearest {
                        Some((min_t, _, _)) if hit.0 < min_t => nearest = Some(hit),
                        None => nearest = Some(hit),
                        _ => {}
                    }
                }
            }
            nearest
        })
        .collect()
}

/// Raycasts an array of rays against a polygon mesh in parallel via Rayon on the CPU.
pub fn raycast_mesh_rayon(
    mesh: &PolygonMesh,
    rays: &[(Point3, Vector3)],
) -> Vec<Option<(f64, Point3, Vector3)>> {
    let positions = mesh.positions();
    let triangles: Vec<[Point3; 3]> = mesh
        .face_iter()
        .flat_map(|face| {
            (2..face.len()).map(move |i| {
                [
                    positions[face[0].pos],
                    positions[face[i - 1].pos],
                    positions[face[i].pos],
                ]
            })
        })
        .collect();

    rays.par_iter()
        .map(|&(origin, dir)| {
            let dir_norm = dir.normalize();
            let mut nearest: Option<(f64, Point3, Vector3)> = None;
            for tri in &triangles {
                if let Some(hit) = intersect_ray_triangle(origin, dir_norm, *tri) {
                    match nearest {
                        Some((min_t, _, _)) if hit.0 < min_t => nearest = Some(hit),
                        None => nearest = Some(hit),
                        _ => {}
                    }
                }
            }
            nearest
        })
        .collect()
}

/// Trait for raycasting against polygon meshes.
pub trait RaycastMesh {
    /// Raycasts a batch of rays against the mesh, returning the nearest intersection
    /// `(distance, hit_point, outward_normal)` for each ray, or `None` if the ray misses.
    fn raycast(&self, rays: &[(Point3, Vector3)]) -> Vec<Option<(f64, Point3, Vector3)>>;
}

impl RaycastMesh for PolygonMesh {
    fn raycast(&self, rays: &[(Point3, Vector3)]) -> Vec<Option<(f64, Point3, Vector3)>> {
        crate::dispatch::Dispatcher::global().raycast_mesh(self, rays)
    }
}
