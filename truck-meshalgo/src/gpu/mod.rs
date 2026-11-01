//! GPU-accelerated mesh algorithms based on wgpu / Vulkan.

use bytemuck::{Pod, Zeroable};
use truck_base::cgmath64::*;
use truck_base::hash;
use truck_geometry::prelude::*;
use truck_platform::wgpu::*;
use truck_platform::*;
use truck_polymesh::PolygonMesh;

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
}
