//! A thread ends its own process (`coord proc kill <pid>` from a project's thread): only a
//! process whose folder, executable or command line is inside the thread's own project folders,
//! and with `tree` every process of its tree too. Checked on a fresh sample, killed by pid.

use crate::error::ProcsError;
use crate::ports::{Activity, Level};
use crate::scope::{Scope, refusal};
use crate::service::{Inner, locked};
use crate::text::base_name;
use crate::tree::descendants_of;
use crate::types::{ProcRow, Sample};
use serde::Serialize;

/// The kind of the activity lines of a thread's own kills.
pub const SELF_KILL_KIND: &str = "proc-kill";

/// A thread of `project` asks to end `pid`, and with `tree` every process it started.
#[derive(Debug, Clone, Default)]
pub struct SelfKillRequest {
    pub project: String,
    pub thread: String,
    pub pid: u32,
    pub tree: bool,
    pub reason: Option<String>,
}

/// The answer to a thread's kill.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelfKillOutcome {
    pub killed: bool,
    pub pid: u32,
    pub name: String,
    pub exe: String,
    /// The other processes of the tree that were ended with it.
    pub tree: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// The executable of a process as a line names it: its path, else its name.
pub(crate) fn exe_of(row: &ProcRow) -> String {
    row.path
        .clone()
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| row.name.clone())
}

fn why_text(reason: Option<&str>) -> &str {
    reason.unwrap_or("no reason given")
}

impl Inner {
    fn self_note(&self, level: Level, text: String) {
        self.ports.host.activity(&Activity {
            kind: SELF_KILL_KIND,
            level,
            text,
        });
    }

    /// Reads the folder of each row that has none.
    pub(crate) fn fill_cwds(&self, rows: &mut [ProcRow]) {
        let missing: Vec<u32> = rows
            .iter()
            .filter(|r| r.cwd.is_none())
            .map(|r| r.pid)
            .collect();
        if missing.is_empty() {
            return;
        }
        let found = self.ports.sampler.cwds(&missing);
        for row in rows.iter_mut().filter(|r| r.cwd.is_none()) {
            row.cwd = found.get(&row.pid).cloned();
        }
    }

    /// The process first, then (with `tree`) every process it started, with their folders.
    pub(crate) fn members(&self, raw: &Sample, root: &ProcRow, tree: bool) -> Vec<ProcRow> {
        let mut rows = vec![root.clone()];
        if tree {
            let kids = descendants_of(root.pid, &raw.procs);
            let mut found: Vec<ProcRow> = raw
                .procs
                .iter()
                .filter(|p| kids.contains(&p.pid))
                .cloned()
                .collect();
            found.sort_by_key(|p| p.pid);
            rows.extend(found);
        }
        self.fill_cwds(&mut rows);
        rows
    }

    /// Why a thread of `own` may not kill these rows (the first is the one it named).
    pub(crate) fn scope_refusal(
        &self,
        raw: &Sample,
        rows: &[ProcRow],
        own: &str,
    ) -> Option<String> {
        let root = rows.first()?;
        let guard = self.guard_for(raw);
        let scope = Scope::new(&self.ports.host.project_folders());
        let windows = raw.is_windows();
        rows.iter().find_map(|row| {
            let why = refusal(row, guard.guard_of(row, windows), &scope, own)?;
            let (name, pid) = (base_name(&row.name), row.pid);
            Some(if pid == root.pid {
                format!("{name} {pid} {why}")
            } else {
                format!(
                    "the tree of {} {} holds {name} {pid}, which {why}",
                    base_name(&root.name),
                    root.pid
                )
            })
        })
    }

    fn self_refuse(&self, request: &SelfKillRequest, error: ProcsError) -> ProcsError {
        let reason = request
            .reason
            .as_deref()
            .map_or(String::new(), |r| format!(" (reason given: {r})"));
        self.self_note(
            Level::Warn,
            format!(
                "refused thread {} of {} the kill of {}: {}{reason}",
                request.thread,
                request.project,
                request.pid,
                error.message()
            ),
        );
        error
    }

    fn self_target(&self, request: &SelfKillRequest) -> Result<(Sample, ProcRow), ProcsError> {
        let raw = locked(&self.view).raw.clone().ok_or_else(|| {
            ProcsError::internal("the process table could not be read: nothing was killed")
        })?;
        let root = raw
            .procs
            .iter()
            .find(|p| p.pid == request.pid)
            .cloned()
            .ok_or_else(|| {
                ProcsError::not_found(format!(
                    "process {} is not running, or its details cannot be read: nothing was killed",
                    request.pid
                ))
            })?;
        Ok((raw, root))
    }

    fn self_end(&self, request: &SelfKillRequest, rows: &[ProcRow]) -> SelfKillOutcome {
        let root = &rows[0];
        let kids: Vec<u32> = rows.iter().skip(1).map(|r| r.pid).collect();
        let result = self.ports.killer.kill(root.pid, request.tree, &kids);
        let gone = result.ok && self.wait_gone(root.pid);
        let exe = exe_of(root);
        let who = format!("thread {} of {}", request.thread, request.project);
        let why = why_text(request.reason.as_deref());
        if result.ok {
            let tree = match (request.tree, kids.len()) {
                (false, _) => String::new(),
                (true, 0) => " and its tree".to_string(),
                (true, n) => format!(" and its tree ({n} more)"),
            };
            let closing = if gone {
                ""
            } else {
                ", it may still be closing"
            };
            self.self_note(
                Level::Info,
                format!("{who} killed {exe} {}{tree}{closing}: {why}", root.pid),
            );
        } else {
            let detail = result.detail.as_deref().unwrap_or("refused");
            self.self_note(
                Level::Error,
                format!("{who} could not kill {exe} {}: {detail}: {why}", root.pid),
            );
        }
        SelfKillOutcome {
            killed: result.ok,
            pid: root.pid,
            name: root.name.clone(),
            exe,
            tree: if result.ok { kids } else { Vec::new() },
            error: (!result.ok).then(|| result.detail.unwrap_or_else(|| "refused".to_string())),
        }
    }

    /// `{killed, pid, name, exe, tree}`; `forbidden`, `not_found` or `bad_request` otherwise.
    pub(crate) fn self_kill(
        &self,
        request: &SelfKillRequest,
    ) -> Result<SelfKillOutcome, ProcsError> {
        if request.pid == 0 {
            return Err(ProcsError::bad_request("pid must be a positive integer"));
        }
        let mut engine = locked(&self.engine);
        self.sample_once(&mut engine);
        let (raw, root) = self
            .self_target(request)
            .map_err(|e| self.self_refuse(request, e))?;
        let rows = self.members(&raw, &root, request.tree);
        if let Some(why) = self.scope_refusal(&raw, &rows, &request.project) {
            return Err(self.self_refuse(request, ProcsError::forbidden(why)));
        }
        let outcome = self.self_end(request, &rows);
        self.sample_once(&mut engine);
        Ok(outcome)
    }
}
