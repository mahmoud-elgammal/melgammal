use desktop_sdk::egui::{self, Color32, CornerRadius, Rect, Vec2};
use desktop_sdk::{document_window, AppletContext, DesktopApplet};

#[derive(Default)]
pub struct AboutApp;

impl AboutApp {
    pub fn new() -> Self {
        Self
    }
}

impl DesktopApplet for AboutApp {
    fn id(&self) -> &'static str {
        "about"
    }

    fn label(&self) -> &'static str {
        "About.txt"
    }

    fn window_title(&self) -> &'static str {
        "About — Mahmoud S. El-Gammal"
    }

    fn accent_color(&self) -> Color32 {
        Color32::from_rgb(88, 101, 231)
    }

    fn draw_glyph(&self, painter: &egui::Painter, badge: Rect) {
        let w = badge.width();
        let h = badge.height();
        let cx = badge.center().x;
        let cy = badge.center().y;

        painter.circle_filled(egui::pos2(cx, cy - h * 0.14), w * 0.16, Color32::WHITE);
        let shoulders =
            Rect::from_center_size(egui::pos2(cx, cy + h * 0.24), Vec2::new(w * 0.5, h * 0.3));
        painter.rect_filled(shoulders, CornerRadius::from(h * 0.15), Color32::WHITE);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &mut AppletContext) {
        document_window(ui, |ui| {
            ui.heading("Mahmoud S. El-Gammal");
            ui.label("Cross-Functional Developer");
            ui.separator();
            ui.label(
                "I work across design, engineering, and product, \
                 prioritizing human understanding over just lines of \
                 code to bridge technology with psychology and art.",
            );
            ui.add_space(8.0);
            ui.label(
                "Background in Psychology, driven by curiosity, \
                 logic, and craft. Based in Egypt.",
            );
        });
    }
}
