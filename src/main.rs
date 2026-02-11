use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

struct Camera {
    position: [f32; 2],
    zoom: f32,
}

impl Camera {
    fn build_view_proj(&self, aspect: f32) -> CameraUniform {
        let sx = self.zoom / aspect;
        let sy = self.zoom;
        let tx = -self.position[0] * sx;
        let ty = -self.position[1] * sy;
        CameraUniform {
            view_proj: [
                [sx, 0.0, 0.0, 0.0],
                [0.0, sy, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [tx, ty, 0.0, 1.0],
            ],
        }
    }
}

struct AppState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    line_pipeline: wgpu::RenderPipeline,
    window: Arc<Window>,

    // 카메라
    camera: Camera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    // 데이터
    block_positions: Vec<InstanceRaw>,
    selected_idx: Option<usize>,
    mouse_ndc: [f32; 2],

    // 팬 상태
    space_pressed: bool,
    is_panning: bool,
    pan_start_ndc: [f32; 2],
    pan_start_camera: [f32; 2],
}

impl AppState {
    fn aspect(&self) -> f32 {
        self.config.width as f32 / self.config.height as f32
    }

    fn ndc_to_world(&self, ndc: [f32; 2]) -> [f32; 2] {
        let aspect = self.aspect();
        [
            ndc[0] * aspect / self.camera.zoom + self.camera.position[0],
            ndc[1] / self.camera.zoom + self.camera.position[1],
        ]
    }

    fn update_camera_buffer(&self) {
        let uniform = self.camera.build_view_proj(self.aspect());
        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }
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
                        .with_title("Rust 10k Blocks — Infinite Canvas"),
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

        // 카메라 초기화 — 격자 중앙에 위치, 전체가 보이도록 줌아웃
        let camera = Camera {
            position: [7.5, 7.5],
            zoom: 0.1,
        };
        let aspect = config.width as f32 / config.height as f32;
        let camera_uniform = camera.build_view_proj(aspect);
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        // 1. 블록(사각형) 파이프라인
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Block Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3],
                    },
                    wgpu::VertexBufferLayout {
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
            layout: Some(&pipeline_layout),
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

        // 10,000개 블록 초기 위치 (world space 격자)
        let mut block_positions = Vec::new();
        for i in 0..100 {
            let x = (i % 100) as f32 * 0.15;
            let y = (i / 100) as f32 * 0.15;
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
            camera,
            camera_buffer,
            camera_bind_group,
            block_positions,
            selected_idx: None,
            mouse_ndc: [0.0, 0.0],
            space_pressed: false,
            is_panning: false,
            pan_start_ndc: [0.0, 0.0],
            pan_start_camera: [0.0, 0.0],
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(state) = self.state.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            // Space 키 추적
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Space),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                state.space_pressed = key_state == ElementState::Pressed;
                if !state.space_pressed {
                    state.is_panning = false;
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                let size = state.window.inner_size();
                state.mouse_ndc = [
                    (position.x as f32 / size.width as f32) * 2.0 - 1.0,
                    (position.y as f32 / size.height as f32) * -2.0 + 1.0,
                ];

                if state.is_panning {
                    // NDC 차이를 world space 이동량으로 변환
                    let aspect = state.aspect();
                    let dx_ndc = state.mouse_ndc[0] - state.pan_start_ndc[0];
                    let dy_ndc = state.mouse_ndc[1] - state.pan_start_ndc[1];
                    state.camera.position[0] =
                        state.pan_start_camera[0] - dx_ndc * aspect / state.camera.zoom;
                    state.camera.position[1] =
                        state.pan_start_camera[1] - dy_ndc / state.camera.zoom;
                    state.update_camera_buffer();
                    state.window.request_redraw();
                } else if let Some(idx) = state.selected_idx {
                    let world = state.ndc_to_world(state.mouse_ndc);
                    state.block_positions[idx].position = world;
                    state.window.request_redraw();
                }
            }

            WindowEvent::MouseInput {
                state: button_state,
                button: MouseButton::Left,
                ..
            } => {
                if button_state == ElementState::Pressed {
                    if state.space_pressed {
                        // Space + 좌클릭 → 팬 시작
                        state.is_panning = true;
                        state.pan_start_ndc = state.mouse_ndc;
                        state.pan_start_camera = state.camera.position;
                    } else {
                        // 블록 선택
                        let mouse_world = state.ndc_to_world(state.mouse_ndc);
                        state.selected_idx = state.block_positions.iter().position(|pos| {
                            let dx = pos.position[0] - mouse_world[0];
                            let dy = pos.position[1] - mouse_world[1];
                            (dx * dx + dy * dy).sqrt() < 0.05
                        });
                    }
                } else {
                    state.selected_idx = None;
                    state.is_panning = false;
                }
            }

            // 스크롤 줌 (마우스 위치 기준)
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 50.0,
                };

                let zoom_factor = if scroll_y > 0.0 { 1.1 } else { 1.0 / 1.1 };

                // 줌 전 마우스 위치의 world 좌표
                let world_before = state.ndc_to_world(state.mouse_ndc);

                state.camera.zoom *= zoom_factor;
                // 줌 범위 제한
                state.camera.zoom = state.camera.zoom.clamp(0.001, 100.0);

                // 줌 후 같은 NDC 위치의 world 좌표
                let world_after = state.ndc_to_world(state.mouse_ndc);

                // 마우스 아래 포인트가 고정되도록 카메라 위치 보정
                state.camera.position[0] += world_before[0] - world_after[0];
                state.camera.position[1] += world_before[1] - world_after[1];

                state.update_camera_buffer();
                state.window.request_redraw();
            }

            WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    state.config.width = new_size.width;
                    state.config.height = new_size.height;
                    state.surface.configure(&state.device, &state.config);
                    state.update_camera_buffer();
                    state.window.request_redraw();
                }
            }

            WindowEvent::RedrawRequested => {
                state.update_camera_buffer();

                let output = state.surface.get_current_texture().unwrap();
                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = state
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                // 블록 정점 데이터
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

                // 선 데이터 생성
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

                    rpass.set_bind_group(0, &state.camera_bind_group, &[]);

                    // 선 먼저 그리기
                    rpass.set_pipeline(&state.line_pipeline);
                    rpass.set_vertex_buffer(0, l_buf.slice(..));
                    rpass.draw(0..line_verts.len() as u32, 0..1);

                    // 블록 그리기 (인스턴싱)
                    rpass.set_pipeline(&state.render_pipeline);
                    rpass.set_vertex_buffer(0, v_buf.slice(..));
                    rpass.set_vertex_buffer(1, i_buf.slice(..));
                    rpass.draw(0..4, 0..100);
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
