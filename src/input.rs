use winit::dpi::PhysicalSize;
use winit::event::MouseScrollDelta;
use winit::keyboard::KeyCode;

use crate::state::AppState;
use crate::types::*;

impl AppState {
    pub fn handle_keyboard(&mut self, key: KeyCode, pressed: bool) {
        if key == KeyCode::Space {
            self.space_pressed = pressed;
            if !pressed {
                self.is_panning = false;
            }
        }
    }

    pub fn handle_cursor_moved(&mut self, x: f64, y: f64) {
        let size = self.window.inner_size();
        self.mouse_ndc = [
            (x as f32 / size.width as f32) * 2.0 - 1.0,
            (y as f32 / size.height as f32) * -2.0 + 1.0,
        ];

        if self.is_panning {
            let aspect = self.aspect();
            let dx_ndc = self.mouse_ndc[0] - self.pan_start_ndc[0];
            let dy_ndc = self.mouse_ndc[1] - self.pan_start_ndc[1];
            self.camera.position[0] =
                self.pan_start_camera[0] - dx_ndc * aspect / self.camera.zoom;
            self.camera.position[1] = self.pan_start_camera[1] - dy_ndc / self.camera.zoom;
            self.update_camera_buffer();
            self.window.request_redraw();
        } else if let Some(idx) = self.selected_idx {
            let world = self.camera.ndc_to_world(self.mouse_ndc, self.aspect());
            self.block_positions[idx].position = world;
            self.window.request_redraw();
        }
    }

    pub fn handle_mouse_button(&mut self, pressed: bool) {
        if pressed {
            if self.space_pressed {
                self.is_panning = true;
                self.pan_start_ndc = self.mouse_ndc;
                self.pan_start_camera = self.camera.position;
            } else if self.mouse_ndc[0].abs() < ADD_BTN_HALF
                && (self.mouse_ndc[1] - ADD_BTN_Y).abs() < ADD_BTN_HALF
            {
                let color = CARD_COLORS[self.block_positions.len() % CARD_COLORS.len()];
                self.block_positions.push(InstanceRaw {
                    position: self.camera.position,
                    color,
                });
                self.window.request_redraw();
            } else {
                let mouse_world = self.camera.ndc_to_world(self.mouse_ndc, self.aspect());
                self.selected_idx = self.block_positions.iter().position(|pos| {
                    let dx = (pos.position[0] - mouse_world[0]).abs();
                    let dy = (pos.position[1] - mouse_world[1]).abs();
                    dx < CARD_HALF_W && dy < CARD_HALF_H
                });
            }
        } else {
            self.selected_idx = None;
            self.is_panning = false;
        }
    }

    pub fn handle_scroll(&mut self, delta: MouseScrollDelta) {
        let scroll_y = match delta {
            MouseScrollDelta::LineDelta(_, y) => y,
            MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 50.0,
        };

        let factor = if scroll_y > 0.0 { 1.1 } else { 1.0 / 1.1 };
        self.camera.zoom_at(self.mouse_ndc, factor, self.aspect());
        self.update_camera_buffer();
        self.window.request_redraw();
    }

    pub fn handle_resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.update_camera_buffer();
            self.window.request_redraw();
        }
    }
}
