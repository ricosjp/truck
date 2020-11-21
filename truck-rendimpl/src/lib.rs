extern crate truck_modeling;
extern crate truck_polymesh;
pub use truck_modeling::*;
use truck_platform::{
    bytemuck::*,
    wgpu::util::{BufferInitDescriptor, DeviceExt},
    wgpu::*,
    *,
};
pub use truck_polymesh::*;

pub mod polymesh;
