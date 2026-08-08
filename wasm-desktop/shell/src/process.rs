use desktop_sdk::ProcessInfo;

/// The kernel's first process — everything else is its descendant.
pub const INIT_PID: u32 = 1;

/// The GUI, spawned by `init` at boot. The desktop you're looking at is
/// itself a process in this table, not special-cased code running outside
/// the model — every app window you open is spawned as its child.
pub const WINDOW_SERVER_PID: u32 = 2;

pub struct ProcessTable {
    next_pid: u32,
    processes: Vec<ProcessInfo>,
}

impl ProcessTable {
    /// Boots the table the way a real OS would: the kernel starts `init`
    /// (pid 1), which starts the GUI (`WindowServer`, pid 2).
    pub fn boot() -> Self {
        let mut table = Self {
            next_pid: 1,
            processes: Vec::new(),
        };
        let init = table.spawn(0, "init");
        debug_assert_eq!(init, INIT_PID);
        let gui = table.spawn(init, "WindowServer");
        debug_assert_eq!(gui, WINDOW_SERVER_PID);
        table
    }

    pub fn spawn(&mut self, ppid: u32, name: &'static str) -> u32 {
        let pid = self.next_pid;
        self.next_pid += 1;
        self.processes.push(ProcessInfo { pid, ppid, name });
        pid
    }

    /// Terminates a process. Returns `false` (and does nothing) for `init`
    /// or `WindowServer` — same as a real OS refusing `kill -9 1`.
    pub fn kill(&mut self, pid: u32) -> bool {
        if pid == INIT_PID || pid == WINDOW_SERVER_PID {
            return false;
        }
        let before = self.processes.len();
        self.processes.retain(|p| p.pid != pid);
        self.processes.len() != before
    }

    pub fn snapshot(&self) -> Vec<ProcessInfo> {
        self.processes.clone()
    }
}
