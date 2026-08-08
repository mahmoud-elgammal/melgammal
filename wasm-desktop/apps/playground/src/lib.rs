use desktop_sdk::egui::{self, Color32, Rect, RichText, Stroke};
use desktop_sdk::{document_window, AppletContext, DesktopApplet};

#[derive(Default)]
pub struct PlaygroundApp;

impl PlaygroundApp {
    pub fn new() -> Self {
        Self
    }
}

impl DesktopApplet for PlaygroundApp {
    fn id(&self) -> &'static str {
        "playground"
    }

    fn label(&self) -> &'static str {
        "Playground"
    }

    fn window_title(&self) -> &'static str {
        "Playground"
    }

    fn accent_color(&self) -> Color32 {
        Color32::from_rgb(255, 92, 122)
    }

    fn draw_glyph(&self, painter: &egui::Painter, badge: Rect) {
        let w = badge.width();
        let h = badge.height();
        let cx = badge.center().x;
        let cy = badge.center().y;
        let accent = self.accent_color();

        let neck_top_l = egui::pos2(cx - w * 0.05, cy - h * 0.3);
        let neck_top_r = egui::pos2(cx + w * 0.05, cy - h * 0.3);
        let neck_bot_l = egui::pos2(cx - w * 0.16, cy - h * 0.02);
        let neck_bot_r = egui::pos2(cx + w * 0.16, cy - h * 0.02);
        let stroke = Stroke::new(w * 0.05, Color32::WHITE);
        painter.line_segment([neck_top_l, neck_bot_l], stroke);
        painter.line_segment([neck_top_r, neck_bot_r], stroke);
        painter.line_segment([neck_top_l, neck_top_r], stroke);
        painter.circle_filled(egui::pos2(cx, cy + h * 0.18), w * 0.22, Color32::WHITE);
        painter.circle_filled(egui::pos2(cx - w * 0.06, cy + h * 0.12), w * 0.035, accent);
        painter.circle_filled(egui::pos2(cx + w * 0.07, cy + h * 0.2), w * 0.03, accent);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &mut AppletContext) {
        document_window(ui, |ui| {
            ui.heading("Playground");
            ui.separator();
            ui.label(
                "A collection of creative-coding experiments — \
                 physics, shaders, generative art, and kinetic type.",
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new(
                    "This desktop itself is one: Rust + egui, \
                     compiled to WebAssembly, with every app \
                     living in its own crate.",
                )
                .italics(),
            );
        });
    }
}
