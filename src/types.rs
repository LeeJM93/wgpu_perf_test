// 카드 렌더링 크기 (셰이더 fs_card의 half_size와 일치해야 함)
pub const CARD_HALF_W: f32 = 0.25;
pub const CARD_HALF_H: f32 = 0.15;
pub const CARD_QUAD_W: f32 = 0.28; // 그림자 여백 포함
pub const CARD_QUAD_H: f32 = 0.18;

// 블록 배치 간격
pub const GRID_COLS: usize = 10;
pub const GRID_SPACING_X: f32 = 0.7;
pub const GRID_SPACING_Y: f32 = 0.5;
pub const DEFAULT_GRID_COUNT: usize = 100;

// UI 레이아웃
pub const TOP_BAR_HEIGHT: f32 = 55.0;
pub const LEFT_TAB_WIDTH: f32 = 75.0;
pub const INSPECTOR_WIDTH: f32 = 351.0;
pub const TOOLBAR_HALF_WIDTH: f32 = 380.0;
pub const TOOLBAR_BOTTOM_OFFSET: f32 = 55.0;
pub const AI_BUTTON_OFFSET: f32 = 60.0;

// 카드 테두리 색상 팔레트
pub const CARD_COLORS: [[f32; 3]; 6] = [
    [0.94, 0.33, 0.46], // 분홍
    [0.55, 0.48, 0.82], // 보라
    [0.95, 0.73, 0.15], // 노랑
    [0.30, 0.82, 0.88], // 시안
    [0.96, 0.58, 0.22], // 주황
    [0.25, 0.25, 0.30], // 다크
];

pub fn create_default_grid() -> Vec<InstanceRaw> {
    (0..DEFAULT_GRID_COUNT)
        .map(|i| {
            let col = (i % GRID_COLS) as f32;
            let row = (i / GRID_COLS) as f32;
            InstanceRaw {
                position: [col * GRID_SPACING_X, row * GRID_SPACING_Y],
                color: CARD_COLORS[i % CARD_COLORS.len()],
            }
        })
        .collect()
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}
