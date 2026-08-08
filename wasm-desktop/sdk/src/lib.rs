//! The interface every desktop app — first-party or third-party — plugs
//! into the shell through. An app crate depends only on this SDK, never on
//! the shell or on any other app: it gets an icon, a window, and its own
//! private state, and can ask the shell to open a sibling app by id.

pub use egui;

mod vfs;
pub use vfs::{Vfs, VfsNode};

/// Public metadata about an installed app: enough for another app (e.g. a
/// terminal's `ls`, or a file browser's shortcuts) to list or launch it by
/// id, without depending on its crate.
#[derive(Clone, Copy)]
pub struct AppInfo {
    pub id: &'static str,
    pub label: &'static str,
}

/// A live entry in the shell's process table. An app only has one of these
/// while a window for it is actually open — same as a real OS, where an
/// installed app isn't a running process until it's launched.
#[derive(Clone, Copy)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub name: &'static str,
}

/// Handed to [`DesktopApplet::ui`] each frame. This is the *only* way an
/// app can reach outside itself — it can read the installed-app registry
/// and the live process table, and request that another app be opened or
/// a process be killed, but it can't touch the shell's window manager,
/// other apps' state, or draw outside its own window.
pub struct AppletContext<'a> {
    pub apps: &'a [AppInfo],
    pub processes: &'a [ProcessInfo],
    /// The shared filesystem. Mutating it (Terminal's `mkdir`/`touch`/`rm`)
    /// takes effect immediately — file operations don't need the shell's
    /// approval the way opening or killing a window does.
    pub vfs: &'a mut Vfs,
    /// Set for exactly one frame when another app (or the shell itself)
    /// opened this one via [`Self::open_with`] — e.g. Finder handing a
    /// path to Preview, or the Trash icon handing Finder `/Trash`.
    pub payload: Option<&'a str>,
    open_requests: &'a mut Vec<&'static str>,
    kill_requests: &'a mut Vec<u32>,
    open_with_requests: &'a mut Vec<(&'static str, String)>,
}

impl<'a> AppletContext<'a> {
    pub fn new(
        apps: &'a [AppInfo],
        processes: &'a [ProcessInfo],
        vfs: &'a mut Vfs,
        payload: Option<&'a str>,
        open_requests: &'a mut Vec<&'static str>,
        kill_requests: &'a mut Vec<u32>,
        open_with_requests: &'a mut Vec<(&'static str, String)>,
    ) -> Self {
        Self {
            apps,
            processes,
            vfs,
            payload,
            open_requests,
            kill_requests,
            open_with_requests,
        }
    }

    /// Request that the app with this id be opened. Takes effect after the
    /// current frame's windows have all been drawn.
    pub fn open(&mut self, app_id: &'static str) {
        self.open_requests.push(app_id);
    }

    /// Like [`Self::open`], but the target app receives `payload` as
    /// [`Self::payload`] on the frame it's opened — e.g. a path to
    /// navigate to or preview.
    pub fn open_with(&mut self, app_id: &'static str, payload: impl Into<String>) {
        self.open_with_requests.push((app_id, payload.into()));
    }

    /// Request that the process with this pid be terminated. `init` (pid 1)
    /// and `WindowServer` (pid 2) can't be killed this way — same as a real
    /// OS refusing to let you kill pid 1.
    pub fn kill(&mut self, pid: u32) {
        self.kill_requests.push(pid);
    }
}

/// One installable desktop app.
pub trait DesktopApplet {
    /// Stable, lowercase identifier used for window ids, the `open`
    /// requests above, and the terminal's `open <id>` command.
    fn id(&self) -> &'static str;

    /// Caption shown under its desktop/dock icon.
    fn label(&self) -> &'static str;

    /// Title drawn in its window's traffic-light strip.
    fn window_title(&self) -> &'static str;

    /// Badge fill color behind [`Self::draw_glyph`].
    fn accent_color(&self) -> egui::Color32;

    /// Hand-drawn vector glyph for this app's icon, centered in `badge`.
    fn draw_glyph(&self, painter: &egui::Painter, badge: egui::Rect);

    /// Initial window size; apps that need more room (a terminal, a
    /// browser-style app) can override this.
    fn default_size(&self) -> egui::Vec2 {
        egui::vec2(380.0, 300.0)
    }

    /// Draws this app's window content. Called once per frame while its
    /// window is open — the shell already drew the traffic-light strip
    /// above `ui`.
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut AppletContext);
}

/// Common chrome for the simple "document" apps: dark text on the shell's
/// light window background, padded, scrollable. Apps aren't required to
/// use this — it's a convenience, not part of the trait contract.
pub fn document_window(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.visuals_mut().override_text_color = Some(egui::Color32::from_gray(35));
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(16, 12))
        .show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, add_contents);
        });
}
