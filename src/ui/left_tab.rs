use egui::{self, Color32, RichText, Vec2};

pub struct LeftTabState {
    pub active_section: usize,
}

impl Default for LeftTabState {
    fn default() -> Self {
        Self { active_section: 0 }
    }
}

struct TabButton {
    icon: &'static str,
    label: &'static str,
}

pub fn show(ui: &mut egui::Ui, state: &mut LeftTabState) {
    ui.vertical_centered(|ui| {
        ui.add_space(50.0);

        // 프로필 아바타
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(40.0), egui::Sense::hover());
        ui.painter()
            .circle_filled(rect.center(), 20.0, Color32::from_rgb(180, 160, 220));
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "👤",
            egui::FontId::proportional(18.0),
            Color32::WHITE,
        );

        ui.add_space(40.0);

        // 섹션 1
        ui.label(
            RichText::new("섹션")
                .size(13.0)
                .color(Color32::from_rgba_premultiplied(82, 82, 82, 128)),
        );
        ui.add_space(12.0);

        let section1 = [
            TabButton {
                icon: "📁",
                label: "프로젝트",
            },
            TabButton {
                icon: "📊",
                label: "대시보드",
            },
        ];

        for (i, btn) in section1.iter().enumerate() {
            let is_active = state.active_section == i;
            show_tab_button(ui, btn, is_active);
            if ui.ctx().input(|i| i.pointer.any_click()) {
                // 클릭 처리는 show_tab_button 내부 response로
            }
            ui.add_space(12.0);
        }

        ui.add_space(30.0);

        // 섹션 2
        ui.label(
            RichText::new("섹션")
                .size(13.0)
                .color(Color32::from_rgba_premultiplied(82, 82, 82, 128)),
        );
        ui.add_space(12.0);

        let section2 = [
            TabButton {
                icon: "📁",
                label: "프로젝트",
            },
            TabButton {
                icon: "📊",
                label: "대시보드",
            },
            TabButton {
                icon: "⚙",
                label: "설정",
            },
        ];

        for btn in &section2 {
            show_tab_button(ui, btn, false);
            ui.add_space(12.0);
        }
    });
}

fn show_tab_button(ui: &mut egui::Ui, btn: &TabButton, _is_active: bool) {
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(btn.icon).size(15.0));
        ui.label(
            RichText::new(btn.label)
                .size(10.0)
                .color(Color32::from_rgb(82, 82, 82)),
        );
    });
}
