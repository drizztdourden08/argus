//! The environment of the launcher wakes: the boards, the Play runs, the wake engine and the
//! activity log.

use std::sync::{Arc, Mutex};

use argus_launchers::launcher::{LauncherBoard, StaleEnv, StaleNote, WakeEvent};
use argus_launchers::{ListFilter, RunService, parse_iso};
use argus_store::Db;
use argus_wake::service::WakeService;
use argus_wake::service::driver::lock;

use super::activity::ActivityAdapter;
use super::runs::RunHost;
use crate::boards::BoardReader;
use crate::{WakeHook, WakeRequest};

/// What the launcher wakes read, and the one thing they do.
pub struct StaleHost {
    boards: BoardReader,
    runs: Option<Arc<RunService<RunHost>>>,
    db: Arc<Db>,
    activity: ActivityAdapter,
    hook: WakeHook,
    wake: Option<Arc<Mutex<WakeService>>>,
    shadow: bool,
}

impl StaleHost {
    /// The wakes' environment. In shadow mode the newest Play of a launcher is not remembered
    /// in the database.
    #[must_use]
    pub fn new(
        boards: BoardReader,
        runs: Option<Arc<RunService<RunHost>>>,
        db: Arc<Db>,
        hook: WakeHook,
        wake: Option<Arc<Mutex<WakeService>>>,
        activity: ActivityAdapter,
        shadow: bool,
    ) -> Self {
        StaleHost {
            boards,
            runs,
            db,
            activity,
            hook,
            wake,
            shadow,
        }
    }

    fn live_play_ms(&self, scope: &str, name: &str) -> i64 {
        let filter = ListFilter {
            name: Some(format!("launcher-{name}")),
            limit: Some(1),
            ..ListFilter::default()
        };
        self.runs
            .as_ref()
            .and_then(|runs| runs.list(scope, &filter).into_iter().next())
            .and_then(|run| parse_iso(&run.queued_at))
            .unwrap_or(0)
    }
}

impl StaleEnv for StaleHost {
    fn scopes(&self) -> Vec<String> {
        self.boards
            .projects()
            .into_iter()
            .filter(|p| p.has_board)
            .map(|p| p.key)
            .collect()
    }

    fn board(&self, scope: &str) -> Option<LauncherBoard> {
        let mut board = self.boards.board(scope)?;
        board.as_object_mut()?.remove("runDefs");
        serde_json::from_value(board).ok()
    }

    fn lead(&self, scope: &str) -> Option<String> {
        self.boards.lead(scope)
    }

    /// The newest Play run of the launcher (the runs table keeps a few weeks of them), kept in
    /// the database too so a use older than that is not forgotten.
    fn played_ms(&self, scope: &str, name: &str) -> i64 {
        let live = self.live_play_ms(scope, name);
        let key = format!("launcher-played:{scope}:{name}");
        let seen = self
            .db
            .kv_get(&key)
            .ok()
            .flatten()
            .and_then(|v| v.trim().parse::<i64>().ok())
            .unwrap_or(0);
        if live > seen && !self.shadow {
            let _ = self.db.kv_set(&key, &live.to_string());
        }
        live.max(seen)
    }

    fn last_sent_ms(&self, scope: &str, thread: &str, reason: &str, subject: &str) -> i64 {
        let Some(wake) = &self.wake else {
            return 0;
        };
        let service = lock(wake);
        service
            .engine()
            .store()
            .last_at(scope, thread, reason, Some(subject))
            .unwrap_or(0)
    }

    fn submit(&self, events: Vec<WakeEvent>) -> Result<(), String> {
        let mut wakes = Vec::with_capacity(events.len());
        for event in events {
            let WakeEvent {
                scope,
                thread,
                reason,
                subject,
                text,
            } = event;
            wakes.push(WakeRequest::about(&scope, &thread, reason, &subject, text));
        }
        self.hook.submit(&wakes)
    }

    fn note(&self, note: StaleNote) {
        self.activity.stale_note(note);
    }
}
