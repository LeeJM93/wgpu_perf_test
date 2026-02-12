use crate::state::{AppState, InteractionMode};
use crate::types::*;
use crate::ui;

struct EguiFrameResult {
    full_output: egui::FullOutput,
    toolbar_action: ui::toolbar::ToolbarAction,
}

impl AppState {
    pub fn render(&mut self) {
        self.update_camera_buffer();

        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // 1) egui UI 구축
        let frame_result = self.build_egui_frame();

        // 2) 툴바 액션 처리
        self.apply_toolbar_actions(&frame_result.toolbar_action);

        // 3) egui 텍스처/버퍼 업데이트
        self.egui
            .winit_state
            .handle_platform_output(&self.window, frame_result.full_output.platform_output);

        let paint_jobs = self.egui.ctx.tessellate(
            frame_result.full_output.shapes,
            self.egui.ctx.pixels_per_point(),
        );

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        for (id, delta) in &frame_result.full_output.textures_delta.set {
            self.egui
                .renderer
                .update_texture(&self.device, &self.queue, *id, delta);
        }
        self.egui.renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        self.update_gpu_buffers();

        // 4) wgpu 캔버스 렌더패스
        self.render_canvas_pass(&mut encoder, &view);

        // 5) egui 렌더패스
        self.render_egui_pass(&mut encoder, &view, &paint_jobs, &screen_descriptor);

        self.queue.submit(std::iter::once(encoder.finish()));

        for id in &frame_result.full_output.textures_delta.free {
            self.egui.renderer.free_texture(id);
        }

        output.present();

        // egui가 재렌더를 요청할 때만 다시 그리기
        let needs_repaint = frame_result
            .full_output
            .viewport_output
            .values()
            .any(|v| v.repaint_delay.is_zero());
        if needs_repaint {
            self.window.request_redraw();
        }
    }

    fn build_egui_frame(&mut self) -> EguiFrameResult {
        let raw_input = self.egui.winit_state.take_egui_input(&self.window);
        let ctx = self.egui.ctx.clone();

        let mut canvas_rect = self.egui.canvas_rect;
        let mut top_bar_state = std::mem::take(&mut self.top_bar_state);
        let mut left_tab_state = std::mem::take(&mut self.left_tab_state);
        let mut inspector_state = std::mem::take(&mut self.inspector_state);
        let camera_position = self.camera.position;
        let camera_zoom = self.camera.zoom;
        let drag_select = match self.interaction {
            InteractionMode::DragSelecting { start, end } => Some((start, end)),
            _ => None,
        };
        let mut toolbar_action = ui::toolbar::ToolbarAction::default();

        let full_output = ctx.run(raw_input, |ctx| {
            egui::TopBottomPanel::top("top_bar")
                .exact_height(TOP_BAR_HEIGHT)
                .frame(ui::top_bar::frame())
                .show(ctx, |ui| {
                    ui::top_bar::show(ui, &mut top_bar_state);
                });

            egui::SidePanel::left("left_tab")
                .exact_width(LEFT_TAB_WIDTH)
                .resizable(false)
                .frame(ui::left_tab::frame())
                .show(ctx, |ui| {
                    ui::left_tab::show(ui, &mut left_tab_state);
                });

            if inspector_state.open {
                egui::SidePanel::right("inspector")
                    .exact_width(INSPECTOR_WIDTH)
                    .resizable(true)
                    .frame(ui::inspector::frame())
                    .show(ctx, |ui| {
                        ui::inspector::show(ui, &mut inspector_state);
                    });
            }

            egui::CentralPanel::default()
                .frame(egui::Frame::new().fill(egui::Color32::TRANSPARENT))
                .show(ctx, |ui| {
                    canvas_rect = ui.available_rect_before_wrap();

                    toolbar_action = ui::toolbar::show(ctx, canvas_rect);

                    // 드래그 선택 사각형
                    if let Some((sel_start, sel_end)) = drag_select {
                        let aspect = if canvas_rect.width() > 0.0 && canvas_rect.height() > 0.0 {
                            canvas_rect.width() / canvas_rect.height()
                        } else {
                            1.0
                        };

                        let ndc_to_screen = |ndc: [f32; 2]| -> egui::Pos2 {
                            egui::pos2(
                                canvas_rect.min.x + (ndc[0] + 1.0) * 0.5 * canvas_rect.width(),
                                canvas_rect.min.y + (-ndc[1] + 1.0) * 0.5 * canvas_rect.height(),
                            )
                        };

                        let ndc_start = [
                            (sel_start[0] - camera_position[0]) * camera_zoom / aspect,
                            (sel_start[1] - camera_position[1]) * camera_zoom,
                        ];
                        let ndc_end = [
                            (sel_end[0] - camera_position[0]) * camera_zoom / aspect,
                            (sel_end[1] - camera_position[1]) * camera_zoom,
                        ];

                        let p1 = ndc_to_screen(ndc_start);
                        let p2 = ndc_to_screen(ndc_end);

                        let select_rect = egui::Rect::from_two_pos(p1, p2);
                        let painter = ui.painter();
                        painter.rect_filled(
                            select_rect,
                            0.0,
                            egui::Color32::from_rgba_premultiplied(59, 130, 246, 30),
                        );
                        painter.rect_stroke(
                            select_rect,
                            egui::CornerRadius::ZERO,
                            egui::Stroke::new(1.5, egui::Color32::from_rgb(59, 130, 246)),
                            egui::StrokeKind::Outside,
                        );
                    }

                    ui::ai_button::show(ctx, canvas_rect);
                });
        });

        // 상태 복원
        self.egui.canvas_rect = canvas_rect;
        self.top_bar_state = top_bar_state;
        self.left_tab_state = left_tab_state;
        self.inspector_state = inspector_state;

        EguiFrameResult {
            full_output,
            toolbar_action,
        }
    }

    fn apply_toolbar_actions(&mut self, action: &ui::toolbar::ToolbarAction) {
        if action.add_node {
            let color = CARD_COLORS[self.block_positions.len() % CARD_COLORS.len()];
            self.block_positions.push(InstanceRaw {
                position: self.camera.position,
                color,
            });
            self.positions_dirty = true;
        }

        if action.add_batch > 0 {
            let start = self.block_positions.len();
            let cols = (action.add_batch as f32).sqrt().ceil() as usize;
            for i in 0..action.add_batch {
                let col = (i % cols) as f32;
                let row = (i / cols) as f32;
                let color = CARD_COLORS[(start + i) % CARD_COLORS.len()];
                self.block_positions.push(InstanceRaw {
                    position: [
                        self.camera.position[0] + col * GRID_SPACING_X,
                        self.camera.position[1] + row * GRID_SPACING_Y,
                    ],
                    color,
                });
            }
            self.positions_dirty = true;
        }

        if action.reset {
            self.block_positions = create_default_grid();
            self.positions_dirty = true;
        }
    }

    fn render_canvas_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let canvas = self.egui.canvas_rect;
        let scale = self.window.scale_factor() as f32;

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Canvas Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });

        if canvas.width() > 0.0 && canvas.height() > 0.0 {
            let vp_x = (canvas.min.x * scale).floor();
            let vp_y = (canvas.min.y * scale).floor();
            let vp_w = (canvas.width() * scale).ceil();
            let vp_h = (canvas.height() * scale).ceil();

            rpass.set_viewport(vp_x, vp_y, vp_w, vp_h, 0.0, 1.0);
            rpass.set_scissor_rect(vp_x as u32, vp_y as u32, vp_w as u32, vp_h as u32);
        }

        rpass.set_bind_group(0, &self.camera_bind_group, &[]);

        rpass.set_pipeline(&self.line_pipeline);
        rpass.set_vertex_buffer(0, self.line_buffer.slice(..));
        rpass.draw(0..self.line_vertex_count, 0..1);

        rpass.set_pipeline(&self.card_pipeline);
        rpass.set_vertex_buffer(0, self.card_quad_buffer.slice(..));
        rpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        rpass.draw(0..4, 0..self.block_positions.len() as u32);
    }

    fn render_egui_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        paint_jobs: &[egui::ClippedPrimitive],
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
    ) {
        let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });

        let mut rpass = rpass.forget_lifetime();
        self.egui
            .renderer
            .render(&mut rpass, paint_jobs, screen_descriptor);
    }
}
