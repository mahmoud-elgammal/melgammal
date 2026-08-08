use desktop_sdk::egui::{self, Color32, CornerRadius, Rect, RichText, Vec2};
use desktop_sdk::{document_window, AppletContext, DesktopApplet};

#[derive(Default)]
pub struct ProjectsApp;

impl ProjectsApp {
    pub fn new() -> Self {
        Self
    }
}

impl DesktopApplet for ProjectsApp {
    fn id(&self) -> &'static str {
        "projects"
    }

    fn label(&self) -> &'static str {
        "Projects"
    }

    fn window_title(&self) -> &'static str {
        "Projects"
    }

    fn accent_color(&self) -> Color32 {
        Color32::from_rgb(255, 176, 59)
    }

    fn draw_glyph(&self, painter: &egui::Painter, badge: Rect) {
        let w = badge.width();
        let h = badge.height();

        let tab = Rect::from_min_size(
            egui::pos2(badge.left() + w * 0.22, badge.top() + h * 0.26),
            Vec2::new(w * 0.28, h * 0.12),
        );
        painter.rect_filled(tab, CornerRadius::from(h * 0.05), Color32::WHITE);

        let body = Rect::from_min_size(
            egui::pos2(badge.left() + w * 0.18, badge.top() + h * 0.36),
            Vec2::new(w * 0.64, h * 0.4),
        );
        painter.rect_filled(body, CornerRadius::from(h * 0.08), Color32::WHITE);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &mut AppletContext) {
        document_window(ui, |ui| {
            ui.heading("Selected Work");
            ui.separator();
            for (title, stack) in [
                ("Syriazone", "Astro · Rust · PostgreSQL"),
                ("Crtly", "React Native · Node.js"),
                ("Tansto", "Framer · HealthTech"),
                ("Kleaner", "Astro · Tailwind"),
            ] {
                ui.label(RichText::new(title).strong());
                ui.label(RichText::new(stack).size(11.0).weak());
                ui.add_space(6.0);
            }
        });
    }
}
