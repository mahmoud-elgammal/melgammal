use desktop_sdk::egui::{self, Color32, CornerRadius, Rect, RichText, Stroke, Vec2};
use desktop_sdk::{AppletContext, DesktopApplet};

pub struct TextEditApp {
    current_path: Option<String>,
    buffer: String,
    saved_snapshot: String,
    new_file_name: String,
}

impl TextEditApp {
    pub fn new() -> Self {
        Self {
            current_path: None,
            buffer: String::new(),
            saved_snapshot: String::new(),
            new_file_name: "/Home/NewFile.txt".to_string(),
        }
    }

    fn load(&mut self, actx: &mut AppletContext, path: &str) {
        let content = actx.vfs.read_file(path).unwrap_or("").to_string();
        self.current_path = Some(path.to_string());
        self.buffer = content.clone();
        self.saved_snapshot = content;
    }
}

impl DesktopApplet for TextEditApp {
    fn id(&self) -> &'static str {
        "textedit"
    }

    fn label(&self) -> &'static str {
        "TextEdit"
    }

    fn window_title(&self) -> &'static str {
        "TextEdit"
    }

    fn accent_color(&self) -> Color32 {
        Color32::from_rgb(94, 143, 255)
    }

    fn draw_glyph(&self, painter: &egui::Painter, badge: Rect) {
        let w = badge.width();
        let h = badge.height();

        let doc = Rect::from_center_size(badge.center(), Vec2::new(w * 0.5, h * 0.66));
        painter.rect_filled(doc, CornerRadius::from(3.0), Color32::WHITE);

        for i in 0..3 {
            let y = doc.top() + doc.height() * (0.3 + i as f32 * 0.22);
            painter.line_segment(
                [
                    egui::pos2(doc.left() + w * 0.08, y),
                    egui::pos2(doc.right() - w * 0.08, y),
                ],
                Stroke::new(w * 0.04, self.accent_color()),
            );
        }

        let pencil_tip = egui::pos2(doc.right() - w * 0.02, doc.bottom() - h * 0.02);
        painter.line_segment(
            [pencil_tip - Vec2::new(w * 0.2, h * 0.2), pencil_tip],
            Stroke::new(w * 0.06, Color32::from_rgb(255, 204, 0)),
        );
    }

    fn default_size(&self) -> Vec2 {
        Vec2::new(500.0, 380.0)
    }

    fn ui(&mut self, ui: &mut egui::Ui, actx: &mut AppletContext) {
        ui.visuals_mut().override_text_color = Some(Color32::from_gray(35));

        ui.horizontal_top(|ui| {
            egui::Frame::new()
                .fill(Color32::from_rgb(238, 238, 238))
                .inner_margin(egui::Margin::symmetric(10, 10))
                .show(ui, |ui| {
                    ui.set_width(150.0);
                    ui.set_min_height(320.0);

                    for (folder, section_label) in [("/Home", "Home"), ("/System", "System")] {
                        ui.label(
                            RichText::new(section_label.to_uppercase())
                                .size(10.0)
                                .color(Color32::from_gray(120))
                                .strong(),
                        );
                        ui.add_space(4.0);

                        let entries = actx.vfs.list_dir(folder).unwrap_or_default();
                        for (name, is_dir) in entries {
                            if is_dir {
                                continue;
                            }
                            let path = format!("{folder}/{name}");
                            let selected = self.current_path.as_deref() == Some(path.as_str());
                            if ui.selectable_label(selected, &name).clicked() {
                                self.load(actx, &path);
                            }
                        }
                        ui.add_space(10.0);
                    }

                    ui.separator();
                    ui.add_space(6.0);
                    ui.label(RichText::new("NEW FILE").size(10.0).weak());
                    ui.text_edit_singleline(&mut self.new_file_name);
                    if ui.button("Create").clicked() && !self.new_file_name.is_empty() {
                        let path = self.new_file_name.clone();
                        if actx.vfs.write_file(&path, "").is_ok() {
                            self.load(actx, &path);
                        }
                    }
                });

            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(14, 12))
                .show(ui, |ui| {
                    ui.set_min_width(280.0);
                    ui.set_min_height(320.0);

                    match self.current_path.clone() {
                        Some(path) => {
                            let dirty = self.buffer != self.saved_snapshot;
                            ui.horizontal(|ui| {
                                let title = if dirty {
                                    format!("● {path}")
                                } else {
                                    path.clone()
                                };
                                ui.label(RichText::new(title).monospace().size(11.0));
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui
                                            .add_enabled(dirty, egui::Button::new("Save"))
                                            .clicked()
                                        {
                                            let _ =
                                                actx.vfs.write_file(&path, self.buffer.clone());
                                            self.saved_snapshot = self.buffer.clone();
                                        }
                                    },
                                );
                            });
                            ui.add_space(6.0);
                            ui.separator();
                            ui.add_space(6.0);

                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.buffer)
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(16)
                                        .font(egui::TextStyle::Monospace),
                                );
                            });
                        }
                        None => {
                            ui.label(
                                RichText::new(
                                    "No file open — pick one on the left, or create a new one.",
                                )
                                .weak(),
                            );
                        }
                    }
                });
        });
    }
}
