use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::error::EventLoopError;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Default)]
struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    // Called when it's safe to create windows
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title("LeoExplorer")
            .with_inner_size(LogicalSize::new(1100.0, 800.0))
            .with_min_inner_size(LogicalSize::new(600.0, 400.0))
            .with_decorations(true); // false = frameless, draw your own title bar

        self.window = Some(event_loop.create_window(attrs).expect("failed to create window"));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                // TODO: render your PDF page here
            }
            WindowEvent::Resized(_size) => {
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            _ => {}
        }
    }
}

pub fn run() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait); // sleep until an event arrives
    event_loop.run_app(&mut App::default())
}