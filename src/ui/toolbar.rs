use egui::{self, Color32, CornerRadius, RichText, Vec2};

use crate::types::{TOOLBAR_BOTTOM_OFFSET, TOOLBAR_HALF_WIDTH};

pub struct ToolbarAction {
    pub add_node: bool,
    pub add_batch: usize, // 0이면 미사용, 100/1000/10000
    pub reset: bool,
}

impl Default for ToolbarAction {
    fn default() -> Self {
        Self {
            add_node: false,
            add_batch: 0,
            reset: false,
        }
    }
}

pub fn show(ctx: &egui::Context, canvas_rect: egui::Rect) -> ToolbarAction {
    let mut action = ToolbarAction::default();

    let toolbar_y = canvas_rect.max.y - TOOLBAR_BOTTOM_OFFSET;
    let toolbar_x = canvas_rect.center().x;

    egui::Area::new(egui::Id::new("toolbar"))
        .fixed_pos(egui::pos2(toolbar_x - TOOLBAR_HALF_WIDTH, toolbar_y))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let frame = egui::Frame::new()
                .fill(Color32::from_rgba_premultiplied(255, 255, 255, 240))
                .corner_radius(CornerRadius::same(24))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(229, 231, 235)))
                .shadow(egui::epaint::Shadow {
                    offset: [0, 20],
                    blur: 30,
                    spread: 0,
                    color: Color32::from_rgba_premultiplied(0, 0, 0, 38),
                })
                .inner_margin(egui::Margin::symmetric(13, 9));

            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    // 노드 생성 (보라색 메인 버튼)
                    let add_btn = egui::Button::new(
                        RichText::new("✦ 노드 생성")
                            .size(14.0)
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(79, 57, 246))
                    .corner_radius(CornerRadius::same(14))
                    .min_size(Vec2::new(96.0, 48.0));
                    if ui.add(add_btn).clicked() {
                        action.add_node = true;
                    }

                    // 대량 생성 버튼들
                    for count in [100, 1000, 10000] {
                        if batch_button(ui, count).clicked() {
                            action.add_batch = count;
                        }
                    }

                    ui.add_space(4.0);
                    separator(ui);
                    ui.add_space(4.0);

                    // 초기화
                    if toolbar_button(ui, "↺", "초기화").clicked() {
                        action.reset = true;
                    }

                    ui.add_space(4.0);
                    separator(ui);
                    ui.add_space(4.0);

                    // 저장 / 내보내기 / 가져오기 (미구현)
                    toolbar_button(ui, "💾", "저장");
                    toolbar_button(ui, "📤", "내보내기");
                    toolbar_button(ui, "📥", "가져오기");

                    ui.add_space(4.0);
                    separator(ui);
                    ui.add_space(4.0);

                    // 우측 패널 컨트롤 탭
                    panel_tab(ui, "회의", true);
                    panel_tab(ui, "생각정리", false);
                    panel_tab(ui, "리서치", false);
                });
            });
        });

    action
}

fn toolbar_button(ui: &mut egui::Ui, icon: &str, label: &str) -> egui::Response {
    let btn = egui::Button::new(egui::WidgetText::from(egui::text::LayoutJob::simple(
        format!("{}\n{}", icon, label),
        egui::FontId::proportional(10.0),
        Color32::from_rgb(54, 65, 83),
        56.0,
    )))
    .fill(Color32::TRANSPARENT)
    .corner_radius(CornerRadius::same(14))
    .min_size(Vec2::new(56.0, 42.0));
    ui.add(btn)
}

fn panel_tab(ui: &mut egui::Ui, label: &str, is_active: bool) {
    let bg = if is_active {
        Color32::WHITE
    } else {
        Color32::TRANSPARENT
    };
    let text_color = if is_active {
        Color32::from_rgb(77, 145, 255)
    } else {
        Color32::BLACK
    };

    let radius: u8 = if is_active { 8 } else { 5 };
    let frame = egui::Frame::new()
        .fill(bg)
        .corner_radius(CornerRadius::same(radius))
        .inner_margin(egui::Margin::same(4));

    if is_active {
        let frame = frame.shadow(egui::epaint::Shadow {
            offset: [0, 4],
            blur: 4,
            spread: 0,
            color: Color32::from_rgba_premultiplied(0, 0, 0, 13),
        });
        frame.show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("💼").size(11.0));
                ui.label(RichText::new(label).size(7.0).color(text_color));
            });
        });
    } else {
        frame.show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("💼").size(11.0));
                ui.label(RichText::new(label).size(7.0).color(text_color));
            });
        });
    }
}

fn batch_button(ui: &mut egui::Ui, count: usize) -> egui::Response {
    let label = if count >= 10000 {
        format!("{}만", count / 10000)
    } else if count >= 1000 {
        format!("{}천", count / 1000)
    } else {
        format!("{}", count)
    };
    let btn = egui::Button::new(
        RichText::new(format!("+{}", label))
            .size(11.0)
            .color(Color32::from_rgb(79, 57, 246)),
    )
    .fill(Color32::from_rgba_premultiplied(79, 57, 246, 20))
    .corner_radius(CornerRadius::same(10))
    .min_size(Vec2::new(42.0, 32.0));
    ui.add(btn)
}

fn separator(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, 32.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::ZERO, Color32::from_rgb(209, 213, 220));
}
