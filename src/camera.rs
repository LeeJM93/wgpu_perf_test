use crate::types::CameraUniform;

pub struct Camera {
    pub position: [f32; 2],
    pub zoom: f32,
}

impl Camera {
    pub fn new(position: [f32; 2], zoom: f32) -> Self {
        Self { position, zoom }
    }

    pub fn build_view_proj(&self, aspect: f32) -> CameraUniform {
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

    pub fn ndc_to_world(&self, ndc: [f32; 2], aspect: f32) -> [f32; 2] {
        [
            ndc[0] * aspect / self.zoom + self.position[0],
            ndc[1] / self.zoom + self.position[1],
        ]
    }

    pub fn world_to_ndc(&self, world: [f32; 2], aspect: f32) -> [f32; 2] {
        [
            (world[0] - self.position[0]) * self.zoom / aspect,
            (world[1] - self.position[1]) * self.zoom,
        ]
    }

    pub fn zoom_at(&mut self, ndc: [f32; 2], factor: f32, aspect: f32) {
        let world_before = self.ndc_to_world(ndc, aspect);
        self.zoom = (self.zoom * factor).clamp(0.001, 100.0);
        let world_after = self.ndc_to_world(ndc, aspect);
        self.position[0] += world_before[0] - world_after[0];
        self.position[1] += world_before[1] - world_after[1];
    }
}
