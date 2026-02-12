use crate::state::AppState;
use crate::types::*;
use crate::ui;

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

        // ===== egui 프레임 실행 =====
        let raw_input = self.egui.winit_state.take_egui_input(&self.window);
        let ctx = self.egui.ctx.clone();

        // build_ui에 필요한 상태를 빌려서 클로저에 전달
        let mut canvas_rect = self.egui.canvas_rect;
        let mut top_bar_state = std::mem::take(&mut self.top_bar_state);
        let mut left_tab_state = std::mem::take(&mut self.left_tab_state);
        let mut inspector_state = std::mem::take(&mut self.inspector_state);
        let camera_position = self.camera.position;
        let camera_zoom = self.camera.zoom;
        let is_drag_selecting = self.is_drag_selecting;
        let drag_select_start = self.drag_select_start;
        let drag_select_end = self.drag_select_end;
        let mut toolbar_action = ui::toolbar::ToolbarAction::default();

        let full_output = ctx.run(raw_input, |ctx| {
            // TopBar (55px)
            egui::TopBottomPanel::top("top_bar")
                .exact_height(55.0)
                .frame(
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgba_premultiplied(244, 244, 244, 210))
                        .stroke(egui::Stroke::new(
                            0.5,
                            egui::Color32::from_rgba_premultiplied(0, 0, 0, 50),
                        ))
                        .inner_margin(egui::Margin::ZERO),
                )
                .show(ctx, |ui| {
                    ui::top_bar::show(ui, &mut top_bar_state);
                });

            // Left Tab (~75px)
            egui::SidePanel::left("left_tab")
                .exact_width(75.0)
                .resizable(false)
                .frame(
                    egui::Frame::new()
                        .fill(egui::Color32::TRANSPARENT)
                        .inner_margin(egui::Margin::ZERO),
                )
                .show(ctx, |ui| {
                    ui::left_tab::show(ui, &mut left_tab_state);
                });

            // Inspector (우측 패널, 351px)
            if inspector_state.open {
                egui::SidePanel::right("inspector")
                    .exact_width(351.0)
                    .resizable(true)
                    .frame(
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgb(252, 252, 252))
                            .stroke(egui::Stroke::new(
                                1.0,
                                egui::Color32::from_rgba_premultiplied(0, 0, 0, 38),
                            ))
                            .inner_margin(egui::Margin::ZERO),
                    )
                    .show(ctx, |ui| {
                        ui::inspector::show(ui, &mut inspector_state);
                    });
            }

            // Central Panel (캔버스 영역 캡처)
            egui::CentralPanel::default()
                .frame(egui::Frame::new().fill(egui::Color32::TRANSPARENT))
                .show(ctx, |ui| {
                    canvas_rect = ui.available_rect_before_wrap();

                    // 플로팅 툴바 (액션은 클로저 밖에서 처리)
                    toolbar_action = ui::toolbar::show(ctx, canvas_rect);

                    // 드래그 선택 사각형
                    if is_drag_selecting {
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
                            (drag_select_start[0] - camera_position[0]) * camera_zoom / aspect,
                            (drag_select_start[1] - camera_position[1]) * camera_zoom,
                        ];
                        let ndc_end = [
                            (drag_select_end[0] - camera_position[0]) * camera_zoom / aspect,
                            (drag_select_end[1] - camera_position[1]) * camera_zoom,
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

                    // AI 버튼
                    ui::ai_button::show(ctx, canvas_rect);
                });
        });

        // 상태 복원
        self.egui.canvas_rect = canvas_rect;
        self.top_bar_state = top_bar_state;
        self.left_tab_state = left_tab_state;
        self.inspector_state = inspector_state;

        // 툴바 액션 처리 (clone 없이 직접 수정)
        if toolbar_action.add_node {
            let color = CARD_COLORS[self.block_positions.len() % CARD_COLORS.len()];
            self.block_positions.push(InstanceRaw {
                position: self.camera.position,
                color,
            });
            self.positions_dirty = true;
        }

        if toolbar_action.add_batch > 0 {
            let start = self.block_positions.len();
            let cols = (toolbar_action.add_batch as f32).sqrt().ceil() as usize;
            for i in 0..toolbar_action.add_batch {
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

        if toolbar_action.reset {
            self.block_positions.clear();
            for i in 0..100 {
                let col = (i % GRID_COLS) as f32;
                let row = (i / GRID_COLS) as f32;
                let color = CARD_COLORS[i % CARD_COLORS.len()];
                self.block_positions.push(InstanceRaw {
                    position: [col * GRID_SPACING_X, row * GRID_SPACING_Y],
                    color,
                });
            }
            self.positions_dirty = true;
        }

        self.egui
            .winit_state
            .handle_platform_output(&self.window, full_output.platform_output);

        let paint_jobs = self
            .egui
            .ctx
            .tessellate(full_output.shapes, self.egui.ctx.pixels_per_point());

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        for (id, delta) in &full_output.textures_delta.set {
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

        // ===== GPU 버퍼 업데이트 (dirty일 때만) =====
        self.update_gpu_buffers();

        // ===== wgpu 캔버스 렌더 (viewport/scissor 제한) =====
        let canvas = self.egui.canvas_rect;
        let scale = self.window.scale_factor() as f32;

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Canvas Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            // 캔버스 영역만 렌더링 (egui 패널 제외)
            if canvas.width() > 0.0 && canvas.height() > 0.0 {
                let vp_x = (canvas.min.x * scale).floor();
                let vp_y = (canvas.min.y * scale).floor();
                let vp_w = (canvas.width() * scale).ceil();
                let vp_h = (canvas.height() * scale).ceil();

                rpass.set_viewport(vp_x, vp_y, vp_w, vp_h, 0.0, 1.0);
                rpass.set_scissor_rect(vp_x as u32, vp_y as u32, vp_w as u32, vp_h as u32);
            }

            rpass.set_bind_group(0, &self.camera_bind_group, &[]);

            // 선
            rpass.set_pipeline(&self.line_pipeline);
            rpass.set_vertex_buffer(0, self.line_buffer.slice(..));
            rpass.draw(0..self.line_vertex_count, 0..1);

            // 카드
            rpass.set_pipeline(&self.card_pipeline);
            rpass.set_vertex_buffer(0, self.card_quad_buffer.slice(..));
            rpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            rpass.draw(0..4, 0..self.block_positions.len() as u32);
        }

        // ===== egui 렌더 (UI를 위에 덧그림) =====
        {
            let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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
                .render(&mut rpass, &paint_jobs, &screen_descriptor);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        // 사용하지 않는 egui 텍스처 해제
        for id in &full_output.textures_delta.free {
            self.egui.renderer.free_texture(id);
        }

        output.present();

        // egui가 재렌더를 요청할 때만 다시 그리기
        let needs_repaint = full_output
            .viewport_output
            .values()
            .any(|v| v.repaint_delay.is_zero());
        if needs_repaint {
            self.window.request_redraw();
        }
    }
}
