use desktop_sdk::egui::{self, Color32, CornerRadius, Rect, RichText, Shape, Stroke, Vec2};
use desktop_sdk::{document_window, AppletContext, DesktopApplet};

#[derive(Default)]
pub struct ResumeApp;

impl ResumeApp {
    pub fn new() -> Self {
        Self
    }
}

impl DesktopApplet for ResumeApp {
    fn id(&self) -> &'static str {
        "resume"
    }

    fn label(&self) -> &'static str {
        "Resume.pdf"
    }

    fn window_title(&self) -> &'static str {
        "Resume"
    }

    fn accent_color(&self) -> Color32 {
        Color32::from_rgb(147, 112, 219)
    }

    fn draw_glyph(&self, painter: &egui::Painter, badge: Rect) {
        let w = badge.width();
        let h = badge.height();
        let accent = self.accent_color();

        let doc = Rect::from_center_size(badge.center(), Vec2::new(w * 0.46, h * 0.62));
        painter.rect_filled(doc, CornerRadius::from(h * 0.05), Color32::WHITE);

        let fold = h * 0.14;
        let p1 = egui::pos2(doc.right() - fold, doc.top());
        let p2 = egui::pos2(doc.right(), doc.top());
        let p3 = egui::pos2(doc.right(), doc.top() + fold);
        painter.add(Shape::convex_polygon(vec![p1, p2, p3], accent, Stroke::NONE));

        for i in 0..3 {
            let y = doc.top() + doc.height() * (0.4 + i as f32 * 0.2);
            painter.line_segment(
                [
                    egui::pos2(doc.left() + w * 0.06, y),
                    egui::pos2(doc.right() - w * 0.12, y),
                ],
                Stroke::new(w * 0.035, Color32::from_gray(190)),
            );
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &mut AppletContext) {
        document_window(ui, |ui| {
            ui.heading("Resume");
            ui.separator();
            ui.label("Creative Technologist & System Architect");
            ui.add_space(8.0);
            ui.label(
                "Node.js · TypeScript · React · Astro · Docker · \
                 Kubernetes · AWS",
            );
            ui.add_space(8.0);
            ui.label(RichText::new("Full resume available at /resume").weak());
        });
    }
}
