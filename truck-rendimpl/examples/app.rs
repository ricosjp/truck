use std::sync::{Arc, Mutex};
use std::time::*;
use truck_platform::{wgpu::*, DeviceHandler};
use winit::dpi::*;
use winit::event::*;
use winit::event_loop::ControlFlow;

pub trait App: Sized + 'static {
    fn init(handler: &DeviceHandler) -> Self;
    fn clear_color() -> Color {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
    fn app_title<'a>() -> Option<&'a str> { None }
    fn depth_stencil_attachment_descriptor(
        &self,
    ) -> Option<RenderPassDepthStencilAttachmentDescriptor> {
        None
    }
    fn update(&mut self, _handler: &DeviceHandler) {}
    fn default_control_flow() -> ControlFlow {
        let next_frame_time = Instant::now() + Duration::from_nanos(16_666_667);
        ControlFlow::WaitUntil(next_frame_time)
    }
    fn render(&self, _frame: &SwapChainFrame) {}
    fn resized(&mut self, _size: PhysicalSize<u32>) -> ControlFlow { Self::default_control_flow() }
    fn moved(&mut self, _position: PhysicalPosition<i32>) -> ControlFlow {
        Self::default_control_flow()
    }
    fn closed_requested(&mut self) -> ControlFlow { ControlFlow::Exit }
    fn destroyed(&mut self) -> ControlFlow { Self::default_control_flow() }
    fn dropped_file(&mut self, _path: std::path::PathBuf) -> ControlFlow {
        Self::default_control_flow()
    }
    fn hovered_file(&mut self, _path: std::path::PathBuf) -> ControlFlow {
        Self::default_control_flow()
    }
    fn keyboard_input(&mut self, _input: KeyboardInput, _is_synthetic: bool) -> ControlFlow {
        Self::default_control_flow()
    }
    fn mouse_input(&mut self, _state: ElementState, _button: MouseButton) -> ControlFlow {
        Self::default_control_flow()
    }
    fn mouse_wheel(&mut self, _delta: MouseScrollDelta, _phase: TouchPhase) -> ControlFlow {
        Self::default_control_flow()
    }
    fn cursor_moved(&mut self, _position: PhysicalPosition<f64>) -> ControlFlow {
        Self::default_control_flow()
    }

    fn run() {
        let event_loop = winit::event_loop::EventLoop::new();
        let mut wb = winit::window::WindowBuilder::new();
        if let Some(title) = Self::app_title() {
            wb = wb.with_title(title);
        }
        let window = wb.build(&event_loop).unwrap();
        let size = window.inner_size();
        let instance = Instance::new(BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };

        let (device, queue) = futures::executor::block_on(init_device(&instance, &surface));

        let sc_desc = SwapChainDescriptor {
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            format: TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Mailbox,
        };

        let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);
        let handler = DeviceHandler::new(
            Arc::new(device),
            Arc::new(queue),
            Arc::new(Mutex::new(sc_desc)),
        );

        let mut app = Self::init(&handler);

        event_loop.run(move |ev, _, control_flow| {
            *control_flow = match ev {
                Event::MainEventsCleared => {
                    window.request_redraw();
                    Self::default_control_flow()
                }
                Event::RedrawRequested(_) => {
                    app.update(&handler);
                    let frame = swap_chain
                        .get_current_frame()
                        .expect("Timeout when acquiring next swap chain texture");
                    app.render(&frame);
                    Self::default_control_flow()
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        let mut sc_desc = handler.lock_sc_desc().unwrap();
                        sc_desc.width = size.width;
                        sc_desc.height = size.height;
                        swap_chain = handler.device().create_swap_chain(&surface, &sc_desc);
                        Self::default_control_flow()
                    }
                    WindowEvent::Moved(position) => app.moved(position),
                    WindowEvent::CloseRequested => app.closed_requested(),
                    WindowEvent::Destroyed => app.destroyed(),
                    WindowEvent::DroppedFile(path) => app.dropped_file(path),
                    WindowEvent::HoveredFile(path) => app.hovered_file(path),
                    WindowEvent::KeyboardInput {
                        input,
                        is_synthetic,
                        ..
                    } => app.keyboard_input(input, is_synthetic),
                    WindowEvent::MouseInput { state, button, .. } => app.mouse_input(state, button),
                    WindowEvent::MouseWheel { delta, phase, .. } => app.mouse_wheel(delta, phase),
                    WindowEvent::CursorMoved { position, .. } => app.cursor_moved(position),
                    _ => Self::default_control_flow(),
                },
                _ => Self::default_control_flow(),
            };
        })
    }
}

async fn init_device(instance: &Instance, surface: &Surface) -> (Device, Queue) {
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::Default,
            compatible_surface: Some(surface),
        })
        .await
        .unwrap();

    adapter
        .request_device(
            &DeviceDescriptor {
                features: Default::default(),
                limits: Limits::default(),
                shader_validation: true,
            },
            None,
        )
        .await
        .unwrap()
}

#[allow(dead_code)]
fn main() {}
