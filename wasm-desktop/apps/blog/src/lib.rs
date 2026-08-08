use desktop_sdk::egui::{self, Color32, CornerRadius, Rect, Stroke, Vec2};
use desktop_sdk::{document_window, AppletContext, DesktopApplet};

#[derive(Default)]
pub struct BlogApp;

impl BlogApp {
    pub fn new() -> Self {
        Self
    }
}

impl DesktopApplet for BlogApp {
    fn id(&self) -> &'static str {
        "blog"
    }

    fn label(&self) -> &'static str {
        "Blog"
    }

    fn window_title(&self) -> &'static str {
        "Blog"
    }

    fn accent_color(&self) -> Color32 {
        Color32::from_rgb(64, 176, 166)
    }

    fn draw_glyph(&self, painter: &egui::Painter, badge: Rect) {
        let w = badge.width();
        let h = badge.height();
        let accent = self.accent_color();

        let page = Rect::from_center_size(badge.center(), Vec2::new(w * 0.5, h * 0.6));
        painter.rect_filled(page, CornerRadius::from(h * 0.06), Color32::WHITE);
        for i in 0..3 {
            let y = page.top() + page.height() * (0.26 + i as f32 * 0.26);
            painter.line_segment(
                [
                    egui::pos2(page.left() + w * 0.08, y),
                    egui::pos2(page.right() - w * 0.08, y),
                ],
                Stroke::new(w * 0.045, accent),
            );
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &mut AppletContext) {
        document_window(ui, |ui| {
            ui.heading("Latest Posts");
            ui.separator();
            for title in [
                "Building a 2D Physics Engine from First Principles in TypeScript",
                "Kinetic Typography: The Mathematics of Motion",
                "Architectural UX: Building Digital Monoliths",
                "Generative Systems: Coding Chaos at Scale",
            ] {
                ui.label(format!("• {title}"));
                ui.add_space(4.0);
            }
        });
    }
}
