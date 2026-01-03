//#![windows_subsystem = "windows"]

mod engine;

use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes},
};

use crate::engine::Engine;

#[derive(Default)]
struct Application {
    window: Option<Arc<dyn Window>>,
    engine: Option<Engine>,
}

impl ApplicationHandler for Application {
    fn can_create_surfaces(&mut self, event_loop: &dyn winit::event_loop::ActiveEventLoop) {
        let window_attributes = WindowAttributes::default().with_title("Vulkan Engine");

        self.window = match event_loop.create_window(window_attributes) {
            Ok(window) => Some(Arc::from(window)),
            Err(_) => panic!("Failed to create window!"),
        };

        self.engine = Some(Engine::new(self.window.clone()));
    }

    fn window_event(
        &mut self,
        event_loop: &dyn winit::event_loop::ActiveEventLoop,
        _: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested
            | winit::event::WindowEvent::KeyboardInput {
                device_id: _,
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        logical_key: _,
                        text: _,
                        location: _,
                        state: ElementState::Pressed,
                        repeat: _,
                        text_with_all_modifiers: _,
                        key_without_modifiers: _,
                    },
                is_synthetic: _,
            } => {
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                let window = unsafe { self.window.as_ref().unwrap_unchecked() };

                if let Some(engine) = &mut self.engine {
                    engine.update();
                }

                window.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.run_app(Application::default()).unwrap();
}
