//! Making a request: who may ask, whether the ask has a sane shape, and the record it leaves.

use super::record::{Mode, RequestRec, RequestStatus, Target};
use super::text::{idle_text, one_line, queued_text, what_text};
use super::{IDLE_MAX_MINUTES, IDLE_MIN_MINUTES, MAX_ACTIVE, MAX_TARGETS, record::Brief};
use crate::board::Caller;
use crate::clock::iso_from_ms;
use crate::doc::WorktreeDoc;
use crate::error::{ErrorCode, Result, WtError};
use crate::guards::check_caller;
use crate::paths::is_worktree_id;
use crate::remove::forbidden;
use crate::request::RemoveInput;
use crate::service::Service;
use serde::Serialize;

const OWNER_ENTRY_MAX: usize = 60;
const REASON_MAX: usize = 200;
const TARGET_MAX: usize = 400;

/// What a thread asks for.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug)]
pub struct RequestInput {
    pub project: String,
    pub caller: Caller,
    /// Named worktrees or leftover folders (ids or paths).
    pub targets: Vec<String>,
    pub finished: bool,
    pub all_leftovers: bool,
    pub branch: bool,
    pub discard: bool,
    pub owner_entry: Option<String>,
    pub reason: Option<String>,
    pub older_than_days: Option<f64>,
    /// `finished` only: minutes (60 to 43200).
    pub idle_minutes: Option<f64>,
}

impl RequestInput {
    /// An ask with nothing in it yet.
    pub fn new(project: &str, caller: Caller) -> Self {
        Self {
            project: project.to_string(),
            caller,
            targets: Vec::new(),
            finished: false,
            all_leftovers: false,
            branch: false,
            discard: false,
            owner_entry: None,
            reason: None,
            older_than_days: None,
            idle_minutes: None,
        }
    }
}

/// The answer to an ask: the request as queued, before the worker starts on it.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Queued {
    pub id: String,
    pub status: RequestStatus,
    pub text: String,
    pub request: Brief,
}

fn bad(message: impl Into<String>) -> WtError {
    WtError::new(ErrorCode::BadRequest, message)
}

/// The ask must be for exactly one thing, and fit what each thing allows.
fn mode_of(input: &RequestInput, named: &[&String]) -> Result<Mode> {
    let asked = [!named.is_empty(), input.finished, input.all_leftovers];
    if asked.iter().filter(|a| **a).count() != 1 {
        return Err(bad(
            "ask for exactly one of: worktrees (ids or paths), finished, or all leftovers (coord worktree request <id|path>... | --finished | --all-leftovers)",
        ));
    }
    let mode = if input.finished {
        Mode::Finished
    } else if input.all_leftovers {
        Mode::Leftovers
    } else {
        Mode::Targets
    };
    if named.len() > MAX_TARGETS {
        return Err(bad(format!(
            "a request names {MAX_TARGETS} worktrees at most ({} given); use --finished or --all-leftovers",
            named.len()
        )));
    }
    Ok(mode)
}

fn check_flags(input: &RequestInput, mode: Mode) -> Result<()> {
    let owner = input.owner_entry.as_deref().is_some_and(|e| !e.is_empty());
    if mode != Mode::Targets && (input.discard || owner) {
        return Err(bad(
            "--finished and --all-leftovers never discard work: --discard and --owner-entry go with named worktrees",
        ));
    }
    if owner && !input.discard {
        return Err(bad(
            "--owner-entry goes with --discard (the owner's say to discard the work in it)",
        ));
    }
    if mode == Mode::Leftovers && input.branch {
        return Err(bad(
            "leftover folders have no branch: --branch does not apply to --all-leftovers",
        ));
    }
    if mode != Mode::Leftovers && input.older_than_days.is_some() {
        return Err(bad("--older-than goes with --all-leftovers"));
    }
    Ok(())
}

/// The idle time asked for: only with `finished`, from an hour to 30 days.
fn check_idle(input: &RequestInput, mode: Mode) -> Result<Option<f64>> {
    let Some(minutes) = input.idle_minutes else {
        return Ok(None);
    };
    if mode != Mode::Finished {
        return Err(bad("--idle goes with --finished"));
    }
    if !minutes.is_finite() || !(IDLE_MIN_MINUTES..=IDLE_MAX_MINUTES).contains(&minutes) {
        return Err(bad(format!(
            "--idle takes a time from {} to {} (6h, 2d, 90m)",
            idle_text(IDLE_MIN_MINUTES),
            idle_text(IDLE_MAX_MINUTES)
        )));
    }
    Ok(Some(minutes))
}

fn target_of(given: &str, docs: &[WorktreeDoc]) -> Target {
    let given: String = given.trim().chars().take(TARGET_MAX).collect();
    let same_path = |d: &&WorktreeDoc| {
        d.path.replace('\\', "/").to_lowercase() == given.replace('\\', "/").to_lowercase()
    };
    let doc = if is_worktree_id(&given) {
        docs.iter().find(|d| d.id == given)
    } else {
        docs.iter().find(same_path)
    };
    Target {
        label: doc.map_or_else(|| given.clone(), |d| d.path.clone()),
        id: given,
    }
}

/// The named targets, the same one only once (the first place it was named keeps its place).
fn targets_of(named: &[&String], docs: &[WorktreeDoc]) -> Vec<Target> {
    let mut targets: Vec<Target> = Vec::new();
    for given in named {
        let target = target_of(given, docs);
        match targets.iter_mut().find(|t| t.id == target.id) {
            Some(same) => *same = target,
            None => targets.push(target),
        }
    }
    targets
}

impl Service {
    /// The caller rule of `remove`, checked when the request is made. A refusal is an activity line too.
    fn authorize_request(&self, input: &RequestInput) -> Result<()> {
        let board = self.board(&input.project)?;
        let allowed = if input.caller.is_page() {
            Err(forbidden(
                "a removal request comes from a thread of the project; the owner's page removes directly"
                    .to_string(),
            ))
        } else {
            check_caller(&board, &input.caller, &board.valid_rules())
        };
        if let Err(err) = &allowed {
            let who = input.caller.thread().unwrap_or("a stranger");
            self.note_request(
                &input.project,
                "warn",
                &format!(
                    "refused a worktree removal request from {who}: {}",
                    err.message
                ),
                input.caller.thread(),
            );
        }
        allowed
    }

    /// The age asked for a leftover request, checked as the direct removal checks it (the same 400 sentences).
    fn asked_age(&self, input: &RequestInput) -> Result<Option<f64>> {
        let Some(days) = input.older_than_days else {
            return Ok(None);
        };
        let mut asked = RemoveInput::new(&input.project, "", input.caller.clone());
        asked.older_than_days = Some(days);
        asked.reason.clone_from(&input.reason);
        self.age_limit(&asked).map(Some)
    }

    /// Record a request and queue it. Nothing is removed here.
    pub fn request(&self, input: &RequestInput) -> Result<Queued> {
        self.authorize_request(input)?;
        let named: Vec<&String> = input
            .targets
            .iter()
            .filter(|t| !t.trim().is_empty())
            .collect();
        let mode = mode_of(input, &named)?;
        check_flags(input, mode)?;
        let idle_minutes = check_idle(input, mode)?;
        let older_than_days = self.asked_age(input)?;
        let docs = self.list(&input.project);
        let rec = self.add_request(
            input,
            mode,
            (older_than_days, idle_minutes),
            targets_of(&named, &docs),
        )?;
        Ok(self.queued(input, &rec))
    }

    /// The hygiene's request for the released worktrees of one thread, checked for the caller as any other.
    pub(crate) fn request_released(&self, input: &RequestInput) -> Result<Queued> {
        self.authorize_request(input)?;
        let named: Vec<&String> = input.targets.iter().collect();
        let docs = self.list(&input.project);
        let rec = self.add_request(
            input,
            Mode::Released,
            (None, None),
            targets_of(&named, &docs),
        )?;
        Ok(self.queued(input, &rec))
    }

    /// A new request is kept, logged and handed to the worker.
    fn queued(&self, input: &RequestInput, rec: &RequestRec) -> Queued {
        self.save_requests(&input.project);
        self.note_request(
            &input.project,
            "info",
            &format!(
                "worktree request {} queued by {}: {}{}",
                rec.id,
                rec.by,
                what_text(rec),
                rec.reason
                    .as_deref()
                    .map(|r| format!(": {r}"))
                    .unwrap_or_default()
            ),
            Some(&rec.by),
        );
        let answer = Queued {
            id: rec.id.clone(),
            status: rec.status,
            text: queued_text(rec),
            request: rec.brief(),
        };
        if self.opts.run_requests {
            self.kick_requests(&input.project);
        }
        answer
    }

    fn add_request(
        &self,
        input: &RequestInput,
        mode: Mode,
        (older_than_days, idle_minutes): (Option<f64>, Option<f64>),
        targets: Vec<Target>,
    ) -> Result<RequestRec> {
        let at = iso_from_ms(self.now());
        self.with_requests(&input.project, |store| {
            if store.active() >= MAX_ACTIVE {
                return Err(WtError::new(
                    ErrorCode::Conflict,
                    format!(
                        "{} already has {MAX_ACTIVE} removal requests waiting: let Argus finish them first (coord worktree requests)",
                        input.project
                    ),
                ));
            }
            store.seq += 1;
            let rec = RequestRec {
                id: format!("R-{}", store.seq),
                key: input.project.clone(),
                by: input.caller.thread().unwrap_or_default().to_string(),
                at,
                mode,
                branch: input.branch,
                discard: input.discard,
                owner_entry: input
                    .owner_entry
                    .as_deref()
                    .map(|e| e.trim().chars().take(OWNER_ENTRY_MAX).collect::<String>())
                    .filter(|e| !e.is_empty()),
                reason: input
                    .reason
                    .as_deref()
                    .map(|r| one_line(r, REASON_MAX))
                    .filter(|r| !r.is_empty()),
                older_than_days,
                idle_minutes,
                resolved: mode.named(),
                targets: if mode.named() { targets } else { Vec::new() },
                ..RequestRec::default()
            };
            store.list.push(rec.clone());
            Ok(rec)
        })
    }
}
