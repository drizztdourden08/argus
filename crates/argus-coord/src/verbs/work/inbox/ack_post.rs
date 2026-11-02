//! The tell of an own inbox read: `POST /api/p/:key/inbox/read`, so Argus stamps the read and
//! acknowledges the dependency requests the thread has now seen. Best effort, as `send` is:
//! every failure is ignored and the read is never held up.

use argus_board::json::text;
use serde_json::{Value, json};

use crate::client::dashboard_client;
use crate::verbs::work::run::Run;
use crate::who::Caller;

/// What a thread learned from reading its own inbox.
pub(super) struct Read {
    thread: String,
    actor: String,
    deps: Vec<String>,
}

impl Read {
    /// The read of `who` of the messages it was shown, before an `--ack` cleared them.
    pub(super) fn of(who: &Caller, messages: &[Value]) -> Read {
        let deps = messages
            .iter()
            .filter(|m| text(m, "kind") == Some("dep"))
            .filter_map(|m| text(m, "ref"))
            .map(str::to_owned)
            .collect();
        Read {
            thread: who.thread.clone(),
            actor: who.actor.clone(),
            deps,
        }
    }

    fn body(&self) -> Value {
        json!({"thread": self.thread, "deps": self.deps, "actor": self.actor})
    }
}

/// Tell the dashboard about the read. Only a nudge: every failure is ignored.
pub(super) fn post(r: &Run<'_>, read: &Read) {
    let paths = r.inv.env.paths(&r.inv.cwd);
    let Ok(client) = dashboard_client(&r.inv.env, &paths, false) else {
        return;
    };
    client
        .post(&format!("/api/p/{}/inbox/read", r.project), &read.body())
        .ok();
}
