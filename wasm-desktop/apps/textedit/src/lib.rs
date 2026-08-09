use desktop_sdk::egui::{self, Color32, CornerRadius, Key, Rect, RichText, Stroke, Vec2};
use desktop_sdk::{AppletContext, DesktopApplet};

/// A navigation the user asked for while there were unsaved changes —
/// held until they confirm discarding, instead of silently overwriting
/// their edits (the previous version just clobbered the buffer).
enum PendingNav {
    Open(String),
    New(String),
}

pub struct TextEditApp {
    current_path: Option<String>,
    buffer: String,
    saved_snapshot: String,
    new_file_name: String,
    pending_nav: Option<PendingNav>,
}

impl TextEditApp {
    pub fn new() -> Self {
        Self {
            current_path: None,
            buffer: String::new(),
            saved_snapshot: String::new(),
            new_file_name: "/Home/NewFile.txt".to_string(),
            pending_nav: None,
        }
    }

    fn is_dirty(&self) -> bool {
        self.buffer != self.saved_snapshot
    }

    fn load(&mut self, actx: &mut AppletContext, path: &str) {
        let content = actx.vfs.read_file(path).unwrap_or("").to_string();
        self.current_path = Some(path.to_string());
        self.buffer = content.clone();
        self.saved_snapshot = content;
        self.pending_nav = None;
    }

    fn create_and_load(&mut self, actx: &mut AppletContext, path: &str) {
        if actx.vfs.write_file(path, "").is_ok() {
            self.load(actx, path);
        }
    }

    fn save(&mut self, actx: &mut AppletContext) {
        if let Some(path) = self.current_path.clone() {
            if actx.vfs.write_file(&path, self.buffer.clone()).is_ok() {
                self.saved_snapshot = self.buffer.clone();
            }
        }
    }

    /// Opening an existing file always just opens it (no data loss
    /// possible); only re-routes through the confirm bar if doing so
    /// would discard unsaved edits to a *different* file.
    fn request_open(&mut self, actx: &mut AppletContext, path: &str) {
        if self.current_path.as_deref() == Some(path) {
            return;
        }
        if self.is_dirty() {
            self.pending_nav = Some(PendingNav::Open(path.to_string()));
        } else {
            self.load(actx, path);
        }
    }

    fn request_new(&mut self, actx: &mut AppletContext, path: String) {
        // Someone already created this path — open it instead of wiping
        // it back to empty, which the original "Create" button did.
        if actx.vfs.exists(&path) {
            self.request_open(actx, &path);
            return;
        }
        if self.is_dirty() {
            self.pending_nav = Some(PendingNav::New(path));
        } else {
            self.create_and_load(actx, &path);
        }
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
        Vec2::new(560.0, 420.0)
    }

    fn ui(&mut self, ui: &mut egui::Ui, actx: &mut AppletContext) {
        ui.visuals_mut().override_text_color = Some(Color32::from_gray(35));

        if ui.input(|i| i.modifiers.command && i.key_pressed(Key::S)) {
            self.save(actx);
        }

        ui.horizontal_top(|ui| {
            egui::Frame::new()
                .fill(Color32::from_rgb(238, 238, 238))
                .inner_margin(egui::Margin::symmetric(10, 10))
                .show(ui, |ui| {
                    ui.set_width(150.0);
                    ui.set_min_height(360.0);

                    // `Frame::show`'s inner Ui inherits whatever layout was
                    // active where the frame was created — since this frame
                    // lives inside `ui.horizontal_top`, without this it lays
                    // its children out left-to-right instead of stacked.
                    ui.vertical(|ui| {
                        for (folder, section_label) in [("/Home", "Home"), ("/System", "System")]
                        {
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
                                let selected =
                                    self.current_path.as_deref() == Some(path.as_str());
                                if ui.selectable_label(selected, &name).clicked() {
                                    self.request_open(actx, &path);
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
                            self.request_new(actx, path);
                        }
                    });
                });

            egui::Frame::new()
                .inner_margin(egui::Margin::same(0))
                .show(ui, |ui| {
                    ui.set_min_width(300.0);
                    ui.set_min_height(360.0);

                    ui.painter().rect_filled(
                        Rect::from_min_size(ui.min_rect().min, Vec2::new(ui.available_width().max(1.0), 3.0)),
                        CornerRadius::ZERO,
                        self.accent_color(),
                    );

                    egui::Frame::new()
                        .inner_margin(egui::Margin::symmetric(14, 12))
                        .show(ui, |ui| {
                          ui.vertical(|ui| {
                            if let Some(nav) = &self.pending_nav {
                                let target = match nav {
                                    PendingNav::Open(p) | PendingNav::New(p) => p.clone(),
                                };
                                egui::Frame::new()
                                    .fill(Color32::from_rgb(255, 244, 214))
                                    .stroke(Stroke::new(1.0, Color32::from_rgb(230, 190, 110)))
                                    .corner_radius(CornerRadius::from(6.0))
                                    .inner_margin(egui::Margin::symmetric(10, 8))
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new(format!(
                                                "Discard unsaved changes and open {target}?"
                                            ))
                                            .size(12.0),
                                        );
                                        ui.horizontal(|ui| {
                                            if ui.button("Discard & Continue").clicked() {
                                                match self.pending_nav.take().unwrap() {
                                                    PendingNav::Open(p) => self.load(actx, &p),
                                                    PendingNav::New(p) => {
                                                        self.create_and_load(actx, &p)
                                                    }
                                                }
                                            }
                                            if ui.button("Cancel").clicked() {
                                                self.pending_nav = None;
                                            }
                                        });
                                    });
                                ui.add_space(8.0);
                            }

                            match self.current_path.clone() {
                                Some(path) => {
                                    let dirty = self.is_dirty();
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
                                                    self.save(actx);
                                                }
                                                ui.label(
                                                    RichText::new("⌘S")
                                                        .size(10.0)
                                                        .color(Color32::from_gray(140)),
                                                );
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
                                                .desired_rows(20)
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
                });
        });
    }
}
