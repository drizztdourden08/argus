//! What the monitor writes to the store, and when: only what a reader would see change, plus a
//! slow heartbeat that keeps the "sampled at" time fresh.
//!
//! The sampler runs every few seconds. A write per tick would churn the database and push a
//! stream frame to every open page, so a sample whose steady fields match the last write is
//! skipped. The fields that move on every tick by nature (the CPU share, the age and idle
//! minutes, the sample time and how long it took) do not count as a change on their own.

use crate::service::{Inner, locked};
use crate::types::{Meta, ProcDoc};

/// The longest the store goes without a write while samples keep coming, in milliseconds.
pub const HEARTBEAT_MS: i64 = 60_000;

/// The docs and meta as a reader sees them, minus the fields that change on every tick.
#[derive(Debug, PartialEq)]
struct Print {
    docs: Vec<ProcDoc>,
    meta: Option<Meta>,
}

fn steady_doc(mut doc: ProcDoc) -> ProcDoc {
    doc.cpu_pct = None;
    doc.age_min = 0;
    doc.idle_min = 0;
    doc.mem_mb = 0;
    doc
}

fn steady_meta(mut meta: Meta) -> Meta {
    meta.sampled_at = None;
    meta.took_ms = None;
    meta.total_mem_mb = 0;
    for rule in &mut meta.by_rule {
        rule.mem_mb = 0;
    }
    meta
}

impl Print {
    fn of(docs: &[ProcDoc], meta: Option<&Meta>) -> Self {
        let mut docs: Vec<ProcDoc> = docs.iter().cloned().map(steady_doc).collect();
        docs.sort_by(|a, b| a.id.cmp(&b.id));
        Self {
            docs,
            meta: meta.cloned().map(steady_meta),
        }
    }
}

/// What the last successful write held and when it was made (the sample time, in milliseconds).
#[derive(Debug, Default)]
pub(crate) struct Published {
    print: Option<Print>,
    at: Option<i64>,
}

impl Published {
    fn due(&self, print: &Print, now: Option<i64>) -> bool {
        self.print.as_ref() != Some(print) || self.heartbeat_due(now)
    }

    fn heartbeat_due(&self, now: Option<i64>) -> bool {
        match (now, self.at) {
            (Some(now), Some(at)) => now.saturating_sub(at) >= HEARTBEAT_MS,
            (Some(_), None) => true,
            (None, _) => false,
        }
    }

    fn record(&mut self, print: Print, now: Option<i64>) {
        self.print = Some(print);
        self.at = now.or(self.at);
    }
}

impl Inner {
    /// Writes the listing and its meta when a reader would see a change or the heartbeat is due.
    /// `now` is the sample time; none (a failed sample) never counts as a heartbeat.
    pub(crate) fn publish_if_changed(&self, now: Option<i64>) {
        self.write(now, false);
    }

    /// Writes the listing and its meta again whatever the last write held, after a hard resync
    /// wiped the scope.
    pub(crate) fn republish(&self) {
        let now = locked(&self.view).raw.as_ref().map(|s| s.at);
        self.write(now, true);
    }

    fn write(&self, now: Option<i64>, forced: bool) {
        let (docs, meta) = {
            let view = locked(&self.view);
            (view.listing.clone(), view.meta.clone())
        };
        let print = Print::of(&docs, meta.as_ref());
        if !forced && !locked(&self.published).due(&print, now) {
            return;
        }
        match self.ports.store.replace(&docs, meta.as_ref()) {
            Ok(()) => locked(&self.published).record(print, now),
            Err(error) => self.ports.host.warn("process docs not written", &error),
        }
    }
}
