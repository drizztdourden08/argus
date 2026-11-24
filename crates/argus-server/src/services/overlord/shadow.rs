//! Shadow mode of the hosted Overlord (I-172): the child forks the desktop Overlord's session,
//! gets a copy of every wake the desktop Overlord gets, may only read, and its words go to the
//! "would do" log beside the transcripts. The pure rules are in `argus_overlord::shadow`.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use argus_accounts::find_app_data;
use argus_accounts::session_map::{local_ids, seat_of};
use argus_core::clock::now_iso;
use argus_core::log::Logger;
use argus_core::redact::Redactor;
use argus_overlord::approvals::{Prompt, allow, deny, summarize};
use argus_overlord::ports::HostFault;
use argus_overlord::session::args::SessionArgs;
use argus_overlord::session::guard::GuardFault;
use argus_overlord::session::supervisor::{Link, Observer};
use argus_overlord::shadow::{
    DENIED, Seen, THREAD, Turn, Verdict, denial_entry, intro, mirrors, seen, shadow_argv, verdict,
};
use argus_wake::engine::AttachOptions;
use argus_wake::hosted::HostedSink;
use argus_wake::service::driver::SharedWake;
use serde_json::{Value, json};

use super::attachment::Attachment;
use crate::Services;
use crate::boards::{BoardReader, lead_in};

const ALL_THREADS: &str = "*";
const LABEL: &str = "shadow overlord";
const FILE: &str = "shadow-would-do.jsonl";
/// The most entries `GET /api/overlord/shadow` answers.
pub(crate) const RECENT: usize = 200;

/// The file of the "would do" log, beside the transcripts.
#[derive(Clone)]
pub(crate) struct ShadowLog {
    path: PathBuf,
}

impl ShadowLog {
    pub(crate) fn new(home: &Path) -> Self {
        Self {
            path: home.join("overlord").join(FILE),
        }
    }

    /// Appends one entry; a log that cannot be written is not worth stopping the shadow for.
    pub(crate) fn append(&self, entry: &Value) {
        if let Some(dir) = self.path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let text = Redactor::default().redact(&entry.to_string());
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path);
        if let Ok(mut file) = file {
            let _ = writeln!(file, "{text}");
        }
    }

    /// The last `limit` entries, oldest first.
    pub(crate) fn recent(&self, limit: usize) -> Vec<Value> {
        let text = fs::read_to_string(&self.path).unwrap_or_default();
        let mut entries: Vec<Value> = text
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();
        let skip = entries.len().saturating_sub(limit);
        entries.drain(..skip);
        entries
    }
}

/// What the shadow's spec needs to find the desktop session.
#[derive(Clone)]
pub(crate) struct ShadowInput {
    pub(crate) boards: BoardReader,
}

/// The CLI session id the desktop Overlord runs now: the board thread's session id, looked up in
/// the desktop app's records (its current id after a compaction), else the thread's own.
pub(super) fn desktop_session(boards: &BoardReader) -> Option<String> {
    let board = boards.overlord_board()?;
    let thread = lead_in(&board)?;
    let session = board["threads"][thread.as_str()]["sessionId"]
        .as_str()
        .filter(|id| !id.is_empty())?
        .to_owned();
    let current = find_app_data(None).and_then(|app| {
        let local = local_ids(&app, std::slice::from_ref(&session)).remove(&session)?;
        seat_of(&app, &local)?.cli_session_id
    });
    Some(current.unwrap_or(session))
}

/// The argument line of the shadow child.
pub(crate) fn argv(input: &ShadowInput, base: &SessionArgs) -> Result<Vec<String>, HostFault> {
    let desktop = desktop_session(&input.boards)
        .ok_or_else(|| HostFault("the desktop Overlord has no session to fork".to_owned()))?;
    Ok(shadow_argv(base, &desktop))
}

/// Answers a permission ask of the shadow: read-only calls pass, every other is denied at once,
/// with no row and no toast for the owner, and the denial goes to the log.
pub(crate) fn judge(svc: &Services, prompt: &Prompt) -> Value {
    if verdict(&prompt.tool_name, &prompt.input) == Verdict::Allow {
        return allow(&prompt.input);
    }
    let entry = denial_entry(&now_iso(), &prompt.tool_name, &summarize(&prompt.input));
    ShadowLog::new(&svc.paths.home).append(&entry);
    deny(DENIED)
}

/// Mirrors the desktop Overlord's wakes to the child and logs what the child says.
pub(super) struct Shadow {
    log: Arc<Logger>,
    file: ShadowLog,
    turn: Arc<Mutex<Turn>>,
    hold: Attachment,
}

impl Shadow {
    pub(super) fn new(svc: &Services, wake: SharedWake) -> Self {
        Self {
            log: Arc::clone(&svc.log),
            file: ShadowLog::new(&svc.paths.home),
            turn: Arc::default(),
            hold: Attachment::new(
                wake,
                Arc::clone(&svc.overlord_approvals),
                svc.paths.coord_home.clone(),
            ),
        }
    }

    fn turn(&self) -> MutexGuard<'_, Turn> {
        self.turn.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Observer for Shadow {
    fn started(&self, link: &Link) {
        let _ = link.send(intro());
        let (to_child, turn) = (link.clone(), Arc::clone(&self.turn));
        let push = move |text: String| {
            let taken = to_child.send(text.clone());
            if taken {
                turn.lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .wake(text);
            }
            taken
        };
        let sink = HostedSink::mirror(Box::new(push), mirrors);
        let options = AttachOptions {
            v: 1,
            label: Some(LABEL.to_owned()),
        };
        self.hold.attach(ALL_THREADS, sink, options);
    }

    fn stopped(&self) {
        self.hold.end();
    }

    fn guard_failed(&self, fault: &GuardFault) {
        self.log.warn(
            "shadow overlord ended",
            json!({ "thread": THREAD, "fault": fault.to_string() }),
        );
    }

    fn line(&self, line: &str) {
        match seen(line) {
            Seen::Said(text) => self.turn().said(text),
            Seen::TurnEnd => {
                let entry = self.turn().finish(&now_iso());
                if let Some(entry) = entry {
                    self.file.append(&entry);
                }
            }
            Seen::Nothing => {}
        }
    }
}
