use desktop_sdk::egui::{self, Color32, CornerRadius, Rect, RichText, Shadow, Stroke, Vec2};
use desktop_sdk::{AppletContext, DesktopApplet};

/// Preview reads straight from the shared filesystem — no bundled copies
/// of its own. These are just the two folders it offers as a sidebar
/// shortcut; anything else on disk is reachable through Finder instead.
const SECTIONS: [(&str, &str); 2] = [("/Home", "Documents"), ("/System", "About This Desktop")];

pub struct ReaderApp {
    selected: String,
}

impl ReaderApp {
    pub fn new() -> Self {
        Self {
            selected: "/Home/About.txt".to_string(),
        }
    }

    fn draw_page_icon(painter: &egui::Painter, badge: Rect, accent: Color32) {
        let w = badge.width();
        let h = badge.height();
        let doc = Rect::from_center_size(badge.center(), Vec2::new(w * 0.56, h * 0.72));
        painter.rect_filled(doc, CornerRadius::from(2.0), Color32::WHITE);
        painter.rect_stroke(
            doc,
            CornerRadius::from(2.0),
            Stroke::new(1.0, Color32::from_black_alpha(30)),
            egui::StrokeKind::Outside,
        );
        for i in 0..3 {
            let y = doc.top() + doc.height() * (0.28 + i as f32 * 0.24);
            painter.line_segment(
                [
                    egui::pos2(doc.left() + w * 0.1, y),
                    egui::pos2(doc.right() - w * 0.1, y),
                ],
                Stroke::new(1.4, accent),
            );
        }
    }
}

impl DesktopApplet for ReaderApp {
    fn id(&self) -> &'static str {
        "reader"
    }

    fn label(&self) -> &'static str {
        "Preview"
    }

    fn window_title(&self) -> &'static str {
        "Preview"
    }

    fn accent_color(&self) -> Color32 {
        Color32::from_rgb(237, 106, 94)
    }

    fn draw_glyph(&self, painter: &egui::Painter, badge: Rect) {
        let w = badge.width();
        let h = badge.height();
        let cx = badge.center().x;
        let cy = badge.center().y;
        let accent = self.accent_color();

        let page_w = w * 0.24;
        let page_h = h * 0.5;
        let left = Rect::from_min_size(
            egui::pos2(cx - page_w - 2.0, cy - page_h / 2.0),
            Vec2::new(page_w, page_h),
        );
        let right = Rect::from_min_size(
            egui::pos2(cx + 2.0, cy - page_h / 2.0),
            Vec2::new(page_w, page_h),
        );
        painter.rect_filled(left, CornerRadius::from(3.0), Color32::WHITE);
        painter.rect_filled(right, CornerRadius::from(3.0), Color32::WHITE);

        for i in 0..2 {
            let y = left.top() + left.height() * (0.32 + i as f32 * 0.32);
            painter.line_segment(
                [
                    egui::pos2(left.left() + 3.0, y),
                    egui::pos2(left.right() - 3.0, y),
                ],
                Stroke::new(w * 0.03, accent),
            );
            painter.line_segment(
                [
                    egui::pos2(right.left() + 3.0, y),
                    egui::pos2(right.right() - 3.0, y),
                ],
                Stroke::new(w * 0.03, accent),
            );
        }
    }

    fn default_size(&self) -> Vec2 {
        Vec2::new(560.0, 400.0)
    }

    fn ui(&mut self, ui: &mut egui::Ui, actx: &mut AppletContext) {
        if let Some(path) = actx.payload {
            self.selected = path.to_string();
        }

        let accent = self.accent_color();

        ui.horizontal_top(|ui| {
            egui::Frame::new()
                .fill(Color32::from_rgb(233, 233, 236))
                .inner_margin(egui::Margin::symmetric(12, 12))
                .show(ui, |ui| {
                    ui.set_width(150.0);
                    ui.set_min_height(320.0);

                    // `Frame::show` inherits the layout of wherever it was
                    // created — this frame lives inside `ui.horizontal_top`,
                    // so without this its children lay out left-to-right
                    // instead of stacking into a real sidebar list.
                    ui.vertical(|ui| {
                        for (folder, section_label) in SECTIONS {
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
                                let selected = self.selected == path;

                                let row = ui.horizontal(|ui| {
                                    let (rect, _) = ui.allocate_exact_size(
                                        Vec2::new(18.0, 18.0),
                                        egui::Sense::hover(),
                                    );
                                    Self::draw_page_icon(ui.painter(), rect, accent);
                                    ui.selectable_label(selected, &name)
                                });

                                if row.inner.clicked() {
                                    self.selected = path;
                                }
                            }

                            ui.add_space(12.0);
                        }
                    });
                });

            egui::Frame::new()
                .fill(Color32::WHITE)
                .stroke(Stroke::new(1.0, Color32::from_black_alpha(15)))
                .corner_radius(CornerRadius::from(6.0))
                .shadow(Shadow {
                    offset: [0, 3],
                    blur: 14,
                    spread: 0,
                    color: Color32::from_black_alpha(35),
                })
                .inner_margin(egui::Margin::same(0))
                .show(ui, |ui| {
                    ui.set_min_width(260.0);
                    ui.set_min_height(320.0);

                    ui.painter().rect_filled(
                        Rect::from_min_size(
                            ui.min_rect().min,
                            Vec2::new(ui.available_width().max(1.0), 4.0),
                        ),
                        CornerRadius::ZERO,
                        accent,
                    );

                    egui::Frame::new()
                        .inner_margin(egui::Margin::symmetric(24, 20))
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.visuals_mut().override_text_color = Some(Color32::from_gray(35));
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    let name = self
                                        .selected
                                        .rsplit('/')
                                        .next()
                                        .unwrap_or(&self.selected)
                                        .to_string();

                                    ui.label(
                                        RichText::new(&self.selected)
                                            .monospace()
                                            .size(10.0)
                                            .color(Color32::from_gray(150)),
                                    );
                                    ui.add_space(2.0);
                                    ui.heading(name);
                                    ui.add_space(10.0);

                                    match actx.vfs.read_file(&self.selected) {
                                        Ok(content) => {
                                            ui.label(RichText::new(content).size(13.0));
                                        }
                                        Err(e) => {
                                            ui.label(
                                                RichText::new(format!(
                                                    "Couldn't open this file: {e}"
                                                ))
                                                .italics()
                                                .weak(),
                                            );
                                        }
                                    }
                                });
                            });
                        });
                });
        });
    }
}
