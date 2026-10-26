//! The log writer of one run: output chunks become redacted lines, lines are batched, and a
//! flush (every 250 ms or 64 KB, driven by the run's supervisor) commits a batch to the
//! store in one call and publishes the lines to the live tail afterwards.

use std::sync::Arc;

use crate::events::RunEvents;
use crate::log_splitter::LineSplitter;
use crate::log_window::{LogWindow, PendingLine, size_of};
use crate::record::{LogEvent, RunLimits, RunRow, Stream};
use crate::redact::Redactor;
use crate::store::Storage;
use crate::time::format_iso;
use argus_core::clock::Clock;

/// Bytes of pending lines that make a flush due at once.
pub const FLUSH_BYTES: u64 = 64 * 1024;

/// What a writer needs to store and publish.
pub struct LogContext {
    pub store: Arc<dyn Storage>,
    pub events: Arc<dyn RunEvents>,
    pub clock: Arc<dyn Clock>,
    pub scope: String,
    pub rid: String,
    pub limits: RunLimits,
    pub redact: Redactor,
}

pub struct LogWriter {
    ctx: LogContext,
    splitter: LineSplitter,
    window: LogWindow,
    pending: Vec<PendingLine>,
    pending_bytes: u64,
    ended: bool,
}

impl LogWriter {
    /// `start` carries the counters of the run row.
    pub fn new(ctx: LogContext, start: &RunRow) -> Self {
        let window = LogWindow::new(start, &ctx.limits);
        Self {
            ctx,
            splitter: LineSplitter::new(),
            window,
            pending: Vec::new(),
            pending_bytes: 0,
            ended: false,
        }
    }

    fn queue(&mut self, stream: Stream, lines: Vec<String>) {
        for line in lines {
            let text = self.ctx.redact.redact(&line);
            self.pending_bytes += size_of(&text);
            self.pending.push(PendingLine {
                stream,
                at: format_iso(self.ctx.clock.now_ms()),
                text,
            });
        }
    }

    /// True when enough is pending that the caller should flush now.
    pub fn flush_due(&self) -> bool {
        self.pending_bytes >= FLUSH_BYTES
    }

    /// A decoded chunk of the child's stdout or stderr.
    pub fn push(&mut self, stream: Stream, text: &str) {
        if self.ended {
            return;
        }
        let lines = self.splitter.push(stream, text, self.ctx.clock.now_ms());
        self.queue(stream, lines);
    }

    /// A line from the dashboard itself (start, stop, exit, timeout).
    pub fn sys(&mut self, text: &str) {
        if !self.ended {
            self.queue(Stream::Sys, vec![text.to_owned()]);
        }
    }

    /// True while something waits for a flush or a partial line waits for its pause.
    pub fn has_work(&self) -> bool {
        !self.pending.is_empty() || self.splitter.has_partial()
    }

    /// Commits the pending batch. Counters change only after the store accepted it, so a
    /// failed commit loses the batch (reported to the events) and leaves the writer
    /// consistent. Returns true when lines were committed.
    pub fn flush(&mut self) -> bool {
        self.release(false);
        if self.pending.is_empty() {
            return false;
        }
        let batch = std::mem::take(&mut self.pending);
        self.pending_bytes = 0;
        let plan = self.window.plan(&self.ctx.limits, &batch);
        let (scope, rid) = (&self.ctx.scope, &self.ctx.rid);
        if let Err(message) = self.ctx.store.commit(scope, rid, &plan.commit) {
            self.ctx
                .events
                .warn("run log write failed", &format!("{scope}:{rid}: {message}"));
            return false;
        }
        self.window.accept(&plan);
        for stored in &plan.commit.lines {
            self.ctx.events.log_line(&LogEvent {
                scope: scope.clone(),
                rid: rid.clone(),
                seq: stored.line.seq,
                stream: stored.line.stream,
                at: stored.line.at.clone(),
                text: stored.line.text.clone(),
            });
        }
        true
    }

    fn release(&mut self, force: bool) {
        for (stream, lines) in self.splitter.release(self.ctx.clock.now_ms(), force) {
            self.queue(stream, lines);
        }
    }

    /// Partial lines go out now, ahead of the dashboard's own closing notes.
    pub fn release_partials(&mut self) {
        self.release(true);
    }

    /// The run is over: partial lines go out and everything is committed. Later pushes
    /// are ignored.
    pub fn end(&mut self) {
        self.release(true);
        self.flush();
        self.ended = true;
    }
}
