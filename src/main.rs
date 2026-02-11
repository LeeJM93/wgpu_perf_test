mod camera;
mod egui_integration;
mod input;
mod pipeline;
mod renderer;
mod state;
mod types;
mod ui;

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::PhysicalKey,
    window::WindowId,
};

use state::AppState;

struct App {
    state: Option<AppState>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(
                    winit::window::WindowAttributes::default().with_title("Weaving — Canvas"),
                )
                .unwrap(),
        );

        self.state = Some(AppState::new(window));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(state) = self.state.as_mut() else {
            return;
        };

        // egui에 먼저 이벤트 전달
        let _ = state
            .egui
            .winit_state
            .on_window_event(&state.window, &event);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_keyboard(code, key_state == ElementState::Pressed),

            WindowEvent::CursorMoved { position, .. } => {
                state.handle_cursor_moved(position.x, position.y);
            }

            WindowEvent::MouseInput {
                state: button_state,
                button: MouseButton::Left,
                ..
            } => state.handle_mouse_button(button_state == ElementState::Pressed),

            WindowEvent::MouseWheel { delta, .. } => state.handle_scroll(delta),

            WindowEvent::Resized(new_size) => state.handle_resize(new_size),

            WindowEvent::RedrawRequested => state.render(),

            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App { state: None };
    event_loop.run_app(&mut app).unwrap();
}
