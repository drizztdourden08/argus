//! The live tail of a run (`GET /api/p/:key/runs/:rid/logs/stream`). The stream listens before
//! it replays, and one cursor under one lock orders the replay and the live lines, so no line
//! is missed or sent twice. It ends after a final `status` event.

use std::any::Any;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use argus_core::Coll;
use argus_launchers::{LogEvent, LogOptions, RunService, RunStatus};
use serde_json::json;

use super::logs::number;
use super::{ids, service, unknown_run};
use crate::adapters::runs::RunHost;
use crate::router::Reply;
use crate::sse::{SseSender, open_stream};
use crate::{Ctx, Router};

const REPLAY_PAGE: usize = 2000;

type Watching = Mutex<Vec<Box<dyn Any + Send>>>;

struct Tail {
    sender: SseSender,
    cursor: Mutex<u64>,
    runs: Arc<RunService<RunHost>>,
    key: String,
    rid: String,
    watching: Arc<Watching>,
}

fn locked<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Tail {
    fn line(&self, event: &LogEvent) {
        let mut cursor = locked(&self.cursor);
        if event.seq <= *cursor {
            return;
        }
        *cursor = event.seq;
        let data =
            json!({ "seq": event.seq, "stream": event.stream, "at": event.at, "text": event.text });
        let _ = self
            .sender
            .event("log", &data, i64::try_from(event.seq).ok());
    }

    fn status(&self) {
        let Some(run) = self.runs.get(&self.key, &self.rid) else {
            self.finish();
            return;
        };
        let _ = self.sender.event("status", &json!({ "run": run }), None);
        if run.status != RunStatus::Queued && run.status != RunStatus::Running {
            self.finish();
        }
    }

    fn finish(&self) {
        self.sender.close();
        locked(&self.watching).clear();
    }

    fn replay(&self) {
        loop {
            let after = i64::try_from(*locked(&self.cursor)).ok();
            let options = LogOptions {
                after,
                limit: i64::try_from(REPLAY_PAGE).ok(),
                ..LogOptions::default()
            };
            let Ok(page) = self.runs.logs(&self.key, &self.rid, &options) else {
                return;
            };
            for l in &page.lines {
                self.line(&LogEvent {
                    scope: self.key.clone(),
                    rid: self.rid.clone(),
                    seq: l.seq,
                    stream: l.stream,
                    at: l.at.clone(),
                    text: l.text.clone(),
                });
            }
            if page.lines.len() < REPLAY_PAGE {
                return;
            }
        }
    }
}

fn start_cursor(ctx: &Ctx) -> u64 {
    let last = ctx
        .header("last-event-id")
        .and_then(|v| v.trim().parse::<i64>().ok());
    let seen = number(ctx, "after").or(last).unwrap_or(0);
    u64::try_from(seen).unwrap_or(0)
}

fn watch(ctx: &Ctx, tail: &Arc<Tail>) -> Vec<Box<dyn Any + Send>> {
    let lines = Arc::clone(tail);
    let docs = Arc::clone(tail);
    let line_watch = ctx.svc.run_logs.listen(move |e| {
        if e.scope == lines.key && e.rid == lines.rid {
            lines.line(e);
        }
    });
    let doc_watch = ctx.svc.bus.on_docs(move |e| {
        let mine = e.scope == docs.key
            && e.changes
                .iter()
                .any(|c| c.change.coll == Coll::Runs && c.change.id == docs.rid);
        if mine {
            docs.status();
        }
    });
    vec![Box::new(line_watch), Box::new(doc_watch)]
}

async fn stream(ctx: Ctx) -> Reply {
    let runs = service(&ctx)?;
    let (key, rid) = ids(&ctx)?;
    runs.get(&key, &rid)
        .ok_or_else(|| unknown_run(&key, &rid))?;
    let watching: Arc<Watching> = Arc::default();
    let held = Arc::clone(&watching);
    let on_drop = Box::new(move || locked(&held).clear());
    let (sender, res) = open_stream(ctx.svc.heartbeat, &ctx.svc.streams, Some(on_drop));
    let tail = Arc::new(Tail {
        sender,
        cursor: Mutex::new(start_cursor(&ctx)),
        runs,
        key,
        rid,
        watching: Arc::clone(&watching),
    });
    *locked(&watching) = watch(&ctx, &tail);
    tail.replay();
    tail.status();
    Ok(res)
}

pub(super) fn register(r: &mut Router) {
    r.get("/api/p/:key/runs/:rid/logs/stream", stream);
}
