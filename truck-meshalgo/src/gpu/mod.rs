//! GPU-accelerated mesh algorithms based on wgpu / Vulkan.

use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use truck_base::cgmath64::*;
use truck_base::hash;
use truck_geometry::prelude::*;
use truck_platform::wgpu::*;
use truck_platform::*;
use std::mem::size_of;
use truck_polymesh::{Faces, PolygonMesh, StandardVertex};
use crate::filters::StructuringFilter;

#[cfg(feature = "cuda")]
pub mod cuda;
#[cfg(feature = "cuda")]
pub use cuda::*;

/// GPU representation of a 3D triangle with 16-byte aligned vertices.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuTriangle {
    /// First vertex `[x, y, z, 0.0]`.
    pub v0: [f32; 4],
    /// Second vertex `[x, y, z, 0.0]`.
    pub v1: [f32; 4],
    /// Third vertex `[x, y, z, 0.0]`.
    pub v2: [f32; 4],
}

impl GpuTriangle {
    /// Creates a new GPU triangle from 3 points.
    #[inline(always)]
    pub fn new(p0: Point3, p1: Point3, p2: Point3) -> Self {
        Self {
            v0: [p0.x as f32, p0.y as f32, p0.z as f32, 0.0],
            v1: [p1.x as f32, p1.y as f32, p1.z as f32, 0.0],
            v2: [p2.x as f32, p2.y as f32, p2.z as f32, 0.0],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct ComputeUniforms {
    num_points: u32,
    num_triangles: u32,
    _pad: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct TriPointUniforms {
    num_triangles: u32,
    num_points: u32,
    _pad: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct SsiUniforms {
    n_u1: u32,
    n_v1: u32,
    n_u2: u32,
    n_v2: u32,
    tol: f32,
    max_seeds: u32,
    _pad0: u32,
    _pad1: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct ExtractPlanesUniforms {
    num_faces: u32,
    tol: f32,
    _pad0: u32,
    _pad1: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct SubdivideUniforms {
    num_triangles: u32,
    _pad: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct NurbsUniforms {
    num_u_knots: u32,
    num_v_knots: u32,
    u_degree: u32,
    v_degree: u32,
    cp_stride_v: u32,
    num_points: u32,
    _pad: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct NurbsCurveUniforms {
    num_knots: u32,
    degree: u32,
    num_control_points: u32,
    num_points: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct RaycastUniforms {
    num_rays: u32,
    num_triangles: u32,
    _pad: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct WeldUniforms {
    num_attrs: u32,
    dim: u32,
    tol: f32,
    table_mask: u32,
    min_x: f32,
    min_y: f32,
    min_z: f32,
    inv_tol: f32,
}

/// GPU-compatible sample point representation for B-spline surfaces.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct GpuSurfaceSample {
    /// Evaluated 3D position `[x, y, z, 1.0]`.
    pub pos: [f32; 4],
    /// Evaluated 3D unit surface normal `[nx, ny, nz, 0.0]`.
    pub normal: [f32; 4],
    /// Surface parameter coordinates `[u, v, 0.0, 0.0]`.
    pub uv: [f32; 4],
}

/// Evaluates a regular grid of sample points and normals over a B-spline surface for GPU processing.
pub fn sample_surface_grid(
    surface: &BSplineSurface<Point3>,
    res: (usize, usize),
) -> (Vec<GpuSurfaceSample>, usize, usize) {
    let (n_u, n_v) = (res.0.max(2), res.1.max(2));
    let ((u_min, u_max), (v_min, v_max)) = surface.range_tuple();
    let mut samples = Vec::with_capacity(n_u * n_v);

    for i in 0..n_u {
        let u = u_min + (u_max - u_min) * (i as f64) / ((n_u - 1) as f64);
        for j in 0..n_v {
            let v = v_min + (v_max - v_min) * (j as f64) / ((n_v - 1) as f64);
            let pt = surface.subs(u, v);
            let nor = surface.normal(u, v);
            let nor_unit = if nor.magnitude2() > 1e-12 {
                nor.normalize()
            } else {
                Vector3::unit_z()
            };
            samples.push(GpuSurfaceSample {
                pos: [pt.x as f32, pt.y as f32, pt.z as f32, 1.0],
                normal: [nor_unit.x as f32, nor_unit.y as f32, nor_unit.z as f32, 0.0],
                uv: [u as f32, v as f32, 0.0, 0.0],
            });
        }
    }
    (samples, n_u, n_v)
}

/// Filters and clusters candidate intersection seeds by parameter proximity, keeping the best within each cluster.
pub fn filter_and_cluster_seeds(
    raw_seeds: Vec<(Point2, Point2)>,
    s1: &BSplineSurface<Point3>,
    s2: &BSplineSurface<Point3>,
) -> Vec<(Point2, Point2)> {
    if raw_seeds.is_empty() {
        return Vec::new();
    }
    let ((u1_min, u1_max), (v1_min, v1_max)) = s1.range_tuple();
    let ((u2_min, u2_max), (v2_min, v2_max)) = s2.range_tuple();
    let du1 = (u1_max - u1_min) * 0.03;
    let dv1 = (v1_max - v1_min) * 0.03;
    let du2 = (u2_max - u2_min) * 0.03;
    let dv2 = (v2_max - v2_min) * 0.03;
    let cluster_radius2 = du1 * du1 + dv1 * dv1 + du2 * du2 + dv2 * dv2;

    let mut clustered: Vec<(Point2, Point2, f64)> = Vec::new();
    for seed in raw_seeds {
        let p1 = Point2::new(seed.0.x.clamp(u1_min, u1_max), seed.0.y.clamp(v1_min, v1_max));
        let p2 = Point2::new(seed.1.x.clamp(u2_min, u2_max), seed.1.y.clamp(v2_min, v2_max));
        let dist = s1.subs(p1.x, p1.y).distance2(s2.subs(p2.x, p2.y));

        let mut merged = false;
        for c in &mut clustered {
            let dp1 = (p1.x - c.0.x).powi(2) + (p1.y - c.0.y).powi(2);
            let dp2 = (p2.x - c.1.x).powi(2) + (p2.y - c.1.y).powi(2);
            if dp1 + dp2 < cluster_radius2 {
                if dist < c.2 {
                    *c = (p1, p2, dist);
                }
                merged = true;
                break;
            }
        }
        if !merged {
            clustered.push((p1, p2, dist));
        }
    }
    clustered.into_iter().map(|(p1, p2, _)| (p1, p2)).collect()
}

const DISTANCE_SHADER: &str = r#"
struct Triangle {
    v0: vec4<f32>,
    v1: vec4<f32>,
    v2: vec4<f32>,
};

struct Uniforms {
    num_points: u32,
    num_triangles: u32,
    pad0: u32,
    pad1: u32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> points: array<vec4<f32>>;

@group(0) @binding(2)
var<storage, read> triangles: array<Triangle>;

@group(0) @binding(3)
var<storage, read_write> min_distances: array<f32>;

fn distance2_point_triangle(point: vec3<f32>, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> f32 {
    let ab = v1 - v0;
    let ap = point - v0;
    let bc = v2 - v1;
    let bp = point - v1;
    let ca = v0 - v2;
    let cp = point - v2;
    let nor = cross(ab, ca);

    let s0 = sign(dot(cross(ab, nor), ap));
    let s1 = sign(dot(cross(bc, nor), bp));
    let s2 = sign(dot(cross(ca, nor), cp));
    let coef = s0 + s1 + s2;
    let nor_len2 = dot(nor, nor);

    if (coef < 2.0 || nor_len2 < 1.0e-12) {
        let dot_ab = dot(ab, ab);
        let da = ap - ab * clamp(dot(ap, ab) / max(dot_ab, 1.0e-12), 0.0, 1.0);
        let d2_a = dot(da, da);

        let dot_bc = dot(bc, bc);
        let db = bp - bc * clamp(dot(bp, bc) / max(dot_bc, 1.0e-12), 0.0, 1.0);
        let d2_b = dot(db, db);

        let dot_ca = dot(ca, ca);
        let dc = cp - ca * clamp(dot(cp, ca) / max(dot_ca, 1.0e-12), 0.0, 1.0);
        let d2_c = dot(dc, dc);

        return min(min(d2_a, d2_b), d2_c);
    } else {
        let d = dot(nor, ap);
        return (d * d) / nor_len2;
    }
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= uniforms.num_points) {
        return;
    }
    let pt = points[idx].xyz;
    var min_d2: f32 = 1.0e30;
    for (var i: u32 = 0u; i < uniforms.num_triangles; i = i + 1u) {
        let tri = triangles[i];
        let d2 = distance2_point_triangle(pt, tri.v0.xyz, tri.v1.xyz, tri.v2.xyz);
        min_d2 = min(min_d2, d2);
    }
    min_distances[idx] = min_d2;
}
"#;

const IN_OUT_SHADER: &str = r#"
struct Triangle {
    v0: vec4<f32>,
    v1: vec4<f32>,
    v2: vec4<f32>,
};

struct Uniforms {
    num_points: u32,
    num_triangles: u32,
    pad0: u32,
    pad1: u32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> points: array<vec4<f32>>;

@group(0) @binding(2)
var<storage, read> ray_dirs: array<vec4<f32>>;

@group(0) @binding(3)
var<storage, read> triangles: array<Triangle>;

@group(0) @binding(4)
var<storage, read_write> crossing_counts: array<i32>;

fn check_crossing(origin: vec3<f32>, dir: vec3<f32>, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> i32 {
    let a = v0 - v1;
    let b = v0 - v2;
    let det = dot(a, cross(b, dir));
    if (abs(det) < 1.0e-7) {
        return 0;
    }

    let rhs = v0 - origin;
    let u = dot(rhs, cross(b, dir)) / det;
    if (u <= 0.0 || u >= 1.0) {
        return 0;
    }
    let v = dot(a, cross(rhs, dir)) / det;
    if (v <= 0.0 || u + v >= 1.0) {
        return 0;
    }
    let t = dot(a, cross(b, rhs)) / det;
    if (t <= 0.0) {
        return 0;
    }

    let nor = cross(v1 - v0, v2 - v0);
    if (dot(nor, dir) > 0.0) {
        return 1;
    } else {
        return -1;
    }
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= uniforms.num_points) {
        return;
    }
    let origin = points[idx].xyz;
    let dir = ray_dirs[idx].xyz;
    var count: i32 = 0;
    for (var i: u32 = 0u; i < uniforms.num_triangles; i = i + 1u) {
        let tri = triangles[i];
        count += check_crossing(origin, dir, tri.v0.xyz, tri.v1.xyz, tri.v2.xyz);
    }
    crossing_counts[idx] = count;
}
"#;

const FACE_NORMALS_SHADER: &str = r#"
struct Triangle {
    v0: vec4<f32>,
    v1: vec4<f32>,
    v2: vec4<f32>,
};

struct Uniforms {
    num_triangles: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> triangles: array<Triangle>;

@group(0) @binding(2)
var<storage, read_write> face_normals: array<vec4<f32>>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= uniforms.num_triangles) {
        return;
    }
    let tri = triangles[idx];
    let v0 = tri.v0.xyz;
    let v1 = tri.v1.xyz;
    let v2 = tri.v2.xyz;
    let nor = cross(v1 - v0, v2 - v0);
    let len2 = dot(nor, nor);
    if (len2 < 1.0e-12) {
        face_normals[idx] = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    } else {
        face_normals[idx] = vec4<f32>(normalize(nor), 0.0);
    }
}
"#;

const TRIMMING_LOOP_SHADER: &str = r#"
struct Uniforms {
    num_segments: u32,
    num_points: u32,
    pad0: u32,
    pad1: u32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> segments: array<vec4<f32>>;

@group(0) @binding(2)
var<storage, read> points: array<vec2<f32>>;

@group(0) @binding(3)
var<storage, read> ray_dirs: array<vec2<f32>>;

@group(0) @binding(4)
var<storage, read_write> inside_flags: array<i32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= uniforms.num_points) {
        return;
    }
    let c = points[idx];
    let r = ray_dirs[idx];
    var counter: i32 = 0;
    var valid: bool = true;

    for (var i: u32 = 0u; i < uniforms.num_segments; i = i + 1u) {
        let seg = segments[i];
        let p0 = seg.xy;
        let p1 = seg.zw;
        let a = p0 - c;
        let b = p1 - c;
        let s0 = r.x * a.y - r.y * a.x;
        let s1 = r.x * b.y - r.y * b.x;
        let s2 = a.x * b.y - a.y * b.x;
        let denom = s1 - s0;
        if (abs(denom) < 1.0e-12) {
            continue;
        }
        let x = s2 / denom;
        if (abs(x) < 1.0e-7 && s0 * s1 < 0.0) {
            valid = false;
            break;
        } else if (x > 0.0 && s0 <= 0.0 && s1 > 0.0) {
            counter = counter + 1;
        } else if (x > 0.0 && s0 >= 0.0 && s1 < 0.0) {
            counter = counter - 1;
        }
    }
    if (valid && counter > 0) {
        inside_flags[idx] = 1;
    } else {
        inside_flags[idx] = 0;
    }
}
"#;

const COLLIDE_TRIANGLES_SHADER: &str = r#"
struct Triangle {
    v0: vec4<f32>,
    v1: vec4<f32>,
    v2: vec4<f32>,
};

struct CollisionUniforms {
    num_tri0: u32,
    num_tri1: u32,
    max_segments: u32,
    pad0: u32,
};

@group(0) @binding(0)
var<uniform> uniforms: CollisionUniforms;

@group(0) @binding(1)
var<storage, read> mesh0: array<Triangle>;

@group(0) @binding(2)
var<storage, read> mesh1: array<Triangle>;

@group(0) @binding(3)
var<storage, read_write> num_interferences: atomic<u32>;

@group(0) @binding(4)
var<storage, read_write> out_segments: array<vec4<f32>>;

fn collide_seg_triangle(p: vec3<f32>, q: vec3<f32>, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>, out_h: ptr<function, vec3<f32>>) -> bool {
    let ab = v1 - v0;
    let bc = v2 - v1;
    let ca = v0 - v2;
    let nor = cross(ab, ca);
    let nor_len2 = dot(nor, nor);
    if (nor_len2 < 1.0e-12) {
        return false;
    }

    let ap = p - v0;
    let aq = q - v0;
    let dotapnor = dot(ap, nor);
    let dotaqnor = dot(aq, nor);
    if (dotapnor * dotaqnor > 0.0) {
        return false;
    }

    let denom = dotapnor - dotaqnor;
    if (abs(denom) < 1.0e-12) {
        return false;
    }

    let t = dotapnor / denom;
    let h = p + t * (q - p);

    let s0 = sign(dot(cross(ab, nor), h - v0) + 1.0e-6);
    let s1 = sign(dot(cross(bc, nor), h - v1) + 1.0e-6);
    let s2 = sign(dot(cross(ca, nor), h - v2) + 1.0e-6);

    if (s0 + s1 + s2 >= 2.0) {
        *out_h = h;
        return true;
    }
    return false;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx0 = global_id.x;
    if (idx0 >= uniforms.num_tri0) {
        return;
    }

    let t0 = mesh0[idx0];
    let a0 = t0.v0.xyz;
    let a1 = t0.v1.xyz;
    let a2 = t0.v2.xyz;

    let a_min = min(min(a0, a1), a2);
    let a_max = max(max(a0, a1), a2);

    for (var idx1: u32 = 0u; idx1 < uniforms.num_tri1; idx1 = idx1 + 1u) {
        let t1 = mesh1[idx1];
        let b0 = t1.v0.xyz;
        let b1 = t1.v1.xyz;
        let b2 = t1.v2.xyz;

        let b_min = min(min(b0, b1), b2);
        let b_max = max(max(b0, b1), b2);

        if (a_max.x < b_min.x || b_max.x < a_min.x ||
            a_max.y < b_min.y || b_max.y < a_min.y ||
            a_max.z < b_min.z || b_max.z < a_min.z) {
            continue;
        }

        var p = vec3<f32>(0.0);
        var q = vec3<f32>(0.0);
        var has_p = false;
        var has_q = false;

        var pt = vec3<f32>(0.0);
        if (collide_seg_triangle(a0, a1, b0, b1, b2, &pt)) {
            p = pt; has_p = true;
        }
        if (collide_seg_triangle(a1, a2, b0, b1, b2, &pt)) {
            if (!has_p) { p = pt; has_p = true; }
            else if (!has_q) { q = pt; has_q = true; }
            else {
                let d0 = dot(pt - p, pt - p);
                let d1 = dot(pt - q, pt - q);
                let d2 = dot(p - q, p - q);
                if (d2 < d0) { q = pt; } else if (d2 < d1) { p = pt; }
            }
        }
        if (collide_seg_triangle(a2, a0, b0, b1, b2, &pt)) {
            if (!has_p) { p = pt; has_p = true; }
            else if (!has_q) { q = pt; has_q = true; }
            else {
                let d0 = dot(pt - p, pt - p);
                let d1 = dot(pt - q, pt - q);
                let d2 = dot(p - q, p - q);
                if (d2 < d0) { q = pt; } else if (d2 < d1) { p = pt; }
            }
        }
        if (collide_seg_triangle(b0, b1, a0, a1, a2, &pt)) {
            if (!has_p) { p = pt; has_p = true; }
            else if (!has_q) { q = pt; has_q = true; }
            else {
                let d0 = dot(pt - p, pt - p);
                let d1 = dot(pt - q, pt - q);
                let d2 = dot(p - q, p - q);
                if (d2 < d0) { q = pt; } else if (d2 < d1) { p = pt; }
            }
        }
        if (collide_seg_triangle(b1, b2, a0, a1, a2, &pt)) {
            if (!has_p) { p = pt; has_p = true; }
            else if (!has_q) { q = pt; has_q = true; }
            else {
                let d0 = dot(pt - p, pt - p);
                let d1 = dot(pt - q, pt - q);
                let d2 = dot(p - q, p - q);
                if (d2 < d0) { q = pt; } else if (d2 < d1) { p = pt; }
            }
        }
        if (collide_seg_triangle(b2, b0, a0, a1, a2, &pt)) {
            if (!has_p) { p = pt; has_p = true; }
            else if (!has_q) { q = pt; has_q = true; }
            else {
                let d0 = dot(pt - p, pt - p);
                let d1 = dot(pt - q, pt - q);
                let d2 = dot(p - q, p - q);
                if (d2 < d0) { q = pt; } else if (d2 < d1) { p = pt; }
            }
        }

        if (has_p && has_q) {
            let slot = atomicAdd(&num_interferences, 1u);
            if (slot < uniforms.max_segments) {
                out_segments[slot * 2u + 0u] = vec4<f32>(p, 0.0);
                out_segments[slot * 2u + 1u] = vec4<f32>(q, 0.0);
            }
        }
    }
}
"#;

const TRIANGLE_POINT_DISTANCE_SHADER: &str = r#"
struct ComputeUniforms {
    num_triangles: u32,
    num_points: u32,
    _pad0: u32,
    _pad1: u32,
};

struct GpuTriangle {
    v0: vec4<f32>,
    v1: vec4<f32>,
    v2: vec4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: ComputeUniforms;
@group(0) @binding(1) var<storage, read> triangles: array<GpuTriangle>;
@group(0) @binding(2) var<storage, read> points: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> min_distances: array<f32>;

fn distance2_point_triangle(point: vec3<f32>, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> f32 {
    let ab = v1 - v0;
    let ap = point - v0;
    let bc = v2 - v1;
    let bp = point - v1;
    let ca = v0 - v2;
    let cp = point - v2;
    let nor = cross(ab, ca);

    let s0 = sign(dot(cross(ab, nor), ap));
    let s1 = sign(dot(cross(bc, nor), bp));
    let s2 = sign(dot(cross(ca, nor), cp));
    let coef = s0 + s1 + s2;
    let nor_len2 = dot(nor, nor);

    if (coef < 2.0 || nor_len2 < 1.0e-12) {
        let t_a = clamp(dot(ap, ab) / max(dot(ab, ab), 1.0e-12), 0.0, 1.0);
        let da = ap - ab * t_a;
        let d2_a = dot(da, da);

        let t_b = clamp(dot(bp, bc) / max(dot(bc, bc), 1.0e-12), 0.0, 1.0);
        let db = bp - bc * t_b;
        let d2_b = dot(db, db);

        let t_c = clamp(dot(cp, ca) / max(dot(ca, ca), 1.0e-12), 0.0, 1.0);
        let dc = cp - ca * t_c;
        let d2_c = dot(dc, dc);

        return min(min(d2_a, d2_b), d2_c);
    } else {
        let d = dot(nor, ap);
        return (d * d) / nor_len2;
    }
}

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= uniforms.num_triangles) { return; }

    let tri = triangles[idx];
    var min_d2: f32 = 1.0e30;

    for (var i: u32 = 0u; i < uniforms.num_points; i = i + 1u) {
        let pt = points[i].xyz;
        let d2 = distance2_point_triangle(pt, tri.v0.xyz, tri.v1.xyz, tri.v2.xyz);
        if (d2 < min_d2) {
            min_d2 = d2;
        }
    }

    min_distances[idx] = min_d2;
}
"#;

const SSI_SEEDS_SHADER: &str = r#"
struct SsiUniforms {
    n_u1: u32,
    n_v1: u32,
    n_u2: u32,
    n_v2: u32,
    tol: f32,
    max_seeds: u32,
    _pad0: u32,
    _pad1: u32,
};

struct GpuSurfaceSample {
    pos: vec4<f32>,
    normal: vec4<f32>,
    uv: vec4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: SsiUniforms;
@group(0) @binding(1) var<storage, read> s1_samples: array<GpuSurfaceSample>;
@group(0) @binding(2) var<storage, read> s2_samples: array<GpuSurfaceSample>;
@group(0) @binding(3) var<storage, read_write> num_seeds: atomic<u32>;
@group(0) @binding(4) var<storage, read_write> out_seeds: array<vec4<f32>>;

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    let n1 = uniforms.n_u1 * uniforms.n_v1;
    let num_cells2 = (uniforms.n_u2 - 1u) * (uniforms.n_v2 - 1u);
    let total_tasks = n1 * num_cells2;

    if (idx >= total_tasks) { return; }

    let pt1_idx = idx / num_cells2;
    let cell2_idx = idx % num_cells2;
    let k = cell2_idx / (uniforms.n_v2 - 1u);
    let l = cell2_idx % (uniforms.n_v2 - 1u);

    let p1 = s1_samples[pt1_idx];
    let p1_pos = p1.pos.xyz;
    let p1_nor = p1.normal.xyz;

    let p2_00 = s2_samples[k * uniforms.n_v2 + l];
    let p2_10 = s2_samples[(k + 1u) * uniforms.n_v2 + l];
    let p2_01 = s2_samples[k * uniforms.n_v2 + (l + 1u)];
    let p2_11 = s2_samples[(k + 1u) * uniforms.n_v2 + (l + 1u)];

    let pos00 = p2_00.pos.xyz;
    let pos10 = p2_10.pos.xyz;
    let pos01 = p2_01.pos.xyz;
    let pos11 = p2_11.pos.xyz;

    let d2_00 = dot(pos00 - p1_pos, pos00 - p1_pos);
    let d2_10 = dot(pos10 - p1_pos, pos10 - p1_pos);
    let d2_01 = dot(pos01 - p1_pos, pos01 - p1_pos);
    let d2_11 = dot(pos11 - p1_pos, pos11 - p1_pos);

    let min_d2 = min(min(d2_00, d2_10), min(d2_01, d2_11));

    let h00 = dot(pos00 - p1_pos, p1_nor);
    let h10 = dot(pos10 - p1_pos, p1_nor);
    let h01 = dot(pos01 - p1_pos, p1_nor);
    let h11 = dot(pos11 - p1_pos, p1_nor);

    let min_h = min(min(h00, h10), min(h01, h11));
    let max_h = max(max(h00, h10), max(h01, h11));

    let diag_d2 = dot(pos11 - pos00, pos11 - pos00);
    let cell_reach2 = max(uniforms.tol * uniforms.tol, diag_d2 * 2.25);

    let is_close = (min_d2 < uniforms.tol * uniforms.tol);
    let is_crossing = (min_h <= 0.0 && max_h >= 0.0 && min_d2 < cell_reach2);

    if (is_close || is_crossing) {
        var u2 = p2_00.uv.x;
        var v2 = p2_00.uv.y;

        if (is_crossing) {
            if (h00 * h10 <= 0.0) {
                let t = abs(h00) / (abs(h00) + abs(h10) + 1.0e-12);
                u2 = p2_00.uv.x + t * (p2_10.uv.x - p2_00.uv.x);
                v2 = p2_00.uv.y;
            } else if (h01 * h11 <= 0.0) {
                let t = abs(h01) / (abs(h01) + abs(h11) + 1.0e-12);
                u2 = p2_01.uv.x + t * (p2_11.uv.x - p2_01.uv.x);
                v2 = p2_01.uv.y;
            } else if (h00 * h01 <= 0.0) {
                let t = abs(h00) / (abs(h00) + abs(h01) + 1.0e-12);
                u2 = p2_00.uv.x;
                v2 = p2_00.uv.y + t * (p2_01.uv.y - p2_00.uv.y);
            } else if (h10 * h11 <= 0.0) {
                let t = abs(h10) / (abs(h10) + abs(h11) + 1.0e-12);
                u2 = p2_10.uv.x;
                v2 = p2_10.uv.y + t * (p2_11.uv.y - p2_10.uv.y);
            }
        } else {
            if (min_d2 == d2_10) {
                u2 = p2_10.uv.x; v2 = p2_10.uv.y;
            } else if (min_d2 == d2_01) {
                u2 = p2_01.uv.x; v2 = p2_01.uv.y;
            } else if (min_d2 == d2_11) {
                u2 = p2_11.uv.x; v2 = p2_11.uv.y;
            }
        }

        let slot = atomicAdd(&num_seeds, 1u);
        if (slot < uniforms.max_seeds) {
            out_seeds[slot] = vec4<f32>(p1.uv.x, p1.uv.y, u2, v2);
        }
    }
}
"#;

const SLICING_SHADER: &str = r#"
struct SlicingUniforms {
    num_triangles: u32,
    num_planes: u32,
    max_segments_per_plane: u32,
    _pad: u32,
};

struct GpuTriangle {
    v0: vec4<f32>,
    v1: vec4<f32>,
    v2: vec4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: SlicingUniforms;
@group(0) @binding(1) var<storage, read> triangles: array<GpuTriangle>;
@group(0) @binding(2) var<storage, read> planes: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> counts: array<atomic<u32>>;
@group(0) @binding(4) var<storage, read_write> segments: array<vec4<f32>>;

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let tri_idx = global_id.x;
    let plane_idx = global_id.y;

    if (tri_idx >= uniforms.num_triangles || plane_idx >= uniforms.num_planes) {
        return;
    }

    let tri = triangles[tri_idx];
    let plane = planes[plane_idx];
    let n = plane.xyz;
    let d = plane.w;

    var d0 = dot(tri.v0.xyz, n) + d;
    var d1 = dot(tri.v1.xyz, n) + d;
    var d2 = dot(tri.v2.xyz, n) + d;

    if (abs(d0) < 1.0e-7) { d0 = select(-1.0e-7, 1.0e-7, d0 >= 0.0); }
    if (abs(d1) < 1.0e-7) { d1 = select(-1.0e-7, 1.0e-7, d1 >= 0.0); }
    if (abs(d2) < 1.0e-7) { d2 = select(-1.0e-7, 1.0e-7, d2 >= 0.0); }

    let s0 = d0 > 0.0;
    let s1 = d1 > 0.0;
    let s2 = d2 > 0.0;

    if ((s0 == s1) && (s1 == s2)) {
        return;
    }

    var p1 = vec3<f32>(0.0);
    var p2 = vec3<f32>(0.0);

    if (s0 != s1 && s0 != s2) {
        let t01 = d0 / (d0 - d1);
        let t02 = d0 / (d0 - d2);
        p1 = tri.v0.xyz + t01 * (tri.v1.xyz - tri.v0.xyz);
        p2 = tri.v0.xyz + t02 * (tri.v2.xyz - tri.v0.xyz);
    } else if (s1 != s0 && s1 != s2) {
        let t10 = d1 / (d1 - d0);
        let t12 = d1 / (d1 - d2);
        p1 = tri.v1.xyz + t10 * (tri.v0.xyz - tri.v1.xyz);
        p2 = tri.v1.xyz + t12 * (tri.v2.xyz - tri.v1.xyz);
    } else {
        let t20 = d2 / (d2 - d0);
        let t21 = d2 / (d2 - d1);
        p1 = tri.v2.xyz + t20 * (tri.v0.xyz - tri.v2.xyz);
        p2 = tri.v2.xyz + t21 * (tri.v1.xyz - tri.v2.xyz);
    }

    let slot = atomicAdd(&counts[plane_idx], 1u);
    if (slot < uniforms.max_segments_per_plane) {
        let base = (plane_idx * uniforms.max_segments_per_plane + slot) * 2u;
        segments[base] = vec4<f32>(p1, 0.0);
        segments[base + 1u] = vec4<f32>(p2, 0.0);
    }
}
"#;

const VOLUME_COG_SHADER: &str = r#"
struct VolumeUniforms {
    num_triangles: u32,
    num_workgroups: u32,
    _pad0: u32,
    _pad1: u32,
};

struct GpuTriangle {
    v0: vec4<f32>,
    v1: vec4<f32>,
    v2: vec4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: VolumeUniforms;
@group(0) @binding(1) var<storage, read> triangles: array<GpuTriangle>;
@group(0) @binding(2) var<storage, read_write> partial_sums: array<vec4<f32>>;

var<workgroup> s_data: array<vec4<f32>, 64>;

@compute @workgroup_size(64, 1, 1)
fn main(
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>,
    @builtin(num_workgroups) num_groups: vec3<u32>
) {
    let tid = local_id.x;
    let gid = group_id.x * 64u + tid;
    let stride = num_groups.x * 64u;

    var sum = vec4<f32>(0.0);

    var i = gid;
    while (i < uniforms.num_triangles) {
        let tri = triangles[i];
        let v0 = tri.v0.xyz;
        let v1 = tri.v1.xyz;
        let v2 = tri.v2.xyz;

        let det = dot(cross(v0, v1), v2);
        let vol = det / 6.0;
        let c = (v0 + v1 + v2) * 0.25;

        sum = sum + vec4<f32>(c * vol, vol);
        i = i + stride;
    }

    s_data[tid] = sum;
    workgroupBarrier();

    for (var s: u32 = 32u; s > 0u; s = s >> 1u) {
        if (tid < s) {
            s_data[tid] = s_data[tid] + s_data[tid + s];
        }
        workgroupBarrier();
    }

    if (tid == 0u) {
        partial_sums[group_id.x] = s_data[0];
    }
}
"#;

const EXTRACT_PLANES_SHADER: &str = r#"
struct ExtractPlanesUniforms {
    num_faces: u32,
    tol: f32,
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(0) var<uniform> uniforms: ExtractPlanesUniforms;
@group(0) @binding(1) var<storage, read> face_offsets: array<u32>;
@group(0) @binding(2) var<storage, read> face_positions: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> face_normals: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> counts: array<atomic<u32>>;
@group(0) @binding(5) var<storage, read_write> planar_indices: array<u32>;
@group(0) @binding(6) var<storage, read_write> other_indices: array<u32>;

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= uniforms.num_faces) {
        return;
    }

    let start = face_offsets[idx];
    let end = face_offsets[idx + 1u];
    let count = end - start;
    if (count < 3u) {
        let slot = atomicAdd(&counts[1], 1u);
        other_indices[slot] = idx;
        return;
    }

    var normal_sum = vec3<f32>(0.0);
    if (count == 3u) {
        let p0 = face_positions[start].xyz;
        let p1 = face_positions[start + 1u].xyz;
        let p2 = face_positions[start + 2u].xyz;
        normal_sum = cross(p1 - p0, p2 - p0);
    } else {
        for (var k = 0u; k < count; k = k + 1u) {
            let pk = face_positions[start + k].xyz;
            let next_idx = start + ((k + 1u) % count);
            let pnext = face_positions[next_idx].xyz;
            normal_sum = normal_sum + cross(pk, pnext);
        }
    }

    let len2 = dot(normal_sum, normal_sum);
    var is_planar = false;
    if (len2 > 1.0e-12) {
        let fn_norm = normalize(normal_sum);
        let tol2 = uniforms.tol * uniforms.tol;

        for (var k = 0u; k < count; k = k + 1u) {
            let vn = face_normals[start + k];
            if (vn.w > 0.5) {
                let diff = fn_norm - vn.xyz;
                if (dot(diff, diff) < tol2) {
                    is_planar = true;
                    break;
                }
            }
        }
    }

    if (is_planar) {
        let slot = atomicAdd(&counts[0], 1u);
        planar_indices[slot] = idx;
    } else {
        let slot = atomicAdd(&counts[1], 1u);
        other_indices[slot] = idx;
    }
}
"#;

const SUBDIVIDE_SHADER: &str = r#"
struct SubdivideUniforms {
    num_triangles: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

struct GpuTriangle {
    v0: vec4<f32>,
    v1: vec4<f32>,
    v2: vec4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: SubdivideUniforms;
@group(0) @binding(1) var<storage, read> in_triangles: array<GpuTriangle>;
@group(0) @binding(2) var<storage, read_write> out_triangles: array<GpuTriangle>;

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= uniforms.num_triangles) {
        return;
    }

    let tri = in_triangles[idx];
    let v0 = tri.v0;
    let v1 = tri.v1;
    let v2 = tri.v2;

    let m01 = vec4<f32>((v0.xyz + v1.xyz) * 0.5, 0.0);
    let m12 = vec4<f32>((v1.xyz + v2.xyz) * 0.5, 0.0);
    let m20 = vec4<f32>((v2.xyz + v0.xyz) * 0.5, 0.0);

    let out_base = idx * 4u;
    out_triangles[out_base + 0u] = GpuTriangle(v0, m01, m20);
    out_triangles[out_base + 1u] = GpuTriangle(v1, m12, m01);
    out_triangles[out_base + 2u] = GpuTriangle(v2, m20, m12);
    out_triangles[out_base + 3u] = GpuTriangle(m01, m12, m20);
}
"#;

const NURBS_SURFACE_SHADER: &str = r#"
struct NurbsUniforms {
    num_u_knots: u32,
    num_v_knots: u32,
    u_degree: u32,
    v_degree: u32,
    cp_stride_v: u32,
    num_points: u32,
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(0) var<uniform> uniforms: NurbsUniforms;
@group(0) @binding(1) var<storage, read> u_knots: array<f32>;
@group(0) @binding(2) var<storage, read> v_knots: array<f32>;
@group(0) @binding(3) var<storage, read> control_points: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read> uv_points: array<vec2<f32>>;
@group(0) @binding(5) var<storage, read_write> out_points: array<vec4<f32>>;
@group(0) @binding(6) var<storage, read_write> out_normals: array<vec4<f32>>;

fn inv_or_zero(delta: f32) -> f32 {
    if (abs(delta) > 1.0e-7) {
        return 1.0 / delta;
    }
    return 0.0;
}

struct BasisResult {
    base: u32,
    len: u32,
    vals: array<f32, 16>,
};

fn bspline_basis_u(t: f32, degree: u32, der_rank: u32) -> BasisResult {
    var res: BasisResult;
    let n = uniforms.num_u_knots - 1u;

    var idx = 0u;
    while (idx < n && u_knots[idx + 1u] <= t) {
        idx = idx + 1u;
    }
    if (idx == n) {
        var mult = 0u;
        var k = n;
        loop {
            if (abs(u_knots[k] - u_knots[n]) < 1.0e-7) {
                mult = mult + 1u;
            } else {
                break;
            }
            if (k == 0u) { break; }
            k = k - 1u;
        }
        idx = n - mult;
    }

    var global_base = 0u;
    if (idx >= degree) {
        global_base = idx - degree;
    }
    res.base = global_base;

    var eval: array<f32, 16>;
    for (var i = 0u; i < 16u; i = i + 1u) {
        eval[i] = 0.0;
    }
    eval[idx - global_base] = 1.0;

    if (degree >= der_rank) {
        let k_limit_1 = degree - der_rank;
        for (var k = 1u; k <= k_limit_1; k = k + 1u) {
            var base = 0u;
            if (idx >= k) {
                base = idx - k;
            }
            let delta = u_knots[base + k] - u_knots[base];
            var a = inv_or_zero(delta) * (t - u_knots[base]);
            var max_i = idx;
            if ((n - k - 1u) < max_i) {
                max_i = n - k - 1u;
            }
            for (var i = base; i <= max_i; i = i + 1u) {
                let delta2 = u_knots[i + k + 1u] - u_knots[i + 1u];
                let b = inv_or_zero(delta2) * (u_knots[i + k + 1u] - t);
                let j = i - global_base;
                eval[j] = a * eval[j] + b * eval[j + 1u];
                a = 1.0 - b;
            }
        }
    }

    let k_start_2 = degree - der_rank + 1u;
    if (k_start_2 <= degree) {
        for (var k = k_start_2; k <= degree; k = k + 1u) {
            var base = 0u;
            if (idx >= k) {
                base = idx - k;
            }
            let delta = u_knots[base + k] - u_knots[base];
            var a = inv_or_zero(delta);
            var max_i = idx;
            if ((n - k - 1u) < max_i) {
                max_i = n - k - 1u;
            }
            for (var i = base; i <= max_i; i = i + 1u) {
                let delta2 = u_knots[i + k + 1u] - u_knots[i + 1u];
                let b = inv_or_zero(delta2);
                let j = i - global_base;
                eval[j] = (a * eval[j] - b * eval[j + 1u]) * f32(k);
                a = b;
            }
        }
    }

    var len = degree + 1u;
    if ((n - degree - global_base) < len) {
        len = n - degree - global_base;
    }
    res.len = len;
    for (var i = 0u; i < len; i = i + 1u) {
        res.vals[i] = eval[i];
    }
    return res;
}

fn bspline_basis_v(t: f32, degree: u32, der_rank: u32) -> BasisResult {
    var res: BasisResult;
    let n = uniforms.num_v_knots - 1u;

    var idx = 0u;
    while (idx < n && v_knots[idx + 1u] <= t) {
        idx = idx + 1u;
    }
    if (idx == n) {
        var mult = 0u;
        var k = n;
        loop {
            if (abs(v_knots[k] - v_knots[n]) < 1.0e-7) {
                mult = mult + 1u;
            } else {
                break;
            }
            if (k == 0u) { break; }
            k = k - 1u;
        }
        idx = n - mult;
    }

    var global_base = 0u;
    if (idx >= degree) {
        global_base = idx - degree;
    }
    res.base = global_base;

    var eval: array<f32, 16>;
    for (var i = 0u; i < 16u; i = i + 1u) {
        eval[i] = 0.0;
    }
    eval[idx - global_base] = 1.0;

    if (degree >= der_rank) {
        let k_limit_1 = degree - der_rank;
        for (var k = 1u; k <= k_limit_1; k = k + 1u) {
            var base = 0u;
            if (idx >= k) {
                base = idx - k;
            }
            let delta = v_knots[base + k] - v_knots[base];
            var a = inv_or_zero(delta) * (t - v_knots[base]);
            var max_i = idx;
            if ((n - k - 1u) < max_i) {
                max_i = n - k - 1u;
            }
            for (var i = base; i <= max_i; i = i + 1u) {
                let delta2 = v_knots[i + k + 1u] - v_knots[i + 1u];
                let b = inv_or_zero(delta2) * (v_knots[i + k + 1u] - t);
                let j = i - global_base;
                eval[j] = a * eval[j] + b * eval[j + 1u];
                a = 1.0 - b;
            }
        }
    }

    let k_start_2 = degree - der_rank + 1u;
    if (k_start_2 <= degree) {
        for (var k = k_start_2; k <= degree; k = k + 1u) {
            var base = 0u;
            if (idx >= k) {
                base = idx - k;
            }
            let delta = v_knots[base + k] - v_knots[base];
            var a = inv_or_zero(delta);
            var max_i = idx;
            if ((n - k - 1u) < max_i) {
                max_i = n - k - 1u;
            }
            for (var i = base; i <= max_i; i = i + 1u) {
                let delta2 = v_knots[i + k + 1u] - v_knots[i + 1u];
                let b = inv_or_zero(delta2);
                let j = i - global_base;
                eval[j] = (a * eval[j] - b * eval[j + 1u]) * f32(k);
                a = b;
            }
        }
    }

    var len = degree + 1u;
    if ((n - degree - global_base) < len) {
        len = n - degree - global_base;
    }
    res.len = len;
    for (var i = 0u; i < len; i = i + 1u) {
        res.vals[i] = eval[i];
    }
    return res;
}

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= uniforms.num_points) { return; }

    let uv = uv_points[idx];

    let bu = bspline_basis_u(uv.x, uniforms.u_degree, 0u);
    let dbu = bspline_basis_u(uv.x, uniforms.u_degree, 1u);

    let bv = bspline_basis_v(uv.y, uniforms.v_degree, 0u);
    let dbv = bspline_basis_v(uv.y, uniforms.v_degree, 1u);

    var pt = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var Su_h = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var Sv_h = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    for (var i = 0u; i < bu.len; i = i + 1u) {
        let nu = bu.vals[i];
        let dnu = dbu.vals[i];
        let ui = bu.base + i;

        for (var j = 0u; j < bv.len; j = j + 1u) {
            let nv = bv.vals[j];
            let dnv = dbv.vals[j];
            let vj = bv.base + j;

            let cp = control_points[ui * uniforms.cp_stride_v + vj];

            let w = nu * nv;
            pt = pt + cp * w;

            let wu = dnu * nv;
            Su_h = Su_h + cp * wu;

            let wv = nu * dnv;
            Sv_h = Sv_h + cp * wv;
        }
    }

    var inv_w = 1.0;
    if (abs(pt.w) > 1.0e-7) {
        inv_w = 1.0 / pt.w;
    }
    let S = pt.xyz * inv_w;

    let Su = (Su_h.xyz - Su_h.w * S) * inv_w;
    let Sv = (Sv_h.xyz - Sv_h.w * S) * inv_w;

    let cross_prod = cross(Su, Sv);
    let nor_len2 = dot(cross_prod, cross_prod);
    var nor = vec3<f32>(0.0, 0.0, 0.0);
    if (nor_len2 > 1.0e-14) {
        nor = cross_prod * inverseSqrt(nor_len2);
    }

    out_points[idx] = vec4<f32>(S.x, S.y, S.z, 1.0);
    out_normals[idx] = vec4<f32>(nor.x, nor.y, nor.z, 0.0);
}
"#;

const NURBS_CURVE_SHADER: &str = r#"
struct NurbsCurveUniforms {
    num_knots: u32,
    degree: u32,
    num_control_points: u32,
    num_points: u32,
};

@group(0) @binding(0) var<uniform> uniforms: NurbsCurveUniforms;
@group(0) @binding(1) var<storage, read> knots: array<f32>;
@group(0) @binding(2) var<storage, read> control_points: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> t_points: array<f32>;
@group(0) @binding(4) var<storage, read_write> out_points: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> out_derivatives: array<vec4<f32>>;

fn inv_or_zero(delta: f32) -> f32 {
    if (abs(delta) > 1.0e-7) {
        return 1.0 / delta;
    }
    return 0.0;
}

struct BasisResult {
    base: u32,
    len: u32,
    vals: array<f32, 16>,
};

fn bspline_basis(t: f32, degree: u32, der_rank: u32) -> BasisResult {
    var res: BasisResult;
    let n = uniforms.num_knots - 1u;

    var idx = 0u;
    while (idx < n && knots[idx + 1u] <= t) {
        idx = idx + 1u;
    }
    if (idx == n) {
        var mult = 0u;
        var k = n;
        loop {
            if (abs(knots[k] - knots[n]) < 1.0e-7) {
                mult = mult + 1u;
            } else {
                break;
            }
            if (k == 0u) { break; }
            k = k - 1u;
        }
        idx = n - mult;
    }

    var global_base = 0u;
    if (idx >= degree) {
        global_base = idx - degree;
    }
    res.base = global_base;

    var eval: array<f32, 16>;
    for (var i = 0u; i < 16u; i = i + 1u) {
        eval[i] = 0.0;
    }
    eval[idx - global_base] = 1.0;

    if (degree >= der_rank) {
        let k_limit_1 = degree - der_rank;
        for (var k = 1u; k <= k_limit_1; k = k + 1u) {
            var base = 0u;
            if (idx >= k) {
                base = idx - k;
            }
            let delta = knots[base + k] - knots[base];
            var a = inv_or_zero(delta) * (t - knots[base]);
            var max_i = idx;
            if ((n - k - 1u) < max_i) {
                max_i = n - k - 1u;
            }
            for (var i = base; i <= max_i; i = i + 1u) {
                let delta2 = knots[i + k + 1u] - knots[i + 1u];
                let b = inv_or_zero(delta2) * (knots[i + k + 1u] - t);
                let j = i - global_base;
                eval[j] = a * eval[j] + b * eval[j + 1u];
                a = 1.0 - b;
            }
        }
    }

    let k_start_2 = degree - der_rank + 1u;
    if (k_start_2 <= degree) {
        for (var k = k_start_2; k <= degree; k = k + 1u) {
            var base = 0u;
            if (idx >= k) {
                base = idx - k;
            }
            let delta = knots[base + k] - knots[base];
            var a = inv_or_zero(delta);
            var max_i = idx;
            if ((n - k - 1u) < max_i) {
                max_i = n - k - 1u;
            }
            for (var i = base; i <= max_i; i = i + 1u) {
                let delta2 = knots[i + k + 1u] - knots[i + 1u];
                let b = inv_or_zero(delta2);
                let j = i - global_base;
                eval[j] = (a * eval[j] - b * eval[j + 1u]) * f32(k);
                a = b;
            }
        }
    }

    var len = degree + 1u;
    if ((n - degree - global_base) < len) {
        len = n - degree - global_base;
    }
    res.len = len;
    for (var i = 0u; i < len; i = i + 1u) {
        res.vals[i] = eval[i];
    }
    return res;
}

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= uniforms.num_points) { return; }

    let t = t_points[idx];

    let b = bspline_basis(t, uniforms.degree, 0u);
    let db = bspline_basis(t, uniforms.degree, 1u);

    var pt = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var dpt = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    for (var i = 0u; i < b.len; i = i + 1u) {
        let n = b.vals[i];
        let dn = db.vals[i];
        let ci = b.base + i;
        if (ci < uniforms.num_control_points) {
            let cp = control_points[ci];
            pt = pt + cp * n;
            dpt = dpt + cp * dn;
        }
    }

    var inv_w = 1.0;
    if (abs(pt.w) > 1.0e-7) {
        inv_w = 1.0 / pt.w;
    }
    let C = pt.xyz * inv_w;
    let dC = (dpt.xyz - dpt.w * C) * inv_w;

    out_points[idx] = vec4<f32>(C, 1.0);
    out_derivatives[idx] = vec4<f32>(dC, 0.0);
}
"#;

const RAYCAST_MESH_SHADER: &str = r#"
struct RaycastUniforms {
    num_rays: u32,
    num_triangles: u32,
    _pad0: u32,
    _pad1: u32,
};

struct GpuTriangle {
    v0: vec4<f32>,
    v1: vec4<f32>,
    v2: vec4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: RaycastUniforms;
@group(0) @binding(1) var<storage, read> ray_origins: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> ray_dirs: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> triangles: array<GpuTriangle>;
@group(0) @binding(4) var<storage, read_write> out_distances: array<f32>;
@group(0) @binding(5) var<storage, read_write> out_tri_indices: array<i32>;
@group(0) @binding(6) var<storage, read_write> out_hit_points: array<vec4<f32>>;
@group(0) @binding(7) var<storage, read_write> out_normals: array<vec4<f32>>;

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= uniforms.num_rays) { return; }

    let origin = ray_origins[idx].xyz;
    let dir = ray_dirs[idx].xyz;

    var min_t: f32 = 3.402823e+38;
    var hit_tri: i32 = -1;
    var hit_norm = vec3<f32>(0.0, 0.0, 0.0);

    for (var i = 0u; i < uniforms.num_triangles; i = i + 1u) {
        let tri = triangles[i];
        let v0 = tri.v0.xyz;
        let v1 = tri.v1.xyz;
        let v2 = tri.v2.xyz;

        let e1 = v1 - v0;
        let e2 = v2 - v0;
        let pvec = cross(dir, e2);
        let det = dot(e1, pvec);

        if (abs(det) > 1.0e-7) {
            let inv_det = 1.0 / det;
            let tvec = origin - v0;
            let u = dot(tvec, pvec) * inv_det;
            if (u >= 0.0 && u <= 1.0) {
                let qvec = cross(tvec, e1);
                let v = dot(dir, qvec) * inv_det;
                if (v >= 0.0 && (u + v) <= 1.0) {
                    let hit_dist = dot(e2, qvec) * inv_det;
                    if (hit_dist >= 0.0 && hit_dist < min_t) {
                        min_t = hit_dist;
                        hit_tri = i32(i);
                        let geo_norm = cross(e1, e2);
                        let norm_len = length(geo_norm);
                        if (norm_len > 1.0e-7) {
                            hit_norm = geo_norm / norm_len;
                        } else {
                            hit_norm = vec3<f32>(0.0, 0.0, 1.0);
                        }
                    }
                }
            }
        }
    }

    if (hit_tri >= 0) {
        out_distances[idx] = min_t;
        out_tri_indices[idx] = hit_tri;
        out_hit_points[idx] = vec4<f32>(origin + min_t * dir, 1.0);
        out_normals[idx] = vec4<f32>(hit_norm, 0.0);
    } else {
        out_distances[idx] = -1.0;
        out_tri_indices[idx] = -1;
        out_hit_points[idx] = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}
"#;

const LOOP_SUBDIVISION_SHADER: &str = r#"
struct LoopSubdivisionUniforms {
    num_vertices: u32,
    num_edges: u32,
    _pad0: u32,
    _pad1: u32,
};

struct GpuLoopVertexInfo {
    offset: u32,
    valence: u32,
    boundary_type: u32,
    b0: u32,
    b1: u32,
    _pad: u32,
};

struct GpuLoopEdgeInfo {
    v0: u32,
    v1: u32,
    w0: u32,
    w1: u32,
    gamma: f32,
    is_boundary: u32,
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(0) var<uniform> uniforms: LoopSubdivisionUniforms;
@group(0) @binding(1) var<storage, read> in_vertices: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> vertex_info: array<GpuLoopVertexInfo>;
@group(0) @binding(3) var<storage, read> vertex_neighbors: array<u32>;
@group(0) @binding(4) var<storage, read> edges: array<GpuLoopEdgeInfo>;
@group(0) @binding(5) var<storage, read_write> out_vertices: array<vec4<f32>>;

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= uniforms.num_vertices + uniforms.num_edges) {
        return;
    }

    if (idx < uniforms.num_vertices) {
        let info = vertex_info[idx];
        let v = in_vertices[idx].xyz;
        if (info.boundary_type == 2u) {
            out_vertices[idx] = vec4<f32>(v, 1.0);
        } else if (info.boundary_type == 1u) {
            let b0 = in_vertices[info.b0].xyz;
            let b1 = in_vertices[info.b1].xyz;
            out_vertices[idx] = vec4<f32>(0.75 * v + 0.125 * (b0 + b1), 1.0);
        } else {
            let n = f32(info.valence);
            if (n > 0.0) {
                let pi = 3.141592653589793;
                let alpha = 0.375 + 0.25 * cos(2.0 * pi / n);
                let beta = (0.625 - alpha * alpha) / n;
                var sum_v = vec3<f32>(0.0, 0.0, 0.0);
                for (var j = 0u; j < info.valence; j = j + 1u) {
                    let nb = vertex_neighbors[info.offset + j];
                    sum_v = sum_v + (in_vertices[nb].xyz - v);
                }
                out_vertices[idx] = vec4<f32>(v + beta * sum_v, 1.0);
            } else {
                out_vertices[idx] = vec4<f32>(v, 1.0);
            }
        }
    } else {
        let e_idx = idx - uniforms.num_vertices;
        let edge = edges[e_idx];
        let v0 = in_vertices[edge.v0].xyz;
        let v1 = in_vertices[edge.v1].xyz;
        if (edge.is_boundary != 0u) {
            out_vertices[idx] = vec4<f32>(0.5 * (v0 + v1), 1.0);
        } else {
            let w0 = in_vertices[edge.w0].xyz;
            let w1 = in_vertices[edge.w1].xyz;
            let gamma = edge.gamma;
            let out_v = v0 + (v1 - v0) * gamma + 0.125 * (w0 - v0) + 0.125 * (w1 - v1);
            out_vertices[idx] = vec4<f32>(out_v, 1.0);
        }
    }
}
"#;

const FEATURE_EDGES_SHADER: &str = r#"
struct FeatureEdgeUniforms {
    num_edges: u32,
    num_faces: u32,
    cos_crease_angle: f32,
    include_creases: u32,
    include_boundaries: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

struct GpuFeatureEdgeInput {
    v0: u32,
    v1: u32,
    f0: i32,
    f1: i32,
};

struct GpuEdgeSegment {
    p0: vec4<f32>,
    p1: vec4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: FeatureEdgeUniforms;
@group(0) @binding(1) var<storage, read> vertices: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> face_normals: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> edges: array<GpuFeatureEdgeInput>;
@group(0) @binding(4) var<storage, read_write> out_count: atomic<u32>;
@group(0) @binding(5) var<storage, read_write> out_segments: array<GpuEdgeSegment>;

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= uniforms.num_edges) {
        return;
    }

    let edge = edges[idx];
    var is_feature = false;

    if (edge.f1 < 0) {
        if (uniforms.include_boundaries != 0u) {
            is_feature = true;
        }
    } else if (uniforms.include_creases != 0u) {
        let f0 = u32(edge.f0);
        let f1 = u32(edge.f1);
        if (f0 < uniforms.num_faces && f1 < uniforms.num_faces) {
            let n0 = face_normals[f0].xyz;
            let n1 = face_normals[f1].xyz;
            let d = dot(n0, n1);
            if (d < uniforms.cos_crease_angle) {
                is_feature = true;
            }
        }
    }

    if (is_feature) {
        let slot = atomicAdd(&out_count, 1u);
        let p0 = vertices[edge.v0];
        let p1 = vertices[edge.v1];
        out_segments[slot] = GpuEdgeSegment(p0, p1);
    }
}
"#;

const QEM_SHADER: &str = r#"
struct QemUniforms {
    num_edges: u32,
    num_vertices: u32,
    _pad0: u32,
    _pad1: u32,
};

struct GpuQuadric {
    q0: f32, q1: f32, q2: f32, q3: f32,
    q4: f32, q5: f32, q6: f32, q7: f32,
    q8: f32, q9: f32, _pad0: f32, _pad1: f32,
};

struct GpuEdge {
    v0: u32,
    v1: u32,
};

@group(0) @binding(0) var<uniform> uniforms: QemUniforms;
@group(0) @binding(1) var<storage, read> vertices: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> vertex_quadrics: array<GpuQuadric>;
@group(0) @binding(3) var<storage, read> edges: array<GpuEdge>;
@group(0) @binding(4) var<storage, read_write> out_costs: array<f32>;
@group(0) @binding(5) var<storage, read_write> out_targets: array<vec4<f32>>;

fn eval_quadric_cost(q: GpuQuadric, p: vec3<f32>) -> f32 {
    let cost = q.q0 * p.x * p.x + 2.0 * q.q1 * p.x * p.y + 2.0 * q.q2 * p.x * p.z + 2.0 * q.q3 * p.x
             + q.q4 * p.y * p.y + 2.0 * q.q5 * p.y * p.z + 2.0 * q.q6 * p.y
             + q.q7 * p.z * p.z + 2.0 * q.q8 * p.z + q.q9;
    return max(0.0, cost);
}

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let e = global_id.x;
    if (e >= uniforms.num_edges) {
        return;
    }

    let edge = edges[e];
    let v0 = edge.v0;
    let v1 = edge.v1;
    if (v0 >= uniforms.num_vertices || v1 >= uniforms.num_vertices) {
        return;
    }

    let p0 = vertices[v0].xyz;
    let p1 = vertices[v1].xyz;

    let q0 = vertex_quadrics[v0];
    let q1 = vertex_quadrics[v1];

    var Q: GpuQuadric;
    Q.q0 = q0.q0 + q1.q0;
    Q.q1 = q0.q1 + q1.q1;
    Q.q2 = q0.q2 + q1.q2;
    Q.q3 = q0.q3 + q1.q3;
    Q.q4 = q0.q4 + q1.q4;
    Q.q5 = q0.q5 + q1.q5;
    Q.q6 = q0.q6 + q1.q6;
    Q.q7 = q0.q7 + q1.q7;
    Q.q8 = q0.q8 + q1.q8;
    Q.q9 = q0.q9 + q1.q9;

    let pm = 0.5 * (p0 + p1);

    let c0 = eval_quadric_cost(Q, p0);
    let c1 = eval_quadric_cost(Q, p1);
    let cm = eval_quadric_cost(Q, pm);

    var best_cost = cm;
    var best_target = pm;

    if (c0 < best_cost) {
        best_cost = c0;
        best_target = p0;
    }
    if (c1 < best_cost) {
        best_cost = c1;
        best_target = p1;
    }

    let C00 = Q.q4 * Q.q7 - Q.q5 * Q.q5;
    let C01 = Q.q2 * Q.q5 - Q.q1 * Q.q7;
    let C02 = Q.q1 * Q.q5 - Q.q2 * Q.q4;
    let C11 = Q.q0 * Q.q7 - Q.q2 * Q.q2;
    let C12 = Q.q1 * Q.q2 - Q.q0 * Q.q5;
    let C22 = Q.q0 * Q.q4 - Q.q1 * Q.q1;
    let det = Q.q0 * C00 + Q.q1 * C01 + Q.q2 * C02;

    if (abs(det) > 1.0e-7) {
        let inv_det = 1.0 / det;
        let opt_x = -(C00 * Q.q3 + C01 * Q.q6 + C02 * Q.q8) * inv_det;
        let opt_y = -(C01 * Q.q3 + C11 * Q.q6 + C12 * Q.q8) * inv_det;
        let opt_z = -(C02 * Q.q3 + C12 * Q.q6 + C22 * Q.q8) * inv_det;
        let opt_pt = vec3<f32>(opt_x, opt_y, opt_z);
        let c_opt = eval_quadric_cost(Q, opt_pt);
        if (c_opt <= best_cost) {
            best_cost = c_opt;
            best_target = opt_pt;
        }
    }

    out_costs[e] = best_cost;
    out_targets[e] = vec4<f32>(best_target, 1.0);
}
"#;

const POINT_IN_2D_LOOPS_SHADER: &str = r#"
struct PointIn2dLoopsUniforms {
    num_points: u32,
    num_segments: u32,
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(0) var<uniform> uniforms: PointIn2dLoopsUniforms;
@group(0) @binding(1) var<storage, read> query_points: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read> segments: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> out_inside_flags: array<u32>;

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= uniforms.num_points) {
        return;
    }

    let p = query_points[idx];
    var crossings = 0u;

    for (var k = 0u; k < uniforms.num_segments; k = k + 1u) {
        let seg = segments[k];
        let x0 = seg.x;
        let y0 = seg.y;
        let x1 = seg.z;
        let y1 = seg.w;

        if ((y0 > p.y) != (y1 > p.y)) {
            let x_int = x0 + (p.y - y0) * (seg.z - seg.x) / (y1 - y0);
            if (p.x < x_int) {
                crossings = crossings + 1u;
            }
        }
    }

    out_inside_flags[idx] = crossings % 2u;
}
"#;

const SEGMENT_MESH_DIST_SHADER: &str = r#"
struct SegmentMeshDistUniforms {
    num_segments: u32,
    num_triangles: u32,
    _pad0: u32,
    _pad1: u32,
};

struct GpuSegment {
    p0: vec4<f32>,
    p1: vec4<f32>,
};

struct GpuTriangle {
    v0: vec4<f32>,
    v1: vec4<f32>,
    v2: vec4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: SegmentMeshDistUniforms;
@group(0) @binding(1) var<storage, read> segments: array<GpuSegment>;
@group(0) @binding(2) var<storage, read> triangles: array<GpuTriangle>;
@group(0) @binding(3) var<storage, read_write> out_distances: array<f32>;

fn distance2_pt_tri(point: vec3<f32>, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> f32 {
    let ab = v1 - v0;
    let ap = point - v0;
    let bc = v2 - v1;
    let bp = point - v1;
    let ca = v0 - v2;
    let cp = point - v2;
    let nor = cross(ab, ca);

    let s0 = sign(dot(cross(ab, nor), ap));
    let s1 = sign(dot(cross(bc, nor), bp));
    let s2 = sign(dot(cross(ca, nor), cp));
    let coef = s0 + s1 + s2;
    let nor_len2 = dot(nor, nor);

    if (coef < 2.0 || nor_len2 < 1.0e-12) {
        let dot_ab = dot(ab, ab);
        let t_a = clamp(dot(ap, ab) / max(dot_ab, 1.0e-12), 0.0, 1.0);
        let da = ap - ab * t_a;
        let d2_a = dot(da, da);

        let dot_bc = dot(bc, bc);
        let t_b = clamp(dot(bp, bc) / max(dot_bc, 1.0e-12), 0.0, 1.0);
        let db = bp - bc * t_b;
        let d2_b = dot(db, db);

        let dot_ca = dot(ca, ca);
        let t_c = clamp(dot(cp, ca) / max(dot_ca, 1.0e-12), 0.0, 1.0);
        let dc = cp - ca * t_c;
        let d2_c = dot(dc, dc);

        return min(min(d2_a, d2_b), d2_c);
    } else {
        let d = dot(nor, ap);
        return (d * d) / nor_len2;
    }
}

fn distance2_seg_seg(p0: vec3<f32>, p1: vec3<f32>, q0: vec3<f32>, q1: vec3<f32>) -> f32 {
    let u = p1 - p0;
    let v = q1 - q0;
    let w0 = p0 - q0;
    let a = dot(u, u);
    let b = dot(u, v);
    let c = dot(v, v);
    let d = dot(u, w0);
    let e = dot(v, w0);
    let D = a * c - b * b;
    var sN = 0.0;
    var sD = D;
    var tN = 0.0;
    var tD = D;

    if (D < 1.0e-8) {
        sN = 0.0;
        sD = 1.0;
        tN = e;
        tD = c;
    } else {
        sN = b * e - c * d;
        tN = a * e - b * d;
        if (sN < 0.0) {
            sN = 0.0;
            tN = e;
            tD = c;
        } else if (sN > sD) {
            sN = sD;
            tN = e + b;
            tD = c;
        }
    }

    if (tN <= 0.0) {
        tN = 0.0;
        if (-d < 0.0) {
            sN = 0.0;
        } else if (-d > a) {
            sN = sD;
        } else {
            sN = -d;
            sD = a;
        }
    } else if (tN >= tD) {
        tN = tD;
        if ((-d + b) < 0.0) {
            sN = 0.0;
        } else if ((-d + b) > a) {
            sN = sD;
        } else {
            sN = -d + b;
            sD = a;
        }
    }

    let sc = select(sN / sD, 0.0, abs(sN) < 1.0e-8);
    let tc = select(tN / tD, 0.0, abs(tN) < 1.0e-8);
    let dP = (p0 + u * sc) - (q0 + v * tc);
    return dot(dP, dP);
}

fn segment_intersects_triangle(p0: vec3<f32>, p1: vec3<f32>, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> bool {
    let ab = v1 - v0;
    let ca = v0 - v2;
    let nor = cross(ab, ca);
    let nor_len2 = dot(nor, nor);
    if (nor_len2 < 1.0e-12) {
        return false;
    }

    let d0 = dot(nor, p0 - v0);
    let d1 = dot(nor, p1 - v0);
    if ((d0 > 1.0e-6 && d1 > 1.0e-6) || (d0 < -1.0e-6 && d1 < -1.0e-6)) {
        return false;
    }
    let denom = d0 - d1;
    if (abs(denom) < 1.0e-8) {
        return false;
    }
    let t = d0 / denom;
    if (t < 0.0 || t > 1.0) {
        return false;
    }

    let pint = p0 + (p1 - p0) * t;
    let bc = v2 - v1;
    let s0 = dot(cross(ab, nor), pint - v0);
    let s1 = dot(cross(bc, nor), pint - v1);
    let s2 = dot(cross(ca, nor), pint - v2);

    let eps = -1.0e-5 * sqrt(nor_len2);
    return (s0 >= eps && s1 >= eps && s2 >= eps) || (s0 <= -eps && s1 <= -eps && s2 <= -eps);
}

fn distance2_seg_tri(p0: vec3<f32>, p1: vec3<f32>, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> f32 {
    if (segment_intersects_triangle(p0, p1, v0, v1, v2)) {
        return 0.0;
    }
    var d2 = distance2_pt_tri(p0, v0, v1, v2);
    d2 = min(d2, distance2_pt_tri(p1, v0, v1, v2));
    d2 = min(d2, distance2_seg_seg(p0, p1, v0, v1));
    d2 = min(d2, distance2_seg_seg(p0, p1, v1, v2));
    d2 = min(d2, distance2_seg_seg(p0, p1, v2, v0));
    return d2;
}

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= uniforms.num_segments) {
        return;
    }

    let seg = segments[idx];
    let p0 = seg.p0.xyz;
    let p1 = seg.p1.xyz;

    var min_d2 = 1.0e30;
    for (var i = 0u; i < uniforms.num_triangles; i = i + 1u) {
        let tri = triangles[i];
        let v0 = tri.v0.xyz;
        let v1 = tri.v1.xyz;
        let v2 = tri.v2.xyz;

        let d2 = distance2_seg_tri(p0, p1, v0, v1, v2);
        min_d2 = min(min_d2, d2);
    }

    out_distances[idx] = sqrt(min_d2);
}
"#;

const WELD_ATTRIBUTES_SHADER: &str = r#"
struct WeldUniforms {
    num_attrs: u32,
    dim: u32,
    tol: f32,
    table_mask: u32,
    min_x: f32,
    min_y: f32,
    min_z: f32,
    inv_tol: f32,
};

@group(0) @binding(0) var<uniform> uniforms: WeldUniforms;
@group(0) @binding(1) var<storage, read> attrs: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> grid_head: array<atomic<u32>>;
@group(0) @binding(3) var<storage, read_write> grid_next: array<u32>;
@group(0) @binding(4) var<storage, read_write> labels: array<atomic<u32>>;

@compute @workgroup_size(64, 1, 1)
fn build_hash(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    if (i >= uniforms.num_attrs) {
        return;
    }
    atomicStore(&labels[i], i);
    let p = attrs[i];
    let cx = i32(floor((p.x - uniforms.min_x) * uniforms.inv_tol));
    let cy = i32(floor((p.y - uniforms.min_y) * uniforms.inv_tol));
    var cz = 0i;
    if (uniforms.dim == 3u) {
        cz = i32(floor((p.z - uniforms.min_z) * uniforms.inv_tol));
    }
    let nh = ((u32(cx) * 73856093u) ^ (u32(cy) * 19349663u) ^ (u32(cz) * 83492791u)) & uniforms.table_mask;
    let old_head = atomicExchange(&grid_head[nh], i);
    grid_next[i] = old_head;
}

@compute @workgroup_size(64, 1, 1)
fn propagate_labels(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    if (i >= uniforms.num_attrs) {
        return;
    }
    let p = attrs[i];
    let cx = i32(floor((p.x - uniforms.min_x) * uniforms.inv_tol));
    let cy = i32(floor((p.y - uniforms.min_y) * uniforms.inv_tol));
    var cz = 0i;
    if (uniforms.dim == 3u) {
        cz = i32(floor((p.z - uniforms.min_z) * uniforms.inv_tol));
    }
    let tol2 = uniforms.tol * uniforms.tol;
    var min_label = atomicLoad(&labels[i]);

    let dz_max = select(0i, 1i, uniforms.dim == 3u);
    let dz_min = select(0i, -1i, uniforms.dim == 3u);

    for (var dz = dz_min; dz <= dz_max; dz = dz + 1i) {
        for (var dy = -1i; dy <= 1i; dy = dy + 1i) {
            for (var dx = -1i; dx <= 1i; dx = dx + 1i) {
                let ncx = cx + dx;
                let ncy = cy + dy;
                let ncz = cz + dz;
                let nh = ((u32(ncx) * 73856093u) ^ (u32(ncy) * 19349663u) ^ (u32(ncz) * 83492791u)) & uniforms.table_mask;
                var curr = atomicLoad(&grid_head[nh]);
                var steps = 0u;
                while (curr != 0xFFFFFFFFu && steps < 1024u) {
                    steps = steps + 1u;
                    let j = curr;
                    curr = grid_next[j];
                    if (j != i) {
                        let q = attrs[j];
                        let diff = p.xyz - q.xyz;
                        let d2 = dot(diff, diff);
                        if (d2 < tol2) {
                            let lj = atomicLoad(&labels[j]);
                            if (lj < min_label) {
                                min_label = lj;
                            }
                        }
                    }
                }
            }
        }
    }
    atomicMin(&labels[i], min_label);

    let parent = atomicLoad(&labels[i]);
    let root = atomicLoad(&labels[parent]);
    if (root < parent) {
        atomicMin(&labels[i], root);
    }
}
"#;

fn qem_layout_entries() -> [BindGroupLayoutEntry; 6] {
    [
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 3,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 4,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 5,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]
}

fn point_in_2d_loops_layout_entries() -> [BindGroupLayoutEntry; 4] {
    [
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 3,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]
}

fn segment_mesh_dist_layout_entries() -> [BindGroupLayoutEntry; 4] {
    [
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 3,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]
}

fn weld_layout_entries() -> [BindGroupLayoutEntry; 5] {
    [
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 3,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 4,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]
}

fn loop_subdivision_layout_entries() -> [BindGroupLayoutEntry; 6] {
    [
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 3,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 4,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 5,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]
}

fn feature_edges_layout_entries() -> [BindGroupLayoutEntry; 6] {
    [
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 3,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 4,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 5,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]
}

fn nurbs_curve_layout_entries() -> [BindGroupLayoutEntry; 6] {
    [
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 3,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 4,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 5,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]
}

fn raycast_mesh_layout_entries() -> [BindGroupLayoutEntry; 8] {
    [
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 3,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 4,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 5,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 6,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 7,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]
}

fn nurbs_layout_entries() -> [BindGroupLayoutEntry; 7] {
    [
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 3,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 4,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 5,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 6,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]
}

fn extract_planes_layout_entries() -> [BindGroupLayoutEntry; 7] {
    [
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 3,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 4,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 5,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 6,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]
}

fn distance_layout_entries() -> [BindGroupLayoutEntry; 4] {
    [
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 3,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]
}

fn in_out_layout_entries() -> [BindGroupLayoutEntry; 5] {
    [
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 3,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 4,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]
}

fn face_normals_layout_entries() -> [BindGroupLayoutEntry; 3] {
    [
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]
}

fn collision_layout_entries() -> [BindGroupLayoutEntry; 5] {
    [
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 3,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 4,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]
}

/// 32-byte standard vertex format for zero-copy direct GPU tessellation matching `truck-rendimpl::AttrVertex`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod, PartialEq)]
pub struct DirectAttrVertex {
    /// 3D position
    pub position: [f32; 3],
    /// 2D UV coordinate
    pub uv_coord: [f32; 2],
    /// 3D normal vector
    pub normal: [f32; 3],
}

const NURBS_PACK_VERTICES_SHADER: &str = r#"
@group(0) @binding(0) var<storage, read> in_pts: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> in_uv: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read> in_nors: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> out_vertex_buf: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    if (idx >= arrayLength(&in_pts)) {
        return;
    }
    let base = idx * 8u;
    let p = in_pts[idx];
    let uv = in_uv[idx];
    let n = in_nors[idx];
    out_vertex_buf[base + 0u] = p.x;
    out_vertex_buf[base + 1u] = p.y;
    out_vertex_buf[base + 2u] = p.z;
    out_vertex_buf[base + 3u] = uv.x;
    out_vertex_buf[base + 4u] = uv.y;
    out_vertex_buf[base + 5u] = n.x;
    out_vertex_buf[base + 6u] = n.y;
    out_vertex_buf[base + 7u] = n.z;
}
"#;

const GRID_INDEX_GEN_SHADER: &str = r#"
struct GridDims {
    u_div: u32,
    v_div: u32,
    num_quads: u32,
    _pad: u32,
};

@group(0) @binding(0) var<uniform> dims: GridDims;
@group(0) @binding(1) var<storage, read_write> out_indices: array<u32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let qidx = id.x;
    if (qidx >= dims.num_quads) {
        return;
    }
    let v_cells = dims.v_div - 1u;
    let i = qidx / v_cells;
    let j = qidx % v_cells;
    let v_div = dims.v_div;

    let v00 = i * v_div + j;
    let v01 = i * v_div + (j + 1u);
    let v10 = (i + 1u) * v_div + j;
    let v11 = (i + 1u) * v_div + (j + 1u);

    let base = qidx * 6u;
    out_indices[base + 0u] = v00;
    out_indices[base + 1u] = v10;
    out_indices[base + 2u] = v01;
    out_indices[base + 3u] = v01;
    out_indices[base + 4u] = v10;
    out_indices[base + 5u] = v11;
}
"#;

fn pack_vertices_layout_entries() -> [BindGroupLayoutEntry; 4] {
    [
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 3,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]
}

fn grid_index_layout_entries() -> [BindGroupLayoutEntry; 2] {
    [
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]
}

/// Metadata describing a Vulkan GPU device.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct VulkanDeviceInfo {
    /// Device adapter name (e.g. "NVIDIA GeForce RTX 3080").
    pub name: String,
    /// Backend API name (e.g. "Vulkan").
    pub backend: String,
    /// Driver name.
    pub driver: String,
    /// Driver information string.
    pub driver_info: String,
}

/// Initializes a Vulkan device handler if a suitable Vulkan GPU is found.
pub fn try_init_vulkan_device() -> Option<DeviceHandler> {
    pollster::block_on(async {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::VULKAN,
            flags: InstanceFlags::from_build_config(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .ok()?;
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("truck_meshalgo_vulkan_device"),
                ..Default::default()
            })
            .await
            .ok()?;
        Some(DeviceHandler::new(adapter, device, queue))
    })
}

/// GPU-accelerated mesh analyzer for distance queries, in-out containment, normals, trimming, and collisions.
#[derive(Debug)]
pub struct GpuMeshAnalyzer {
    handler: DeviceHandler,
    dist_engine: ComputeEngine,
    in_out_engine: ComputeEngine,
    face_normals_engine: ComputeEngine,
    trimming_loop_engine: ComputeEngine,
    collide_triangles_engine: ComputeEngine,
    tri_point_dist_engine: ComputeEngine,
    ssi_seeds_engine: ComputeEngine,
    slicing_engine: ComputeEngine,
    volume_cog_engine: ComputeEngine,
    extract_planes_engine: ComputeEngine,
    subdivide_engine: ComputeEngine,
    nurbs_surface_engine: ComputeEngine,
    nurbs_curve_engine: ComputeEngine,
    raycast_mesh_engine: ComputeEngine,
    loop_subdivision_engine: ComputeEngine,
    feature_edges_engine: ComputeEngine,
    qem_engine: ComputeEngine,
    point_in_2d_loops_engine: ComputeEngine,
    segment_mesh_dist_engine: ComputeEngine,
    weld_build_engine: ComputeEngine,
    weld_propagate_engine: ComputeEngine,
    pack_vertices_engine: ComputeEngine,
    grid_index_engine: ComputeEngine,
}

impl GpuMeshAnalyzer {
    /// Attempts to create a new GPU mesh analyzer using the default Vulkan device.
    pub fn try_create_default() -> Option<Self> {
        let handler = try_init_vulkan_device()?;
        Some(Self::new(&handler))
    }

    /// Queries the Vulkan device information for this analyzer.
    pub fn device_info(&self) -> VulkanDeviceInfo {
        let info = self.handler.adapter().get_info();
        VulkanDeviceInfo {
            name: info.name,
            backend: format!("{:?}", info.backend),
            driver: info.driver,
            driver_info: info.driver_info,
        }
    }

    /// Creates a new GPU mesh analyzer.
    pub fn new(handler: &DeviceHandler) -> Self {
        let dist_entries = distance_layout_entries();
        let dist_engine = ComputeEngine::new(handler, DISTANCE_SHADER, "main", &dist_entries);

        let in_out_entries = in_out_layout_entries();
        let in_out_engine = ComputeEngine::new(handler, IN_OUT_SHADER, "main", &in_out_entries);

        let fn_entries = face_normals_layout_entries();
        let face_normals_engine = ComputeEngine::new(handler, FACE_NORMALS_SHADER, "main", &fn_entries);

        let trim_entries = in_out_layout_entries();
        let trimming_loop_engine = ComputeEngine::new(handler, TRIMMING_LOOP_SHADER, "main", &trim_entries);

        let col_entries = collision_layout_entries();
        let collide_triangles_engine = ComputeEngine::new(handler, COLLIDE_TRIANGLES_SHADER, "main", &col_entries);

        let tri_point_entries = distance_layout_entries();
        let tri_point_dist_engine = ComputeEngine::new(handler, TRIANGLE_POINT_DISTANCE_SHADER, "main", &tri_point_entries);

        let ssi_entries = collision_layout_entries();
        let ssi_seeds_engine = ComputeEngine::new(handler, SSI_SEEDS_SHADER, "main", &ssi_entries);

        let slice_entries = collision_layout_entries();
        let slicing_engine = ComputeEngine::new(handler, SLICING_SHADER, "main", &slice_entries);

        let vol_entries = face_normals_layout_entries();
        let volume_cog_engine = ComputeEngine::new(handler, VOLUME_COG_SHADER, "main", &vol_entries);

        let ep_entries = extract_planes_layout_entries();
        let extract_planes_engine = ComputeEngine::new(handler, EXTRACT_PLANES_SHADER, "main", &ep_entries);

        let sub_entries = face_normals_layout_entries();
        let subdivide_engine = ComputeEngine::new(handler, SUBDIVIDE_SHADER, "main", &sub_entries);

        let nurbs_entries = nurbs_layout_entries();
        let nurbs_surface_engine = ComputeEngine::new(handler, NURBS_SURFACE_SHADER, "main", &nurbs_entries);

        let nurbs_curve_entries = nurbs_curve_layout_entries();
        let nurbs_curve_engine = ComputeEngine::new(handler, NURBS_CURVE_SHADER, "main", &nurbs_curve_entries);

        let raycast_entries = raycast_mesh_layout_entries();
        let raycast_mesh_engine = ComputeEngine::new(handler, RAYCAST_MESH_SHADER, "main", &raycast_entries);

        let loop_entries = loop_subdivision_layout_entries();
        let loop_subdivision_engine = ComputeEngine::new(handler, LOOP_SUBDIVISION_SHADER, "main", &loop_entries);

        let fe_entries = feature_edges_layout_entries();
        let feature_edges_engine = ComputeEngine::new(handler, FEATURE_EDGES_SHADER, "main", &fe_entries);

        let qem_entries = qem_layout_entries();
        let qem_engine = ComputeEngine::new(handler, QEM_SHADER, "main", &qem_entries);

        let p2d_entries = point_in_2d_loops_layout_entries();
        let point_in_2d_loops_engine = ComputeEngine::new(handler, POINT_IN_2D_LOOPS_SHADER, "main", &p2d_entries);

        let smd_entries = segment_mesh_dist_layout_entries();
        let segment_mesh_dist_engine = ComputeEngine::new(handler, SEGMENT_MESH_DIST_SHADER, "main", &smd_entries);

        let weld_entries = weld_layout_entries();
        let weld_build_engine = ComputeEngine::new(handler, WELD_ATTRIBUTES_SHADER, "build_hash", &weld_entries);
        let weld_propagate_engine = ComputeEngine::new(handler, WELD_ATTRIBUTES_SHADER, "propagate_labels", &weld_entries);

        let pack_entries = pack_vertices_layout_entries();
        let pack_vertices_engine = ComputeEngine::new(handler, NURBS_PACK_VERTICES_SHADER, "main", &pack_entries);

        let grid_idx_entries = grid_index_layout_entries();
        let grid_index_engine = ComputeEngine::new(handler, GRID_INDEX_GEN_SHADER, "main", &grid_idx_entries);

        Self {
            handler: handler.clone(),
            dist_engine,
            in_out_engine,
            face_normals_engine,
            trimming_loop_engine,
            collide_triangles_engine,
            tri_point_dist_engine,
            ssi_seeds_engine,
            slicing_engine,
            volume_cog_engine,
            extract_planes_engine,
            subdivide_engine,
            nurbs_surface_engine,
            nurbs_curve_engine,
            raycast_mesh_engine,
            loop_subdivision_engine,
            feature_edges_engine,
            qem_engine,
            point_in_2d_loops_engine,
            segment_mesh_dist_engine,
            weld_build_engine,
            weld_propagate_engine,
            pack_vertices_engine,
            grid_index_engine,
        }
    }

    /// Extracts triangles from a polygon mesh into GPU triangle format.
    pub fn extract_triangles(mesh: &PolygonMesh) -> Vec<GpuTriangle> {
        let positions = mesh.positions();
        mesh.face_iter()
            .flat_map(|face| {
                (2..face.len()).map(move |i| {
                    let p0 = positions[face[0].pos];
                    let p1 = positions[face[i - 1].pos];
                    let p2 = positions[face[i].pos];
                    GpuTriangle::new(p0, p1, p2)
                })
            })
            .collect()
    }

    /// Computes the minimum squared distance from each query point to the polygon mesh on the GPU.
    pub fn distance2_point_cloud(&self, mesh: &PolygonMesh, points: &[Point3]) -> Vec<f32> {
        if points.is_empty() {
            return Vec::new();
        }
        let triangles = Self::extract_triangles(mesh);
        if triangles.is_empty() {
            return vec![f32::INFINITY; points.len()];
        }

        let num_points = points.len() as u32;
        let num_triangles = triangles.len() as u32;

        let uniforms = ComputeUniforms {
            num_points,
            num_triangles,
            _pad: [0, 0],
        };
        let uniform_buf = BufferHandler::from_slice(
            &[uniforms],
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let pt_vec: Vec<[f32; 4]> = points
            .iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32, 0.0])
            .collect();
        let pt_buf = BufferHandler::storage_from_slice(&pt_vec, self.handler.device());
        let tri_buf = BufferHandler::storage_from_slice(&triangles, self.handler.device());
        let out_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_points as usize * size_of::<f32>()) as u64,
        );

        let bind_group = self.dist_engine.create_bind_group([
            uniform_buf.binding_resource(),
            pt_buf.binding_resource(),
            tri_buf.binding_resource(),
            out_buf.binding_resource(),
        ]);

        let workgroups = ((num_points + 63) / 64, 1, 1);
        self.dist_engine.dispatch(&bind_group, workgroups);

        out_buf.read_to_vec_block(&self.handler)
    }

    /// Computes signed crossing face counts for specified rays on the GPU.
    pub fn signed_crossing_rays(
        &self,
        mesh: &PolygonMesh,
        points: &[Point3],
        ray_directions: &[Vector3],
    ) -> Vec<i32> {
        assert_eq!(points.len(), ray_directions.len());
        if points.is_empty() {
            return Vec::new();
        }
        let triangles = Self::extract_triangles(mesh);
        if triangles.is_empty() {
            return vec![0; points.len()];
        }

        let num_points = points.len() as u32;
        let num_triangles = triangles.len() as u32;

        let uniforms = ComputeUniforms {
            num_points,
            num_triangles,
            _pad: [0, 0],
        };
        let uniform_buf = BufferHandler::from_slice(
            &[uniforms],
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let pt_vec: Vec<[f32; 4]> = points
            .iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32, 0.0])
            .collect();
        let dir_vec: Vec<[f32; 4]> = ray_directions
            .iter()
            .map(|d| [d.x as f32, d.y as f32, d.z as f32, 0.0])
            .collect();

        let pt_buf = BufferHandler::storage_from_slice(&pt_vec, self.handler.device());
        let dir_buf = BufferHandler::storage_from_slice(&dir_vec, self.handler.device());
        let tri_buf = BufferHandler::storage_from_slice(&triangles, self.handler.device());
        let out_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_points as usize * size_of::<i32>()) as u64,
        );

        let bind_group = self.in_out_engine.create_bind_group([
            uniform_buf.binding_resource(),
            pt_buf.binding_resource(),
            dir_buf.binding_resource(),
            tri_buf.binding_resource(),
            out_buf.binding_resource(),
        ]);

        let workgroups = ((num_points + 63) / 64, 1, 1);
        self.in_out_engine.dispatch(&bind_group, workgroups);

        out_buf.read_to_vec_block(&self.handler)
    }

    /// Computes signed crossing face counts for rays starting at each query point on the GPU along a common direction.
    pub fn signed_crossing_faces(
        &self,
        mesh: &PolygonMesh,
        points: &[Point3],
        ray_direction: Vector3,
    ) -> Vec<i32> {
        let dirs = vec![ray_direction; points.len()];
        self.signed_crossing_rays(mesh, points, &dirs)
    }

    /// Determines whether each point in `points` is inside the domain bounded by the closed mesh on the GPU.
    /// Uses `truck_base::hash::take_one_unit` for each point to match CPU parity.
    pub fn inside(&self, mesh: &PolygonMesh, points: &[Point3]) -> Vec<bool> {
        let dirs: Vec<Vector3> = points.iter().map(|&p| hash::take_one_unit(p)).collect();
        let counts = self.signed_crossing_rays(mesh, points, &dirs);
        counts.into_iter().map(|c| c >= 1).collect()
    }

    /// Computes normalized face normals for all triangular faces in the mesh on the GPU via Vulkan.
    pub fn compute_face_normals(&self, mesh: &PolygonMesh) -> Vec<Vector3> {
        let triangles = Self::extract_triangles(mesh);
        if triangles.is_empty() {
            return Vec::new();
        }

        let num_triangles = triangles.len() as u32;
        let uniforms = [num_triangles, 0, 0, 0];
        let uniform_buf = BufferHandler::from_slice(
            &uniforms,
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let tri_buf = BufferHandler::storage_from_slice(&triangles, self.handler.device());
        let out_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_triangles as usize * size_of::<[f32; 4]>()) as u64,
        );

        let bind_group = self.face_normals_engine.create_bind_group([
            uniform_buf.binding_resource(),
            tri_buf.binding_resource(),
            out_buf.binding_resource(),
        ]);

        let workgroups = ((num_triangles + 63) / 64, 1, 1);
        self.face_normals_engine.dispatch(&bind_group, workgroups);

        let raw: Vec<[f32; 4]> = out_buf.read_to_vec_block(&self.handler);
        raw.into_iter()
            .map(|n| Vector3::new(n[0] as f64, n[1] as f64, n[2] as f64))
            .collect()
    }

    /// Tests whether 2D query points are included within a 2D boundary represented by segments on the GPU via Vulkan.
    pub fn trimming_loop_include(
        &self,
        segments: &[[Point2; 2]],
        points: &[Point2],
    ) -> Vec<bool> {
        if points.is_empty() {
            return Vec::new();
        }
        if segments.is_empty() {
            return vec![false; points.len()];
        }

        let num_segments = segments.len() as u32;
        let num_points = points.len() as u32;

        let uniforms = [num_segments, num_points, 0, 0];
        let uniform_buf = BufferHandler::from_slice(
            &uniforms,
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let seg_vec: Vec<[f32; 4]> = segments
            .iter()
            .map(|seg| [seg[0].x as f32, seg[0].y as f32, seg[1].x as f32, seg[1].y as f32])
            .collect();
        let pt_vec: Vec<[f32; 2]> = points
            .iter()
            .map(|p| [p.x as f32, p.y as f32])
            .collect();
        let ray_vec: Vec<[f32; 2]> = points
            .iter()
            .map(|&c| {
                let t = 2.0 * std::f64::consts::PI * HashGen::hash1(c);
                [t.cos() as f32, t.sin() as f32]
            })
            .collect();

        let seg_buf = BufferHandler::storage_from_slice(&seg_vec, self.handler.device());
        let pt_buf = BufferHandler::storage_from_slice(&pt_vec, self.handler.device());
        let ray_buf = BufferHandler::storage_from_slice(&ray_vec, self.handler.device());
        let out_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_points as usize * size_of::<i32>()) as u64,
        );

        let bind_group = self.trimming_loop_engine.create_bind_group([
            uniform_buf.binding_resource(),
            seg_buf.binding_resource(),
            pt_buf.binding_resource(),
            ray_buf.binding_resource(),
            out_buf.binding_resource(),
        ]);

        let workgroups = ((num_points + 63) / 64, 1, 1);
        self.trimming_loop_engine.dispatch(&bind_group, workgroups);

        let flags: Vec<i32> = out_buf.read_to_vec_block(&self.handler);
        flags.into_iter().map(|f| f > 0).collect()
    }

    /// Extracts interference line segments between two polygon meshes on the GPU via Vulkan.
    pub fn collide_triangles(
        &self,
        mesh0: &PolygonMesh,
        mesh1: &PolygonMesh,
    ) -> Vec<(Point3, Point3)> {
        let tri0 = Self::extract_triangles(mesh0);
        let tri1 = Self::extract_triangles(mesh1);
        if tri0.is_empty() || tri1.is_empty() {
            return Vec::new();
        }

        let num_tri0 = tri0.len() as u32;
        let num_tri1 = tri1.len() as u32;
        let max_segments = 65536u32;

        let uniforms = [num_tri0, num_tri1, max_segments, 0];
        let uniform_buf = BufferHandler::from_slice(
            &uniforms,
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let tri0_buf = BufferHandler::storage_from_slice(&tri0, self.handler.device());
        let tri1_buf = BufferHandler::storage_from_slice(&tri1, self.handler.device());
        let count_buf = BufferHandler::from_slice(
            &[0u32],
            self.handler.device(),
            BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        );
        let out_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (max_segments as usize * 2 * size_of::<[f32; 4]>()) as u64,
        );

        let bind_group = self.collide_triangles_engine.create_bind_group([
            uniform_buf.binding_resource(),
            tri0_buf.binding_resource(),
            tri1_buf.binding_resource(),
            count_buf.binding_resource(),
            out_buf.binding_resource(),
        ]);

        let workgroups = ((num_tri0 + 63) / 64, 1, 1);
        self.collide_triangles_engine.dispatch(&bind_group, workgroups);

        let count_vec: Vec<u32> = count_buf.read_to_vec_block(&self.handler);
        let count = count_vec[0].min(max_segments) as usize;
        if count == 0 {
            return Vec::new();
        }

        let segs: Vec<[f32; 4]> = out_buf.read_to_vec_block(&self.handler);
        (0..count)
            .map(|i| {
                let p = Point3::new(segs[i * 2][0] as f64, segs[i * 2][1] as f64, segs[i * 2][2] as f64);
                let q = Point3::new(segs[i * 2 + 1][0] as f64, segs[i * 2 + 1][1] as f64, segs[i * 2 + 1][2] as f64);
                (p, q)
            })
            .collect()
    }

    /// Tests if two polygon meshes collide on the GPU via Vulkan.
    pub fn collide_with(
        &self,
        mesh0: &PolygonMesh,
        mesh1: &PolygonMesh,
    ) -> Option<(Point3, Point3)> {
        let lines = self.collide_triangles(mesh0, mesh1);
        lines.into_iter().next()
    }

    /// Checks whether all faces of the polygon mesh intersect the neighborhood of `point_cloud` on the GPU.
    pub fn is_clung_to_by(&self, mesh: &PolygonMesh, point_cloud: &[Point3], tol: f64) -> bool {
        if point_cloud.is_empty() {
            return false;
        }
        let triangles = Self::extract_triangles(mesh);
        if triangles.is_empty() {
            return false;
        }

        let num_triangles = triangles.len() as u32;
        let num_points = point_cloud.len() as u32;

        let uniforms = TriPointUniforms {
            num_triangles,
            num_points,
            _pad: [0, 0],
        };
        let uniform_buf = BufferHandler::from_slice(
            &[uniforms],
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let pt_vec: Vec<[f32; 4]> = point_cloud
            .iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32, 0.0])
            .collect();
        let tri_buf = BufferHandler::storage_from_slice(&triangles, self.handler.device());
        let pt_buf = BufferHandler::storage_from_slice(&pt_vec, self.handler.device());
        let out_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_triangles as usize * size_of::<f32>()) as u64,
        );

        let bind_group = self.tri_point_dist_engine.create_bind_group([
            uniform_buf.binding_resource(),
            tri_buf.binding_resource(),
            pt_buf.binding_resource(),
            out_buf.binding_resource(),
        ]);

        let workgroups = ((num_triangles + 63) / 64, 1, 1);
        self.tri_point_dist_engine.dispatch(&bind_group, workgroups);

        let min_d2: Vec<f32> = out_buf.read_to_vec_block(&self.handler);
        let tol2 = (tol * tol) as f32;
        min_d2.into_iter().all(|d| d < tol2 + 1e-5)
    }

    /// Checks whether the neighborhood of the polygon mesh includes all points in `point_cloud` on the GPU.
    pub fn neighborhood_include(&self, mesh: &PolygonMesh, point_cloud: &[Point3], tol: f64) -> bool {
        if point_cloud.is_empty() {
            return true;
        }
        let triangles = Self::extract_triangles(mesh);
        if triangles.is_empty() {
            return false;
        }

        let min_d2 = self.distance2_point_cloud(mesh, point_cloud);
        let tol2 = (tol * tol) as f32;
        min_d2.into_iter().all(|d| d < tol2 + 1e-5)
    }

    /// Evaluates dense parameter grid separation on the GPU to locate candidate intersection seeds between two B-spline surfaces.
    pub fn find_surface_intersection_seeds(
        &self,
        s1: &BSplineSurface<Point3>,
        s2: &BSplineSurface<Point3>,
        tol: f64,
    ) -> Vec<(Point2, Point2)> {
        self.find_surface_intersection_seeds_with_res(s1, s2, (32, 32), (32, 32), tol)
    }

    /// Evaluates dense parameter grid separation on the GPU with custom grid resolutions.
    pub fn find_surface_intersection_seeds_with_res(
        &self,
        s1: &BSplineSurface<Point3>,
        s2: &BSplineSurface<Point3>,
        res1: (usize, usize),
        res2: (usize, usize),
        tol: f64,
    ) -> Vec<(Point2, Point2)> {
        let (samples1, n_u1, n_v1) = sample_surface_grid(s1, res1);
        let (samples2, n_u2, n_v2) = sample_surface_grid(s2, res2);

        const MAX_SEEDS: u32 = 8192;
        let uniforms = SsiUniforms {
            n_u1: n_u1 as u32,
            n_v1: n_v1 as u32,
            n_u2: n_u2 as u32,
            n_v2: n_v2 as u32,
            tol: tol as f32,
            max_seeds: MAX_SEEDS,
            _pad0: 0,
            _pad1: 0,
        };

        let uniform_buf = BufferHandler::from_slice(
            &[uniforms],
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let s1_buf = BufferHandler::storage_from_slice(&samples1, self.handler.device());
        let s2_buf = BufferHandler::storage_from_slice(&samples2, self.handler.device());
        let count_buf = BufferHandler::from_slice(
            &[0u32],
            self.handler.device(),
            BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        );
        let out_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (MAX_SEEDS as usize * size_of::<[f32; 4]>()) as u64,
        );

        let bind_group = self.ssi_seeds_engine.create_bind_group([
            uniform_buf.binding_resource(),
            s1_buf.binding_resource(),
            s2_buf.binding_resource(),
            count_buf.binding_resource(),
            out_buf.binding_resource(),
        ]);

        let total_tasks = (n_u1 * n_v1) * ((n_u2 - 1) * (n_v2 - 1));
        let workgroups = (((total_tasks as u32) + 63) / 64, 1, 1);
        self.ssi_seeds_engine.dispatch(&bind_group, workgroups);

        let counts: Vec<u32> = count_buf.read_to_vec_block(&self.handler);
        let valid_count = (counts.first().copied().unwrap_or(0) as usize).min(MAX_SEEDS as usize);

        let out_seeds: Vec<[f32; 4]> = out_buf.read_to_vec_block(&self.handler);
        let raw_seeds: Vec<(Point2, Point2)> = out_seeds[..valid_count]
            .iter()
            .map(|s| (Point2::new(s[0] as f64, s[1] as f64), Point2::new(s[2] as f64, s[3] as f64)))
            .collect();

        filter_and_cluster_seeds(raw_seeds, s1, s2)
    }

    /// Slices a polygon mesh against multiple planes on the GPU via Vulkan compute.
    pub fn slice_mesh_planes(&self, mesh: &PolygonMesh, planes: &[[f64; 4]]) -> Vec<Vec<(Point3, Point3)>> {
        if planes.is_empty() {
            return Vec::new();
        }
        let tri = Self::extract_triangles(mesh);
        if tri.is_empty() {
            return vec![Vec::new(); planes.len()];
        }

        let num_tri = tri.len() as u32;
        let num_planes = planes.len() as u32;
        let max_segments_per_plane = num_tri.min(65536);

        let uniforms = [num_tri, num_planes, max_segments_per_plane, 0];
        let uniform_buf = BufferHandler::from_slice(
            &uniforms,
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let planes_f32: Vec<[f32; 4]> = planes
            .iter()
            .map(|p| [p[0] as f32, p[1] as f32, p[2] as f32, p[3] as f32])
            .collect();

        let tri_buf = BufferHandler::storage_from_slice(&tri, self.handler.device());
        let planes_buf = BufferHandler::storage_from_slice(&planes_f32, self.handler.device());
        let count_buf = BufferHandler::from_slice(
            &vec![0u32; planes.len()],
            self.handler.device(),
            BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        );
        let total_segments = (num_planes as usize) * (max_segments_per_plane as usize);
        let out_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (total_segments * 2 * size_of::<[f32; 4]>()) as u64,
        );

        let bind_group = self.slicing_engine.create_bind_group([
            uniform_buf.binding_resource(),
            tri_buf.binding_resource(),
            planes_buf.binding_resource(),
            count_buf.binding_resource(),
            out_buf.binding_resource(),
        ]);

        let workgroups = ((num_tri + 63) / 64, num_planes, 1);
        self.slicing_engine.dispatch(&bind_group, workgroups);

        let counts: Vec<u32> = count_buf.read_to_vec_block(&self.handler);
        let segs: Vec<[f32; 4]> = out_buf.read_to_vec_block(&self.handler);

        (0..planes.len())
            .map(|k| {
                let count = (counts[k] as usize).min(max_segments_per_plane as usize);
                let base = k * (max_segments_per_plane as usize) * 2;
                (0..count)
                    .map(|i| {
                        let seg_base = base + i * 2;
                        let p1 = Point3::new(
                            segs[seg_base][0] as f64,
                            segs[seg_base][1] as f64,
                            segs[seg_base][2] as f64,
                        );
                        let p2 = Point3::new(
                            segs[seg_base + 1][0] as f64,
                            segs[seg_base + 1][1] as f64,
                            segs[seg_base + 1][2] as f64,
                        );
                        (p1, p2)
                    })
                    .collect()
            })
            .collect()
    }

    /// Evaluates divergence-theorem prism volume and center of gravity on the GPU via Vulkan compute.
    pub fn mass_properties(&self, mesh: &PolygonMesh) -> (f64, Point3) {
        let tri = Self::extract_triangles(mesh);
        if tri.is_empty() {
            return (0.0, Point3::origin());
        }

        let num_tri = tri.len() as u32;
        let num_workgroups = ((num_tri + 63) / 64).min(256);

        let uniforms = [num_tri, num_workgroups, 0, 0];
        let uniform_buf = BufferHandler::from_slice(
            &uniforms,
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let tri_buf = BufferHandler::storage_from_slice(&tri, self.handler.device());
        let partial_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_workgroups as usize * size_of::<[f32; 4]>()) as u64,
        );

        let bind_group = self.volume_cog_engine.create_bind_group([
            uniform_buf.binding_resource(),
            tri_buf.binding_resource(),
            partial_buf.binding_resource(),
        ]);

        let workgroups = (num_workgroups, 1, 1);
        self.volume_cog_engine.dispatch(&bind_group, workgroups);

        let partial_sums: Vec<[f32; 4]> = partial_buf.read_to_vec_block(&self.handler);

        let mut total_cx = 0.0f64;
        let mut total_cy = 0.0f64;
        let mut total_cz = 0.0f64;
        let mut total_vol = 0.0f64;

        for p in partial_sums {
            total_cx += p[0] as f64;
            total_cy += p[1] as f64;
            total_cz += p[2] as f64;
            total_vol += p[3] as f64;
        }

        let cog = if total_vol.abs() > 1e-12 {
            Point3::new(total_cx / total_vol, total_cy / total_vol, total_cz / total_vol)
        } else {
            Point3::origin()
        };

        (total_vol, cog)
    }

    /// Evaluates the divergence-theorem prism volume of a mesh on the GPU via Vulkan compute.
    #[inline(always)]
    pub fn volume(&self, mesh: &PolygonMesh) -> f64 {
        self.mass_properties(mesh).0
    }

    /// Evaluates the center of gravity of a mesh on the GPU via Vulkan compute.
    #[inline(always)]
    pub fn center_of_gravity(&self, mesh: &PolygonMesh) -> Point3 {
        self.mass_properties(mesh).1
    }

    /// Extracts polygons such that their face normal matches their vertex normal within `tol`.
    pub fn extract_planes(&self, mesh: &PolygonMesh, tol: f64) -> (Vec<usize>, Vec<usize>) {
        if mesh.faces().is_empty() {
            return (Vec::new(), Vec::new());
        }
        let (offsets, face_pos, face_nor) = prepare_extract_planes_buffers_f32(mesh);
        let num_faces = mesh.faces().len() as u32;

        let uniform_data = ExtractPlanesUniforms {
            num_faces,
            tol: tol as f32,
            _pad0: 0,
            _pad1: 0,
        };
        let uniform_buf = BufferHandler::from_slice(
            &[uniform_data],
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        let offset_buf = BufferHandler::storage_from_slice(&offsets, self.handler.device());
        let pos_buf = BufferHandler::storage_from_slice(&face_pos, self.handler.device());
        let nor_buf = BufferHandler::storage_from_slice(&face_nor, self.handler.device());
        let count_buf = BufferHandler::from_slice(
            &[0u32, 0u32],
            self.handler.device(),
            BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        );
        let planar_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_faces as usize * size_of::<u32>()) as u64,
        );
        let other_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_faces as usize * size_of::<u32>()) as u64,
        );

        let bind_group = self.extract_planes_engine.create_bind_group([
            uniform_buf.binding_resource(),
            offset_buf.binding_resource(),
            pos_buf.binding_resource(),
            nor_buf.binding_resource(),
            count_buf.binding_resource(),
            planar_buf.binding_resource(),
            other_buf.binding_resource(),
        ]);

        let workgroups = ((num_faces + 63) / 64, 1, 1);
        self.extract_planes_engine.dispatch(&bind_group, workgroups);

        let counts: Vec<u32> = count_buf.read_to_vec_block(&self.handler);
        let p_count = counts[0].min(num_faces) as usize;
        let o_count = counts[1].min(num_faces) as usize;

        let p_indices: Vec<u32> = planar_buf.read_to_vec_block(&self.handler);
        let o_indices: Vec<u32> = other_buf.read_to_vec_block(&self.handler);

        let mut planes: Vec<usize> = p_indices[..p_count].iter().map(|&i| i as usize).collect();
        let mut others: Vec<usize> = o_indices[..o_count].iter().map(|&i| i as usize).collect();

        planes.sort_unstable();
        others.sort_unstable();

        (planes, others)
    }

    /// Subdivides triangles into 4-to-1 midpoint-refined triangles on the GPU via Vulkan compute.
    pub fn subdivide_triangles(&self, triangles: &[GpuTriangle]) -> Vec<GpuTriangle> {
        if triangles.is_empty() {
            return Vec::new();
        }
        let num_triangles = triangles.len() as u32;
        let uniforms = SubdivideUniforms {
            num_triangles,
            _pad: [0, 0, 0],
        };
        let uniform_buf = BufferHandler::from_slice(
            &[uniforms],
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        let in_buf = BufferHandler::storage_from_slice(&triangles.to_vec(), self.handler.device());
        let out_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_triangles as usize * 4 * size_of::<GpuTriangle>()) as u64,
        );

        let bind_group = self.subdivide_engine.create_bind_group([
            uniform_buf.binding_resource(),
            in_buf.binding_resource(),
            out_buf.binding_resource(),
        ]);

        let workgroups = ((num_triangles + 63) / 64, 1, 1);
        self.subdivide_engine.dispatch(&bind_group, workgroups);

        out_buf.read_to_vec_block(&self.handler)
    }

    /// Generates a 4-to-1 linearly subdivided mesh on the GPU via Vulkan compute.
    pub fn linear_subdivide(&self, mesh: &PolygonMesh) -> PolygonMesh {
        let triangles = Self::extract_triangles(mesh);
        if triangles.is_empty() {
            return mesh.clone();
        }
        let sub_tris = self.subdivide_triangles(&triangles);
        create_mesh_from_triangles(&sub_tris)
    }

    /// Evaluates a rational NURBS surface on a batch of `(u, v)` coordinates on the GPU via Vulkan compute.
    pub fn evaluate_nurbs_surface(
        &self,
        u_knots: &[f64],
        v_knots: &[f64],
        u_degree: usize,
        v_degree: usize,
        control_points: &[Vec<Vector4>],
        uv_points: &[Point2],
    ) -> (Vec<Point3>, Vec<Vector3>) {
        if uv_points.is_empty() || control_points.is_empty() || control_points[0].is_empty() {
            return (Vec::new(), Vec::new());
        }

        let num_u_knots = u_knots.len() as u32;
        let num_v_knots = v_knots.len() as u32;
        let num_points = uv_points.len() as u32;
        let cp_stride_v = control_points[0].len() as u32;

        let uniforms = NurbsUniforms {
            num_u_knots,
            num_v_knots,
            u_degree: u_degree as u32,
            v_degree: v_degree as u32,
            cp_stride_v,
            num_points,
            _pad: [0, 0],
        };
        let uniform_buf = BufferHandler::from_slice(
            &[uniforms],
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let u_knots_f32: Vec<f32> = u_knots.iter().map(|&x| x as f32).collect();
        let v_knots_f32: Vec<f32> = v_knots.iter().map(|&x| x as f32).collect();

        let mut flat_cps = Vec::with_capacity(control_points.len() * control_points[0].len());
        for row in control_points {
            for pt in row {
                flat_cps.push([pt.x as f32, pt.y as f32, pt.z as f32, pt.w as f32]);
            }
        }

        let uv_f32: Vec<[f32; 2]> = uv_points.iter().map(|p| [p.x as f32, p.y as f32]).collect();

        let u_knots_buf = BufferHandler::storage_from_slice(&u_knots_f32, self.handler.device());
        let v_knots_buf = BufferHandler::storage_from_slice(&v_knots_f32, self.handler.device());
        let cps_buf = BufferHandler::storage_from_slice(&flat_cps, self.handler.device());
        let uv_buf = BufferHandler::storage_from_slice(&uv_f32, self.handler.device());

        let out_pts_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_points as usize * size_of::<[f32; 4]>()) as u64,
        );
        let out_nors_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_points as usize * size_of::<[f32; 4]>()) as u64,
        );

        let bind_group = self.nurbs_surface_engine.create_bind_group([
            uniform_buf.binding_resource(),
            u_knots_buf.binding_resource(),
            v_knots_buf.binding_resource(),
            cps_buf.binding_resource(),
            uv_buf.binding_resource(),
            out_pts_buf.binding_resource(),
            out_nors_buf.binding_resource(),
        ]);

        let workgroups = ((num_points + 63) / 64, 1, 1);
        self.nurbs_surface_engine.dispatch(&bind_group, workgroups);

        let pts_f32: Vec<[f32; 4]> = out_pts_buf.read_to_vec_block(&self.handler);
        let nors_f32: Vec<[f32; 4]> = out_nors_buf.read_to_vec_block(&self.handler);

        let pts = pts_f32
            .into_iter()
            .map(|p| Point3::new(p[0] as f64, p[1] as f64, p[2] as f64))
            .collect();
        let nors = nors_f32
            .into_iter()
            .map(|n| Vector3::new(n[0] as f64, n[1] as f64, n[2] as f64))
            .collect();

        (pts, nors)
    }

    /// Evaluates a rational NURBS surface directly into GPU vertex and index buffers suitable for zero-copy rendering.
    ///
    /// The vertex buffer has layout `[DirectAttrVertex]` (pos: [f32; 3], uv: [f32; 2], nor: [f32; 3]) with `BufferUsages::VERTEX`,
    /// and the index buffer contains `u32` triangle indices with `BufferUsages::INDEX`.
    /// Zero vertex or index bytes are transferred back to host CPU memory.
    #[allow(clippy::too_many_arguments)]
    pub fn tessellate_nurbs_surface_direct(
        &self,
        u_knots: &[f64],
        v_knots: &[f64],
        u_degree: usize,
        v_degree: usize,
        control_points: &[Vec<Vector4>],
        u_div: usize,
        v_div: usize,
    ) -> (Arc<BufferHandler>, Arc<BufferHandler>, usize) {
        if u_div < 2 || v_div < 2 || control_points.is_empty() || control_points[0].is_empty() {
            let vb = Arc::new(BufferHandler::storage_empty(self.handler.device(), 0));
            let ib = Arc::new(BufferHandler::storage_empty(self.handler.device(), 0));
            return (vb, ib, 0);
        }

        let u_min = u_knots[u_degree];
        let u_max = u_knots[u_knots.len() - 1 - u_degree];
        let v_min = v_knots[v_degree];
        let v_max = v_knots[v_knots.len() - 1 - v_degree];

        let num_points = (u_div * v_div) as u32;
        let mut uv_f32 = Vec::with_capacity(num_points as usize);
        for i in 0..u_div {
            let u = u_min + (u_max - u_min) * (i as f64) / ((u_div - 1) as f64);
            for j in 0..v_div {
                let v = v_min + (v_max - v_min) * (j as f64) / ((v_div - 1) as f64);
                uv_f32.push([u as f32, v as f32]);
            }
        }

        let num_u_knots = u_knots.len() as u32;
        let num_v_knots = v_knots.len() as u32;
        let cp_stride_v = control_points[0].len() as u32;

        let uniforms = NurbsUniforms {
            num_u_knots,
            num_v_knots,
            u_degree: u_degree as u32,
            v_degree: v_degree as u32,
            cp_stride_v,
            num_points,
            _pad: [0, 0],
        };
        let uniform_buf = BufferHandler::from_slice(
            &[uniforms],
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let u_knots_f32: Vec<f32> = u_knots.iter().map(|&x| x as f32).collect();
        let v_knots_f32: Vec<f32> = v_knots.iter().map(|&x| x as f32).collect();

        let mut flat_cps = Vec::with_capacity(control_points.len() * control_points[0].len());
        for row in control_points {
            for pt in row {
                flat_cps.push([pt.x as f32, pt.y as f32, pt.z as f32, pt.w as f32]);
            }
        }

        let u_knots_buf = BufferHandler::storage_from_slice(&u_knots_f32, self.handler.device());
        let v_knots_buf = BufferHandler::storage_from_slice(&v_knots_f32, self.handler.device());
        let cps_buf = BufferHandler::storage_from_slice(&flat_cps, self.handler.device());
        let uv_buf = BufferHandler::storage_from_slice(&uv_f32, self.handler.device());

        let out_pts_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_points as usize * size_of::<[f32; 4]>()) as u64,
        );
        let out_nors_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_points as usize * size_of::<[f32; 4]>()) as u64,
        );

        let bind_group = self.nurbs_surface_engine.create_bind_group([
            uniform_buf.binding_resource(),
            u_knots_buf.binding_resource(),
            v_knots_buf.binding_resource(),
            cps_buf.binding_resource(),
            uv_buf.binding_resource(),
            out_pts_buf.binding_resource(),
            out_nors_buf.binding_resource(),
        ]);

        let workgroups = ((num_points + 63) / 64, 1, 1);
        self.nurbs_surface_engine.dispatch(&bind_group, workgroups);

        // Direct packing into AttrVertex buffer entirely on GPU
        let vertex_buf_size = (num_points as usize * size_of::<DirectAttrVertex>()) as u64;
        let vertex_raw_buffer = self.handler.device().create_buffer(&BufferDescriptor {
            label: Some("direct_vertex_buffer"),
            size: vertex_buf_size,
            usage: BufferUsages::VERTEX | BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let pack_bind_group = self.pack_vertices_engine.create_bind_group([
            out_pts_buf.binding_resource(),
            uv_buf.binding_resource(),
            out_nors_buf.binding_resource(),
            BindingResource::Buffer(BufferBinding {
                buffer: &vertex_raw_buffer,
                offset: 0,
                size: None,
            }),
        ]);
        self.pack_vertices_engine.dispatch(&pack_bind_group, workgroups);

        // Generate triangle indices on GPU
        let num_quads = ((u_div - 1) * (v_div - 1)) as u32;
        let index_count = (num_quads * 6) as usize;
        let index_buf_size = (index_count * size_of::<u32>()) as u64;

        #[repr(C)]
        #[derive(Clone, Copy, Pod, Zeroable)]
        struct GridDims {
            u_div: u32,
            v_div: u32,
            num_quads: u32,
            _pad: u32,
        }
        let dims_uniform = BufferHandler::from_slice(
            &[GridDims {
                u_div: u_div as u32,
                v_div: v_div as u32,
                num_quads,
                _pad: 0,
            }],
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let index_raw_buffer = self.handler.device().create_buffer(&BufferDescriptor {
            label: Some("direct_index_buffer"),
            size: index_buf_size,
            usage: BufferUsages::INDEX | BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let idx_bind_group = self.grid_index_engine.create_bind_group([
            dims_uniform.binding_resource(),
            BindingResource::Buffer(BufferBinding {
                buffer: &index_raw_buffer,
                offset: 0,
                size: None,
            }),
        ]);
        let idx_workgroups = ((num_quads + 63) / 64, 1, 1);
        self.grid_index_engine.dispatch(&idx_bind_group, idx_workgroups);

        let vertex_handler = Arc::new(BufferHandler::from_buffer(
            vertex_raw_buffer,
            vertex_buf_size,
            size_of::<DirectAttrVertex>() as u64,
        ));
        let index_handler = Arc::new(BufferHandler::from_buffer(
            index_raw_buffer,
            index_buf_size,
            size_of::<u32>() as u64,
        ));

        (vertex_handler, index_handler, index_count)
    }

    /// Evaluates a rational NURBS curve on a batch of parameters on the GPU via Vulkan compute.
    ///
    /// Returns a tuple of `(evaluated_3d_points, evaluated_derivatives)`.
    pub fn evaluate_nurbs_curve(
        &self,
        knots: &[f64],
        degree: usize,
        control_points: &[Vector4],
        t_points: &[f64],
    ) -> (Vec<Point3>, Vec<Vector3>) {
        if t_points.is_empty() || control_points.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let num_knots = knots.len() as u32;
        let num_control_points = control_points.len() as u32;
        let num_points = t_points.len() as u32;

        let uniforms = NurbsCurveUniforms {
            num_knots,
            degree: degree as u32,
            num_control_points,
            num_points,
        };
        let uniform_buf = BufferHandler::from_slice(
            &[uniforms],
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let knots_f32: Vec<f32> = knots.iter().map(|&x| x as f32).collect();
        let cps_f32: Vec<[f32; 4]> = control_points
            .iter()
            .map(|pt| [pt.x as f32, pt.y as f32, pt.z as f32, pt.w as f32])
            .collect();
        let t_f32: Vec<f32> = t_points.iter().map(|&x| x as f32).collect();

        let knots_buf = BufferHandler::storage_from_slice(&knots_f32, self.handler.device());
        let cps_buf = BufferHandler::storage_from_slice(&cps_f32, self.handler.device());
        let t_buf = BufferHandler::storage_from_slice(&t_f32, self.handler.device());

        let out_pts_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_points as usize * size_of::<[f32; 4]>()) as u64,
        );
        let out_ders_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_points as usize * size_of::<[f32; 4]>()) as u64,
        );

        let bind_group = self.nurbs_curve_engine.create_bind_group([
            uniform_buf.binding_resource(),
            knots_buf.binding_resource(),
            cps_buf.binding_resource(),
            t_buf.binding_resource(),
            out_pts_buf.binding_resource(),
            out_ders_buf.binding_resource(),
        ]);

        let workgroups = ((num_points + 63) / 64, 1, 1);
        self.nurbs_curve_engine.dispatch(&bind_group, workgroups);

        let pts_f32: Vec<[f32; 4]> = out_pts_buf.read_to_vec_block(&self.handler);
        let ders_f32: Vec<[f32; 4]> = out_ders_buf.read_to_vec_block(&self.handler);

        let pts = pts_f32
            .into_iter()
            .map(|p| Point3::new(p[0] as f64, p[1] as f64, p[2] as f64))
            .collect();
        let ders = ders_f32
            .into_iter()
            .map(|d| Vector3::new(d[0] as f64, d[1] as f64, d[2] as f64))
            .collect();

        (pts, ders)
    }

    /// Raycasts an array of rays against a polygon mesh on the GPU via Vulkan compute.
    pub fn raycast_mesh(
        &self,
        mesh: &PolygonMesh,
        rays: &[(Point3, Vector3)],
    ) -> Vec<Option<(f64, Point3, Vector3)>> {
        if rays.is_empty() {
            return Vec::new();
        }
        let triangles = Self::extract_triangles(mesh);
        self.raycast_triangles(&triangles, rays)
    }

    /// Alias for [`Self::raycast_mesh`].
    #[inline(always)]
    pub fn raycast(
        &self,
        mesh: &PolygonMesh,
        rays: &[(Point3, Vector3)],
    ) -> Vec<Option<(f64, Point3, Vector3)>> {
        self.raycast_mesh(mesh, rays)
    }

    /// Raycasts an array of rays against a slice of GPU triangles on the GPU via Vulkan compute.
    pub fn raycast_triangles(
        &self,
        triangles: &[GpuTriangle],
        rays: &[(Point3, Vector3)],
    ) -> Vec<Option<(f64, Point3, Vector3)>> {
        if rays.is_empty() {
            return Vec::new();
        }

        let num_rays = rays.len() as u32;
        let num_triangles = triangles.len() as u32;

        let uniforms = RaycastUniforms {
            num_rays,
            num_triangles,
            _pad: [0, 0],
        };
        let uniform_buf = BufferHandler::from_slice(
            &[uniforms],
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let ray_origins: Vec<[f32; 4]> = rays
            .iter()
            .map(|(o, _)| [o.x as f32, o.y as f32, o.z as f32, 1.0f32])
            .collect();
        let ray_dirs: Vec<[f32; 4]> = rays
            .iter()
            .map(|(_, d)| {
                let norm = d.normalize();
                [norm.x as f32, norm.y as f32, norm.z as f32, 0.0f32]
            })
            .collect();

        let origins_buf = BufferHandler::storage_from_slice(&ray_origins, self.handler.device());
        let dirs_buf = BufferHandler::storage_from_slice(&ray_dirs, self.handler.device());
        let tri_buf = BufferHandler::storage_from_slice(&triangles.to_vec(), self.handler.device());

        let out_dists_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_rays as usize * size_of::<f32>()) as u64,
        );
        let out_tris_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_rays as usize * size_of::<i32>()) as u64,
        );
        let out_hits_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_rays as usize * size_of::<[f32; 4]>()) as u64,
        );
        let out_nors_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_rays as usize * size_of::<[f32; 4]>()) as u64,
        );

        let bind_group = self.raycast_mesh_engine.create_bind_group([
            uniform_buf.binding_resource(),
            origins_buf.binding_resource(),
            dirs_buf.binding_resource(),
            tri_buf.binding_resource(),
            out_dists_buf.binding_resource(),
            out_tris_buf.binding_resource(),
            out_hits_buf.binding_resource(),
            out_nors_buf.binding_resource(),
        ]);

        let workgroups = ((num_rays + 63) / 64, 1, 1);
        self.raycast_mesh_engine.dispatch(&bind_group, workgroups);

        let dists_f32: Vec<f32> = out_dists_buf.read_to_vec_block(&self.handler);
        let hits_f32: Vec<[f32; 4]> = out_hits_buf.read_to_vec_block(&self.handler);
        let nors_f32: Vec<[f32; 4]> = out_nors_buf.read_to_vec_block(&self.handler);

        dists_f32
            .into_iter()
            .zip(hits_f32)
            .zip(nors_f32)
            .map(|((dist, hit_pt), norm)| {
                if dist >= 0.0 {
                    Some((
                        dist as f64,
                        Point3::new(hit_pt[0] as f64, hit_pt[1] as f64, hit_pt[2] as f64),
                        Vector3::new(norm[0] as f64, norm[1] as f64, norm[2] as f64),
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Smooth Loop subdivision surface refinement on the GPU via Vulkan compute.
    pub fn loop_subdivide(&self, mesh: &PolygonMesh) -> PolygonMesh {
        let tri_faces = mesh.tri_faces();
        if tri_faces.is_empty() {
            return mesh.clone();
        }
        let topo = prepare_loop_subdivision_topology(mesh);
        let num_vertices = mesh.positions().len() as u32;
        let num_edges = topo.edges_f32.len() as u32;

        let in_pos_f32: Vec<[f32; 4]> = mesh
            .positions()
            .iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32, 1.0])
            .collect();

        let uniforms = LoopSubdivisionUniforms {
            num_vertices,
            num_edges,
            _pad0: 0,
            _pad1: 0,
        };

        let uniform_buf = BufferHandler::from_slice(
            &[uniforms],
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        let in_vertices_buf = BufferHandler::storage_from_slice(&in_pos_f32, self.handler.device());
        let vertex_info_buf = BufferHandler::storage_from_slice(&topo.vertex_info, self.handler.device());
        let vertex_neighbors_buf = BufferHandler::storage_from_slice(&topo.vertex_neighbors, self.handler.device());
        let edges_buf = BufferHandler::storage_from_slice(&topo.edges_f32, self.handler.device());
        let out_vertices_buf = BufferHandler::storage_empty(
            self.handler.device(),
            ((num_vertices + num_edges) as usize * size_of::<[f32; 4]>()) as u64,
        );

        let bind_group = self.loop_subdivision_engine.create_bind_group([
            uniform_buf.binding_resource(),
            in_vertices_buf.binding_resource(),
            vertex_info_buf.binding_resource(),
            vertex_neighbors_buf.binding_resource(),
            edges_buf.binding_resource(),
            out_vertices_buf.binding_resource(),
        ]);

        let total_work = num_vertices + num_edges;
        let workgroups = ((total_work + 63) / 64, 1, 1);
        self.loop_subdivision_engine.dispatch(&bind_group, workgroups);

        let out_f32: Vec<[f32; 4]> = out_vertices_buf.read_to_vec_block(&self.handler);
        let positions: Vec<Point3> = out_f32
            .into_iter()
            .map(|p| Point3::new(p[0] as f64, p[1] as f64, p[2] as f64))
            .collect();

        let mut out_mesh = mesh.clone();
        {
            let editor = out_mesh.debug_editor();
            editor.attributes.positions = positions;
            *editor.faces = Faces::from_tri_and_quad_faces(topo.new_tri_faces, Vec::new());
        }
        out_mesh
    }

    /// Alias for [`Self::loop_subdivide`].
    #[inline(always)]
    pub fn loop_subdivision(&self, mesh: &PolygonMesh) -> PolygonMesh {
        self.loop_subdivide(mesh)
    }

    /// Extracts crease edges and open boundary feature edges as 3D line segments via Vulkan compute.
    pub fn extract_feature_edges(&self, mesh: &PolygonMesh, crease_angle_rad: f64) -> Vec<(Point3, Point3)> {
        let tri_faces = mesh.tri_faces();
        if tri_faces.is_empty() {
            return Vec::new();
        }
        let topo = prepare_feature_edges_topology(mesh);
        if topo.edges.is_empty() {
            return Vec::new();
        }
        let num_edges = topo.edges.len() as u32;
        let num_faces = topo.face_normals_f32.len() as u32;

        let vert_f32: Vec<[f32; 4]> = mesh
            .positions()
            .iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32, 1.0])
            .collect();

        let uniforms = FeatureEdgeUniforms {
            num_edges,
            num_faces,
            cos_crease_angle: crease_angle_rad.cos() as f32,
            include_creases: 1,
            include_boundaries: 1,
            _pad0: 0,
            _pad1: 0,
            _pad2: 0,
        };

        let uniform_buf = BufferHandler::from_slice(
            &[uniforms],
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        let vert_buf = BufferHandler::storage_from_slice(&vert_f32, self.handler.device());
        let norm_buf = BufferHandler::storage_from_slice(&topo.face_normals_f32, self.handler.device());
        let edge_buf = BufferHandler::storage_from_slice(&topo.edges, self.handler.device());
        let count_buf = BufferHandler::from_slice(
            &[0u32],
            self.handler.device(),
            BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        );
        let seg_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_edges as usize * size_of::<GpuEdgeSegment>()) as u64,
        );

        let bind_group = self.feature_edges_engine.create_bind_group([
            uniform_buf.binding_resource(),
            vert_buf.binding_resource(),
            norm_buf.binding_resource(),
            edge_buf.binding_resource(),
            count_buf.binding_resource(),
            seg_buf.binding_resource(),
        ]);

        let workgroups = ((num_edges + 63) / 64, 1, 1);
        self.feature_edges_engine.dispatch(&bind_group, workgroups);

        let count = count_buf.read_to_vec_block::<u32>(&self.handler)[0] as usize;
        let segs: Vec<GpuEdgeSegment> = seg_buf.read_to_vec_block(&self.handler);
        let valid_segs = &segs[..count.min(num_edges as usize)];

        valid_segs
            .iter()
            .map(|s| {
                (
                    Point3::new(s.p0[0] as f64, s.p0[1] as f64, s.p0[2] as f64),
                    Point3::new(s.p1[0] as f64, s.p1[1] as f64, s.p1[2] as f64),
                )
            })
            .collect()
    }

    /// Extracts open boundary edges as 3D line segments via Vulkan compute.
    pub fn extract_boundary_edges(&self, mesh: &PolygonMesh) -> Vec<(Point3, Point3)> {
        let tri_faces = mesh.tri_faces();
        if tri_faces.is_empty() {
            return Vec::new();
        }
        let topo = prepare_feature_edges_topology(mesh);
        if topo.edges.is_empty() {
            return Vec::new();
        }
        let num_edges = topo.edges.len() as u32;
        let num_faces = topo.face_normals_f32.len() as u32;

        let vert_f32: Vec<[f32; 4]> = mesh
            .positions()
            .iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32, 1.0])
            .collect();

        let uniforms = FeatureEdgeUniforms {
            num_edges,
            num_faces,
            cos_crease_angle: 1.0,
            include_creases: 0,
            include_boundaries: 1,
            _pad0: 0,
            _pad1: 0,
            _pad2: 0,
        };

        let uniform_buf = BufferHandler::from_slice(
            &[uniforms],
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        let vert_buf = BufferHandler::storage_from_slice(&vert_f32, self.handler.device());
        let norm_buf = BufferHandler::storage_from_slice(&topo.face_normals_f32, self.handler.device());
        let edge_buf = BufferHandler::storage_from_slice(&topo.edges, self.handler.device());
        let count_buf = BufferHandler::from_slice(
            &[0u32],
            self.handler.device(),
            BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        );
        let seg_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_edges as usize * size_of::<GpuEdgeSegment>()) as u64,
        );

        let bind_group = self.feature_edges_engine.create_bind_group([
            uniform_buf.binding_resource(),
            vert_buf.binding_resource(),
            norm_buf.binding_resource(),
            edge_buf.binding_resource(),
            count_buf.binding_resource(),
            seg_buf.binding_resource(),
        ]);

        let workgroups = ((num_edges + 63) / 64, 1, 1);
        self.feature_edges_engine.dispatch(&bind_group, workgroups);

        let count = count_buf.read_to_vec_block::<u32>(&self.handler)[0] as usize;
        let segs: Vec<GpuEdgeSegment> = seg_buf.read_to_vec_block(&self.handler);
        let valid_segs = &segs[..count.min(num_edges as usize)];

        valid_segs
            .iter()
            .map(|s| {
                (
                    Point3::new(s.p0[0] as f64, s.p0[1] as f64, s.p0[2] as f64),
                    Point3::new(s.p1[0] as f64, s.p1[1] as f64, s.p1[2] as f64),
                )
            })
            .collect()
    }

    /// Evaluates Quadric Error Metric (Garland & Heckbert) edge collapse costs and optimal targets via Vulkan compute.
    pub fn compute_edge_collapse_costs(&self, mesh: &PolygonMesh) -> Vec<([usize; 2], f64, Point3)> {
        let triangulated;
        let mesh = if mesh.tri_faces().is_empty() && !mesh.faces().is_empty() {
            let mut m = mesh.clone();
            m.triangulate();
            triangulated = m;
            &triangulated
        } else {
            mesh
        };
        let tri_faces = mesh.tri_faces();
        if tri_faces.is_empty() {
            return Vec::new();
        }
        let mut edge_set = rustc_hash::FxHashSet::default();
        for tri in tri_faces {
            let v0 = tri[0].pos;
            let v1 = tri[1].pos;
            let v2 = tri[2].pos;
            edge_set.insert((v0.min(v1), v0.max(v1)));
            edge_set.insert((v1.min(v2), v1.max(v2)));
            edge_set.insert((v2.min(v0), v2.max(v0)));
        }
        let mut edges: Vec<[usize; 2]> = edge_set.into_iter().map(|(a, b)| [a, b]).collect();
        edges.sort_unstable();

        let num_vertices = mesh.positions().len() as u32;
        let num_edges = edges.len() as u32;

        let positions = mesh.positions();
        let mut vertex_quadrics = vec![[0.0f32; 12]; num_vertices as usize];
        for tri in tri_faces {
            let p0 = positions[tri[0].pos];
            let p1 = positions[tri[1].pos];
            let p2 = positions[tri[2].pos];
            let u = p1 - p0;
            let v = p2 - p0;
            let n = u.cross(v);
            let len = n.magnitude();
            let (a, b, c) = if len > 1e-12 {
                let inv = 1.0 / len;
                ((n.x * inv) as f32, (n.y * inv) as f32, (n.z * inv) as f32)
            } else {
                (0.0f32, 0.0f32, 1.0f32)
            };
            let d = -(a * p0.x as f32 + b * p0.y as f32 + c * p0.z as f32);
            let k = [
                a * a, a * b, a * c, a * d,
                b * b, b * c, b * d,
                c * c, c * d,
                d * d, 0.0f32, 0.0f32,
            ];
            for &idx in &[tri[0].pos, tri[1].pos, tri[2].pos] {
                for m in 0..10 {
                    vertex_quadrics[idx][m] += k[m];
                }
            }
        }

        let in_vertices: Vec<[f32; 4]> = positions
            .iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32, 1.0])
            .collect();

        let in_edges: Vec<[u32; 2]> = edges
            .iter()
            .map(|e| [e[0] as u32, e[1] as u32])
            .collect();

        let uniforms = QemUniforms {
            num_edges,
            num_vertices,
            _pad0: 0,
            _pad1: 0,
        };

        let uniform_buf = BufferHandler::from_slice(
            &[uniforms],
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        let vert_buf = BufferHandler::storage_from_slice(&in_vertices, self.handler.device());
        let q_buf = BufferHandler::storage_from_slice(&vertex_quadrics, self.handler.device());
        let edge_buf = BufferHandler::storage_from_slice(&in_edges, self.handler.device());
        let cost_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_edges as usize * size_of::<f32>()) as u64,
        );
        let target_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_edges as usize * size_of::<[f32; 4]>()) as u64,
        );

        let bind_group = self.qem_engine.create_bind_group([
            uniform_buf.binding_resource(),
            vert_buf.binding_resource(),
            q_buf.binding_resource(),
            edge_buf.binding_resource(),
            cost_buf.binding_resource(),
            target_buf.binding_resource(),
        ]);

        let workgroups = ((num_edges + 63) / 64, 1, 1);
        self.qem_engine.dispatch(&bind_group, workgroups);

        let out_costs: Vec<f32> = cost_buf.read_to_vec_block(&self.handler);
        let out_targets: Vec<[f32; 4]> = target_buf.read_to_vec_block(&self.handler);

        edges
            .into_iter()
            .zip(out_costs)
            .zip(out_targets)
            .map(|((edge, cost), tgt)| {
                (
                    edge,
                    cost as f64,
                    Point3::new(tgt[0] as f64, tgt[1] as f64, tgt[2] as f64),
                )
            })
            .collect()
    }

    /// Evaluates 2D point-in-polygon containment across nested loops (outer boundary + holes) via Vulkan compute.
    pub fn point_in_2d_loops(&self, points: &[Point2], loops: &[Vec<Point2>]) -> Vec<bool> {
        if points.is_empty() {
            return Vec::new();
        }
        let mut segments: Vec<[f32; 4]> = Vec::new();
        for lp in loops {
            let n = lp.len();
            if n < 3 {
                continue;
            }
            let is_closed = lp[0].distance2(lp[n - 1]) < 1e-14;
            let seg_count = if is_closed { n - 1 } else { n };
            for i in 0..seg_count {
                let p0 = lp[i];
                let p1 = lp[(i + 1) % n];
                segments.push([p0.x as f32, p0.y as f32, p1.x as f32, p1.y as f32]);
            }
        }

        let num_points = points.len() as u32;
        let num_segments = segments.len() as u32;

        let query_points: Vec<[f32; 2]> = points
            .iter()
            .map(|p| [p.x as f32, p.y as f32])
            .collect();

        let uniforms = PointIn2dLoopsUniforms {
            num_points,
            num_segments,
            _pad0: 0,
            _pad1: 0,
        };

        let uniform_buf = BufferHandler::from_slice(
            &[uniforms],
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        let pt_buf = BufferHandler::storage_from_slice(&query_points, self.handler.device());
        let seg_buf = BufferHandler::storage_from_slice(&segments, self.handler.device());
        let out_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_points as usize * size_of::<u32>()) as u64,
        );

        let bind_group = self.point_in_2d_loops_engine.create_bind_group([
            uniform_buf.binding_resource(),
            pt_buf.binding_resource(),
            seg_buf.binding_resource(),
            out_buf.binding_resource(),
        ]);

        let workgroups = ((num_points + 63) / 64, 1, 1);
        self.point_in_2d_loops_engine.dispatch(&bind_group, workgroups);

        let flags: Vec<u32> = out_buf.read_to_vec_block(&self.handler);
        flags.into_iter().map(|f| f != 0).collect()
    }

    /// Evaluates minimum clearance distance from 3D line segments to a workpiece mesh via Vulkan compute.
    pub fn segment_mesh_distances(&self, mesh: &PolygonMesh, segments: &[(Point3, Point3)]) -> Vec<f64> {
        if segments.is_empty() {
            return Vec::new();
        }
        let triangles = Self::extract_triangles(mesh);
        if triangles.is_empty() {
            return vec![f64::INFINITY; segments.len()];
        }
        let num_segments = segments.len() as u32;
        let num_triangles = triangles.len() as u32;

        let gpu_segments: Vec<GpuSegment> = segments
            .iter()
            .map(|(p0, p1)| GpuSegment {
                p0: [p0.x as f32, p0.y as f32, p0.z as f32, 1.0],
                p1: [p1.x as f32, p1.y as f32, p1.z as f32, 1.0],
            })
            .collect();

        let uniforms = SegmentMeshDistUniforms {
            num_segments,
            num_triangles,
            _pad0: 0,
            _pad1: 0,
        };

        let uniform_buf = BufferHandler::from_slice(
            &[uniforms],
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        let seg_buf = BufferHandler::storage_from_slice(&gpu_segments, self.handler.device());
        let tri_buf = BufferHandler::storage_from_slice(&triangles, self.handler.device());
        let dist_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_segments as usize * size_of::<f32>()) as u64,
        );

        let bind_group = self.segment_mesh_dist_engine.create_bind_group([
            uniform_buf.binding_resource(),
            seg_buf.binding_resource(),
            tri_buf.binding_resource(),
            dist_buf.binding_resource(),
        ]);

        let workgroups = ((num_segments + 63) / 64, 1, 1);
        self.segment_mesh_dist_engine.dispatch(&bind_group, workgroups);

        let distances: Vec<f32> = dist_buf.read_to_vec_block(&self.handler);
        distances.into_iter().map(|d| d as f64).collect()
    }

    /// Welds and deduplicates 3D attributes within tolerance `tol` via Vulkan compute.
    pub fn weld_attributes_3d(&self, attrs: &[Point3], tol: f64) -> Vec<usize> {
        if attrs.is_empty() {
            return Vec::new();
        }
        if attrs.len() == 1 || tol <= 0.0 {
            return (0..attrs.len()).collect();
        }
        let gpu_attrs: Vec<[f32; 4]> = attrs
            .iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32, 0.0])
            .collect();
        self.weld_attributes_gpu(&gpu_attrs, 3, tol)
    }

    /// Welds and deduplicates 2D attributes within tolerance `tol` via Vulkan compute.
    pub fn weld_attributes_2d(&self, attrs: &[Point2], tol: f64) -> Vec<usize> {
        if attrs.is_empty() {
            return Vec::new();
        }
        if attrs.len() == 1 || tol <= 0.0 {
            return (0..attrs.len()).collect();
        }
        let gpu_attrs: Vec<[f32; 4]> = attrs
            .iter()
            .map(|p| [p.x as f32, p.y as f32, 0.0, 0.0])
            .collect();
        self.weld_attributes_gpu(&gpu_attrs, 2, tol)
    }

    fn weld_attributes_gpu(&self, gpu_attrs: &[[f32; 4]], dim: u32, tol: f64) -> Vec<usize> {
        let num_attrs = gpu_attrs.len() as u32;
        let mut min_x = gpu_attrs[0][0];
        let mut min_y = gpu_attrs[0][1];
        let mut min_z = if dim == 3 { gpu_attrs[0][2] } else { 0.0 };
        for a in &gpu_attrs[1..] {
            if a[0] < min_x { min_x = a[0]; }
            if a[1] < min_y { min_y = a[1]; }
            if dim == 3 && a[2] < min_z { min_z = a[2]; }
        }

        let mut table_size = 64u32;
        while table_size < 2 * num_attrs && table_size < (1u32 << 26) {
            table_size <<= 1;
        }
        let table_mask = table_size - 1;

        let uniforms = WeldUniforms {
            num_attrs,
            dim,
            tol: tol as f32,
            table_mask,
            min_x,
            min_y,
            min_z,
            inv_tol: (1.0 / tol) as f32,
        };

        let uniform_buf = BufferHandler::from_slice(
            &[uniforms],
            self.handler.device(),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        let attr_buf = BufferHandler::storage_from_slice(&gpu_attrs, self.handler.device());
        let initial_heads = vec![0xFFFFFFFFu32; table_size as usize];
        let head_buf = BufferHandler::storage_from_slice(&initial_heads, self.handler.device());
        let next_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_attrs as usize * size_of::<u32>()) as u64,
        );
        let label_buf = BufferHandler::storage_empty(
            self.handler.device(),
            (num_attrs as usize * size_of::<u32>()) as u64,
        );

        let bind_group = self.weld_build_engine.create_bind_group([
            uniform_buf.binding_resource(),
            attr_buf.binding_resource(),
            head_buf.binding_resource(),
            next_buf.binding_resource(),
            label_buf.binding_resource(),
        ]);

        let workgroups = ((num_attrs + 63) / 64, 1, 1);
        self.weld_build_engine.dispatch(&bind_group, workgroups);

        for _ in 0..8 {
            self.weld_propagate_engine.dispatch(&bind_group, workgroups);
        }

        let mut labels: Vec<u32> = label_buf.read_to_vec_block(&self.handler);
        for i in 0..labels.len() {
            let mut root = labels[i] as usize;
            while (labels[root] as usize) != root {
                root = labels[root] as usize;
            }
            labels[i] = root as u32;
        }

        labels.into_iter().map(|idx| idx as usize).collect()
    }
}

/// Uniforms for Quadric Error Metric (QEM) compute shader.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct QemUniforms {
    /// Number of unique mesh edges.
    pub num_edges: u32,
    /// Number of mesh vertices.
    pub num_vertices: u32,
    /// Alignment padding.
    pub _pad0: u32,
    /// Alignment padding.
    pub _pad1: u32,
}

/// Uniforms for 2D multi-loop point-in-polygon compute shader.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct PointIn2dLoopsUniforms {
    /// Number of 2D query points.
    pub num_points: u32,
    /// Total number of directed 2D line segments across all loops.
    pub num_segments: u32,
    /// Alignment padding.
    pub _pad0: u32,
    /// Alignment padding.
    pub _pad1: u32,
}

/// Uniforms for 3D segment-to-mesh clearance distance compute shader.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SegmentMeshDistUniforms {
    /// Number of 3D line segments.
    pub num_segments: u32,
    /// Number of mesh triangles.
    pub num_triangles: u32,
    /// Alignment padding.
    pub _pad0: u32,
    /// Alignment padding.
    pub _pad1: u32,
}

/// 3D line segment input for segment-mesh distance calculations.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct GpuSegment {
    /// First segment endpoint `[x, y, z, 1.0]`.
    pub p0: [f32; 4],
    /// Second segment endpoint `[x, y, z, 1.0]`.
    pub p1: [f32; 4],
}

/// Uniforms for Loop subdivision compute shader.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct LoopSubdivisionUniforms {
    /// Number of vertices in the original mesh.
    pub num_vertices: u32,
    /// Number of unique edges in the original mesh.
    pub num_edges: u32,
    /// Alignment padding.
    pub _pad0: u32,
    /// Alignment padding.
    pub _pad1: u32,
}

/// Topology and boundary condition metadata for an original vertex.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct GpuLoopVertexInfo {
    /// Offset into the vertex neighbors CSR array.
    pub offset: u32,
    /// Valence (number of adjacent neighbors).
    pub valence: u32,
    /// Boundary classification: 0 = Inner, 1 = Boundary, 2 = Corner.
    pub boundary_type: u32,
    /// First adjacent boundary neighbor vertex index (if boundary).
    pub b0: u32,
    /// Second adjacent boundary neighbor vertex index (if boundary).
    pub b1: u32,
    /// Struct alignment padding.
    pub _pad: u32,
}

/// Edge connectivity and weighting metadata for Loop subdivision (f32 for Vulkan).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct GpuLoopEdgeInfoF32 {
    /// Index of first endpoint vertex.
    pub v0: u32,
    /// Index of second endpoint vertex.
    pub v1: u32,
    /// Index of first opposite wing vertex.
    pub w0: u32,
    /// Index of second opposite wing vertex (or 0xFFFFFFFF if boundary).
    pub w1: u32,
    /// Weighting factor gamma.
    pub gamma: f32,
    /// 1 if open boundary edge, 0 if interior.
    pub is_boundary: u32,
    /// Alignment padding.
    pub _pad0: u32,
    /// Alignment padding.
    pub _pad1: u32,
}

/// Edge connectivity and weighting metadata for Loop subdivision (f64 for CUDA).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct GpuLoopEdgeInfoF64 {
    /// Index of first endpoint vertex.
    pub v0: u32,
    /// Index of second endpoint vertex.
    pub v1: u32,
    /// Index of first opposite wing vertex.
    pub w0: u32,
    /// Index of second opposite wing vertex (or 0xFFFFFFFF if boundary).
    pub w1: u32,
    /// Weighting factor gamma (FP64 precision).
    pub gamma: f64,
    /// 1 if open boundary edge, 0 if interior.
    pub is_boundary: u32,
    /// Alignment padding.
    pub _pad: u32,
}

/// Uniform parameters for feature edge extraction.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct FeatureEdgeUniforms {
    /// Total number of unique mesh edges.
    pub num_edges: u32,
    /// Total number of triangle faces.
    pub num_faces: u32,
    /// Cosine of the crease threshold angle.
    pub cos_crease_angle: f32,
    /// 1 to include crease edges, 0 to ignore.
    pub include_creases: u32,
    /// 1 to include open boundary edges, 0 to ignore.
    pub include_boundaries: u32,
    /// Alignment padding.
    pub _pad0: u32,
    /// Alignment padding.
    pub _pad1: u32,
    /// Alignment padding.
    pub _pad2: u32,
}

/// Feature edge input struct describing edge endpoints and adjacent faces.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct GpuFeatureEdgeInput {
    /// First endpoint vertex index.
    pub v0: u32,
    /// Second endpoint vertex index.
    pub v1: u32,
    /// First adjacent face index.
    pub f0: i32,
    /// Second adjacent face index (-1 if open boundary).
    pub f1: i32,
}

/// 3D line segment output from GPU edge extraction.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct GpuEdgeSegment {
    /// First segment endpoint `[x, y, z, 1.0]`.
    pub p0: [f32; 4],
    /// Second segment endpoint `[x, y, z, 1.0]`.
    pub p1: [f32; 4],
}

/// Precomputed topology buffers for GPU smooth Loop subdivision.
#[derive(Clone, Debug)]
pub struct LoopSubdivisionTopology {
    /// Original vertex topology and boundary information.
    pub vertex_info: Vec<GpuLoopVertexInfo>,
    /// Flattened adjacency list of neighbor vertex indices.
    pub vertex_neighbors: Vec<u32>,
    /// Edge definitions with f32 weights for Vulkan.
    pub edges_f32: Vec<GpuLoopEdgeInfoF32>,
    /// Edge definitions with f64 weights for CUDA.
    pub edges_f64: Vec<GpuLoopEdgeInfoF64>,
    /// Subdivided 4-to-1 triangle face indices.
    pub new_tri_faces: Vec<[StandardVertex; 3]>,
}

/// Precomputed topology buffers for GPU feature and boundary edge extraction.
#[derive(Clone, Debug)]
pub struct FeatureEdgesTopology {
    /// Unique edges with face adjacency.
    pub edges: Vec<GpuFeatureEdgeInput>,
    /// Face normal vectors in FP64 precision.
    pub face_normals_f64: Vec<[f64; 3]>,
    /// Face normal vectors in FP32 precision for Vulkan.
    pub face_normals_f32: Vec<[f32; 4]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SubdivEdge(usize, usize);

#[derive(Clone, Copy, Debug)]
struct SubdivEdgeInfo {
    idx: usize,
    first_wing: usize,
    second_wing: Option<usize>,
}

fn add_subdiv_edge(
    edges: &mut rustc_hash::FxHashMap<SubdivEdge, SubdivEdgeInfo>,
    vertex_adjacency: &mut [Vec<usize>],
    v0: usize,
    v1: usize,
    v2: usize,
) {
    let idx = edges.len();
    edges
        .entry(SubdivEdge(usize::min(v0, v1), usize::max(v0, v1)))
        .and_modify(|tuple| tuple.second_wing = Some(v2))
        .or_insert_with(|| {
            vertex_adjacency[v0].push(v1);
            vertex_adjacency[v1].push(v0);
            SubdivEdgeInfo {
                idx,
                first_wing: v2,
                second_wing: None,
            }
        });
}

/// Prepares full topology buffers for smooth Loop subdivision surface refinement.
pub fn prepare_loop_subdivision_topology(mesh: &PolygonMesh) -> LoopSubdivisionTopology {
    let num_vertices = mesh.positions().len();
    let mut edges = rustc_hash::FxHashMap::default();
    let mut vertex_adjacency = vec![Vec::new(); num_vertices];

    mesh.tri_faces().iter().for_each(|face| {
        let f = [face[0].pos, face[1].pos, face[2].pos];
        add_subdiv_edge(&mut edges, &mut vertex_adjacency, f[1], f[2], f[0]);
        add_subdiv_edge(&mut edges, &mut vertex_adjacency, f[2], f[0], f[1]);
        add_subdiv_edge(&mut edges, &mut vertex_adjacency, f[0], f[1], f[2]);
    });

    let mut vertex_boundary = Vec::with_capacity(num_vertices);
    for (v, adj) in vertex_adjacency.iter().enumerate() {
        let binfo = adj.iter().copied().fold((None, None), |binfo, w| {
            let edge = edges.get(&SubdivEdge(usize::min(v, w), usize::max(v, w))).unwrap();
            match (edge.second_wing, binfo) {
                (None, (None, _)) => (Some(w), None),
                (None, (Some(x), None)) => (Some(x), Some(w)),
                _ => binfo,
            }
        });
        let cond = if let (Some(x), Some(y)) = binfo {
            if adj.len() == 2 {
                (2u32, x as u32, y as u32) // Corner
            } else {
                (1u32, x as u32, y as u32) // Boundary
            }
        } else {
            (0u32, 0u32, 0u32) // Inner
        };
        vertex_boundary.push(cond);
    }

    let mut vertex_info = Vec::with_capacity(num_vertices);
    let mut vertex_neighbors = Vec::new();
    for (adj, &(b_type, b0, b1)) in vertex_adjacency.iter().zip(&vertex_boundary) {
        let offset = vertex_neighbors.len() as u32;
        let valence = adj.len() as u32;
        vertex_neighbors.extend(adj.iter().map(|&nb| nb as u32));
        vertex_info.push(GpuLoopVertexInfo {
            offset,
            valence,
            boundary_type: b_type,
            b0,
            b1,
            _pad: 0,
        });
    }

    let num_edges = edges.len();
    let mut edges_f32 = vec![GpuLoopEdgeInfoF32::default(); num_edges];
    let mut edges_f64 = vec![GpuLoopEdgeInfoF64::default(); num_edges];
    for (&SubdivEdge(v0, v1), info) in edges.iter() {
        let is_boundary = info.second_wing.is_none();
        let (w0, w1) = match info.second_wing {
            Some(w1) => (info.first_wing as u32, w1 as u32),
            None => (info.first_wing as u32, 0xFFFFFFFFu32),
        };
        let gamma = if is_boundary {
            0.5
        } else {
            let (cond0, _, _) = vertex_boundary[v0];
            let (cond1, _, _) = vertex_boundary[v1];
            if cond0 == 1 {
                5.0 / 8.0 - f64::cos(std::f64::consts::PI / vertex_adjacency[v0].len() as f64) / 4.0
            } else if cond1 == 1 {
                3.0 / 8.0 + f64::cos(std::f64::consts::PI / vertex_adjacency[v1].len() as f64) / 4.0
            } else {
                0.5
            }
        };

        edges_f32[info.idx] = GpuLoopEdgeInfoF32 {
            v0: v0 as u32,
            v1: v1 as u32,
            w0,
            w1,
            gamma: gamma as f32,
            is_boundary: if is_boundary { 1 } else { 0 },
            _pad0: 0,
            _pad1: 0,
        };
        edges_f64[info.idx] = GpuLoopEdgeInfoF64 {
            v0: v0 as u32,
            v1: v1 as u32,
            w0,
            w1,
            gamma,
            is_boundary: if is_boundary { 1 } else { 0 },
            _pad: 0,
        };
    }

    let new_tri_faces = mesh.tri_faces().iter().flat_map(|v| {
        let len = num_vertices;
        let e: [StandardVertex; 3] = [
            (edges.get(&SubdivEdge(usize::min(v[1].pos, v[2].pos), usize::max(v[1].pos, v[2].pos))).unwrap().idx + len).into(),
            (edges.get(&SubdivEdge(usize::min(v[2].pos, v[0].pos), usize::max(v[2].pos, v[0].pos))).unwrap().idx + len).into(),
            (edges.get(&SubdivEdge(usize::min(v[0].pos, v[1].pos), usize::max(v[0].pos, v[1].pos))).unwrap().idx + len).into(),
        ];
        vec![
            [v[0], e[2], e[1]],
            [e[0], v[2], e[1]],
            [e[0], e[2], v[1]],
            [e[0], e[1], e[2]],
        ]
    }).collect();

    LoopSubdivisionTopology {
        vertex_info,
        vertex_neighbors,
        edges_f32,
        edges_f64,
        new_tri_faces,
    }
}

/// Prepares topology buffers and face normals for feature/boundary edge extraction.
pub fn prepare_feature_edges_topology(mesh: &PolygonMesh) -> FeatureEdgesTopology {
    let positions = mesh.positions();
    let tri_faces = mesh.tri_faces();
    let num_faces = tri_faces.len();
    let mut face_normals_f64 = Vec::with_capacity(num_faces);
    let mut face_normals_f32 = Vec::with_capacity(num_faces);

    for face in tri_faces.iter() {
        let p0 = positions[face[0].pos];
        let p1 = positions[face[1].pos];
        let p2 = positions[face[2].pos];
        let u = p1 - p0;
        let v = p2 - p0;
        let n = u.cross(v);
        let len = n.magnitude();
        if len > 1e-12 {
            let inv = 1.0 / len;
            let unit = [n.x * inv, n.y * inv, n.z * inv];
            face_normals_f64.push(unit);
            face_normals_f32.push([unit[0] as f32, unit[1] as f32, unit[2] as f32, 0.0]);
        } else {
            face_normals_f64.push([0.0, 0.0, 1.0]);
            face_normals_f32.push([0.0, 0.0, 1.0, 0.0]);
        }
    }

    let mut edge_map = rustc_hash::FxHashMap::default();
    for (f_idx, face) in tri_faces.iter().enumerate() {
        let v = [face[0].pos, face[1].pos, face[2].pos];
        let f_edges = [
            (usize::min(v[0], v[1]), usize::max(v[0], v[1])),
            (usize::min(v[1], v[2]), usize::max(v[1], v[2])),
            (usize::min(v[2], v[0]), usize::max(v[2], v[0])),
        ];
        for &e in &f_edges {
            edge_map.entry(e)
                .and_modify(|info: &mut (usize, Option<usize>)| {
                    if info.1.is_none() {
                        info.1 = Some(f_idx);
                    }
                })
                .or_insert((f_idx, None));
        }
    }

    let mut edges = Vec::with_capacity(edge_map.len());
    for (&(v0, v1), &(f0, f1)) in edge_map.iter() {
        edges.push(GpuFeatureEdgeInput {
            v0: v0 as u32,
            v1: v1 as u32,
            f0: f0 as i32,
            f1: f1.map(|idx| idx as i32).unwrap_or(-1),
        });
    }

    FeatureEdgesTopology {
        edges,
        face_normals_f64,
        face_normals_f32,
    }
}

/// Helper to construct a PolygonMesh from a slice of GpuTriangles.
pub fn create_mesh_from_triangles(triangles: &[GpuTriangle]) -> PolygonMesh {
    use truck_polymesh::{Faces, StandardAttributes};
    let mut positions = Vec::with_capacity(triangles.len() * 3);
    let mut tri_faces = Vec::with_capacity(triangles.len());
    for (i, tri) in triangles.iter().enumerate() {
        positions.push(Point3::new(tri.v0[0] as f64, tri.v0[1] as f64, tri.v0[2] as f64));
        positions.push(Point3::new(tri.v1[0] as f64, tri.v1[1] as f64, tri.v1[2] as f64));
        positions.push(Point3::new(tri.v2[0] as f64, tri.v2[1] as f64, tri.v2[2] as f64));
        let base = i * 3;
        tri_faces.push([base.into(), (base + 1).into(), (base + 2).into()]);
    }
    PolygonMesh::new(
        StandardAttributes {
            positions,
            ..Default::default()
        },
        Faces::from_tri_and_quad_faces(tri_faces, Vec::new()),
    )
}

/// Prepares flattened buffers for extract_planes (f64 precision for CUDA).
pub fn prepare_extract_planes_buffers_f64(mesh: &PolygonMesh) -> (Vec<u32>, Vec<[f64; 3]>, Vec<[f64; 4]>) {
    let positions = mesh.positions();
    let normals = mesh.normals();
    let num_faces = mesh.faces().len();
    let mut offsets = Vec::with_capacity(num_faces + 1);
    let mut face_pos = Vec::new();
    let mut face_nor = Vec::new();

    offsets.push(0u32);
    for face in mesh.face_iter() {
        for v in face {
            let p = positions[v.pos];
            face_pos.push([p.x, p.y, p.z]);
            if let Some(nor_idx) = v.nor {
                let n = normals[nor_idx];
                face_nor.push([n.x, n.y, n.z, 1.0]);
            } else {
                face_nor.push([0.0, 0.0, 0.0, 0.0]);
            }
        }
        offsets.push(face_pos.len() as u32);
    }
    (offsets, face_pos, face_nor)
}

/// Prepares flattened buffers for extract_planes (f32 precision for Vulkan).
pub fn prepare_extract_planes_buffers_f32(mesh: &PolygonMesh) -> (Vec<u32>, Vec<[f32; 4]>, Vec<[f32; 4]>) {
    let positions = mesh.positions();
    let normals = mesh.normals();
    let num_faces = mesh.faces().len();
    let mut offsets = Vec::with_capacity(num_faces + 1);
    let mut face_pos = Vec::new();
    let mut face_nor = Vec::new();

    offsets.push(0u32);
    for face in mesh.face_iter() {
        for v in face {
            let p = positions[v.pos];
            face_pos.push([p.x as f32, p.y as f32, p.z as f32, 0.0]);
            if let Some(nor_idx) = v.nor {
                let n = normals[nor_idx];
                face_nor.push([n.x as f32, n.y as f32, n.z as f32, 1.0]);
            } else {
                face_nor.push([0.0, 0.0, 0.0, 0.0]);
            }
        }
        offsets.push(face_pos.len() as u32);
    }
    (offsets, face_pos, face_nor)
}
