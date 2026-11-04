#![allow(unsafe_code)]

//! Bounding Volume Hierarchy (BVH) spatial acceleration structure for GPU ray tracing.
//!
//! Provides hardware-oriented Surface Area Heuristic (SAH) BVH building and memory-compact
//! 32-byte node representations for GPU raycasting, solid containment, and toolpath clearance.

use super::GpuTriangle;
#[allow(unused_imports)]
use truck_base::cgmath64::*;
use truck_polymesh::PolygonMesh;

/// 32-byte GPU-aligned BVH node matching native hardware ray traversal layouts.
///
/// Layout:
/// - `aabb_min`: 12 bytes `[min_x, min_y, min_z]`
/// - `left_or_tri_start`: 4 bytes. If `>= 0`, internal node pointing to left child. If `< 0`, leaf node with triangle start index `!val`.
/// - `aabb_max`: 12 bytes `[max_x, max_y, max_z]`
/// - `right_or_tri_count`: 4 bytes. If internal, index of right child. If leaf, count of triangles.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuBvhNode {
    /// Bounding box minimum corner.
    pub aabb_min: [f32; 3],
    /// Child index if internal (`>= 0`), or bitwise NOT of triangle start index if leaf (`< 0`).
    pub left_or_tri_start: i32,
    /// Bounding box maximum corner.
    pub aabb_max: [f32; 3],
    /// Right child index if internal, or triangle count if leaf.
    pub right_or_tri_count: i32,
}

impl GpuBvhNode {
    /// Returns true if this node is a leaf containing triangles.
    #[inline(always)]
    pub fn is_leaf(&self) -> bool {
        self.left_or_tri_start < 0
    }

    /// Returns the triangle range `(start_idx, count)` if this is a leaf node.
    #[inline(always)]
    pub fn tri_range(&self) -> Option<(usize, usize)> {
        if self.is_leaf() {
            Some(((!self.left_or_tri_start) as usize, self.right_or_tri_count as usize))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct TriInfo {
    aabb_min: [f32; 3],
    aabb_max: [f32; 3],
    centroid: [f32; 3],
    orig_idx: usize,
}

fn compute_tri_info(tri: &GpuTriangle, orig_idx: usize) -> TriInfo {
    let v0 = tri.v0;
    let v1 = tri.v1;
    let v2 = tri.v2;

    let min_x = v0[0].min(v1[0]).min(v2[0]);
    let min_y = v0[1].min(v1[1]).min(v2[1]);
    let min_z = v0[2].min(v1[2]).min(v2[2]);

    let max_x = v0[0].max(v1[0]).max(v2[0]);
    let max_y = v0[1].max(v1[1]).max(v2[1]);
    let max_z = v0[2].max(v1[2]).max(v2[2]);

    TriInfo {
        aabb_min: [min_x, min_y, min_z],
        aabb_max: [max_x, max_y, max_z],
        centroid: [(min_x + max_x) * 0.5, (min_y + max_y) * 0.5, (min_z + max_z) * 0.5],
        orig_idx,
    }
}

#[inline(always)]
fn aabb_surface_area(min: [f32; 3], max: [f32; 3]) -> f32 {
    let dx = (max[0] - min[0]).max(0.0);
    let dy = (max[1] - min[1]).max(0.0);
    let dz = (max[2] - min[2]).max(0.0);
    2.0 * (dx * dy + dy * dz + dz * dx)
}

/// Builds a high-quality Binned Surface Area Heuristic (SAH) BVH over triangles.
///
/// Returns `(nodes, reordered_triangles, original_indices)`.
pub fn build_bvh(
    triangles: &[GpuTriangle],
) -> (Vec<GpuBvhNode>, Vec<GpuTriangle>, Vec<u32>) {
    if triangles.is_empty() {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    let mut tri_infos: Vec<TriInfo> = triangles
        .iter()
        .enumerate()
        .map(|(i, t)| compute_tri_info(t, i))
        .collect();

    let mut nodes = Vec::with_capacity(triangles.len() * 2);
    let mut reordered_triangles = Vec::with_capacity(triangles.len());
    let mut original_indices = Vec::with_capacity(triangles.len());

    build_recursive(
        &mut tri_infos,
        triangles,
        &mut nodes,
        &mut reordered_triangles,
        &mut original_indices,
    );

    (nodes, reordered_triangles, original_indices)
}

/// Builds a high-quality BVH directly from a polygon mesh.
pub fn build_mesh_bvh(mesh: &PolygonMesh) -> (Vec<GpuBvhNode>, Vec<GpuTriangle>, Vec<u32>) {
    let triangles = crate::gpu::GpuMeshAnalyzer::extract_triangles(mesh);
    build_bvh(&triangles)
}

fn build_recursive(
    tri_infos: &mut [TriInfo],
    orig_triangles: &[GpuTriangle],
    nodes: &mut Vec<GpuBvhNode>,
    reordered_triangles: &mut Vec<GpuTriangle>,
    original_indices: &mut Vec<u32>,
) -> usize {
    let node_idx = nodes.len();
    nodes.push(GpuBvhNode {
        aabb_min: [0.0; 3],
        left_or_tri_start: 0,
        aabb_max: [0.0; 3],
        right_or_tri_count: 0,
    });

    // Compute enclosing AABB of all triangles in this range
    let mut node_min = [f32::INFINITY; 3];
    let mut node_max = [f32::NEG_INFINITY; 3];
    let mut c_min = [f32::INFINITY; 3];
    let mut c_max = [f32::NEG_INFINITY; 3];

    for t in tri_infos.iter() {
        for a in 0..3 {
            node_min[a] = node_min[a].min(t.aabb_min[a]);
            node_max[a] = node_max[a].max(t.aabb_max[a]);
            c_min[a] = c_min[a].min(t.centroid[a]);
            c_max[a] = c_max[a].max(t.centroid[a]);
        }
    }

    let count = tri_infos.len();
    let max_leaf_size = 4;

    // Base case: small triangle count or zero centroid spread
    let c_extent = [
        c_max[0] - c_min[0],
        c_max[1] - c_min[1],
        c_max[2] - c_min[2],
    ];

    let max_spread = c_extent[0].max(c_extent[1]).max(c_extent[2]);

    if count <= max_leaf_size || max_spread < 1e-7 {
        let tri_start = reordered_triangles.len();
        for t in tri_infos.iter() {
            reordered_triangles.push(orig_triangles[t.orig_idx]);
            original_indices.push(t.orig_idx as u32);
        }
        nodes[node_idx] = GpuBvhNode {
            aabb_min: node_min,
            left_or_tri_start: !(tri_start as i32),
            aabb_max: node_max,
            right_or_tri_count: count as i32,
        };
        return node_idx;
    }

    // Default split axis by maximum extent
    let mut best_axis = 0;
    if c_extent[1] > c_extent[0] && c_extent[1] > c_extent[2] {
        best_axis = 1;
    } else if c_extent[2] > c_extent[0] && c_extent[2] > c_extent[1] {
        best_axis = 2;
    }

    let mut best_cost = f32::INFINITY;
    let parent_sa = aabb_surface_area(node_min, node_max).max(1e-12);

    const NUM_BINS: usize = 16;

    for axis in 0..3 {
        let extent = c_extent[axis];
        if extent < 1e-7 {
            continue;
        }

        #[derive(Clone, Copy)]
        struct Bin {
            min: [f32; 3],
            max: [f32; 3],
            count: usize,
        }
        let mut bins = [Bin {
            min: [f32::INFINITY; 3],
            max: [f32::NEG_INFINITY; 3],
            count: 0,
        }; NUM_BINS];

        let inv_extent = NUM_BINS as f32 / extent;
        let axis_min = c_min[axis];

        for t in tri_infos.iter() {
            let bin_idx = (((t.centroid[axis] - axis_min) * inv_extent) as usize).min(NUM_BINS - 1);
            let b = &mut bins[bin_idx];
            b.count += 1;
            for a in 0..3 {
                b.min[a] = b.min[a].min(t.aabb_min[a]);
                b.max[a] = b.max[a].max(t.aabb_max[a]);
            }
        }

        // Prefix and suffix sweeps for fast SAH evaluation
        let mut left_min = [f32::INFINITY; 3];
        let mut left_max = [f32::NEG_INFINITY; 3];
        let mut left_counts = [0usize; NUM_BINS];
        let mut left_areas = [0.0f32; NUM_BINS];
        let mut cur_count = 0;

        for i in 0..NUM_BINS - 1 {
            cur_count += bins[i].count;
            left_counts[i] = cur_count;
            for a in 0..3 {
                left_min[a] = left_min[a].min(bins[i].min[a]);
                left_max[a] = left_max[a].max(bins[i].max[a]);
            }
            left_areas[i] = aabb_surface_area(left_min, left_max);
        }

        let mut right_min = [f32::INFINITY; 3];
        let mut right_max = [f32::NEG_INFINITY; 3];
        let mut cur_right_count = 0;

        for i in (1..NUM_BINS).rev() {
            cur_right_count += bins[i].count;
            for a in 0..3 {
                right_min[a] = right_min[a].min(bins[i].min[a]);
                right_max[a] = right_max[a].max(bins[i].max[a]);
            }
            let right_sa = aabb_surface_area(right_min, right_max);
            let left_sa = left_areas[i - 1];
            let n_left = left_counts[i - 1];
            let n_right = cur_right_count;

            if n_left > 0 && n_right > 0 {
                let cost = (left_sa * n_left as f32 + right_sa * n_right as f32) / parent_sa + 1.0;
                if cost < best_cost {
                    best_cost = cost;
                    best_axis = axis;
                }
            }
        }
    }

    // Partition around best axis median
    tri_infos.sort_unstable_by(|a, b| {
        a.centroid[best_axis]
            .partial_cmp(&b.centroid[best_axis])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let best_split_idx = (count / 2).max(1);

    let (left_slice, right_slice) = tri_infos.split_at_mut(best_split_idx);

    let left_child_idx = build_recursive(
        left_slice,
        orig_triangles,
        nodes,
        reordered_triangles,
        original_indices,
    );

    let right_child_idx = build_recursive(
        right_slice,
        orig_triangles,
        nodes,
        reordered_triangles,
        original_indices,
    );

    nodes[node_idx] = GpuBvhNode {
        aabb_min: node_min,
        left_or_tri_start: left_child_idx as i32,
        aabb_max: node_max,
        right_or_tri_count: right_child_idx as i32,
    };

    node_idx
}

#[cfg(feature = "cuda")]
extern "C" {
    fn truck_cuda_set_device(device_id: i32) -> i32;

    fn truck_cuda_bvh_create(
        h_nodes: *const GpuBvhNode,
        num_nodes: u32,
        h_triangles: *const f32,
        h_orig_indices: *const u32,
        num_triangles: u32,
    ) -> *mut std::ffi::c_void;

    fn truck_cuda_bvh_destroy(bvh: *mut std::ffi::c_void);

    fn truck_cuda_bvh_raycast(
        bvh: *const std::ffi::c_void,
        h_ray_origins: *const f32,
        h_ray_dirs: *const f32,
        num_rays: u32,
        h_out_distances: *mut f32,
        h_out_tri_indices: *mut i32,
        h_out_hit_points: *mut f32,
        h_out_normals: *mut f32,
    ) -> i32;

    fn truck_cuda_bvh_signed_crossing_rays(
        bvh: *const std::ffi::c_void,
        h_points: *const f32,
        h_ray_dirs: *const f32,
        num_points: u32,
        h_crossing_counts: *mut i32,
    ) -> i32;
}

/// High-performance GPU-resident BVH structure for raycasting and solid mesh containment queries.
///
/// Keeps the BVH nodes and mesh triangles resident in GPU memory across queries, eliminating
/// re-building and PCIe data transfer overheads during repeated collision checks or toolpath evaluations.
#[cfg(feature = "cuda")]
pub struct CudaMeshBvh {
    raw: *mut std::ffi::c_void,
    num_nodes: usize,
    num_triangles: usize,
    device_id: i32,
}

#[cfg(feature = "cuda")]
impl std::fmt::Debug for CudaMeshBvh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CudaMeshBvh")
            .field("num_nodes", &self.num_nodes)
            .field("num_triangles", &self.num_triangles)
            .field("device_id", &self.device_id)
            .finish()
    }
}

#[cfg(feature = "cuda")]
unsafe impl Send for CudaMeshBvh {}
#[cfg(feature = "cuda")]
unsafe impl Sync for CudaMeshBvh {}

#[cfg(feature = "cuda")]
impl CudaMeshBvh {
    /// Builds a BVH on the primary/current GPU device from a polygon mesh.
    pub fn new(mesh: &PolygonMesh) -> Self {
        Self::new_on_device(mesh, 0)
    }

    /// Builds a BVH on the specified GPU device from a polygon mesh.
    pub fn new_on_device(mesh: &PolygonMesh, device_id: i32) -> Self {
        let (nodes, reordered_triangles, orig_indices) = build_mesh_bvh(mesh);
        Self::from_bvh_data(&nodes, &reordered_triangles, &orig_indices, device_id)
    }

    /// Builds a BVH on the primary/current GPU device from a slice of GPU triangles.
    pub fn from_triangles(triangles: &[GpuTriangle]) -> Self {
        Self::from_triangles_on_device(triangles, 0)
    }

    /// Builds a BVH on the specified GPU device from a slice of GPU triangles.
    pub fn from_triangles_on_device(triangles: &[GpuTriangle], device_id: i32) -> Self {
        let (nodes, reordered_triangles, orig_indices) = build_bvh(triangles);
        Self::from_bvh_data(&nodes, &reordered_triangles, &orig_indices, device_id)
    }

    /// Creates a GPU-resident BVH from pre-built BVH nodes and reordered triangles.
    pub fn from_bvh_data(
        nodes: &[GpuBvhNode],
        triangles: &[GpuTriangle],
        orig_indices: &[u32],
        device_id: i32,
    ) -> Self {
        if nodes.is_empty() || triangles.is_empty() {
            return Self {
                raw: std::ptr::null_mut(),
                num_nodes: 0,
                num_triangles: 0,
                device_id,
            };
        }

        unsafe {
            let _ = truck_cuda_set_device(device_id);
            let raw = truck_cuda_bvh_create(
                nodes.as_ptr(),
                nodes.len() as u32,
                triangles.as_ptr() as *const f32,
                orig_indices.as_ptr(),
                triangles.len() as u32,
            );
            Self {
                raw,
                num_nodes: nodes.len(),
                num_triangles: triangles.len(),
                device_id,
            }
        }
    }

    /// Returns the number of nodes in the BVH.
    #[inline(always)]
    pub fn num_nodes(&self) -> usize {
        self.num_nodes
    }

    /// Returns the number of triangles in the BVH.
    #[inline(always)]
    pub fn num_triangles(&self) -> usize {
        self.num_triangles
    }

    /// Returns the device ID hosting this BVH.
    #[inline(always)]
    pub fn device_id(&self) -> i32 {
        self.device_id
    }

    /// Intersects a slice of query rays against this BVH structure on the GPU.
    ///
    /// Returns a vector of `Option<(distance, hit_point, surface_normal)>`.
    pub fn raycast(&self, rays: &[(Point3, Vector3)]) -> Vec<Option<(f64, Point3, Vector3)>> {
        if self.raw.is_null() || rays.is_empty() {
            return vec![None; rays.len()];
        }

        let num_rays = rays.len() as u32;
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

        let mut out_distances = vec![0.0f32; rays.len()];
        let mut out_tri_indices = vec![0i32; rays.len()];
        let mut out_hit_points = vec![[0.0f32; 4]; rays.len()];
        let mut out_normals = vec![[0.0f32; 4]; rays.len()];

        let status = unsafe {
            truck_cuda_bvh_raycast(
                self.raw,
                ray_origins.as_ptr() as *const f32,
                ray_dirs.as_ptr() as *const f32,
                num_rays,
                out_distances.as_mut_ptr(),
                out_tri_indices.as_mut_ptr(),
                out_hit_points.as_mut_ptr() as *mut f32,
                out_normals.as_mut_ptr() as *mut f32,
            )
        };
        assert_eq!(status, 0, "truck_cuda_bvh_raycast failed with code: {}", status);

        out_distances
            .into_iter()
            .zip(out_hit_points)
            .zip(out_normals)
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

    /// Computes signed face crossing counts for rays starting at `points` along `dirs`.
    pub fn signed_crossing_rays(&self, points: &[Point3], dirs: &[Vector3]) -> Vec<i32> {
        if self.raw.is_null() || points.is_empty() {
            return vec![0; points.len()];
        }
        assert_eq!(points.len(), dirs.len(), "points and dirs must have identical lengths");

        let num_points = points.len() as u32;
        let pt_vec: Vec<[f32; 4]> = points
            .iter()
            .map(|p| [p.x as f32, p.y as f32, p.z as f32, 1.0f32])
            .collect();
        let dir_vec: Vec<[f32; 4]> = dirs
            .iter()
            .map(|d| {
                let norm = d.normalize();
                [norm.x as f32, norm.y as f32, norm.z as f32, 0.0f32]
            })
            .collect();

        let mut out_counts = vec![0i32; points.len()];

        let status = unsafe {
            truck_cuda_bvh_signed_crossing_rays(
                self.raw,
                pt_vec.as_ptr() as *const f32,
                dir_vec.as_ptr() as *const f32,
                num_points,
                out_counts.as_mut_ptr(),
            )
        };
        assert_eq!(status, 0, "truck_cuda_bvh_signed_crossing_rays failed with code: {}", status);

        out_counts
    }

    /// Determines whether each point in `points` is inside the domain bounded by the mesh.
    pub fn inside(&self, points: &[Point3]) -> Vec<bool> {
        let dirs: Vec<Vector3> = points.iter().map(|&p| truck_base::hash::take_one_unit(p)).collect();
        let counts = self.signed_crossing_rays(points, &dirs);
        counts.into_iter().map(|c| c >= 1).collect()
    }
}

#[cfg(feature = "cuda")]
impl Drop for CudaMeshBvh {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                truck_cuda_bvh_destroy(self.raw);
            }
            self.raw = std::ptr::null_mut();
        }
    }
}
