use desktop_sdk::egui::{self, Color32, CornerRadius, Rect, RichText, Vec2};
use desktop_sdk::{AppletContext, DesktopApplet};

#[derive(Default)]
pub struct MonitorApp;

impl MonitorApp {
    pub fn new() -> Self {
        Self
    }
}

impl DesktopApplet for MonitorApp {
    fn id(&self) -> &'static str {
        "monitor"
    }

    fn label(&self) -> &'static str {
        "Activity Monitor"
    }

    fn window_title(&self) -> &'static str {
        "Activity Monitor"
    }

    fn accent_color(&self) -> Color32 {
        Color32::from_rgb(255, 149, 0)
    }

    fn draw_glyph(&self, painter: &egui::Painter, badge: Rect) {
        let w = badge.width();
        let h = badge.height();
        let heights = [0.35, 0.7, 0.5];
        let bar_w = w * 0.14;
        let gap = w * 0.1;
        let total_w = 3.0 * bar_w + 2.0 * gap;
        let start_x = badge.center().x - total_w / 2.0;
        let base_y = badge.bottom() - h * 0.22;

        for (i, frac) in heights.iter().enumerate() {
            let bar_h = h * 0.56 * frac;
            let x = start_x + i as f32 * (bar_w + gap);
            let rect =
                Rect::from_min_size(egui::pos2(x, base_y - bar_h), Vec2::new(bar_w, bar_h));
            painter.rect_filled(rect, CornerRadius::from(bar_w * 0.25), Color32::WHITE);
        }
    }

    fn default_size(&self) -> Vec2 {
        Vec2::new(420.0, 320.0)
    }

    fn ui(&mut self, ui: &mut egui::Ui, actx: &mut AppletContext) {
        ui.visuals_mut().override_text_color = Some(Color32::from_gray(35));

        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(14, 12))
            .show(ui, |ui| {
                let mut kill_target: Option<u32> = None;

                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("monitor-grid")
                        .num_columns(4)
                        .striped(true)
                        .spacing(Vec2::new(16.0, 6.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("PID").strong().size(11.0));
                            ui.label(RichText::new("PPID").strong().size(11.0));
                            ui.label(RichText::new("NAME").strong().size(11.0));
                            ui.label("");
                            ui.end_row();

                            for p in actx.processes {
                                ui.label(RichText::new(p.pid.to_string()).monospace().size(12.0));
                                ui.label(RichText::new(p.ppid.to_string()).monospace().size(12.0));
                                ui.label(RichText::new(p.name).size(12.0));

                                let protected = p.pid == 1 || p.pid == 2;
                                if ui
                                    .add_enabled(!protected, egui::Button::new("Kill").small())
                                    .clicked()
                                {
                                    kill_target = Some(p.pid);
                                }
                                ui.end_row();
                            }
                        });
                });

                if let Some(pid) = kill_target {
                    actx.kill(pid);
                }
            });
    }
}
