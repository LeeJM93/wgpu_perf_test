use egui_wgpu::Renderer as EguiRenderer;
use egui_winit::State as EguiWinitState;
use winit::window::Window;

pub struct EguiIntegration {
    pub ctx: egui::Context,
    pub winit_state: EguiWinitState,
    pub renderer: EguiRenderer,
    pub canvas_rect: egui::Rect,
}

impl EguiIntegration {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        window: &Window,
    ) -> Self {
        let ctx = egui::Context::default();

        // 한글 폰트 로드
        let mut fonts = egui::FontDefinitions::default();
        if let Ok(font_data) = std::fs::read("/System/Library/Fonts/AppleSDGothicNeo.ttc") {
            fonts.font_data.insert(
                "korean".to_owned(),
                egui::FontData::from_owned(font_data).into(),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "korean".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("korean".to_owned());
        }
        ctx.set_fonts(fonts);

        let winit_state = EguiWinitState::new(
            ctx.clone(),
            egui::ViewportId::ROOT,
            window,
            None,
            None,
            None,
        );
        let renderer = EguiRenderer::new(device, surface_format, None, 1, true);

        Self {
            ctx,
            winit_state,
            renderer,
            canvas_rect: egui::Rect::NOTHING,
        }
    }
}
