use crate::*;
use glium::glutin::dpi::*;
use glium::glutin::event::*;
use glium::glutin::event_loop::ControlFlow;
use glium::Display;
use std::time::*;

pub trait Render: Sized + 'static {
    fn create_display(event_loop: &glium::glutin::event_loop::EventLoop<()>) -> glium::Display {
        let cb = glium::glutin::ContextBuilder::new();
        let wb = glium::glutin::window::WindowBuilder::new();
        glium::Display::new(wb, cb, &event_loop).unwrap()
    }
    fn init(display: &Display) -> Self;
    fn update(&mut self, _display: &Display) {}
    fn default_control_flow() -> ControlFlow {
        let next_frame_time = Instant::now() + Duration::from_nanos(16_666_667);
        glium::glutin::event_loop::ControlFlow::WaitUntil(next_frame_time)
    }
    fn draw(&mut self, _target: &mut glium::Frame) {}
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
        let event_loop = glium::glutin::event_loop::EventLoop::new();
        let display = Self::create_display(&event_loop);
        let mut renderer = Self::init(&display);

        event_loop.run(move |ev, _, control_flow| {
            *control_flow = match ev {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => renderer.resized(size),
                    WindowEvent::Moved(position) => renderer.moved(position),
                    WindowEvent::CloseRequested => renderer.closed_requested(),
                    WindowEvent::Destroyed => renderer.destroyed(),
                    WindowEvent::DroppedFile(path) => renderer.dropped_file(path),
                    WindowEvent::HoveredFile(path) => renderer.hovered_file(path),
                    WindowEvent::KeyboardInput {
                        input,
                        is_synthetic,
                        ..
                    } => renderer.keyboard_input(input, is_synthetic),
                    WindowEvent::MouseInput { state, button, .. } => {
                        renderer.mouse_input(state, button)
                    }
                    WindowEvent::MouseWheel { delta, phase, .. } => {
                        renderer.mouse_wheel(delta, phase)
                    }
                    WindowEvent::CursorMoved { position, .. } => renderer.cursor_moved(position),
                    _ => Self::default_control_flow(),
                },
                _ => Self::default_control_flow(),
            };
            renderer.update(&display);
            let mut target = display.draw();
            renderer.draw(&mut target);
            target.finish().unwrap();
        })
    }
}
