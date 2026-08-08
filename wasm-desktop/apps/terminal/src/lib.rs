use desktop_sdk::egui::{self, Color32, FontId, Frame, Key, Rect, RichText, TextEdit, Vec2};
use desktop_sdk::{AppletContext, DesktopApplet, Vfs};
use js_sys::Date;

pub struct TerminalApp {
    input: String,
    history: Vec<String>,
    command_history: Vec<String>,
    history_index: Option<usize>,
    cwd: String,
}

impl TerminalApp {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            history: vec![
                "Desktop OS [Rust + egui + WebAssembly]".to_string(),
                "Type 'help' to see available commands.".to_string(),
                String::new(),
            ],
            command_history: Vec::new(),
            history_index: None,
            cwd: "/Home".to_string(),
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

    fn recall(&mut self, direction: i32) {
        if self.command_history.is_empty() {
            return;
        }

        let last_index = self.command_history.len() - 1;
        let next_index = match (self.history_index, direction) {
            (None, -1) => Some(last_index),
            (None, _) => None,
            (Some(i), -1) => Some(i.saturating_sub(1)),
            (Some(i), _) if i + 1 <= last_index => Some(i + 1),
            (Some(_), _) => None,
        };

        self.history_index = next_index;
        self.input = match next_index {
            Some(i) => self.command_history[i].clone(),
            None => String::new(),
        };
    }

    fn run(&mut self, actx: &mut AppletContext) {
        let raw = self.input.trim().to_string();
        self.input.clear();
        self.history_index = None;

        if raw.is_empty() {
            self.history.push("$".to_string());
            return;
        }

        self.history.push(format!("$ {raw}"));
        self.command_history.push(raw.clone());

        let mut parts = raw.split_whitespace();
        let cmd = parts.next().unwrap_or("");
        let rest: Vec<&str> = parts.collect();

        match cmd {
            "help" => self.history.push(
                "Commands: help, whoami, about, apps, open <app>, ps, kill <pid>, \
                 pwd, cd, ls, cat, mkdir, touch, rm, echo <text> [> file], date, clear"
                    .to_string(),
            ),
            "whoami" => self.history.push("mahmoud @ desktop-os".to_string()),
            "about" => self.history.push(
                "Mahmoud S. El-Gammal — cross-functional developer working across design, \
                 engineering, and product."
                    .to_string(),
            ),
            "date" => self.history.push(Self::clock_string()),
            "apps" => {
                let listing = actx
                    .apps
                    .iter()
                    .map(|app| app.id)
                    .collect::<Vec<_>>()
                    .join("  ");
                self.history.push(listing);
            }
            "pwd" => self.history.push(self.cwd.clone()),
            "cd" => {
                let target = rest.first().copied().unwrap_or("/Home");
                let resolved = Vfs::resolve(&self.cwd, target);
                if actx.vfs.is_dir(&resolved) {
                    self.cwd = resolved;
                } else {
                    self.history.push(format!("cd: no such directory: {target}"));
                }
            }
            "ls" => {
                let target = rest.first().copied().unwrap_or(".");
                let resolved = Vfs::resolve(&self.cwd, target);
                match actx.vfs.list_dir(&resolved) {
                    Ok(entries) => {
                        let listing = entries
                            .iter()
                            .map(|(name, is_dir)| {
                                if *is_dir {
                                    format!("{name}/")
                                } else {
                                    name.clone()
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("  ");
                        self.history.push(listing);
                    }
                    Err(e) => self.history.push(format!("ls: {target}: {e}")),
                }
            }
            "cat" => match rest.first() {
                Some(target) => {
                    let resolved = Vfs::resolve(&self.cwd, target);
                    match actx.vfs.read_file(&resolved) {
                        Ok(content) => self.history.push(content.to_string()),
                        Err(e) => self.history.push(format!("cat: {target}: {e}")),
                    }
                }
                None => self.history.push("usage: cat <file>".to_string()),
            },
            "mkdir" => match rest.first() {
                Some(target) => {
                    let resolved = Vfs::resolve(&self.cwd, target);
                    if let Err(e) = actx.vfs.mkdir(&resolved) {
                        self.history.push(format!("mkdir: {target}: {e}"));
                    }
                }
                None => self.history.push("usage: mkdir <dir>".to_string()),
            },
            "touch" => match rest.first() {
                Some(target) => {
                    let resolved = Vfs::resolve(&self.cwd, target);
                    if !actx.vfs.exists(&resolved) {
                        if let Err(e) = actx.vfs.write_file(&resolved, "") {
                            self.history.push(format!("touch: {target}: {e}"));
                        }
                    }
                }
                None => self.history.push("usage: touch <file>".to_string()),
            },
            "rm" => match rest.first() {
                Some(target) => {
                    let resolved = Vfs::resolve(&self.cwd, target);
                    if let Err(e) = actx.vfs.trash(&resolved) {
                        self.history.push(format!("rm: {target}: {e}"));
                    }
                }
                None => self.history.push("usage: rm <path>".to_string()),
            },
            "ps" => {
                self.history.push("  PID  PPID  NAME".to_string());
                for p in actx.processes {
                    self.history
                        .push(format!("{:>5} {:>5}  {}", p.pid, p.ppid, p.name));
                }
            }
            "kill" => match rest.first() {
                Some(arg) => match arg.parse::<u32>() {
                    Ok(pid) if pid == 1 || pid == 2 => self
                        .history
                        .push(format!("kill: ({pid}) - Operation not permitted")),
                    Ok(pid) if actx.processes.iter().any(|p| p.pid == pid) => {
                        actx.kill(pid);
                        self.history.push(format!("Terminated process {pid}"));
                    }
                    Ok(pid) => self
                        .history
                        .push(format!("kill: ({pid}) - No such process")),
                    Err(_) => self.history.push(format!("kill: illegal pid: {arg}")),
                },
                None => self.history.push("usage: kill <pid>".to_string()),
            },
            "clear" => self.history.clear(),
            "echo" => match rest.iter().position(|s| *s == ">") {
                Some(gt) => {
                    let text = rest[..gt].join(" ");
                    match rest.get(gt + 1) {
                        Some(target) => {
                            let resolved = Vfs::resolve(&self.cwd, target);
                            if let Err(e) = actx.vfs.write_file(&resolved, text) {
                                self.history.push(format!("echo: {target}: {e}"));
                            }
                        }
                        None => self.history.push("usage: echo <text> > <file>".to_string()),
                    }
                }
                None => self.history.push(rest.join(" ")),
            },
            "open" => match rest.first() {
                Some(target) => match actx.apps.iter().find(|app| app.id == *target) {
                    Some(app) => {
                        self.history.push(format!("Opening {}…", app.label));
                        actx.open(app.id);
                    }
                    None => self.history.push(format!("open: unknown target '{target}'")),
                },
                None => self.history.push("usage: open <app>".to_string()),
            },
            "sudo" => self.history.push("Nice try.".to_string()),
            _ => self.history.push(format!("command not found: {cmd}")),
        }
    }
}

impl DesktopApplet for TerminalApp {
    fn id(&self) -> &'static str {
        "terminal"
    }

    fn label(&self) -> &'static str {
        "Terminal"
    }

    fn window_title(&self) -> &'static str {
        "Terminal — zsh"
    }

    fn accent_color(&self) -> Color32 {
        Color32::from_rgb(38, 40, 44)
    }

    fn draw_glyph(&self, painter: &egui::Painter, badge: Rect) {
        painter.text(
            badge.center(),
            egui::Align2::CENTER_CENTER,
            ">_",
            FontId::monospace(badge.width() * 0.38),
            Color32::from_rgb(120, 255, 140),
        );
    }

    fn default_size(&self) -> Vec2 {
        Vec2::new(460.0, 320.0)
    }

    fn ui(&mut self, ui: &mut egui::Ui, actx: &mut AppletContext) {
        ui.visuals_mut().override_text_color = Some(Color32::from_rgb(210, 255, 210));

        Frame::new()
            .fill(Color32::from_rgb(18, 18, 20))
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.set_min_height(240.0);

                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .max_height(210.0)
                    .show(ui, |ui| {
                        for line in &self.history {
                            ui.label(
                                RichText::new(line)
                                    .monospace()
                                    .size(12.5)
                                    .color(Color32::from_rgb(190, 255, 190)),
                            );
                        }
                    });

                ui.add_space(4.0);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("{} $", self.cwd))
                            .monospace()
                            .color(Color32::from_rgb(120, 255, 140)),
                    );

                    let response = ui.add(
                        TextEdit::singleline(&mut self.input)
                            .font(FontId::monospace(12.5))
                            .text_color(Color32::from_rgb(210, 255, 210))
                            .frame(Frame::NONE)
                            .desired_width(f32::INFINITY)
                            .hint_text("type 'help'"),
                    );

                    if response.has_focus() {
                        if ui.input(|i| i.key_pressed(Key::ArrowUp)) {
                            self.recall(-1);
                        }
                        if ui.input(|i| i.key_pressed(Key::ArrowDown)) {
                            self.recall(1);
                        }
                    }

                    if response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                        self.run(actx);
                        response.request_focus();
                    }
                });
            });
    }
}
