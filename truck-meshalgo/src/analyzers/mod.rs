use crate::*;

mod collision;
mod in_out_judge;
pub(crate) mod point_cloud;
mod slicing;
mod splitting;
mod topology;
mod volume;

pub use collision::Collision;
pub use in_out_judge::IncludingPointInDomain;
pub use point_cloud::WithPointCloud;
pub use slicing::{
    chain_segments, chain_segments_into_loops, intersect_triangle_plane, slice_mesh_planes_cpu,
    slice_mesh_planes_rayon, PlanarSlicing,
};
pub use splitting::ExperimentalSplitters;
pub use splitting::Splitting;
pub(crate) use splitting::is_in_the_plane;
pub use topology::Topology;
pub use truck_topology::shell::ShellCondition;
pub use volume::CalcVolume;
