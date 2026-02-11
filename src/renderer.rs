use wgpu::util::DeviceExt;

use crate::state::AppState;
use crate::types::*;

fn make_quad(cx: f32, cy: f32, hw: f32, hh: f32, color: [f32; 3]) -> [Vertex; 6] {
    let tl = Vertex { position: [cx - hw, cy + hh], color };
    let bl = Vertex { position: [cx - hw, cy - hh], color };
    let tr = Vertex { position: [cx + hw, cy + hh], color };
    let br = Vertex { position: [cx + hw, cy - hh], color };
    [tl, bl, tr, tr, bl, br]
}

pub fn build_ui_vertices() -> Vec<Vertex> {
    let panel_color = [0.18_f32, 0.18, 0.22];
    let btn_color = [0.35_f32, 0.55, 0.95];
    let plus_color = [1.0_f32, 1.0, 1.0];

    let mut verts = Vec::new();

    // 패널 배경
    verts.extend_from_slice(&make_quad(0.0, ADD_BTN_Y, 0.06, 0.035, panel_color));
    // 버튼 배경
    verts.extend_from_slice(&make_quad(0.0, ADD_BTN_Y, ADD_BTN_HALF, ADD_BTN_HALF, btn_color));
    // "+" 가로 막대
    verts.extend_from_slice(&make_quad(0.0, ADD_BTN_Y, 0.012, 0.003, plus_color));
    // "+" 세로 막대
    verts.extend_from_slice(&make_quad(0.0, ADD_BTN_Y, 0.003, 0.012, plus_color));

    verts
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

        // 인스턴스 버퍼 (블록 위치+색상, 매 프레임 갱신)
        let instance_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&self.block_positions),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        // 선 버퍼 (매 프레임 갱신)
        let mut line_verts = Vec::new();
        for i in 0..self.block_positions.len().saturating_sub(1) {
            let color = self.block_positions[i].color;
            line_verts.push(Vertex {
                position: self.block_positions[i].position,
                color,
            });
            line_verts.push(Vertex {
                position: self.block_positions[i + 1].position,
                color,
            });
        }
        let line_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&line_verts),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
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

            rpass.set_bind_group(0, &self.camera_bind_group, &[]);

            // 선
            rpass.set_pipeline(&self.line_pipeline);
            rpass.set_vertex_buffer(0, line_buffer.slice(..));
            rpass.draw(0..line_verts.len() as u32, 0..1);

            // 카드
            rpass.set_pipeline(&self.card_pipeline);
            rpass.set_vertex_buffer(0, self.card_quad_buffer.slice(..));
            rpass.set_vertex_buffer(1, instance_buffer.slice(..));
            rpass.draw(0..4, 0..self.block_positions.len() as u32);

            // UI
            rpass.set_pipeline(&self.ui_pipeline);
            rpass.set_vertex_buffer(0, self.ui_vertex_buffer.slice(..));
            rpass.draw(0..self.ui_vertex_count, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
