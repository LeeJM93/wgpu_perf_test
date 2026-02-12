use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::camera::Camera;
use crate::egui_integration::EguiIntegration;
use crate::pipeline;
use crate::types::*;
use crate::ui;

pub struct AppState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub window: Arc<Window>,

    // 파이프라인
    pub card_pipeline: wgpu::RenderPipeline,
    pub line_pipeline: wgpu::RenderPipeline,

    // 카메라
    pub camera: Camera,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,

    // 정적 버퍼 (초기화 시 한 번만 생성)
    pub card_quad_buffer: wgpu::Buffer,

    // 영속 GPU 버퍼
    pub instance_buffer: wgpu::Buffer,
    pub instance_buffer_capacity: usize,
    pub line_buffer: wgpu::Buffer,
    pub line_buffer_capacity: usize,
    pub line_vertex_count: u32,
    pub positions_dirty: bool,
    pub cached_line_verts: Vec<Vertex>,

    // 데이터
    pub block_positions: Vec<InstanceRaw>,
    pub mouse_ndc: [f32; 2],
    pub mouse_pixel: [f32; 2],

    // 선택 상태
    pub selected_indices: Vec<usize>,
    pub is_drag_selecting: bool,
    pub drag_select_start: [f32; 2],
    pub drag_select_end: [f32; 2],
    pub is_moving_selection: bool,
    pub move_last_world: [f32; 2],

    // 팬 상태
    pub space_pressed: bool,
    pub is_panning: bool,
    pub pan_start_ndc: [f32; 2],
    pub pan_start_camera: [f32; 2],

    // egui
    pub egui: EguiIntegration,

    // UI 상태
    pub top_bar_state: ui::top_bar::TopBarState,
    pub left_tab_state: ui::left_tab::LeftTabState,
    pub inspector_state: ui::inspector::InspectorState,
}

impl AppState {
    pub fn new(window: Arc<Window>) -> Self {
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

        // 카메라
        let camera = Camera::new([3.15, 2.25], 0.3);
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

        // 파이프라인
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let card_pipeline =
            pipeline::create_card_pipeline(&device, &shader, config.format, &pipeline_layout);
        let line_pipeline =
            pipeline::create_line_pipeline(&device, &shader, config.format, &pipeline_layout);

        // 정적 버퍼: 카드 쿼드
        let card_quad_buffer = Self::create_card_quad_buffer(&device);

        // egui 초기화
        let egui = EguiIntegration::new(&device, config.format, &window);

        // 블록 초기 데이터
        let mut block_positions = Vec::new();
        for i in 0..100 {
            let col = (i % GRID_COLS) as f32;
            let row = (i / GRID_COLS) as f32;
            let color = CARD_COLORS[i % CARD_COLORS.len()];
            block_positions.push(InstanceRaw {
                position: [col * GRID_SPACING_X, row * GRID_SPACING_Y],
                color,
            });
        }

        // 영속 GPU 버퍼 사전 할당
        let initial_capacity = block_positions.len().max(1024) * 2;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (initial_capacity * std::mem::size_of::<InstanceRaw>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let line_capacity = initial_capacity * 2;
        let line_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Line Buffer"),
            size: (line_capacity * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            surface,
            device,
            queue,
            config,
            window,
            card_pipeline,
            line_pipeline,
            camera,
            camera_buffer,
            camera_bind_group,
            card_quad_buffer,
            instance_buffer,
            instance_buffer_capacity: initial_capacity,
            line_buffer,
            line_buffer_capacity: line_capacity,
            line_vertex_count: 0,
            positions_dirty: true,
            cached_line_verts: Vec::new(),
            block_positions,
            mouse_ndc: [0.0, 0.0],
            mouse_pixel: [0.0, 0.0],
            selected_indices: Vec::new(),
            is_drag_selecting: false,
            drag_select_start: [0.0, 0.0],
            drag_select_end: [0.0, 0.0],
            is_moving_selection: false,
            move_last_world: [0.0, 0.0],
            space_pressed: false,
            is_panning: false,
            pan_start_ndc: [0.0, 0.0],
            pan_start_camera: [0.0, 0.0],
            egui,
            top_bar_state: Default::default(),
            left_tab_state: Default::default(),
            inspector_state: Default::default(),
        }
    }

    pub fn canvas_aspect(&self) -> f32 {
        let rect = self.egui.canvas_rect;
        if rect.width() > 0.0 && rect.height() > 0.0 {
            rect.width() / rect.height()
        } else {
            self.config.width as f32 / self.config.height as f32
        }
    }

    pub fn mark_positions_dirty(&mut self) {
        self.positions_dirty = true;
    }

    pub fn update_gpu_buffers(&mut self) {
        if !self.positions_dirty {
            return;
        }
        self.positions_dirty = false;

        // 용량 부족 시 버퍼 재할당 (2배 확장)
        let needed = self.block_positions.len();
        if needed > self.instance_buffer_capacity {
            let new_cap = (needed * 2).max(1024);
            self.instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer"),
                size: (new_cap * std::mem::size_of::<InstanceRaw>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.instance_buffer_capacity = new_cap;

            let line_cap = new_cap * 2;
            self.line_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Line Buffer"),
                size: (line_cap * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.line_buffer_capacity = line_cap;
        }

        // 라인 버텍스 캐시 재생성
        self.cached_line_verts.clear();
        for i in 0..self.block_positions.len().saturating_sub(1) {
            let color = self.block_positions[i].color;
            self.cached_line_verts.push(Vertex {
                position: self.block_positions[i].position,
                color,
            });
            self.cached_line_verts.push(Vertex {
                position: self.block_positions[i + 1].position,
                color,
            });
        }
        self.line_vertex_count = self.cached_line_verts.len() as u32;

        // GPU에 업로드
        if !self.block_positions.is_empty() {
            self.queue.write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(&self.block_positions),
            );
        }
        if !self.cached_line_verts.is_empty() {
            self.queue.write_buffer(
                &self.line_buffer,
                0,
                bytemuck::cast_slice(&self.cached_line_verts),
            );
        }
    }

    pub fn update_camera_buffer(&self) {
        let uniform = self.camera.build_view_proj(self.canvas_aspect());
        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    fn create_card_quad_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        let vertices = [
            Vertex {
                position: [-CARD_QUAD_W, CARD_QUAD_H],
                color: [0.0; 3],
            },
            Vertex {
                position: [-CARD_QUAD_W, -CARD_QUAD_H],
                color: [0.0; 3],
            },
            Vertex {
                position: [CARD_QUAD_W, CARD_QUAD_H],
                color: [0.0; 3],
            },
            Vertex {
                position: [CARD_QUAD_W, -CARD_QUAD_H],
                color: [0.0; 3],
            },
        ];
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Card Quad Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }
}
