use winit::dpi::PhysicalSize;
use winit::event::MouseScrollDelta;
use winit::keyboard::KeyCode;

use crate::state::AppState;
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
                self.is_panning = false;
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

        if !in_canvas && !self.is_panning && !self.is_moving_selection && !self.is_drag_selecting {
            return;
        }

        self.mouse_ndc = [
            ((x as f32 - canvas.min.x) / canvas.width()) * 2.0 - 1.0,
            ((y as f32 - canvas.min.y) / canvas.height()) * -2.0 + 1.0,
        ];

        if self.is_panning {
            let aspect = self.canvas_aspect();
            let dx_ndc = self.mouse_ndc[0] - self.pan_start_ndc[0];
            let dy_ndc = self.mouse_ndc[1] - self.pan_start_ndc[1];
            self.camera.position[0] = self.pan_start_camera[0] - dx_ndc * aspect / self.camera.zoom;
            self.camera.position[1] = self.pan_start_camera[1] - dy_ndc / self.camera.zoom;
            self.update_camera_buffer();
            self.window.request_redraw();
        } else if self.is_drag_selecting {
            let world = self.camera.ndc_to_world(self.mouse_ndc, self.canvas_aspect());
            self.drag_select_end = world;
            self.window.request_redraw();
        } else if self.is_moving_selection {
            let world = self.camera.ndc_to_world(self.mouse_ndc, self.canvas_aspect());
            let dx = world[0] - self.move_last_world[0];
            let dy = world[1] - self.move_last_world[1];
            for &idx in &self.selected_indices {
                if idx < self.block_positions.len() {
                    self.block_positions[idx].position[0] += dx;
                    self.block_positions[idx].position[1] += dy;
                }
            }
            self.move_last_world = world;
            self.window.request_redraw();
        }
    }

    pub fn handle_mouse_button(&mut self, pressed: bool) {
        if pressed && !self.is_pointer_in_canvas() {
            return;
        }

        if pressed {
            if self.space_pressed {
                self.is_panning = true;
                self.pan_start_ndc = self.mouse_ndc;
                self.pan_start_camera = self.camera.position;
            } else {
                let mouse_world = self.camera.ndc_to_world(self.mouse_ndc, self.canvas_aspect());

                // 클릭한 위치에 노드가 있는지 확인
                let clicked_node = self.block_positions.iter().position(|pos| {
                    let dx = (pos.position[0] - mouse_world[0]).abs();
                    let dy = (pos.position[1] - mouse_world[1]).abs();
                    dx < CARD_HALF_W && dy < CARD_HALF_H
                });

                if let Some(idx) = clicked_node {
                    if self.selected_indices.contains(&idx) {
                        // 이미 선택된 노드 위 클릭 → 그룹 이동 시작
                        self.is_moving_selection = true;
                        self.move_last_world = mouse_world;
                    } else {
                        // 비선택 노드 클릭 → 해당 노드만 선택 후 이동 시작
                        self.selected_indices.clear();
                        self.selected_indices.push(idx);
                        self.is_moving_selection = true;
                        self.move_last_world = mouse_world;
                    }
                } else {
                    // 빈 공간 클릭 → 드래그 선택 시작
                    self.selected_indices.clear();
                    self.is_drag_selecting = true;
                    self.drag_select_start = mouse_world;
                    self.drag_select_end = mouse_world;
                }

                self.window.request_redraw();
            }
        } else {
            if self.is_drag_selecting {
                // 드래그 선택 완료 → 사각형 안의 노드 전부 선택
                let min_x = self.drag_select_start[0].min(self.drag_select_end[0]);
                let max_x = self.drag_select_start[0].max(self.drag_select_end[0]);
                let min_y = self.drag_select_start[1].min(self.drag_select_end[1]);
                let max_y = self.drag_select_start[1].max(self.drag_select_end[1]);

                self.selected_indices.clear();
                for (i, pos) in self.block_positions.iter().enumerate() {
                    let [px, py] = pos.position;
                    if px >= min_x && px <= max_x && py >= min_y && py <= max_y {
                        self.selected_indices.push(i);
                    }
                }

                self.is_drag_selecting = false;
                self.window.request_redraw();
            }

            self.is_moving_selection = false;
            self.is_panning = false;
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
