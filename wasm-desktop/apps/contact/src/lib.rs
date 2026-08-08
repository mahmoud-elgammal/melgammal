use desktop_sdk::egui::{self, Color32, CornerRadius, Rect, RichText, Stroke, Vec2};
use desktop_sdk::{document_window, AppletContext, DesktopApplet};

#[derive(Default)]
pub struct ContactApp;

impl ContactApp {
    pub fn new() -> Self {
        Self
    }
}

impl DesktopApplet for ContactApp {
    fn id(&self) -> &'static str {
        "contact"
    }

    fn label(&self) -> &'static str {
        "Contact.txt"
    }

    fn window_title(&self) -> &'static str {
        "Contact"
    }

    fn accent_color(&self) -> Color32 {
        Color32::from_rgb(52, 199, 89)
    }

    fn draw_glyph(&self, painter: &egui::Painter, badge: Rect) {
        let w = badge.width();
        let h = badge.height();
        let accent = self.accent_color();

        let env = Rect::from_center_size(badge.center(), Vec2::new(w * 0.62, h * 0.44));
        painter.rect_filled(env, CornerRadius::from(h * 0.06), Color32::WHITE);
        let apex = egui::pos2(env.center().x, env.top() + env.height() * 0.55);
        painter.line_segment([env.left_top(), apex], Stroke::new(w * 0.045, accent));
        painter.line_segment([env.right_top(), apex], Stroke::new(w * 0.045, accent));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &mut AppletContext) {
        document_window(ui, |ui| {
            ui.heading("Get in Touch");
            ui.separator();
            ui.label("hello@melgammal.com");
            ui.add_space(8.0);
            ui.label("X · GitHub · LinkedIn");
            ui.add_space(8.0);
            ui.label(RichText::new("Cairo, Egypt").weak());
        });
    }
}
