//! The approval gate of the hosted Overlord: the queue, the bearer of the running child and the
//! temp mcp-config that carries it. A held `permission_prompt` call waits here until the owner
//! answers, the wait runs out, or the child goes away.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use argus_core::clock::{iso_from_ms, now_ms};
use argus_overlord::approvals::{
    Approval, DecideFault, Decision, Prompt, Queue, Status, bearer_ok, mcp_config_json,
    reply_for, summarize, timed_out,
};
use argus_overlord::ports::{ApprovalStore, HostFault, OwnerToast};
use serde_json::Value;
use tokio::sync::watch;

const BEARER_BYTES: usize = 32;

/// One question for the owner, with where its toast leads.
pub struct Ask {
    pub prompt: Prompt,
    /// How long to wait for an answer, in milliseconds.
    pub wait_ms: i64,
    /// `http://127.0.0.1:<port>`, the base of the toast link.
    pub link_base: String,
}

#[derive(Default)]
struct Credentials {
    bearer: Option<String>,
    file: Option<PathBuf>,
}

/// The queue and the per-start credentials, shared by the endpoint, the routes and the supervisor.
pub struct ApprovalGate {
    queue: Mutex<Queue>,
    credentials: Mutex<Credentials>,
    changed: watch::Sender<u64>,
    store: Option<Arc<dyn ApprovalStore>>,
}

/// Expires the approval when its call is dropped before an answer (the child went away).
struct Abandon<'a> {
    gate: &'a ApprovalGate,
    id: String,
}

impl Drop for Abandon<'_> {
    fn drop(&mut self) {
        self.gate.expire(&self.id, "closed");
    }
}

fn random_bearer() -> String {
    let mut bytes = [0u8; BEARER_BYTES];
    getrandom::fill(&mut bytes).unwrap_or_default();
    argus_core::config::hex(&bytes)
}

impl ApprovalGate {
    /// A gate that keeps its rows in `store` when there is one.
    #[must_use]
    pub fn new(store: Option<Arc<dyn ApprovalStore>>) -> Self {
        Self {
            queue: Mutex::new(Queue::default()),
            credentials: Mutex::new(Credentials::default()),
            changed: watch::channel(0).0,
            store,
        }
    }

    fn queue(&self) -> std::sync::MutexGuard<'_, Queue> {
        self.queue.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn creds(&self) -> std::sync::MutexGuard<'_, Credentials> {
        self.credentials
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn keep(&self, approval: &Approval) {
        if let Some(store) = &self.store {
            let _ = store.save(approval);
        }
        self.changed.send_modify(|n| *n += 1);
    }

    /// A new bearer for a child about to start, and the temp mcp-config file that carries it in
    /// `dir`. Rows left pending by an earlier run are abandoned. The path goes to `--mcp-config`.
    ///
    /// # Errors
    /// A fault when the file cannot be written; the bearer is dropped then.
    pub fn begin(&self, endpoint: &str, dir: &Path) -> Result<PathBuf, HostFault> {
        self.end();
        if let Some(store) = &self.store {
            let _ = store.abandon_pending(&iso_from_ms(now_ms()));
        }
        let bearer = random_bearer();
        let file = dir.join(format!("argus-overlord-mcp-{}.json", &bearer[..12]));
        std::fs::write(&file, mcp_config_json(endpoint, &bearer))
            .map_err(|_| HostFault("the mcp config cannot be written".to_owned()))?;
        let mut creds = self.creds();
        creds.bearer = Some(bearer);
        creds.file = Some(file.clone());
        Ok(file)
    }

    /// The child is gone: the bearer stops working, the file is deleted, every pending approval
    /// expires so no held call waits for nobody.
    pub fn end(&self) {
        let (file, had) = {
            let mut creds = self.creds();
            (creds.file.take(), creds.bearer.take().is_some())
        };
        if let Some(file) = file {
            let _ = std::fs::remove_file(file);
        }
        if had {
            let pending = self.queue().pending();
            for id in pending {
                self.expire(&id, "stop");
            }
        }
    }

    /// True when `header` is the bearer of the running child.
    #[must_use]
    pub fn bearer_ok(&self, header: Option<&str>) -> bool {
        bearer_ok(header, self.creds().bearer.as_deref())
    }

    fn expire(&self, id: &str, by: &str) {
        let done = self.queue().expire(id, by, &iso_from_ms(now_ms()));
        if let Ok(done) = done {
            self.keep(&done);
        }
    }

    /// The owner's answer to one approval.
    ///
    /// # Errors
    /// `Unknown` or `Settled` as the queue says.
    pub fn decide(
        &self,
        id: &str,
        decision: Decision,
        message: Option<&str>,
    ) -> Result<Approval, DecideFault> {
        let done = self
            .queue()
            .decide(id, decision, message, &iso_from_ms(now_ms()))?;
        self.keep(&done);
        Ok(done)
    }

    /// Up to `limit` approvals, newest first.
    #[must_use]
    pub fn recent(&self, limit: usize) -> Vec<Approval> {
        self.queue().recent(limit)
    }

    fn submit(&self, prompt: &Prompt) -> Approval {
        let now = now_ms();
        let summary = summarize(&prompt.input);
        let item = self
            .queue()
            .submit(&prompt.tool_name, &summary, &iso_from_ms(now), now);
        self.keep(&item);
        item
    }

    /// Records the approval, toasts the owner and holds until the answer. The reply is the
    /// documented `allow` or `deny` JSON.
    pub async fn ask(&self, ask: &Ask, toast: Arc<dyn OwnerToast>) -> Value {
        let mut changes = self.changed.subscribe();
        let item = self.submit(&ask.prompt);
        let guard = Abandon {
            gate: self,
            id: item.id.clone(),
        };
        let text = format!("{}: {}", item.tool, item.input_summary);
        let url = format!("{}/?tab=overlord&approval={}", ask.link_base, item.id);
        let _ = tokio::task::spawn_blocking(move || toast.urgent(&text, &url)).await;
        let reply = self.hold(&item, ask.wait_ms, &mut changes).await;
        drop(guard);
        reply_for(&reply, &ask.prompt.input)
    }

    async fn hold(
        &self,
        item: &Approval,
        wait_ms: i64,
        changes: &mut watch::Receiver<u64>,
    ) -> Approval {
        loop {
            let now = now_ms();
            let found = self.queue().get(&item.id);
            match found {
                Some(a) if a.status != Status::Pending => return a,
                Some(_) => {}
                None => return Approval { status: Status::Expired, ..item.clone() },
            }
            if timed_out(item.requested_ms, now, wait_ms) {
                self.expire(&item.id, "timeout");
                continue;
            }
            let left = (item.requested_ms + wait_ms - now).max(1);
            let nap = Duration::from_millis(u64::try_from(left).unwrap_or(1));
            tokio::select! {
                _ = changes.changed() => {}
                () = tokio::time::sleep(nap) => {}
            }
        }
    }
}
