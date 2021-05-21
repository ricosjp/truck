use std::sync::{Arc, Mutex};
use std::f64::consts::PI;
use std::convert::TryInto;
use truck_meshalgo::tessellation::*;
use truck_platform::*;
use truck_polymesh::*;
use truck_rendimpl::*;
use truck_geometry::specifieds::Plane;
use wgpu::{*, Instance};

fn init_device(instance: &Instance) -> (Arc<Device>, Arc<Queue>) {
    futures::executor::block_on(async {
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
            })
            .await
            .unwrap();
        println!("{:?}", adapter.get_info());
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features: Default::default(),
                    limits: Default::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();
        (Arc::new(device), Arc::new(queue))
    })
}

fn texture_descriptor(sc_desc: &SwapChainDescriptor) -> TextureDescriptor<'static> {
    TextureDescriptor {
        label: None,
        size: Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: sc_desc.format,
        usage: TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
    }
}

fn texture_copy_view<'a>(texture: &'a Texture) -> ImageCopyTexture<'a> {
    ImageCopyTexture {
        texture: &texture,
        mip_level: 0,
        origin: Origin3d::ZERO,
    }
}

fn buffer_copy_view<'a>(buffer: &'a Buffer, size: (u32, u32)) -> ImageCopyBuffer<'a> {
    ImageCopyBuffer {
        buffer: &buffer,
        layout: ImageDataLayout {
            offset: 0,
            bytes_per_row: (size.0 * 4).try_into().ok(),
            rows_per_image: size.1.try_into().ok(),
        },
    }
}

fn read_buffer(device: &Device, buffer: &Buffer) -> Vec<u8> {
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

fn read_texture(handler: &DeviceHandler, texture: &Texture) -> Vec<u8> {
    let (device, queue, sc_desc) = (handler.device(), handler.queue(), handler.sc_desc());
    let size = (sc_desc.width * sc_desc.height * 4) as u64;
    let buffer = device.create_buffer(&BufferDescriptor {
        label: None,
        mapped_at_creation: false,
        usage: BufferUsage::COPY_DST | BufferUsage::MAP_READ,
        size,
    });
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
    encoder.copy_texture_to_buffer(
        texture_copy_view(&texture),
        buffer_copy_view(&buffer, (sc_desc.width, sc_desc.height)),
        Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));
    read_buffer(device, &buffer)
}

fn save_buffer<P: AsRef<std::path::Path>>(path: P, vec: &Vec<u8>, size: (u32, u32)) {
    image::save_buffer(path, &vec, size.0, size.1, image::ColorType::Rgba8).unwrap();
}


fn main() {
    let plane = Plane::new(Point3::origin(), Point3::new(1.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0));
    let mut positions = Vec::new();
    let mut polyline = Vec::new();
    const N: usize = 50;
    for i in 0..N {
        let theta = 2.0 * PI * i as f64 / N as f64;
        positions.push(Vector2::new(f64::cos(theta), f64::sin(theta)));
        polyline.push([i, (i + 1) % N]);
    }
    positions.push(Vector2::new(0.25, 0.25));
    positions.push(Vector2::new(0.25, -0.25));
    positions.push(Vector2::new(-0.25, -0.25));
    positions.push(Vector2::new(-0.25, 0.25));
    polyline.push([N, N + 1]);
    polyline.push([N + 1, N + 2]);
    polyline.push([N + 2, N + 3]);
    polyline.push([N + 3, N]);
    for i in 0..(N / 2) {
        let theta = 2.0 * PI * i as f64 / (N / 2) as f64;
        positions.push(0.75 * Vector2::new(f64::cos(theta), f64::sin(theta)));
    } 
    let indices = triangulation::delaunay_2d(&positions, &polyline);
    let polygon = triangulation::create_polymesh(&plane, &positions, &indices);
    println!("{:?}", polygon);
 
    let instance = wgpu::Instance::new(BackendBit::PRIMARY);
    let (device, queue) = init_device(&instance);
    let sc_desc = SwapChainDescriptor {
        usage: TextureUsage::RENDER_ATTACHMENT,
        format: TextureFormat::Rgba8Unorm,
        width: 512,
        height: 512,
        present_mode: PresentMode::Mailbox,
    };
    let screen = device.create_texture(&texture_descriptor(&sc_desc));
    let camera = Camera::parallel_camera(
        Matrix4::look_at_rh(
            Point3::new(0.0, 0.0, 1.0),
            Point3::origin(),
            Vector3::unit_y(),
        ).invert().unwrap(),
        2.0,
        0.1,
        10.0,
    );
    let light = Light {
        position: Point3::new(0.0, 0.0, 1.0),
        color: Vector3::new(1.0, 1.0, 1.0),
        light_type: LightType::Uniform,
    };
    let mut scene = Scene::new(
        DeviceHandler::new(device, queue, Arc::new(Mutex::new(sc_desc))),
        &SceneDescriptor {
            camera,
            lights: vec![light],
            ..Default::default()
        },
    );
    
    let instance: WireFrameInstance = scene.instance_creator().create_instance(&polygon, &Default::default());
    scene.add_object(&instance);
    scene.render_scene(&screen.create_view(&Default::default()));
    let buffer = read_texture(scene.device_handler(), &screen);
    save_buffer("output.png", &buffer, (512, 512));
}
