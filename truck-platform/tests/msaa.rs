mod common;
use common::*;
use std::sync::{Arc, Mutex};
use truck_base::cgmath64::*;
use truck_platform::*;
use wgpu::*;

pub const PICTURE_WIDTH: u32 = 1024;
pub const PICTURE_HEIGHT: u32 = 1024;

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
    let mut scene = Scene::new(handler.clone(), &Default::default());
    let mut plane = new_plane!("shaders/plane.vert", "shaders/unicolor.frag");
    render_one(&mut scene, &texture0, &mut plane);
    let mut plane = new_plane!("shaders/bindgroup.vert", "shaders/bindgroup.frag");
    render_one(&mut scene, &texture1, &mut plane);
    let mut plane = new_plane!("shaders/bindgroup.vert", "shaders/anti-bindgroup.frag");
    assert!(common::same_texture(&handler, &texture0, &texture1));
}

