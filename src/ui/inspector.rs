use egui::{self, Color32, CornerRadius, RichText, Vec2};

pub struct InspectorState {
    pub open: bool,
    pub active_tab: usize, // 0: 속성, 1: 전사문
}

impl Default for InspectorState {
    fn default() -> Self {
        Self {
            open: true,
            active_tab: 1, // 전사문 탭 활성
        }
    }
}

struct TranscriptEntry {
    speaker: &'static str,
    text: &'static str,
    timestamp: &'static str,
    badge_bg: Color32,
    badge_text: Color32,
}

pub fn frame() -> egui::Frame {
    egui::Frame::new()
        .fill(Color32::from_rgb(252, 252, 252))
        .stroke(egui::Stroke::new(
            1.0,
            Color32::from_rgba_premultiplied(0, 0, 0, 38),
        ))
        .inner_margin(egui::Margin::ZERO)
}

pub fn show(ui: &mut egui::Ui, state: &mut InspectorState) {
    // 탭 바
    ui.horizontal(|ui| {
        ui.add_space(12.0);

        // 속성 탭
        let attr_active = state.active_tab == 0;
        let attr_btn =
            egui::Button::new(RichText::new("🔧 속성").size(10.0).color(if attr_active {
                Color32::BLACK
            } else {
                Color32::from_rgb(174, 174, 174)
            }))
            .fill(if attr_active {
                Color32::from_rgb(229, 229, 234)
            } else {
                Color32::TRANSPARENT
            })
            .corner_radius(CornerRadius::same(13))
            .min_size(Vec2::new(0.0, 32.0));
        if ui.add(attr_btn).clicked() {
            state.active_tab = 0;
        }

        // 전사문 탭
        let trans_active = state.active_tab == 1;
        let trans_btn = egui::Button::new(RichText::new("📝 전사문").size(10.0).color(
            if trans_active {
                Color32::BLACK
            } else {
                Color32::from_rgb(174, 174, 174)
            },
        ))
        .fill(if trans_active {
            Color32::from_rgb(229, 229, 234)
        } else {
            Color32::TRANSPARENT
        })
        .corner_radius(CornerRadius::same(13))
        .min_size(Vec2::new(0.0, 32.0));
        if ui.add(trans_btn).clicked() {
            state.active_tab = 1;
        }
    });

    ui.separator();

    match state.active_tab {
        0 => show_properties(ui),
        1 => show_transcript(ui),
        _ => {}
    }
}

fn show_properties(ui: &mut egui::Ui) {
    ui.add_space(20.0);
    ui.centered_and_justified(|ui| {
        ui.label(
            RichText::new("노드를 선택하세요")
                .size(13.0)
                .color(Color32::from_rgb(174, 174, 174)),
        );
    });
}

fn show_transcript(ui: &mut egui::Ui) {
    let entries = [
        TranscriptEntry {
            speaker: "김팀장",
            text: "안녕하세요, 여러분. 오늘은 2026년 상반기 팀 워크샵 기획에 대해 논의하려고 합니다. 팀 결속력 강화와 전략 공유가 주요 목표입니다.",
            timestamp: "00:00",
            badge_bg: Color32::from_rgb(219, 234, 254),
            badge_text: Color32::from_rgb(25, 60, 184),
        },
        TranscriptEntry {
            speaker: "이과장",
            text: "좋습니다. 먼저 워크샵 장소부터 정해야 할 것 같은데요, 참여 인원이 20명 정도 되니까 예산과 접근성을 고려해야 합니다.",
            timestamp: "00:13",
            badge_bg: Color32::from_rgb(220, 252, 231),
            badge_text: Color32::from_rgb(1, 102, 48),
        },
        TranscriptEntry {
            speaker: "김팀장",
            text: "맞습니다. 그리고 어떤 프로그램을 진행할지도 중요하죠. 팀 빌딩 활동, 교육, 전략 회의 등을 어떻게 구성할지 고민이 필요합니다.",
            timestamp: "00:26",
            badge_bg: Color32::from_rgb(219, 234, 254),
            badge_text: Color32::from_rgb(25, 60, 184),
        },
        TranscriptEntry {
            speaker: "박대리",
            text: "제 생각에는 게임형 팀 챌린지를 도입하면 어떨까 싶습니다. 방탈출이나 미션 수행 같은 게임 요소를 넣으면 참여도가 훨씬 높아질 것 같아요.",
            timestamp: "00:39",
            badge_bg: Color32::from_rgb(243, 232, 255),
            badge_text: Color32::from_rgb(110, 17, 176),
        },
        TranscriptEntry {
            speaker: "이과장",
            text: "장소 관련해서는 제주도 리조트를 검토해봤는데요, 3박 4일 패키지로 진행할 수 있습니다. 시설은 정말 좋은데... 문제는 1인당 비용이 35만원 정도로 예산을 초과한다는 점이에요.",
            timestamp: "00:51",
            badge_bg: Color32::from_rgb(220, 252, 231),
            badge_text: Color32::from_rgb(1, 102, 48),
        },
        TranscriptEntry {
            speaker: "최주임",
            text: "제주도면 이동 시간도 문제인 것 같아요. 비행기로 3시간 이상 걸리니까 이동만으로도 하루가 소비되고, 실제 워크숍 시간이 줄어들 것 같습니다.",
            timestamp: "01:09",
            badge_bg: Color32::from_rgb(255, 237, 212),
            badge_text: Color32::from_rgb(159, 45, 0),
        },
    ];

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(7.0);
            for entry in &entries {
                show_transcript_card(ui, entry);
                ui.add_space(10.0);
            }
        });
}

fn show_transcript_card(ui: &mut egui::Ui, entry: &TranscriptEntry) {
    let frame = egui::Frame::new()
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(229, 231, 235)))
        .corner_radius(CornerRadius::same(20))
        .inner_margin(egui::Margin::same(14));

    frame.show(ui, |ui| {
        ui.set_width(ui.available_width());

        // 헤더: 화자 뱃지 + 타임스탬프
        ui.horizontal(|ui| {
            let badge_frame = egui::Frame::new()
                .fill(entry.badge_bg)
                .corner_radius(CornerRadius::same(4))
                .inner_margin(egui::Margin::symmetric(8, 2));
            badge_frame.show(ui, |ui| {
                ui.label(
                    RichText::new(entry.speaker)
                        .size(12.0)
                        .color(entry.badge_text),
                );
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(entry.timestamp)
                        .size(12.0)
                        .color(Color32::from_rgb(106, 114, 130)),
                );
                ui.label(
                    RichText::new("🕐")
                        .size(10.0)
                        .color(Color32::from_rgb(106, 114, 130)),
                );
            });
        });

        ui.add_space(8.0);

        ui.label(
            RichText::new(entry.text)
                .size(12.5)
                .color(Color32::from_rgb(54, 65, 83)),
        );
    });
}
