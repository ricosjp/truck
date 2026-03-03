//! A GUI framework module providing MFC-like API.

// Copyright © 2021 RICOS
// Apache license 2.0
#![allow(deprecated)]

pub use async_trait::async_trait;
use std::sync::Arc;
//use std::time::Duration;
use winit::dpi::*;
use winit::event::*;
use winit::event_loop::{ActiveEventLoop, ControlFlow as WControlFlow};
use winit::window::Window;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

#[allow(unused)]
#[derive(Clone, Copy, Debug)]
pub enum ControlFlow {
    Poll,
    Exit,
    Wait,
    WaitUntil(Instant),
}

impl TryFrom<ControlFlow> for WControlFlow {
    type Error = ();
    fn try_from(value: ControlFlow) -> Result<Self, Self::Error> {
        match value {
            ControlFlow::Exit => Err(()),
            ControlFlow::Poll => Ok(WControlFlow::Poll),
            ControlFlow::Wait => Ok(WControlFlow::Wait),
            ControlFlow::WaitUntil(x) => Ok(WControlFlow::WaitUntil(x)),
        }
    }
}

/// The framework of applications with `winit`.
/// The main function of this file is the smallest usecase of this trait.
#[async_trait(?Send)]
pub trait App: Sized + 'static {
    /// Initialize application
    /// # Arguments
    /// - handler: `DeviceHandler` provided by `wgpu`
    /// - info: information of device and backend
    async fn init(window: Arc<Window>) -> Self;
    /// By overriding this function, you can change the display of the title bar.
    /// It is not possible to change the window while it is running.
    fn app_title<'a>() -> Option<&'a str> { None }
    /// Default is `ControlFlow::WaitUntil(1 / 60 seconds)`.
    fn default_control_flow() -> ControlFlow {
        /*
        let next_frame_time = Instant::now() + Duration::from_nanos(16_666_667);
        ControlFlow::WaitUntil(next_frame_time)
        */
        ControlFlow::Poll
    }
    /// By overriding this function, one can set the rendering process for each frame.
    fn render(&mut self) {}
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
    fn keyboard_input(&mut self, _input: KeyEvent, _is_synthetic: bool) -> ControlFlow {
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
    /// Run the application in the future.
    async fn async_run() {
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        let mut wa = winit::window::Window::default_attributes();
        if let Some(title) = Self::app_title() {
            wa.title = title.to_owned();
        }
        let window = event_loop
            .create_window(wa)
            .expect("failed to build window");
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
                    let canvas = window.canvas()?;
                    canvas.set_width(500);
                    canvas.set_height(500);
                    body.append_child(&web_sys::Element::from(canvas)).ok()
                })
                .expect("couldn't append canvas to document body");
        }

        let window = Arc::new(window);
        let mut app = Self::init(Arc::clone(&window)).await;

        let routine = move |ev: Event<()>, target: &ActiveEventLoop| {
            let control_flow = match ev {
                Event::NewEvents(_) => {
                    window.request_redraw();
                    Self::default_control_flow()
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        app.resized(size);
                        Self::default_control_flow()
                    }
                    WindowEvent::RedrawRequested => {
                        app.render();
                        Self::default_control_flow()
                    }
                    WindowEvent::Moved(position) => app.moved(position),
                    WindowEvent::CloseRequested => app.closed_requested(),
                    WindowEvent::Destroyed => app.destroyed(),
                    WindowEvent::DroppedFile(path) => app.dropped_file(path),
                    WindowEvent::HoveredFile(path) => app.hovered_file(path),
                    WindowEvent::KeyboardInput {
                        event,
                        is_synthetic,
                        ..
                    } => app.keyboard_input(event, is_synthetic),
                    WindowEvent::MouseInput { state, button, .. } => app.mouse_input(state, button),
                    WindowEvent::MouseWheel { delta, phase, .. } => app.mouse_wheel(delta, phase),
                    WindowEvent::CursorMoved { position, .. } => app.cursor_moved(position),
                    _ => Self::default_control_flow(),
                },
                _ => Self::default_control_flow(),
            };
            match control_flow {
                ControlFlow::Exit => target.exit(),
                _ => target.set_control_flow(control_flow.try_into().unwrap()),
            }
        };
        #[cfg(not(target_arch = "wasm32"))]
        event_loop.run(routine).unwrap();
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::EventLoopExtWebSys;
            event_loop.spawn(routine);
        }
    }
    /// Run the application.
    #[inline]
    fn run() { block_on(Self::async_run()) }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn block_on<F: core::future::Future<Output = ()> + 'static>(f: F) { pollster::block_on(f); }

#[cfg(target_arch = "wasm32")]
pub fn block_on<F: core::future::Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

/// The smallest example of the trait `App`.
/// Creates an empty window whose back ground is black.
#[allow(dead_code)]
fn main() {
    struct MyApp;
    #[async_trait(?Send)]
    impl App for MyApp {
        async fn init(_: Arc<Window>) -> Self { MyApp }
    }
    MyApp::run()
}
