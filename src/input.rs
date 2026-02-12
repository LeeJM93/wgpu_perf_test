use winit::dpi::PhysicalSize;
use winit::event::MouseScrollDelta;
use winit::keyboard::KeyCode;

use crate::state::{AppState, InteractionMode};
use crate::types::*;

impl AppState {
    fn is_pointer_in_canvas(&self) -> bool {
        let canvas = self.egui.canvas_rect;
        let [mx, my] = self.mouse_pixel;
        canvas.width() > 0.0
            && canvas.height() > 0.0
            && mx >= canvas.min.x
            && mx <= canvas.max.x
            && my >= canvas.min.y
            && my <= canvas.max.y
    }

    pub fn handle_keyboard(&mut self, key: KeyCode, pressed: bool) {
        if key == KeyCode::Space {
            self.space_pressed = pressed;
            if !pressed {
                if matches!(self.interaction, InteractionMode::Panning { .. }) {
                    self.interaction = InteractionMode::Idle;
                }
            }
        }
    }

    pub fn handle_cursor_moved(&mut self, x: f64, y: f64) {
        self.mouse_pixel = [x as f32, y as f32];

        let canvas = self.egui.canvas_rect;
        if canvas.width() <= 0.0 || canvas.height() <= 0.0 {
            return;
        }

        let in_canvas = x as f32 >= canvas.min.x
            && x as f32 <= canvas.max.x
            && y as f32 >= canvas.min.y
            && y as f32 <= canvas.max.y;

        let is_active = !matches!(self.interaction, InteractionMode::Idle);
        if !in_canvas && !is_active {
            return;
        }

        self.mouse_ndc = [
            ((x as f32 - canvas.min.x) / canvas.width()) * 2.0 - 1.0,
            ((y as f32 - canvas.min.y) / canvas.height()) * -2.0 + 1.0,
        ];

        match &self.interaction {
            InteractionMode::Panning {
                start_ndc,
                start_camera,
            } => {
                let aspect = self.canvas_aspect();
                let dx_ndc = self.mouse_ndc[0] - start_ndc[0];
                let dy_ndc = self.mouse_ndc[1] - start_ndc[1];
                self.camera.position[0] = start_camera[0] - dx_ndc * aspect / self.camera.zoom;
                self.camera.position[1] = start_camera[1] - dy_ndc / self.camera.zoom;
                self.update_camera_buffer();
                self.window.request_redraw();
            }
            InteractionMode::DragSelecting { start, .. } => {
                let world = self.camera.ndc_to_world(self.mouse_ndc, self.canvas_aspect());
                let start = *start;
                self.interaction = InteractionMode::DragSelecting {
                    start,
                    end: world,
                };
                self.window.request_redraw();
            }
            InteractionMode::MovingSelection { last_world } => {
                let world = self.camera.ndc_to_world(self.mouse_ndc, self.canvas_aspect());
                let dx = world[0] - last_world[0];
                let dy = world[1] - last_world[1];
                for &idx in &self.selected_indices {
                    if idx < self.block_positions.len() {
                        self.block_positions[idx].position[0] += dx;
                        self.block_positions[idx].position[1] += dy;
                    }
                }
                self.interaction = InteractionMode::MovingSelection { last_world: world };
                self.mark_positions_dirty();
                self.window.request_redraw();
            }
            InteractionMode::Idle => {}
        }
    }

    pub fn handle_mouse_button(&mut self, pressed: bool) {
        if pressed && !self.is_pointer_in_canvas() {
            return;
        }

        if pressed {
            if self.space_pressed {
                self.interaction = InteractionMode::Panning {
                    start_ndc: self.mouse_ndc,
                    start_camera: self.camera.position,
                };
            } else {
                let mouse_world = self.camera.ndc_to_world(self.mouse_ndc, self.canvas_aspect());

                // 클릭한 위치에 노드가 있는지 확인
                let clicked_node = self.block_positions.iter().position(|pos| {
                    let dx = (pos.position[0] - mouse_world[0]).abs();
                    let dy = (pos.position[1] - mouse_world[1]).abs();
                    dx < CARD_HALF_W && dy < CARD_HALF_H
                });

                if let Some(idx) = clicked_node {
                    if !self.selected_indices.contains(&idx) {
                        self.selected_indices.clear();
                        self.selected_indices.push(idx);
                    }
                    self.interaction = InteractionMode::MovingSelection {
                        last_world: mouse_world,
                    };
                } else {
                    // 빈 공간 클릭 → 드래그 선택 시작
                    self.selected_indices.clear();
                    self.interaction = InteractionMode::DragSelecting {
                        start: mouse_world,
                        end: mouse_world,
                    };
                }

                self.window.request_redraw();
            }
        } else {
            if let InteractionMode::DragSelecting { start, end } = &self.interaction {
                // 드래그 선택 완료 → 사각형 안의 노드 전부 선택
                let min_x = start[0].min(end[0]);
                let max_x = start[0].max(end[0]);
                let min_y = start[1].min(end[1]);
                let max_y = start[1].max(end[1]);

                self.selected_indices.clear();
                for (i, pos) in self.block_positions.iter().enumerate() {
                    let [px, py] = pos.position;
                    if px >= min_x && px <= max_x && py >= min_y && py <= max_y {
                        self.selected_indices.push(i);
                    }
                }
                self.window.request_redraw();
            }

            self.interaction = InteractionMode::Idle;
        }
    }

    pub fn handle_scroll(&mut self, delta: MouseScrollDelta) {
        if !self.is_pointer_in_canvas() {
            return;
        }

        let scroll_y = match delta {
            MouseScrollDelta::LineDelta(_, y) => y,
            MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 50.0,
        };

        let factor = if scroll_y > 0.0 { 1.1 } else { 1.0 / 1.1 };
        self.camera
            .zoom_at(self.mouse_ndc, factor, self.canvas_aspect());
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
