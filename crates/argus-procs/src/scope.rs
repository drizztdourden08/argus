//! Whose process this is, from the project folders its folder, its executable and its command
//! line are in; and why a thread of a project may not kill it. The thread kill and the reaper
//! both go through [`refusal`].

use crate::project::ProjectFolders;
use crate::text::slashed;
use crate::types::{Guard, ProcRow};
use std::collections::{BTreeSet, HashMap};

/// Every folder of every known project, lower case with forward slashes, the longest first.
#[derive(Debug, Clone, Default)]
pub struct Scope {
    folders: Vec<(String, String)>,
}

fn clean(path: &str) -> String {
    let lower = slashed(path.trim()).to_lowercase();
    lower.trim_end_matches('/').to_string()
}

/// A drive or the file system root would claim every process: it is never a project folder.
fn usable(folder: &str) -> bool {
    let bytes = folder.as_bytes();
    let drive = bytes.len() == 2 && bytes[1] == b':';
    !folder.is_empty() && !drive
}

fn below(path: &str, folder: &str) -> bool {
    path == folder
        || path
            .strip_prefix(folder)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn opens(c: Option<char>) -> bool {
    c.is_none_or(|c| c.is_whitespace() || matches!(c, '"' | '\'' | '='))
}

fn closes(c: Option<char>) -> bool {
    c.is_none_or(|c| c.is_whitespace() || matches!(c, '"' | '\'' | '/' | ';' | ','))
}

impl Scope {
    /// The scope of these projects' folders.
    #[must_use]
    pub fn new(projects: &[ProjectFolders]) -> Self {
        let mut folders: Vec<(String, String)> = projects
            .iter()
            .flat_map(|p| p.folders.iter().map(|f| (clean(f), p.key.clone())))
            .filter(|(folder, _)| usable(folder))
            .collect();
        folders.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.cmp(b)));
        folders.dedup();
        Self { folders }
    }

    fn path_owner(&self, path: &str) -> Option<&str> {
        let path = clean(path);
        self.folders
            .iter()
            .find(|(folder, _)| below(&path, folder))
            .map(|(_, key)| key.as_str())
    }

    /// The project of each folder the command line names; at one place the longest folder wins.
    fn cmd_owners(&self, cmd: &str) -> Vec<&str> {
        let text = slashed(cmd).to_lowercase();
        let mut best: HashMap<usize, (usize, &str)> = HashMap::new();
        for (folder, key) in &self.folders {
            for (at, _) in text.match_indices(folder.as_str()) {
                let before = text[..at].chars().next_back();
                let after = text[at + folder.len()..].chars().next();
                if !(opens(before) && closes(after)) {
                    continue;
                }
                let slot = best.entry(at).or_insert((0, key.as_str()));
                if folder.len() > slot.0 {
                    *slot = (folder.len(), key.as_str());
                }
            }
        }
        best.into_values().map(|(_, key)| key).collect()
    }

    /// Every project the process touches: by its folder, its executable and its command line.
    #[must_use]
    pub fn owners(&self, proc: &ProcRow) -> BTreeSet<String> {
        let by_cwd = proc.cwd.as_deref().and_then(|c| self.path_owner(c));
        let by_exe = proc.path.as_deref().and_then(|p| self.path_owner(p));
        let by_cmd = self.cmd_owners(&proc.cmd);
        by_cwd
            .into_iter()
            .chain(by_exe)
            .chain(by_cmd)
            .map(str::to_string)
            .collect()
    }

    /// The project the process runs for: its folder's, else its executable's, else the first
    /// its command line names.
    #[must_use]
    pub fn owner(&self, proc: &ProcRow) -> Option<String> {
        let by_cwd = proc.cwd.as_deref().and_then(|c| self.path_owner(c));
        let by_exe = || proc.path.as_deref().and_then(|p| self.path_owner(p));
        let by_cmd = || self.cmd_owners(&proc.cmd).into_iter().min();
        by_cwd.or_else(by_exe).or_else(by_cmd).map(str::to_string)
    }
}

fn guarded(guard: Guard) -> &'static str {
    match guard {
        Guard::Claude => "is a Claude session process: it is never killed from here",
        Guard::Dashboard | Guard::Tray => {
            "is a process of the dashboard itself (argus serve, the tray or an MCP bridge): it is never killed from here"
        }
        Guard::System | Guard::Windows => "is a system process: it is never killed from here",
    }
}

fn unreadable(proc: &ProcRow) -> bool {
    proc.cwd.is_none()
        && proc.path.as_deref().is_none_or(str::is_empty)
        && proc.cmd.trim().is_empty()
}

/// Why a thread of project `own` may not kill this process, as the words that follow
/// "<name> <pid> ", or `None` when it may. `guard` is what the guard says of the process.
#[must_use]
pub fn refusal(proc: &ProcRow, guard: Option<Guard>, scope: &Scope, own: &str) -> Option<String> {
    if let Some(guard) = guard {
        return Some(guarded(guard).to_string());
    }
    if unreadable(proc) {
        return Some(
            "has details that cannot be read (no folder, executable or command line): it is never killed from here"
                .to_string(),
        );
    }
    let owners = scope.owners(proc);
    if let Some(other) = owners.iter().find(|key| *key != own) {
        return Some(format!(
            "belongs to project {other}, not {own}: only that project's threads may kill it"
        ));
    }
    owners.is_empty().then(|| {
        format!("is not inside a folder of project {own} (its root, its worktrees or its ci root)")
    })
}
