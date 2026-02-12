use egui::{self, Color32, RichText, Vec2};

pub struct TopBarState {
    pub active_tab: usize,
    pub tabs: Vec<String>,
}

impl Default for TopBarState {
    fn default() -> Self {
        Self {
            active_tab: 0,
            tabs: vec![
                "운동 습관 형성 앱 기획".to_string(),
                "기술스택 선택".to_string(),
                "신규서비스 런칭 전략".to_string(),
            ],
        }
    }
}

pub fn frame() -> egui::Frame {
    egui::Frame::new()
        .fill(Color32::from_rgba_premultiplied(244, 244, 244, 210))
        .stroke(egui::Stroke::new(
            0.5,
            Color32::from_rgba_premultiplied(0, 0, 0, 50),
        ))
        .inner_margin(egui::Margin::ZERO)
}

pub fn show(ui: &mut egui::Ui, state: &mut TopBarState) {
    ui.horizontal_centered(|ui| {
        ui.add_space(100.0); // macOS traffic lights 영역

        // 홈 아이콘
        ui.add_space(8.0);
        ui.label(RichText::new("⌂").size(16.0).color(Color32::from_gray(80)));
        ui.add_space(8.0);

        // 구분선
        let rect = ui.available_rect_before_wrap();
        ui.painter().vline(
            rect.left(),
            egui::Rangef::new(rect.top() + 10.0, rect.bottom() - 10.0),
            egui::Stroke::new(0.5, Color32::from_rgba_premultiplied(0, 0, 0, 25)),
        );

        // 페이지 탭들
        for (i, tab_name) in state.tabs.iter().enumerate() {
            let is_active = i == state.active_tab;

            let bg_color = if is_active {
                Color32::from_rgba_premultiplied(255, 255, 255, 200)
            } else {
                Color32::TRANSPARENT
            };

            let text_color = if is_active {
                Color32::from_rgb(16, 24, 40)
            } else {
                Color32::from_rgb(174, 174, 174)
            };

            let font_id = egui::FontId::proportional(14.0);

            // 탭 구분선
            let rect = ui.available_rect_before_wrap();
            ui.painter().vline(
                rect.left(),
                egui::Rangef::new(rect.top() + 10.0, rect.bottom() - 10.0),
                egui::Stroke::new(0.5, Color32::from_rgba_premultiplied(0, 0, 0, 25)),
            );

            let response = ui.allocate_ui_with_layout(
                Vec2::new(0.0, ui.available_height()),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    let padding = egui::Frame::new()
                        .inner_margin(egui::Margin::symmetric(16, 0))
                        .fill(bg_color);
                    padding.show(ui, |ui| {
                        let icon = if is_active { "📄" } else { "📋" };
                        ui.label(RichText::new(icon).size(12.0));
                        ui.add_space(4.0);
                        let label = egui::Label::new(
                            RichText::new(tab_name.as_str())
                                .font(font_id)
                                .color(text_color),
                        )
                        .selectable(false);
                        ui.add(label);
                    });
                },
            );

            if response.response.clicked() {
                state.active_tab = i;
            }
        }
    });
}
