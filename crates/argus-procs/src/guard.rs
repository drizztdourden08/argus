//! The protection no rule can lift: why a process may never be killed from here.

use crate::defaults::{ARGUS_CMD, PROTECTED_CMD, PROTECTED_NAMES};
use crate::text::{base_name, slashed};
use crate::tree::chain_of;
use crate::types::{Guard, ProcRow};
use fancy_regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

static PROTECTED_COMMANDS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    PROTECTED_CMD
        .iter()
        .filter_map(|src| Regex::new(src).ok())
        .collect()
});

static ARGUS_COMMAND: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(ARGUS_CMD).ok());

/// What the guard needs to know of a sample: the dashboard's own chain of processes, the tray and the Windows folder.
#[derive(Debug, Clone, Default)]
pub struct GuardCtx {
    own_chain: HashSet<u32>,
    tray: Option<u32>,
    system_root: Option<String>,
}

fn root_prefix(system_root: &str) -> String {
    format!(
        "{}/",
        slashed(system_root).to_lowercase().trim_end_matches('/')
    )
}

impl GuardCtx {
    /// The guard for one sample: `own_pid` and its parents, the tray's pid, and the Windows folder on Windows.
    #[must_use]
    pub fn new(
        procs: &[ProcRow],
        own_pid: u32,
        tray: Option<u32>,
        system_root: Option<&str>,
    ) -> Self {
        Self {
            own_chain: chain_of(own_pid, procs),
            tray,
            system_root: system_root.map(root_prefix),
        }
    }

    fn under_windows(&self, proc: &ProcRow) -> bool {
        let path = slashed(proc.path.as_deref().unwrap_or("")).to_lowercase();
        self.system_root
            .as_ref()
            .is_some_and(|root| path.starts_with(root))
    }

    fn by_command(cmd: &str) -> Option<Guard> {
        let line = slashed(cmd);
        let argus = ARGUS_COMMAND
            .as_ref()
            .is_some_and(|re| re.is_match(&line).unwrap_or(false));
        if argus {
            return Some(Guard::Dashboard);
        }
        let hit = PROTECTED_COMMANDS
            .iter()
            .any(|re| re.is_match(&line).unwrap_or(false));
        hit.then(|| {
            if line.to_lowercase().contains("claude") {
                Guard::Claude
            } else {
                Guard::Dashboard
            }
        })
    }

    /// Why the process may never be killed, or `None`.
    #[must_use]
    pub fn guard_of(&self, proc: &ProcRow, windows: bool) -> Option<Guard> {
        let base = base_name(&proc.name);
        if proc.pid <= 4 || (!windows && proc.pid == 1) {
            return Some(Guard::System);
        }
        if PROTECTED_NAMES.contains(&base.as_str()) {
            return Some(if base == "claude" {
                Guard::Claude
            } else {
                Guard::System
            });
        }
        if self.under_windows(proc) {
            return Some(Guard::Windows);
        }
        if self.own_chain.contains(&proc.pid) {
            return Some(Guard::Dashboard);
        }
        if self.tray == Some(proc.pid) {
            return Some(Guard::Tray);
        }
        Self::by_command(&proc.cmd)
    }
}
