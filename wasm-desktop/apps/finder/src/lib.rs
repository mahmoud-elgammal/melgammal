use desktop_sdk::egui::{self, Color32, CornerRadius, Rect, RichText, Sense, Shape, Stroke, Vec2};
use desktop_sdk::{AppletContext, DesktopApplet, Vfs};
use std::collections::HashMap;

const FOLDER_COLOR: Color32 = Color32::from_rgb(90, 160, 235);

pub struct FinderApp {
    current_path: String,
    last_click: HashMap<String, f64>,
}

impl FinderApp {
    pub fn new() -> Self {
        Self {
            current_path: "/Home".to_string(),
            last_click: HashMap::new(),
        }
    }

    fn file_color(name: &str) -> Color32 {
        match name {
            "About.txt" => Color32::from_rgb(88, 101, 231),
            "Resume.pdf" => Color32::from_rgb(147, 112, 219),
            "Contact.txt" => Color32::from_rgb(52, 199, 89),
            _ if name.ends_with(".md") => Color32::from_rgb(64, 176, 166),
            _ => Color32::from_rgb(150, 150, 158),
        }
    }

    /// Files under `/Home` are conventionally associated with the app that
    /// "owns" them and open it directly; everything else opens in Preview.
    fn app_for_file(path: &str, name: &str) -> &'static str {
        if path == "/Home" {
            match name {
                "About.txt" => return "about",
                "Resume.pdf" => return "resume",
                "Contact.txt" => return "contact",
                _ => {}
            }
        }
        "reader"
    }

    fn draw_generic_folder_glyph(painter: &egui::Painter, badge: Rect) {
        let w = badge.width();
        let h = badge.height();

        let tab = Rect::from_min_size(
            egui::pos2(badge.left() + w * 0.2, badge.top() + h * 0.22),
            Vec2::new(w * 0.28, h * 0.12),
        );
        painter.rect_filled(tab, CornerRadius::from(h * 0.05), Color32::WHITE);

        let body = Rect::from_min_size(
            egui::pos2(badge.left() + w * 0.14, badge.top() + h * 0.34),
            Vec2::new(w * 0.72, h * 0.42),
        );
        painter.rect_filled(body, CornerRadius::from(h * 0.08), Color32::WHITE);
    }

    fn draw_generic_file_glyph(painter: &egui::Painter, badge: Rect, accent: Color32) {
        let w = badge.width();
        let h = badge.height();

        let doc = Rect::from_center_size(badge.center(), Vec2::new(w * 0.5, h * 0.66));
        painter.rect_filled(doc, CornerRadius::from(3.0), Color32::WHITE);

        let fold = h * 0.16;
        let p1 = egui::pos2(doc.right() - fold, doc.top());
        let p2 = egui::pos2(doc.right(), doc.top());
        let p3 = egui::pos2(doc.right(), doc.top() + fold);
        painter.add(Shape::convex_polygon(vec![p1, p2, p3], accent, Stroke::NONE));

        for i in 0..3 {
            let y = doc.top() + doc.height() * (0.4 + i as f32 * 0.18);
            painter.line_segment(
                [
                    egui::pos2(doc.left() + w * 0.08, y),
                    egui::pos2(doc.right() - w * 0.12, y),
                ],
                Stroke::new(w * 0.035, Color32::from_gray(190)),
            );
        }
    }

    /// Draws one tile. `key` identifies it for double-click timing (a full
    /// path, so files with the same name in different folders don't get
    /// confused); `display` is what's shown under the icon. Returns
    /// `(opened, trash_requested)`; `trashable` controls whether "Move to
    /// Trash" appears in its right-click menu.
    fn draw_tile(
        &mut self,
        ui: &mut egui::Ui,
        key: &str,
        display: &str,
        color: Color32,
        trashable: bool,
        draw_glyph: impl FnOnce(&egui::Painter, Rect),
    ) -> (bool, bool) {
        let mut opened = false;
        let mut trash_requested = false;

        ui.vertical(|ui| {
            ui.set_width(64.0);
            ui.vertical_centered(|ui| {
                let (rect, response) =
                    ui.allocate_exact_size(Vec2::new(48.0, 42.0), Sense::click());
                ui.painter().rect_filled(rect, CornerRadius::from(8.0), color);
                draw_glyph(ui.painter(), rect);
                ui.add_space(2.0);
                ui.label(RichText::new(display).size(10.0));

                if response.clicked() {
                    let now = ui.input(|i| i.time);
                    let last = self.last_click.get(key).copied().unwrap_or(-1.0);
                    if now - last < 0.4 {
                        opened = true;
                        self.last_click.remove(key);
                    } else {
                        self.last_click.insert(key.to_string(), now);
                    }
                }

                if trashable {
                    response.context_menu(|ui| {
                        if ui.button("Move to Trash").clicked() {
                            trash_requested = true;
                            ui.close();
                        }
                    });
                }
            });
        });

        (opened, trash_requested)
    }

    fn draw_items(&mut self, ui: &mut egui::Ui, actx: &mut AppletContext) {
        let mut navigate_to: Option<String> = None;
        let mut open_app: Option<&'static str> = None;
        let mut trash_path: Option<String> = None;

        if self.current_path == "/Trash" {
            let has_items = !actx.vfs.list_dir("/Trash").unwrap_or_default().is_empty();
            if ui.add_enabled(has_items, egui::Button::new("Empty Trash")).clicked() {
                for (name, _) in actx.vfs.list_dir("/Trash").unwrap_or_default() {
                    let _ = actx.vfs.remove(&format!("/Trash/{name}"));
                }
            }
            ui.add_space(10.0);
        }

        if self.current_path != "/" {
            let key = format!("{}/..", self.current_path);
            let (opened, _) =
                self.draw_tile(ui, &key, "..", FOLDER_COLOR, false, Self::draw_generic_folder_glyph);
            if opened {
                navigate_to = Some(Vfs::resolve(&self.current_path, ".."));
            }
        }

        let entries = actx.vfs.list_dir(&self.current_path).unwrap_or_default();
        let in_trash = self.current_path == "/Trash";
        for (name, is_dir) in entries {
            let key = Vfs::resolve(&self.current_path, &name);
            let (opened, trashed) = if is_dir {
                self.draw_tile(
                    ui,
                    &key,
                    &name,
                    FOLDER_COLOR,
                    false,
                    Self::draw_generic_folder_glyph,
                )
            } else {
                let color = Self::file_color(&name);
                self.draw_tile(ui, &key, &name, color, !in_trash, |painter, rect| {
                    Self::draw_generic_file_glyph(painter, rect, color);
                })
            };

            if opened {
                if is_dir {
                    navigate_to = Some(key.clone());
                } else {
                    open_app = Some(Self::app_for_file(&self.current_path, &name));
                }
            }
            if trashed {
                trash_path = Some(key);
            }
        }

        if let Some(path) = navigate_to {
            self.current_path = path;
        }
        if let Some(app_id) = open_app {
            actx.open(app_id);
        }
        if let Some(path) = trash_path {
            let _ = actx.vfs.trash(&path);
        }
    }
}

impl DesktopApplet for FinderApp {
    fn id(&self) -> &'static str {
        "finder"
    }

    fn label(&self) -> &'static str {
        "Finder"
    }

    fn window_title(&self) -> &'static str {
        "Finder"
    }

    fn accent_color(&self) -> Color32 {
        Color32::from_rgb(66, 153, 225)
    }

    fn draw_glyph(&self, painter: &egui::Painter, badge: Rect) {
        let w = badge.width();
        let h = badge.height();
        let accent = self.accent_color();

        let tab = Rect::from_min_size(
            egui::pos2(badge.left() + w * 0.2, badge.top() + h * 0.24),
            Vec2::new(w * 0.26, h * 0.11),
        );
        painter.rect_filled(tab, CornerRadius::from(h * 0.05), Color32::WHITE);

        let body = Rect::from_min_size(
            egui::pos2(badge.left() + w * 0.16, badge.top() + h * 0.33),
            Vec2::new(w * 0.58, h * 0.36),
        );
        painter.rect_filled(body, CornerRadius::from(h * 0.08), Color32::WHITE);

        let lens_center = egui::pos2(badge.right() - w * 0.28, badge.bottom() - h * 0.26);
        painter.circle_stroke(lens_center, w * 0.11, Stroke::new(w * 0.035, accent));
        let handle_dir = Vec2::new(0.7, 0.7).normalized() * (w * 0.1);
        painter.line_segment(
            [
                lens_center + handle_dir * 0.9,
                lens_center + handle_dir * 1.6,
            ],
            Stroke::new(w * 0.04, accent),
        );
    }

    fn default_size(&self) -> Vec2 {
        Vec2::new(480.0, 340.0)
    }

    fn ui(&mut self, ui: &mut egui::Ui, actx: &mut AppletContext) {
        if let Some(path) = actx.payload {
            self.current_path = path.to_string();
        }

        ui.visuals_mut().override_text_color = Some(Color32::from_gray(35));

        ui.horizontal_top(|ui| {
            egui::Frame::new()
                .fill(Color32::from_rgb(238, 238, 238))
                .inner_margin(egui::Margin::symmetric(10, 10))
                .show(ui, |ui| {
                    ui.set_width(112.0);
                    ui.set_min_height(270.0);
                    ui.label(RichText::new("FAVORITES").size(10.5).weak());
                    ui.add_space(6.0);

                    for (path, label) in [
                        ("/", "Computer"),
                        ("/Home", "Home"),
                        ("/Projects", "Projects"),
                        ("/Blog", "Blog"),
                        ("/Playground", "Playground"),
                        ("/System", "System"),
                        ("/Trash", "Trash"),
                    ] {
                        let selected = self.current_path == path;
                        if ui.selectable_label(selected, label).clicked() {
                            self.current_path = path.to_string();
                        }
                    }
                });

            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(14, 12))
                .show(ui, |ui| {
                    ui.set_min_width(220.0);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(&self.current_path)
                                .monospace()
                                .size(10.5)
                                .weak(),
                        );
                        ui.add_space(6.0);
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing = Vec2::new(14.0, 14.0);
                                self.draw_items(ui, actx);
                            });
                        });
                    });
                });
        });
    }
}
