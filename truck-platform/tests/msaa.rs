mod common;
use common::*;
use std::sync::{Arc, Mutex};
use truck_platform::*;
use wgpu::*;

pub const PICTURE_WIDTH: u32 = 512;
pub const PICTURE_HEIGHT: u32 = 512;

fn init_device_with_adptinfo(instance: &Instance) -> (Arc<Device>, Arc<Queue>, AdapterInfo) {
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
        (Arc::new(device), Arc::new(queue), adapter.get_info())
    })
}

#[test]
fn msaa_test() {
    let instance = Instance::new(BackendBit::PRIMARY);
    let (device, queue, info) = init_device_with_adptinfo(&instance);
    match info.backend {
        Backend::Vulkan => {}
        Backend::Dx12 => {}
        _ => {
            eprintln!("Backend: {:?} is not compatible with wgpu MSAA.", info.backend);
            return;
        }
    }
    let sc_desc = SwapChainDescriptor {
        usage: TextureUsage::OUTPUT_ATTACHMENT,
        format: TextureFormat::Rgba8UnormSrgb,
        width: PICTURE_WIDTH,
        height: PICTURE_HEIGHT,
        present_mode: PresentMode::Mailbox,
    };
    let texture0 = device.create_texture(&common::texture_descriptor(&sc_desc));
    let texture1 = device.create_texture(&common::texture_descriptor(&sc_desc));
    let sc_desc = Arc::new(Mutex::new(sc_desc));
    let handler = DeviceHandler::new(device, queue, sc_desc);
    let mut scene = Scene::new(handler.clone(), &SceneDescriptor {
        sample_count: 1,
        ..Default::default()
    });
    let plane = new_plane!("shaders/trapezoid.vert", "shaders/trapezoid.frag");
    render_one(&mut scene, &texture0, &plane);
    scene.descriptor_mut().sample_count = 2;
    let plane = new_plane!("shaders/trapezoid.vert", "shaders/trapezoid.frag");
    render_one(&mut scene, &texture1, &plane);
    assert!(!common::same_texture(&handler, &texture0, &texture1));
}
