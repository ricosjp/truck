//! A GUI framework module providing MFC-like API.

// Copyright © 2021 RICOS
// Apache license 2.0

use instant::Instant;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use truck_platform::{wgpu::*, DeviceHandler};
use winit::dpi::*;
use winit::event::*;
use winit::event_loop::ControlFlow;

/// The framework of applications with `winit`.
/// The main function of this file is the smallest usecase of this trait.
pub trait App: Sized + 'static {
    /// Initialize application
    /// # Arguments
    /// - handler: `DeviceHandler` provided by `wgpu`
    /// - info: informations of device and backend
    fn init(handler: &DeviceHandler, info: AdapterInfo) -> Self;
    /// By overriding this function, you can change the display of the title bar.
    /// It is not possible to change the window while it is running.
    fn app_title<'a>() -> Option<&'a str> { None }
    /// Default is `ControlFlow::WaitUntil(1 / 60 seconds)`.
    fn default_control_flow() -> ControlFlow {
        let next_frame_time = Instant::now() + Duration::from_nanos(16_666_667);
        ControlFlow::WaitUntil(next_frame_time)
    }
    /// By overriding this function, one can set the update process for each frame.
    fn update(&mut self, _handler: &DeviceHandler) {}
    /// By overriding this function, one can set the rendering process for each frame.
    fn render(&mut self, _frame: &TextureView) {}
    /// By overriding this function, one can change the behavior when the window is resized.
    fn resized(&mut self, _size: PhysicalSize<u32>) -> ControlFlow { Self::default_control_flow() }
    /// By overriding this function, one can change the behavior when the window is moved.
    fn moved(&mut self, _position: PhysicalPosition<i32>) -> ControlFlow {
        Self::default_control_flow()
    }
    /// By overriding this function, one can change the behavior when the X button is pushed.
    fn closed_requested(&mut self) -> ControlFlow { ControlFlow::Exit }
    /// By overriding this function, one can change the behavior when the window is destoroyed.
    fn destroyed(&mut self) -> ControlFlow { Self::default_control_flow() }
    /// By overriding this function, one can change the behavior when a file is dropped to the window.
    fn dropped_file(&mut self, _path: std::path::PathBuf) -> ControlFlow {
        Self::default_control_flow()
    }
    /// By overriding this function, one can change the behavior when a file is hovered to the window.
    fn hovered_file(&mut self, _path: std::path::PathBuf) -> ControlFlow {
        Self::default_control_flow()
    }
    /// By overriding this function, one can change the behavior when a keybourd input occurs.
    fn keyboard_input(&mut self, _input: KeyboardInput, _is_synthetic: bool) -> ControlFlow {
        Self::default_control_flow()
    }
    /// By overriding this function, one can change the behavior when a mouse input occurs.
    fn mouse_input(&mut self, _state: ElementState, _button: MouseButton) -> ControlFlow {
        Self::default_control_flow()
    }
    /// By overriding this function, one can change the behavior when a mouse wheel input occurs.
    fn mouse_wheel(&mut self, _delta: MouseScrollDelta, _phase: TouchPhase) -> ControlFlow {
        Self::default_control_flow()
    }
    /// By overriding this function, one can change the behavior when the cursor is moved.
    fn cursor_moved(&mut self, _position: PhysicalPosition<f64>) -> ControlFlow {
        Self::default_control_flow()
    }
    /// Run the application.
    fn run() {
        let event_loop = winit::event_loop::EventLoop::new();
        let mut wb = winit::window::WindowBuilder::new();
        if let Some(title) = Self::app_title() {
            wb = wb.with_title(title);
        }
        let window = wb.build(&event_loop).expect("failed to build window");
        #[cfg(not(target_arch = "wasm32"))]
        futures::executor::block_on(run::<Self>(event_loop, window));
        #[cfg(target_arch = "wasm32")]
        {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init().expect("could not initialize logger");
            use winit::platform::web::WindowExtWebSys;
            // On wasm, append the canvas to the document body
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| {
                    body.append_child(&web_sys::Element::from(window.canvas()))
                        .ok()
                })
                .expect("couldn't append canvas to document body");
            wasm_bindgen_futures::spawn_local(run::<Self>(event_loop, window));
        }
    }
}

async fn init_device(instance: &Instance, surface: &Surface) -> (Device, Queue, Adapter) {
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            #[cfg(not(feature = "webgl"))]
            power_preference: PowerPreference::HighPerformance,
            #[cfg(feature = "webgl")]
            power_preference: PowerPreference::LowPower,
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        })
        .await
        .expect("Failed to find an appropriate adapter");

    let tuple = adapter
        .request_device(
            &DeviceDescriptor {
                features: Default::default(),
                #[cfg(not(feature = "webgl"))]
                limits: Limits::downlevel_defaults().using_resolution(adapter.limits()),
                #[cfg(feature = "webgl")]
                limits: Limits::downlevel_webgl2_defaults(),
                label: None,
            },
            None,
        )
        .await
        .expect("Failed to create device");
    (tuple.0, tuple.1, adapter)
}

async fn run<A: App>(event_loop: winit::event_loop::EventLoop<()>, window: winit::window::Window) {
    let size = window.inner_size();
    #[cfg(not(feature = "webgl"))]
    let instance = Instance::new(Backends::PRIMARY);
    #[cfg(feature = "webgl")]
    let instance = Instance::new(Backends::all());
    let surface = unsafe { instance.create_surface(&window) };

    let (device, queue, adapter) = init_device(&instance, &surface).await;

    let config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_preferred_format(&adapter).unwrap(),
        width: size.width,
        height: size.height,
        present_mode: PresentMode::Mailbox,
    };

    let surface = unsafe { instance.create_surface(&window) };
    surface.configure(&device, &config);

    let handler = DeviceHandler::new(
        Arc::new(device),
        Arc::new(queue),
        Arc::new(Mutex::new(config)),
    );

    let mut app = A::init(&handler, adapter.get_info());

    event_loop.run(move |ev, _, control_flow| {
        *control_flow = match ev {
            Event::MainEventsCleared => {
                window.request_redraw();
                A::default_control_flow()
            }
            Event::RedrawRequested(_) => {
                app.update(&handler);
                let surface_texture = match surface.get_current_texture() {
                    Ok(got) => got,
                    Err(_) => {
                        surface.configure(handler.device(), &handler.config());
                        surface
                            .get_current_texture()
                            .expect("Failed to acquire next surface texture!")
                    }
                };
                let view = surface_texture
                    .texture
                    .create_view(&TextureViewDescriptor::default());
                app.render(&view);
                surface_texture.present();
                A::default_control_flow()
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    let mut config = handler.lock_config().unwrap();
                    config.width = size.width;
                    config.height = size.height;
                    surface.configure(handler.device(), &config);
                    A::default_control_flow()
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
                _ => A::default_control_flow(),
            },
            _ => A::default_control_flow(),
        };
    })
}

/// The smallest example of the trait `App`.
/// Creates an empty window whose back ground is black.
#[allow(dead_code)]
fn main() {
    struct MyApp;
    impl App for MyApp {
        fn init(_: &DeviceHandler, _: AdapterInfo) -> MyApp { MyApp }
    }
    MyApp::run()
}
