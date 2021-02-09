use super::*;
use image::*;

/// Create `Texture` from `DynamicImage`
#[inline(always)]
pub fn image2texture(device_handler: &DeviceHandler, image: &DynamicImage) -> Texture {
    match image {
        DynamicImage::ImageLuma8(buffer) => {
            imagebuffer2texture(device_handler, buffer, TextureFormat::R8Unorm)
        }
        DynamicImage::ImageLumaA8(buffer) => {
            imagebuffer2texture(device_handler, buffer, TextureFormat::Rg8Unorm)
        }
        DynamicImage::ImageRgb8(_) => {
            let rgba = image.to_rgba8();
            imagebuffer2texture(device_handler, &rgba, TextureFormat::Rgba8Unorm)
        }
        DynamicImage::ImageRgba8(buffer) => {
            imagebuffer2texture(device_handler, buffer, TextureFormat::Rgba8Unorm)
        }
        DynamicImage::ImageBgr8(_) => {
            let brga = image.to_bgra8();
            imagebuffer2texture(device_handler, &brga, TextureFormat::Bgra8Unorm)
        }
        DynamicImage::ImageBgra8(buffer) => {
            imagebuffer2texture(device_handler, buffer, TextureFormat::Bgra8Unorm)
        }
        DynamicImage::ImageLuma16(buffer) => {
            imagebuffer2texture(device_handler, buffer, TextureFormat::R16Float)
        }
        DynamicImage::ImageLumaA16(buffer) => {
            imagebuffer2texture(device_handler, buffer, TextureFormat::Rg16Float)
        }
        DynamicImage::ImageRgb16(_) => {
            let rgba = image.to_rgba16();
            imagebuffer2texture(device_handler, &rgba, TextureFormat::Rgba16Float)
        }
        DynamicImage::ImageRgba16(buffer) => {
            imagebuffer2texture(device_handler, buffer, TextureFormat::Rgba16Float)
        }
    }
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
        depth: 1,
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
    let buffer = device.create_buffer_init(&BufferInitDescriptor {
        contents: bytemuck::cast_slice(&image_buffer),
        usage: BufferUsage::COPY_SRC,
        label: None,
    });
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
    encoder.copy_buffer_to_texture(
        BufferCopyView {
            buffer: &buffer,
            layout: TextureDataLayout {
                offset: 0,
                bytes_per_row: size.width * std::mem::size_of::<P>() as u32,
                rows_per_image: size.height,
            },
        },
        TextureCopyView {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
        },
        size,
    );
    queue.submit(vec![encoder.finish()]);
    texture
}
