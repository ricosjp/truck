use glsl_to_spirv::ShaderType;
use std::io::Read;
use std::sync::Arc;
use truck_platform::*;
use wgpu::*;

pub const PICTURE_WIDTH: u32 = 256;
pub const PICTURE_HEIGHT: u32 = 256;

pub fn read_shader(device: &Device, code: &str, shadertype: ShaderType) -> ShaderModule {
    let mut spirv = glsl_to_spirv::compile(&code, shadertype).unwrap();
    let mut compiled = Vec::new();
    spirv.read_to_end(&mut compiled).unwrap();
    device.create_shader_module(wgpu::util::make_spirv(&compiled))
}

pub fn init_device(instance: &Instance) -> (Arc<Device>, Arc<Queue>) {
    futures::executor::block_on(async {
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::Default,
                compatible_surface: None,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features: Default::default(),
                    limits: Limits::default(),
                    shader_validation: true,
                },
                None,
            )
            .await
            .unwrap();
        (Arc::new(device), Arc::new(queue))
    })
}

pub fn swap_chain_descriptor() -> SwapChainDescriptor {
    SwapChainDescriptor {
        usage: TextureUsage::OUTPUT_ATTACHMENT,
        format: TextureFormat::Bgra8UnormSrgb,
        width: PICTURE_WIDTH,
        height: PICTURE_HEIGHT,
        present_mode: PresentMode::Mailbox,
    }
}

pub fn extend3d() -> Extent3d {
    Extent3d {
        width: PICTURE_WIDTH,
        height: PICTURE_HEIGHT,
        depth: 1,
    }
}

pub fn texture_descriptor(sc_desc: &SwapChainDescriptor) -> TextureDescriptor<'static> {
    TextureDescriptor {
        label: None,
        size: extend3d(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: sc_desc.format,
        usage: TextureUsage::OUTPUT_ATTACHMENT | TextureUsage::COPY_SRC,
    }
}

pub fn texture_copy_view<'a>(texture: &'a Texture) -> TextureCopyView<'a> {
    TextureCopyView {
        texture: &texture,
        mip_level: 0,
        origin: Origin3d::ZERO,
    }
}

pub fn buffer_copy_view<'a>(buffer: &'a Buffer) -> BufferCopyView<'a> {
    BufferCopyView {
        buffer: &buffer,
        layout: TextureDataLayout {
            offset: 0,
            bytes_per_row: PICTURE_WIDTH * 4,
            rows_per_image: PICTURE_HEIGHT,
        },
    }
}

pub fn read_buffer(device: &Device, buffer: &Buffer) -> Vec<u8> {
    let buffer_slice = buffer.slice(..);
    let buffer_future = buffer_slice.map_async(MapMode::Read);
    device.poll(Maintain::Wait);
    futures::executor::block_on(async {
        match buffer_future.await {
            Ok(_) => buffer_slice.get_mapped_range().iter().map(|b| *b).collect(),
            Err(_) => panic!("failed to run compute on gpu!"),
        }
    })
}

pub fn read_texture(handler: &DeviceHandler, texture: &Texture) -> Vec<u8> {
    let (device, queue) = (handler.device(), handler.queue());
    let size = (PICTURE_WIDTH * PICTURE_HEIGHT * 4) as u64;
    let buffer = device.create_buffer(&BufferDescriptor {
        label: None,
        mapped_at_creation: false,
        usage: BufferUsage::COPY_DST | BufferUsage::MAP_READ,
        size,
    });
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
    encoder.copy_texture_to_buffer(
        texture_copy_view(&texture),
        buffer_copy_view(&buffer),
        extend3d(),
    );
    queue.submit(Some(encoder.finish()));
    read_buffer(device, &buffer)
}

pub fn same_texture(handler: &DeviceHandler, texture0: &Texture, texture1: &Texture) -> bool {
    let vec0 = read_texture(handler, texture0);
    let vec1 = read_texture(handler, texture1);
    vec0.into_iter()
        .zip(vec1)
        .all(move |(i, j)| std::cmp::max(i, j) - std::cmp::min(i, j) < 3)
}
