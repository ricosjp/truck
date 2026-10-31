//! GPU-accelerated mesh algorithms based on wgpu / Vulkan.

use bytemuck::{Pod, Zeroable};
use truck_base::cgmath64::*;
use truck_base::hash;
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

/// GPU-accelerated mesh analyzer for point-cloud distance queries and in-out containment.
#[derive(Debug)]
pub struct GpuMeshAnalyzer {
    handler: DeviceHandler,
    dist_engine: ComputeEngine,
    in_out_engine: ComputeEngine,
}

impl GpuMeshAnalyzer {
    /// Creates a new GPU mesh analyzer.
    pub fn new(handler: &DeviceHandler) -> Self {
        let dist_entries = distance_layout_entries();
        let dist_engine = ComputeEngine::new(handler, DISTANCE_SHADER, "main", &dist_entries);

        let in_out_entries = in_out_layout_entries();
        let in_out_engine = ComputeEngine::new(handler, IN_OUT_SHADER, "main", &in_out_entries);

        Self {
            handler: handler.clone(),
            dist_engine,
            in_out_engine,
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
}
