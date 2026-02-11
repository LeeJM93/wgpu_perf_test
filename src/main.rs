use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    position: [f32; 2],
}

struct AppState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    line_pipeline: wgpu::RenderPipeline,
    window: Arc<Window>,

    // 데이터
    block_positions: Vec<InstanceRaw>,
    selected_idx: Option<usize>,
    mouse_pos: [f32; 2],
}

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
                    winit::window::WindowAttributes::default()
                        .with_title("Rust 100 Blocks & Lines Test"),
                )
                .unwrap(),
        );
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
                .unwrap();

        let size = window.inner_size();
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        // 1. 블록(사각형) 파이프라인
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Block Pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        // 기본 사각형 정점
                        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3],
                    },
                    wgpu::VertexBufferLayout {
                        // 인스턴스 위치
                        array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![2 => Float32x2],
                    },
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // 2. 선(Line) 파이프라인
        let line_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Line Pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_line"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // 100개 블록 초기 위치 설정 (격자 형태)
        let mut block_positions = Vec::new();
        for i in 0..10_000 {
            let x = (i % 10) as f32 * 0.15 - 0.7;
            let y = (i / 10) as f32 * 0.15 - 0.7;
            block_positions.push(InstanceRaw { position: [x, y] });
        }

        self.state = Some(AppState {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            line_pipeline,
            window,
            block_positions,
            selected_idx: None,
            mouse_pos: [0.0, 0.0],
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::CursorMoved { position, .. } => {
                let size = state.window.inner_size();
                state.mouse_pos = [
                    (position.x as f32 / size.width as f32) * 2.0 - 1.0,
                    (position.y as f32 / size.height as f32) * -2.0 + 1.0,
                ];
                if let Some(idx) = state.selected_idx {
                    state.block_positions[idx].position = state.mouse_pos;
                    state.window.request_redraw();
                }
            }
            WindowEvent::MouseInput {
                state: button_state,
                button: MouseButton::Left,
                ..
            } => {
                if button_state == ElementState::Pressed {
                    state.selected_idx = state.block_positions.iter().position(|pos| {
                        let dx = pos.position[0] - state.mouse_pos[0];
                        let dy = pos.position[1] - state.mouse_pos[1];
                        (dx * dx + dy * dy).sqrt() < 0.05
                    });
                } else {
                    state.selected_idx = None;
                }
            }
            WindowEvent::RedrawRequested => {
                let output = state.surface.get_current_texture().unwrap();
                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = state
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                // 1. 블록 정점 데이터 (단일 사각형 모양)
                let s = 0.03;
                let block_shape = [
                    Vertex {
                        position: [-s, s],
                        color: [0.0, 0.8, 1.0],
                    },
                    Vertex {
                        position: [-s, -s],
                        color: [0.0, 0.8, 1.0],
                    },
                    Vertex {
                        position: [s, s],
                        color: [0.0, 0.8, 1.0],
                    },
                    Vertex {
                        position: [s, -s],
                        color: [0.0, 0.8, 1.0],
                    },
                ];
                let v_buf = state
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&block_shape),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                let i_buf = state
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(&state.block_positions),
                        usage: wgpu::BufferUsages::VERTEX,
                    });

                // 2. 선 데이터 생성 (블록들을 순서대로 연결)
                let mut line_verts = Vec::new();
                for i in 0..state.block_positions.len() - 1 {
                    line_verts.push(Vertex {
                        position: state.block_positions[i].position,
                        color: [1.0, 1.0, 1.0],
                    });
                    line_verts.push(Vertex {
                        position: state.block_positions[i + 1].position,
                        color: [1.0, 1.0, 1.0],
                    });
                }
                let l_buf = state
                    .device
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
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        ..Default::default()
                    });

                    // 선 먼저 그리기
                    rpass.set_pipeline(&state.line_pipeline);
                    rpass.set_vertex_buffer(0, l_buf.slice(..));
                    rpass.draw(0..line_verts.len() as u32, 0..1);

                    // 블록 그리기 (인스턴싱)
                    rpass.set_pipeline(&state.render_pipeline);
                    rpass.set_vertex_buffer(0, v_buf.slice(..));
                    rpass.set_vertex_buffer(1, i_buf.slice(..));
                    rpass.draw(0..4, 0..10_000);
                }
                state.queue.submit(std::iter::once(encoder.finish()));
                output.present();
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App { state: None };
    event_loop.run_app(&mut app).unwrap();
}
