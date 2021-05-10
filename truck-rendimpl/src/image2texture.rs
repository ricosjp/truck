use super::*;
use image::*;
use std::convert::TryInto;

/// Utility for creating `Texture` from `DynamicImage`
#[inline(always)]
pub fn image2texture(device_handler: &DeviceHandler, image: &DynamicImage) -> Texture {
    let buffer = image.to_rgba8();
    imagebuffer2texture(device_handler, &buffer, TextureFormat::Rgba8Unorm)
}

fn imagebuffer2texture<P, Container>(
    device_handler: &DeviceHandler,
    image_buffer: &ImageBuffer<P, Container>,
    format: TextureFormat,
) -> Texture
where
    P: Pixel + 'static,
    P::Subpixel: Pod + Zeroable + 'static,
    Container: std::ops::Deref<Target = [P::Subpixel]>,
{
    let (device, queue) = (device_handler.device(), device_handler.queue());
    let size = Extent3d {
        width: image_buffer.width(),
        height: image_buffer.height(),
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
    });
    queue.write_texture(
        ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
        },
        bytemuck::cast_slice(&image_buffer),
        ImageDataLayout {
            offset: 0,
            bytes_per_row: (size.width * std::mem::size_of::<P>() as u32).try_into().ok(),
            rows_per_image: size.height.try_into().ok(),
        },
        size,
    );
    texture
}
