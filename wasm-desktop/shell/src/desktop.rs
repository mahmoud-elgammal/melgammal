use desktop_sdk::{AppInfo, AppletContext, DesktopApplet, Vfs};
use eframe::egui::{
    self, Align2, Area, Color32, CornerRadius, FontId, Frame, Id, Layout, Margin, Order,
    PaintCallback, RichText, Sense, Shadow, Stroke, Vec2,
};
use eframe::egui_glow;
use js_sys::Date;
use std::collections::HashMap;
use std::sync::Arc;

use crate::gradient::GradientBackground;
use crate::process::{ProcessTable, WINDOW_SERVER_PID};

/// Every installed app, wrapped in one closed enum. Each variant's app
/// lives in its own crate behind the shared `DesktopApplet` trait — this
/// enum is just the shell's dispatch table, not shared implementation.
enum AnyApp {
    About(app_about::AboutApp),
    Projects(app_projects::ProjectsApp),
    Blog(app_blog::BlogApp),
    Contact(app_contact::ContactApp),
    Resume(app_resume::ResumeApp),
    Playground(app_playground::PlaygroundApp),
    Terminal(app_terminal::TerminalApp),
    Finder(app_finder::FinderApp),
    Reader(app_reader::ReaderApp),
    Monitor(app_monitor::MonitorApp),
    Calculator(app_calculator::CalculatorApp),
    TextEdit(app_textedit::TextEditApp),
}

macro_rules! delegate {
    ($self:expr, $app:ident => $body:expr) => {
        match $self {
            AnyApp::About($app) => $body,
            AnyApp::Projects($app) => $body,
            AnyApp::Blog($app) => $body,
            AnyApp::Contact($app) => $body,
            AnyApp::Resume($app) => $body,
            AnyApp::Playground($app) => $body,
            AnyApp::Terminal($app) => $body,
            AnyApp::Finder($app) => $body,
            AnyApp::Reader($app) => $body,
            AnyApp::Monitor($app) => $body,
            AnyApp::Calculator($app) => $body,
            AnyApp::TextEdit($app) => $body,
        }
    };
}

impl DesktopApplet for AnyApp {
    fn id(&self) -> &'static str {
        delegate!(self, app => app.id())
    }

    fn label(&self) -> &'static str {
        delegate!(self, app => app.label())
    }

    fn window_title(&self) -> &'static str {
        delegate!(self, app => app.window_title())
    }

    fn accent_color(&self) -> Color32 {
        delegate!(self, app => app.accent_color())
    }

    fn draw_glyph(&self, painter: &egui::Painter, badge: egui::Rect) {
        delegate!(self, app => app.draw_glyph(painter, badge))
    }

    fn default_size(&self) -> Vec2 {
        delegate!(self, app => app.default_size())
    }

    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut AppletContext) {
        delegate!(self, app => app.ui(ui, ctx))
    }
}

#[derive(Default)]
struct TitleBarAction {
    close: bool,
    minimize: bool,
    toggle_maximize: bool,
}

pub struct Desktop {
    apps: Vec<AnyApp>,
    app_infos: Vec<AppInfo>,
    processes: ProcessTable,
    vfs: Vfs,
    /// App id -> pid, for whichever apps currently have a running process
    /// (i.e. an open window). Absence from this map means "not running".
    running: HashMap<&'static str, u32>,
    /// When each currently-running app was launched (`ctx.input(|i| i.time)`
    /// at spawn time), so its window can ease in instead of just appearing.
    open_time: HashMap<&'static str, f64>,
    /// Apps whose window is hidden (still running — click the dock icon to
    /// bring it back) via the yellow traffic light or dock toggle.
    minimized: std::collections::HashSet<&'static str>,
    /// Apps whose window is filling the screen via the green traffic light.
    maximized: std::collections::HashSet<&'static str>,
    last_icon_click: HashMap<&'static str, f64>,
    gradient: Option<Arc<GradientBackground>>,
    show_desktop_icons: bool,
    /// A pending `open_with` delivery: (target app id, payload). Consumed
    /// the next time that app's window is drawn.
    pending_payload: Option<(&'static str, String)>,
    spotlight_open: bool,
    spotlight_query: String,
    control_center_open: bool,
    wifi_on: bool,
    bluetooth_on: bool,
    dnd_on: bool,
    volume: f32,
    brightness: f32,
    /// True while Cmd is held down mid-switch (Cmd+Tab held).
    switcher_active: bool,
    switcher_index: usize,
}

impl Desktop {
    pub fn new() -> Self {
        let apps = vec![
            AnyApp::About(app_about::AboutApp::new()),
            AnyApp::Projects(app_projects::ProjectsApp::new()),
            AnyApp::Blog(app_blog::BlogApp::new()),
            AnyApp::Contact(app_contact::ContactApp::new()),
            AnyApp::Resume(app_resume::ResumeApp::new()),
            AnyApp::Playground(app_playground::PlaygroundApp::new()),
            AnyApp::Terminal(app_terminal::TerminalApp::new()),
            AnyApp::Finder(app_finder::FinderApp::new()),
            AnyApp::Reader(app_reader::ReaderApp::new()),
            AnyApp::Monitor(app_monitor::MonitorApp::new()),
            AnyApp::Calculator(app_calculator::CalculatorApp::new()),
            AnyApp::TextEdit(app_textedit::TextEditApp::new()),
        ];
        let app_infos = apps
            .iter()
            .map(|app| AppInfo {
                id: app.id(),
                label: app.label(),
            })
            .collect();

        Self {
            apps,
            app_infos,
            // The kernel starts `init` (pid 1), which starts the GUI
            // (`WindowServer`, pid 2) — this desktop is itself a process,
            // not special-cased code sitting outside the model.
            processes: ProcessTable::boot(),
            vfs: {
                let mut vfs = Vfs::new();
                crate::fs_seed::seed(&mut vfs);
                vfs
            },
            running: HashMap::new(),
            open_time: HashMap::new(),
            minimized: std::collections::HashSet::new(),
            maximized: std::collections::HashSet::new(),
            last_icon_click: HashMap::new(),
            gradient: None,
            show_desktop_icons: true,
            pending_payload: None,
            spotlight_open: false,
            spotlight_query: String::new(),
            control_center_open: false,
            wifi_on: true,
            bluetooth_on: true,
            dnd_on: false,
            volume: 0.7,
            brightness: 0.85,
            switcher_active: false,
            switcher_index: 0,
        }
    }

    /// Spawns a process for `id` (child of `WindowServer`) if it isn't
    /// already running. Safe to call from anywhere that isn't already
    /// borrowing `self.apps`.
    fn spawn_by_id(&mut self, id: &'static str, now: f64) {
        if self.running.contains_key(id) {
            return;
        }
        if let Some(info) = self.app_infos.iter().find(|a| a.id == id) {
            let pid = self.processes.spawn(WINDOW_SERVER_PID, info.label);
            self.running.insert(id, pid);
            self.open_time.insert(id, now);
        }
    }

    /// Kills the process behind `id`, if any — closing its window.
    fn kill_by_id(&mut self, id: &str) {
        if let Some(pid) = self.running.remove(id) {
            self.processes.kill(pid);
        }
        self.minimized.remove(id);
        self.maximized.remove(id);
    }

    pub fn draw(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let toggle_spotlight = ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Space));
        if toggle_spotlight {
            self.spotlight_open = !self.spotlight_open;
            self.spotlight_query.clear();
        }

        self.draw_wallpaper(ui, frame);
        self.draw_menu_bar(ui);
        self.draw_icons(ui);
        self.draw_windows(ui.ctx());
        self.draw_dock(ui.ctx());
        self.draw_control_center(ui.ctx());
        self.draw_spotlight(ui.ctx());
        self.handle_app_switcher(ui.ctx());
    }

    /// Hold Cmd and press Tab (repeatedly) to cycle a HUD of running apps;
    /// releasing Cmd switches to whichever one is selected.
    fn handle_app_switcher(&mut self, ctx: &egui::Context) {
        let running_ids: Vec<&'static str> = self
            .apps
            .iter()
            .map(|a| a.id())
            .filter(|id| self.running.contains_key(id))
            .collect();

        if running_ids.len() < 2 {
            self.switcher_active = false;
            return;
        }

        let cmd_held = ctx.input(|i| i.modifiers.command);
        let tab_pressed = ctx.input(|i| i.key_pressed(egui::Key::Tab));

        if cmd_held && tab_pressed {
            if self.switcher_active {
                self.switcher_index = (self.switcher_index + 1) % running_ids.len();
            } else {
                self.switcher_active = true;
                self.switcher_index = 1 % running_ids.len();
            }
        }

        if self.switcher_active && !cmd_held {
            if let Some(&id) = running_ids.get(self.switcher_index) {
                self.minimized.remove(id);
                ctx.move_to_top(egui::LayerId::new(
                    Order::Middle,
                    Id::new(("desktop-window", id)),
                ));
            }
            self.switcher_active = false;
        }

        if !self.switcher_active {
            return;
        }

        let viewport = ctx.viewport_rect();
        Area::new(Id::new("app-switcher"))
            .order(Order::Foreground)
            .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, viewport.height() * -0.08))
            .show(ctx, |ui| {
                Frame::new()
                    .fill(Color32::from_rgba_unmultiplied(30, 30, 34, 235))
                    .corner_radius(CornerRadius::from(16.0))
                    .stroke(Stroke::new(1.0, Color32::from_white_alpha(30)))
                    .shadow(Shadow {
                        offset: [0, 14],
                        blur: 44,
                        spread: 0,
                        color: Color32::from_black_alpha(150),
                    })
                    .inner_margin(Margin::same(18))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 18.0;
                            for (i, &id) in running_ids.iter().enumerate() {
                                let Some(app) = self.apps.iter().find(|a| a.id() == id) else {
                                    continue;
                                };
                                let selected = i == self.switcher_index;
                                ui.vertical(|ui| {
                                    ui.set_width(56.0);
                                    ui.vertical_centered(|ui| {
                                        let (rect, _) = ui.allocate_exact_size(
                                            Vec2::splat(56.0),
                                            Sense::hover(),
                                        );
                                        let badge = egui::Rect::from_center_size(
                                            rect.center(),
                                            Vec2::splat(48.0),
                                        );
                                        ui.painter().rect_filled(
                                            badge,
                                            CornerRadius::from(12.0),
                                            app.accent_color(),
                                        );
                                        app.draw_glyph(ui.painter(), badge);
                                        if selected {
                                            ui.painter().rect_stroke(
                                                rect,
                                                CornerRadius::from(14.0),
                                                Stroke::new(2.0, Color32::WHITE),
                                                egui::StrokeKind::Outside,
                                            );
                                        }
                                        ui.add_space(4.0);
                                        ui.label(
                                            RichText::new(app.label())
                                                .size(10.5)
                                                .color(Color32::WHITE),
                                        );
                                    });
                                });
                            }
                        });
                    });
            });
    }

    /// An animated GLSL gradient, rendered on the background layer via a
    /// raw glow paint callback so it sits behind the menu bar, windows,
    /// and dock. Falls back to a flat fill if the glow context is
    /// unavailable (e.g. a non-WebGL backend).
    fn draw_wallpaper(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let rect = ui.ctx().viewport_rect();

        // The gradient is time-driven, so without this eframe only repaints
        // on input (mouse move, clicks), making the animation advance in
        // big, jerky steps instead of a smooth continuous flow.
        ui.ctx().request_repaint();

        if self.gradient.is_none() {
            if let Some(gl) = frame.gl() {
                self.gradient = Some(Arc::new(GradientBackground::new(gl)));
            }
        }

        if let Some(gradient) = self.gradient.clone() {
            let time = ui.input(|i| i.time) as f32;
            ui.painter().add(PaintCallback {
                rect,
                callback: Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                    gradient.paint(painter.gl(), time);
                })),
            });
        } else {
            ui.painter_at(rect)
                .rect_filled(rect, 0.0, Color32::from_rgb(20, 20, 30));
        }
    }

    fn clock_string() -> String {
        const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        const MONTHS: [&str; 12] = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];

        let now = Date::new_0();
        let weekday = WEEKDAYS[now.get_day() as usize];
        let month = MONTHS[now.get_month() as usize];
        let day = now.get_date();

        let hours24 = now.get_hours();
        let minutes = now.get_minutes();
        let period = if hours24 >= 12 { "PM" } else { "AM" };
        let hours12 = match hours24 % 12 {
            0 => 12,
            h => h,
        };

        format!("{weekday} {month} {day}  {hours12}:{minutes:02} {period}")
    }

    fn draw_menu_bar(&mut self, ui: &mut egui::Ui) {
        let bar_fill = Color32::from_black_alpha(140);

        egui::Panel::top(Id::new("menubar"))
            .exact_size(26.0)
            .frame(
                Frame::new()
                    .fill(bar_fill)
                    .inner_margin(Margin::symmetric(14, 0)),
            )
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    let (logo_rect, _) = ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
                    Self::draw_apple_mark(ui.painter(), logo_rect.center(), 13.0, bar_fill);
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("Finder")
                            .strong()
                            .color(Color32::WHITE)
                            .size(13.0),
                    );
                    ui.add_space(18.0);

                    Self::menu_bar_button(ui, "File", |ui| {
                        if ui.button("New Finder Window").clicked() {
                            let now = ui.input(|i| i.time);
                            self.spawn_by_id("finder", now);
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("Close All Windows").clicked() {
                            for (_, pid) in self.running.drain() {
                                self.processes.kill(pid);
                            }
                            self.minimized.clear();
                            self.maximized.clear();
                            ui.close();
                        }
                    });
                    ui.add_space(4.0);

                    Self::menu_bar_button(ui, "Edit", |ui| {
                        ui.add_enabled(false, egui::Button::new("Undo"));
                        ui.add_enabled(false, egui::Button::new("Cut"));
                        ui.add_enabled(false, egui::Button::new("Copy"));
                        ui.add_enabled(false, egui::Button::new("Paste"));
                    });
                    ui.add_space(4.0);

                    Self::menu_bar_button(ui, "View", |ui| {
                        ui.checkbox(&mut self.show_desktop_icons, "Show Desktop Icons");
                    });
                    ui.add_space(4.0);

                    Self::menu_bar_button(ui, "Window", |ui| {
                        for info in self.app_infos.clone() {
                            let mut is_open = self.running.contains_key(info.id);
                            if ui.checkbox(&mut is_open, info.label).changed() {
                                if is_open {
                                    let now = ui.input(|i| i.time);
                                    self.spawn_by_id(info.id, now);
                                } else {
                                    self.kill_by_id(info.id);
                                }
                            }
                        }
                    });
                    ui.add_space(4.0);

                    Self::menu_bar_button(ui, "Help", |ui| {
                        if ui.button("About This Desktop").clicked() {
                            let now = ui.input(|i| i.time);
                            self.spawn_by_id("reader", now);
                            ui.close();
                        }
                    });

                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(Self::clock_string())
                                .color(Color32::WHITE)
                                .size(12.5),
                        );
                        ui.add_space(14.0);
                        let (battery_rect, _) =
                            ui.allocate_exact_size(Vec2::new(22.0, 12.0), Sense::hover());
                        Self::draw_battery(ui.painter(), battery_rect.center(), 12.0);
                        ui.add_space(10.0);
                        let (signal_rect, _) =
                            ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
                        Self::draw_signal_bars(ui.painter(), signal_rect.center(), 14.0);
                        ui.add_space(12.0);
                        let (spotlight_rect, spotlight_response) =
                            ui.allocate_exact_size(Vec2::splat(14.0), Sense::click());
                        Self::draw_spotlight_icon(ui.painter(), spotlight_rect.center(), 13.0);
                        if spotlight_response.clicked() {
                            self.spotlight_open = !self.spotlight_open;
                            self.spotlight_query.clear();
                            self.control_center_open = false;
                        }
                        ui.add_space(12.0);
                        let (cc_rect, cc_response) =
                            ui.allocate_exact_size(Vec2::splat(14.0), Sense::click());
                        Self::draw_control_center_icon(ui.painter(), cc_rect.center(), 13.0);
                        if cc_response.clicked() {
                            self.control_center_open = !self.control_center_open;
                            self.spotlight_open = false;
                        }
                    });
                });
            });
    }

    /// A flat, frameless menu-bar label that opens a real dropdown on
    /// click — the "File Edit View..." row used to be inert text.
    fn menu_bar_button(ui: &mut egui::Ui, label: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
        let button = egui::Button::new(
            RichText::new(label)
                .color(Color32::from_white_alpha(215))
                .size(13.0),
        )
        .frame(false);
        egui::containers::menu::MenuButton::from_button(button).ui(ui, add_contents);
    }

    fn draw_apple_mark(painter: &egui::Painter, center: egui::Pos2, size: f32, bg: Color32) {
        painter.circle_filled(center, size * 0.5, Color32::WHITE);
        painter.circle_filled(
            center + Vec2::new(size * 0.36, -size * 0.02),
            size * 0.26,
            bg,
        );
        painter.circle_filled(
            center + Vec2::new(-size * 0.04, -size * 0.62),
            size * 0.13,
            Color32::WHITE,
        );
    }

    fn draw_signal_bars(painter: &egui::Painter, center: egui::Pos2, size: f32) {
        let bars = 4;
        let bar_w = size * 0.14;
        let gap = size * 0.07;
        let max_h = size * 0.72;
        let total_w = bars as f32 * bar_w + (bars - 1) as f32 * gap;
        let start_x = center.x - total_w / 2.0;

        for i in 0..bars {
            let h = max_h * (0.35 + 0.65 * (i as f32 / (bars - 1) as f32));
            let x = start_x + i as f32 * (bar_w + gap);
            let rect = egui::Rect::from_min_size(
                egui::pos2(x, center.y + max_h / 2.0 - h),
                Vec2::new(bar_w, h),
            );
            painter.rect_filled(rect, CornerRadius::from(bar_w * 0.3), Color32::WHITE);
        }
    }

    fn draw_battery(painter: &egui::Painter, center: egui::Pos2, size: f32) {
        let body = egui::Rect::from_center_size(center, Vec2::new(size * 1.7, size * 0.85));
        painter.rect_stroke(
            body,
            CornerRadius::from(size * 0.14),
            Stroke::new(1.2, Color32::WHITE),
            egui::StrokeKind::Inside,
        );

        let nub = egui::Rect::from_min_size(
            egui::pos2(body.right() + 1.0, center.y - size * 0.14),
            Vec2::new(size * 0.12, size * 0.28),
        );
        painter.rect_filled(nub, CornerRadius::from(1.0), Color32::WHITE);

        let margin = 2.5;
        let fill = egui::Rect::from_min_size(
            egui::pos2(body.left() + margin, body.top() + margin),
            Vec2::new(body.width() * 0.72 - margin, body.height() - margin * 2.0),
        );
        painter.rect_filled(fill, CornerRadius::from(1.0), Color32::WHITE);
    }

    fn draw_spotlight_icon(painter: &egui::Painter, center: egui::Pos2, size: f32) {
        let lens_center = center - Vec2::new(size * 0.08, size * 0.08);
        painter.circle_stroke(lens_center, size * 0.3, Stroke::new(size * 0.09, Color32::WHITE));
        let dir = Vec2::new(1.0, 1.0).normalized();
        painter.line_segment(
            [
                lens_center + dir * size * 0.28,
                lens_center + dir * size * 0.55,
            ],
            Stroke::new(size * 0.1, Color32::WHITE),
        );
    }

    fn draw_control_center_icon(painter: &egui::Painter, center: egui::Pos2, size: f32) {
        for i in 0..2 {
            let y = center.y + (i as f32 - 0.5) * size * 0.42;
            let rect = egui::Rect::from_center_size(
                egui::pos2(center.x, y),
                Vec2::new(size * 0.62, size * 0.26),
            );
            painter.rect_stroke(
                rect,
                CornerRadius::from(rect.height() * 0.5),
                Stroke::new(size * 0.05, Color32::WHITE),
                egui::StrokeKind::Inside,
            );
            let knob_x = if i == 0 {
                rect.right() - rect.height() * 0.5
            } else {
                rect.left() + rect.height() * 0.5
            };
            painter.circle_filled(egui::pos2(knob_x, y), rect.height() * 0.3, Color32::WHITE);
        }
    }

    /// Desktop icons live top-right, macOS-style, and require a
    /// double-click to open — single clicks just acknowledge focus.
    fn draw_icons(&mut self, ui: &mut egui::Ui) {
        if !self.show_desktop_icons {
            return;
        }

        let spacing = 92.0;
        let margin = 20.0;
        let viewport = ui.ctx().viewport_rect();

        for (i, app) in self.apps.iter().enumerate() {
            let id = app.id();
            let pos = egui::pos2(
                viewport.right() - margin - 76.0,
                viewport.top() + 40.0 + i as f32 * spacing,
            );

            Area::new(Id::new(("desktop-icon", id)))
                .fixed_pos(pos)
                .order(Order::Middle)
                .show(ui.ctx(), |ui| {
                    ui.set_max_width(76.0);
                    ui.vertical_centered(|ui| {
                        let (rect, response) =
                            ui.allocate_exact_size(Vec2::new(64.0, 56.0), Sense::click());

                        if response.hovered() {
                            ui.painter().rect_filled(
                                rect,
                                CornerRadius::from(8.0),
                                Color32::from_white_alpha(30),
                            );
                        }

                        let badge = egui::Rect::from_center_size(
                            rect.center() - Vec2::new(0.0, 4.0),
                            Vec2::splat(40.0),
                        );
                        ui.painter().rect_filled(
                            badge,
                            CornerRadius::from(badge.width() * 0.26),
                            app.accent_color(),
                        );
                        app.draw_glyph(ui.painter(), badge);

                        // Faux drop-shadow: draw the label twice, offset,
                        // so it stays legible over any part of the wallpaper.
                        let label_pos = rect.center_bottom() + Vec2::new(0.0, 10.0);
                        ui.painter().text(
                            label_pos + Vec2::new(0.0, 1.0),
                            Align2::CENTER_TOP,
                            app.label(),
                            FontId::proportional(11.0),
                            Color32::from_black_alpha(160),
                        );
                        ui.painter().text(
                            label_pos,
                            Align2::CENTER_TOP,
                            app.label(),
                            FontId::proportional(11.0),
                            Color32::WHITE,
                        );

                        if response.clicked() {
                            let now = ui.input(|i| i.time);
                            let last = self.last_icon_click.get(id).copied().unwrap_or(-1.0);

                            if now - last < 0.4 {
                                if !self.running.contains_key(id) {
                                    let pid = self.processes.spawn(WINDOW_SERVER_PID, app.label());
                                    self.running.insert(id, pid);
                                    self.open_time.insert(id, now);
                                }
                                self.last_icon_click.remove(id);
                            } else {
                                self.last_icon_click.insert(id, now);
                            }
                        }

                        response.context_menu(|ui| {
                            if self.running.contains_key(id) {
                                if ui.button("Quit").clicked() {
                                    if let Some(pid) = self.running.remove(id) {
                                        self.processes.kill(pid);
                                    }
                                    self.minimized.remove(id);
                                    self.maximized.remove(id);
                                    ui.close();
                                }
                            } else if ui.button("Open").clicked() {
                                let now = ui.input(|i| i.time);
                                let pid = self.processes.spawn(WINDOW_SERVER_PID, app.label());
                                self.running.insert(id, pid);
                                self.open_time.insert(id, now);
                                ui.close();
                            }
                        });
                    });
                });
        }
    }

    fn draw_windows(&mut self, ctx: &egui::Context) {
        let app_infos = self.app_infos.clone();
        let processes = self.processes.snapshot();
        let now = ctx.input(|i| i.time);
        let viewport = ctx.viewport_rect();
        let mut open_requests: Vec<&'static str> = Vec::new();
        let mut kill_requests: Vec<u32> = Vec::new();
        let mut minimize_requests: Vec<&'static str> = Vec::new();
        let mut maximize_toggle_requests: Vec<&'static str> = Vec::new();
        let mut open_with_requests: Vec<(&'static str, String)> = Vec::new();

        for (i, app) in self.apps.iter_mut().enumerate() {
            let id = app.id();
            if !self.running.contains_key(id) || self.minimized.contains(id) {
                continue;
            }

            // Ease the window in over its first 160ms instead of having it
            // just appear — one of the things that made this desktop feel
            // static rather than alive.
            let opened_at = self.open_time.get(id).copied().unwrap_or(now);
            let t = (((now - opened_at) / 0.16).clamp(0.0, 1.0)) as f32;
            let eased_opacity = 1.0 - (1.0 - t) * (1.0 - t) * (1.0 - t);
            if t < 1.0 {
                ctx.request_repaint();
            }

            let mut close_clicked = false;
            let mut minimize_clicked = false;
            let mut maximize_clicked = false;
            let default_pos = egui::pos2(140.0 + i as f32 * 32.0, 70.0 + i as f32 * 26.0);
            let window_title = app.window_title();
            let maximized = self.maximized.contains(id);

            let payload = if self.pending_payload.as_ref().map(|(pid, _)| *pid) == Some(id) {
                self.pending_payload.take().map(|(_, p)| p)
            } else {
                None
            };

            let mut window = egui::Window::new(window_title)
                .id(Id::new(("desktop-window", id)))
                .title_bar(false)
                .frame(Self::window_frame());

            window = if maximized {
                let rect = egui::Rect::from_min_size(
                    egui::pos2(viewport.left() + 20.0, viewport.top() + 36.0),
                    egui::vec2(
                        (viewport.width() - 40.0).max(240.0),
                        (viewport.height() - 36.0 - 100.0).max(160.0),
                    ),
                );
                window.resizable(false).fixed_rect(rect)
            } else {
                window
                    .resizable(true)
                    .default_pos(default_pos)
                    .default_size(app.default_size())
            };

            window.show(ctx, |ui| {
                ui.set_opacity(eased_opacity);
                let action = Self::draw_title_strip(ui, window_title, id);
                close_clicked = action.close;
                minimize_clicked = action.minimize;
                maximize_clicked = action.toggle_maximize;
                ui.add_space(2.0);
                let mut actx = AppletContext::new(
                    &app_infos,
                    &processes,
                    &mut self.vfs,
                    payload.as_deref(),
                    &mut open_requests,
                    &mut kill_requests,
                    &mut open_with_requests,
                );
                app.ui(ui, &mut actx);
            });

            if close_clicked {
                kill_requests.push(*self.running.get(id).unwrap_or(&0));
            }
            if minimize_clicked {
                minimize_requests.push(id);
            }
            if maximize_clicked {
                maximize_toggle_requests.push(id);
            }
        }

        for id in open_requests {
            self.spawn_by_id(id, now);
        }

        for (id, payload) in open_with_requests {
            self.spawn_by_id(id, now);
            self.pending_payload = Some((id, payload));
        }

        for id in minimize_requests {
            self.minimized.insert(id);
        }

        for id in maximize_toggle_requests {
            if !self.maximized.remove(id) {
                self.maximized.insert(id);
            }
        }

        for pid in kill_requests {
            let owning_id = self
                .running
                .iter()
                .find(|(_, &running_pid)| running_pid == pid)
                .map(|(&id, _)| id);
            if let Some(id) = owning_id {
                self.running.remove(id);
                self.maximized.remove(id);
                self.minimized.remove(id);
            }
            self.processes.kill(pid);
        }
    }

    fn window_frame() -> Frame {
        Frame::new()
            .fill(Color32::from_rgba_unmultiplied(246, 246, 246, 250))
            .stroke(Stroke::new(1.0, Color32::from_black_alpha(30)))
            .corner_radius(CornerRadius::from(10.0))
            .shadow(Shadow {
                offset: [0, 10],
                blur: 28,
                spread: 0,
                color: Color32::from_black_alpha(90),
            })
            .inner_margin(Margin::same(0))
    }

    /// Custom macOS traffic-light title strip. Returns `true` if the red
    /// close dot was clicked this frame.
    fn draw_title_strip(ui: &mut egui::Ui, window_title: &str, id: &str) -> TitleBarAction {
        let height = 32.0;
        let width = ui.available_width();
        let (rect, _response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        let painter = ui.painter_at(rect);

        painter.line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            Stroke::new(1.0, Color32::from_black_alpha(25)),
        );

        let cy = rect.top() + height / 2.0;
        let dot_colors = [
            Color32::from_rgb(255, 95, 86),
            Color32::from_rgb(255, 189, 46),
            Color32::from_rgb(39, 201, 63),
        ];
        let salts = ["traffic-close", "traffic-min", "traffic-max"];

        let mut action = TitleBarAction::default();
        for (i, color) in dot_colors.iter().enumerate() {
            let center = egui::pos2(rect.left() + 17.0 + i as f32 * 18.0, cy);
            let hit_rect = egui::Rect::from_center_size(center, Vec2::splat(16.0));
            let response = ui.interact(hit_rect, ui.id().with((salts[i], id)), Sense::click());
            if response.clicked() {
                match i {
                    0 => action.close = true,
                    1 => action.minimize = true,
                    _ => action.toggle_maximize = true,
                }
            }
            let radius = if response.hovered() { 6.5 } else { 6.0 };
            painter.circle_filled(center, radius, *color);
        }

        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            window_title,
            FontId::proportional(13.0),
            Color32::from_gray(60),
        );

        action
    }

    fn draw_control_center(&mut self, ctx: &egui::Context) {
        if !self.control_center_open {
            return;
        }

        let viewport = ctx.viewport_rect();
        Area::new(Id::new("control-center"))
            .order(Order::Foreground)
            .fixed_pos(egui::pos2(viewport.right() - 260.0, 32.0))
            .show(ctx, |ui| {
                Frame::new()
                    .fill(Color32::from_rgba_unmultiplied(40, 40, 44, 235))
                    .corner_radius(CornerRadius::from(12.0))
                    .stroke(Stroke::new(1.0, Color32::from_white_alpha(30)))
                    .shadow(Shadow {
                        offset: [0, 8],
                        blur: 28,
                        spread: 0,
                        color: Color32::from_black_alpha(120),
                    })
                    .inner_margin(Margin::same(14))
                    .show(ui, |ui| {
                        ui.set_width(220.0);
                        ui.visuals_mut().override_text_color = Some(Color32::WHITE);

                        ui.checkbox(&mut self.wifi_on, "Wi-Fi");
                        ui.checkbox(&mut self.bluetooth_on, "Bluetooth");
                        ui.checkbox(&mut self.dnd_on, "Do Not Disturb");
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);
                        ui.label("Brightness");
                        ui.add(egui::Slider::new(&mut self.brightness, 0.0..=1.0).show_value(false));
                        ui.add_space(6.0);
                        ui.label("Volume");
                        ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).show_value(false));
                    });
            });
    }

    fn draw_spotlight(&mut self, ctx: &egui::Context) {
        if !self.spotlight_open {
            return;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.spotlight_open = false;
            self.spotlight_query.clear();
            return;
        }

        let viewport = ctx.viewport_rect();
        let now = ctx.input(|i| i.time);

        Area::new(Id::new("spotlight-backdrop"))
            .order(Order::Foreground)
            .fixed_pos(viewport.min)
            .show(ctx, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(viewport.size(), Sense::click());
                ui.painter()
                    .rect_filled(rect, CornerRadius::ZERO, Color32::from_black_alpha(80));
                if response.clicked() {
                    self.spotlight_open = false;
                    self.spotlight_query.clear();
                }
            });

        let mut launch: Option<&'static str> = None;
        let mut launch_with: Option<(&'static str, String)> = None;

        Area::new(Id::new("spotlight"))
            .order(Order::Foreground)
            .anchor(Align2::CENTER_TOP, Vec2::new(0.0, viewport.height() * 0.16))
            .show(ctx, |ui| {
                Frame::new()
                    .fill(Color32::from_rgba_unmultiplied(40, 40, 44, 245))
                    .corner_radius(CornerRadius::from(12.0))
                    .stroke(Stroke::new(1.0, Color32::from_white_alpha(30)))
                    .shadow(Shadow {
                        offset: [0, 14],
                        blur: 44,
                        spread: 0,
                        color: Color32::from_black_alpha(150),
                    })
                    .inner_margin(Margin::same(14))
                    .show(ui, |ui| {
                        ui.set_width(440.0);

                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.spotlight_query)
                                .font(FontId::proportional(19.0))
                                .text_color(Color32::WHITE)
                                .frame(Frame::NONE)
                                .desired_width(f32::INFINITY)
                                .hint_text("Spotlight Search"),
                        );
                        response.request_focus();

                        let query = self.spotlight_query.to_lowercase();
                        if query.is_empty() {
                            return;
                        }

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(4.0);

                        egui::ScrollArea::vertical()
                            .max_height(280.0)
                            .show(ui, |ui| {
                                for info in &self.app_infos {
                                    if !info.label.to_lowercase().contains(&query)
                                        && !info.id.contains(&query)
                                    {
                                        continue;
                                    }
                                    let text = RichText::new(format!("  {}", info.label))
                                        .color(Color32::WHITE)
                                        .size(14.0);
                                    if ui.selectable_label(false, text).clicked() {
                                        launch = Some(info.id);
                                    }
                                }

                                for (path, is_dir) in self.vfs.all_paths() {
                                    if is_dir || path == "/" || !path.to_lowercase().contains(&query) {
                                        continue;
                                    }
                                    let text = RichText::new(format!("  {path}"))
                                        .color(Color32::from_white_alpha(210))
                                        .size(12.5)
                                        .monospace();
                                    if ui.selectable_label(false, text).clicked() {
                                        launch_with = Some(("reader", path.to_string()));
                                    }
                                }
                            });
                    });
            });

        if let Some(id) = launch {
            self.spawn_by_id(id, now);
            self.spotlight_open = false;
            self.spotlight_query.clear();
        }
        if let Some((id, payload)) = launch_with {
            self.spawn_by_id(id, now);
            self.pending_payload = Some((id, payload));
            self.spotlight_open = false;
            self.spotlight_query.clear();
        }
    }

    fn draw_dock(&mut self, ctx: &egui::Context) {
        let slot = 46.0;
        let gap = 10.0;

        Area::new(Id::new("dock"))
            .order(Order::Foreground)
            .anchor(Align2::CENTER_BOTTOM, Vec2::new(0.0, -14.0))
            .movable(false)
            .show(ctx, |ui| {
                Frame::new()
                    .fill(Color32::from_black_alpha(130))
                    .stroke(Stroke::new(1.0, Color32::from_white_alpha(35)))
                    .corner_radius(CornerRadius::from(20.0))
                    .shadow(Shadow {
                        offset: [0, 6],
                        blur: 20,
                        spread: 0,
                        color: Color32::from_black_alpha(80),
                    })
                    .inner_margin(Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = gap;
                            for i in 0..self.apps.len() {
                                self.draw_dock_icon(ui, i, slot);
                            }

                            ui.add_space(2.0);
                            let (sep_rect, _) = ui.allocate_exact_size(
                                Vec2::new(1.0, slot * 0.8),
                                Sense::hover(),
                            );
                            ui.painter().line_segment(
                                [
                                    egui::pos2(sep_rect.center().x, sep_rect.top()),
                                    egui::pos2(sep_rect.center().x, sep_rect.bottom()),
                                ],
                                Stroke::new(1.0, Color32::from_white_alpha(50)),
                            );
                            ui.add_space(2.0);

                            self.draw_trash_icon(ui, slot);
                        });
                    });
            });
    }

    fn draw_dock_tooltip(painter: &egui::Painter, anchor_rect: egui::Rect, label: &str) {
        let galley =
            painter.layout_no_wrap(label.to_string(), FontId::proportional(11.0), Color32::WHITE);
        let label_pos = egui::pos2(anchor_rect.center().x, anchor_rect.top() - 16.0);
        let bg_rect = egui::Rect::from_center_size(label_pos, galley.size() + Vec2::new(14.0, 8.0));
        painter.rect_filled(bg_rect, CornerRadius::from(6.0), Color32::from_black_alpha(210));
        painter.text(
            label_pos,
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(11.0),
            Color32::WHITE,
        );
    }

    /// A decorative, non-functional Trash icon — every real macOS dock ends
    /// with one, and its absence was one of the things making this dock
    /// read as an obvious mockup rather than the real thing.
    fn draw_trash_icon(&mut self, ui: &mut egui::Ui, slot: f32) {
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(slot), Sense::click());

        if response.clicked() {
            let now = ui.input(|i| i.time);
            self.spawn_by_id("finder", now);
            self.pending_payload = Some(("finder", "/Trash".to_string()));
        }

        let target_scale = if response.hovered() { 1.2 } else { 1.0 };
        let scale =
            ui.ctx()
                .animate_value_with_time(ui.id().with("dock-trash-scale"), target_scale, 0.15);

        let painter = ui.painter();
        let size = slot * 0.6 * scale;
        let center = egui::pos2(rect.center().x, rect.bottom() - size / 2.0 - 3.0);

        let lid = egui::Rect::from_center_size(
            center - Vec2::new(0.0, size * 0.42),
            Vec2::new(size * 0.92, size * 0.1),
        );
        painter.rect_filled(lid, CornerRadius::from(size * 0.05), Color32::from_white_alpha(225));

        let body = egui::Rect::from_center_size(
            center + Vec2::new(0.0, size * 0.06),
            Vec2::new(size * 0.7, size * 0.68),
        );
        painter.rect_stroke(
            body,
            CornerRadius::from(size * 0.1),
            Stroke::new(size * 0.06, Color32::from_white_alpha(225)),
            egui::StrokeKind::Inside,
        );

        for i in -1..=1 {
            let x = center.x + i as f32 * size * 0.16;
            painter.line_segment(
                [
                    egui::pos2(x, body.top() + size * 0.1),
                    egui::pos2(x, body.bottom() - size * 0.1),
                ],
                Stroke::new(size * 0.045, Color32::from_white_alpha(190)),
            );
        }

        if response.hovered() {
            Self::draw_dock_tooltip(painter, rect, "Trash");
        }
    }

    fn draw_dock_icon(&mut self, ui: &mut egui::Ui, index: usize, slot: f32) {
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(slot), Sense::click());
        let id = self.apps[index].id();

        let target_scale = if response.hovered() { 1.3 } else { 1.0 };
        let scale =
            ui.ctx()
                .animate_value_with_time(ui.id().with(("dock-scale", id)), target_scale, 0.15);

        if response.clicked() {
            if self.running.contains_key(id) {
                // Already running: the dock toggles minimize/restore, same
                // as clicking an app's icon on a real dock — it never
                // quits the app. That's what the red traffic light is for.
                if !self.minimized.remove(id) {
                    self.minimized.insert(id);
                }
            } else {
                let now = ui.input(|i| i.time);
                let pid = self.processes.spawn(WINDOW_SERVER_PID, self.apps[index].label());
                self.running.insert(id, pid);
                self.open_time.insert(id, now);
            }
        }

        let app = &self.apps[index];
        let painter = ui.painter();
        let icon_size = slot * scale;
        let icon_center = egui::pos2(rect.center().x, rect.bottom() - icon_size / 2.0);
        let badge = egui::Rect::from_center_size(icon_center, Vec2::splat(icon_size * 0.86));

        painter.rect_filled(
            badge,
            CornerRadius::from(badge.width() * 0.26),
            app.accent_color(),
        );
        app.draw_glyph(painter, badge);

        let pid = self.running.get(id).copied();
        let minimized = self.minimized.contains(id);
        if pid.is_some() {
            painter.circle_filled(
                egui::pos2(rect.center().x, rect.bottom() + 6.0),
                2.2,
                Color32::from_white_alpha(if minimized { 110 } else { 230 }),
            );
        }

        if response.hovered() {
            let tooltip = match pid {
                Some(pid) if minimized => format!("{} (pid {pid}, minimized)", app.label()),
                Some(pid) => format!("{} (pid {pid})", app.label()),
                None => app.label().to_string(),
            };
            Self::draw_dock_tooltip(painter, rect, &tooltip);
        }

        response.context_menu(|ui| {
            if pid.is_some() {
                if minimized {
                    if ui.button("Show").clicked() {
                        self.minimized.remove(id);
                        ui.close();
                    }
                } else if ui.button("Minimize").clicked() {
                    self.minimized.insert(id);
                    ui.close();
                }
                if ui.button("Quit").clicked() {
                    if let Some(pid) = self.running.remove(id) {
                        self.processes.kill(pid);
                    }
                    self.minimized.remove(id);
                    self.maximized.remove(id);
                    ui.close();
                }
            } else if ui.button("Open").clicked() {
                let now = ui.input(|i| i.time);
                let pid = self.processes.spawn(WINDOW_SERVER_PID, self.apps[index].label());
                self.running.insert(id, pid);
                self.open_time.insert(id, now);
                ui.close();
            }
        });
    }
}
