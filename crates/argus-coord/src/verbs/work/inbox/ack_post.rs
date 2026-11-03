//! The tell of an own inbox read: `POST /api/p/:key/inbox/read`, so Argus stamps the read and
//! acknowledges the dependency requests the thread has now seen. Best effort, as `send` is:
//! every failure is ignored and the read is never held up.

use std::time::Duration;

use argus_board::json::text;
use argus_core::Paths;
use serde_json::{Value, json};

use crate::client::{Client, dashboard_client, dashboard_config};
use crate::verbs::work::run::Run;
use crate::who::Caller;

/// What a thread learned from reading its own inbox.
pub(crate) struct Read {
    thread: String,
    actor: String,
    deps: Vec<String>,
}

impl Read {
    /// The read of `who` of the messages it was shown, before an `--ack` cleared them.
    pub(super) fn of(who: &Caller, messages: &[Value]) -> Read {
        Read::new(&who.thread, &who.actor, dep_refs(messages))
    }

    /// The read of `thread`, by `actor`, of the dependency requests `deps`.
    pub(crate) fn new(thread: &str, actor: &str, deps: Vec<String>) -> Read {
        Read {
            thread: thread.to_owned(),
            actor: actor.to_owned(),
            deps,
        }
    }

    fn body(&self) -> Value {
        json!({"thread": self.thread, "deps": self.deps, "actor": self.actor})
    }
}

/// The `ref` of every message of kind `dep`: the dependency requests the thread was shown.
pub(crate) fn dep_refs(messages: &[Value]) -> Vec<String> {
    messages
        .iter()
        .filter(|m| text(m, "kind") == Some("dep"))
        .filter_map(|m| text(m, "ref"))
        .map(str::to_owned)
        .collect()
}

fn path_of(project: &str) -> String {
    format!("/api/p/{project}/inbox/read")
}

/// Tell the dashboard about the read, within `limit`, without a health check first: the server
/// is either up or the nudge is dropped. Only a nudge: every failure is ignored.
pub(crate) fn tell_within(paths: &Paths, project: &str, read: &Read, limit: Duration) {
    let Some(config) = dashboard_config(paths) else {
        return;
    };
    Client::new(&config)
        .request_within(limit, "POST", &path_of(project), Some(&read.body()))
        .ok();
}

/// Tell the dashboard about the read of `coord inbox`.
pub(super) fn post(r: &Run<'_>, read: &Read) {
    let paths = r.inv.env.paths(&r.inv.cwd);
    let Ok(client) = dashboard_client(&r.inv.env, &paths, false) else {
        return;
    };
    client.post(&path_of(&r.project), &read.body()).ok();
}
