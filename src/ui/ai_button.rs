use egui::{self, Color32, Vec2};

use crate::types::AI_BUTTON_OFFSET;

pub fn show(ctx: &egui::Context, canvas_rect: egui::Rect) -> bool {
    let mut clicked = false;

    let btn_x = canvas_rect.max.x - AI_BUTTON_OFFSET;
    let btn_y = canvas_rect.max.y - AI_BUTTON_OFFSET;

    egui::Area::new(egui::Id::new("ai_button"))
        .fixed_pos(egui::pos2(btn_x, btn_y))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(Vec2::splat(70.0), egui::Sense::click());

            // 배경 원 (그라데이션 근사)
            ui.painter()
                .circle_filled(rect.center(), 35.0, Color32::from_rgb(100, 80, 200));

            // 내부 밝은 원
            ui.painter().circle_filled(
                rect.center(),
                28.0,
                Color32::from_rgba_premultiplied(180, 160, 255, 180),
            );

            // 로고 심볼 (✦)
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "✦",
                egui::FontId::proportional(24.0),
                Color32::WHITE,
            );

            if response.clicked() {
                clicked = true;
            }
        });

    clicked
}
