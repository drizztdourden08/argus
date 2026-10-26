//! The supervisor of one run: a task that reads the output pipes, flushes the log every
//! 250 ms, enforces the timeout, notices the exit and hands the ending to the finaliser.
//! After the shell exits it waits a short while for its pipes to drain, then finishes anyway
//! (a grandchild that keeps the pipes open must not keep the run "running").

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::oneshot;
use tokio::time::{Instant, Interval, MissedTickBehavior, interval_at, sleep_until};

use crate::host::{BoxRead, ExitInfo, ProcessHost};
use crate::live_run::LiveRun;
use crate::record::Stream;
use crate::run_finalize::Ending;
use crate::run_launch::ProcessIo;
use crate::run_service::RunService;
use crate::utf8::Utf8Decoder;
use tokio::io::AsyncReadExt;

/// How often the log is flushed and run docs follow progress.
pub(crate) const FLUSH_EVERY: Duration = Duration::from_millis(250);
/// After the shell exits, how long its output pipes get to drain.
const PIPE_DRAIN: Duration = Duration::from_millis(750);
const READ_BYTES: usize = 64 * 1024;
const TOLERANCE: Duration = Duration::from_millis(20);

struct Pipe {
    reader: BoxRead,
    decoder: Utf8Decoder,
    buf: Vec<u8>,
    open: bool,
}

impl Pipe {
    fn new(reader: BoxRead) -> Self {
        Self {
            reader,
            decoder: Utf8Decoder::default(),
            buf: vec![0; READ_BYTES],
            open: true,
        }
    }

    /// The next decoded chunk; at the end of the pipe (or a read error) it closes.
    async fn next(&mut self) -> String {
        match self.reader.read(&mut self.buf).await {
            Ok(n) if n > 0 => self.decoder.push(&self.buf[..n]),
            _ => {
                self.open = false;
                self.decoder.finish()
            }
        }
    }
}

enum Event {
    Output(Stream, String),
    Exited(ExitInfo),
    Tick,
    Timeout,
    DrainOver,
    Force,
    Release,
}

struct Watch {
    out: Pipe,
    err: Pipe,
    exit: Option<oneshot::Receiver<ExitInfo>>,
    exited: Option<ExitInfo>,
    drain_at: Option<Instant>,
    timeout_at: Option<Instant>,
    tick: Interval,
}

async fn pipe_chunk(pipe: &mut Pipe, stream: Stream) -> Event {
    if !pipe.open {
        return std::future::pending().await;
    }
    Event::Output(stream, pipe.next().await)
}

async fn exit_event(exit: &mut Option<oneshot::Receiver<ExitInfo>>) -> Event {
    match exit {
        Some(report) => Event::Exited(report.await.unwrap_or_default()),
        None => std::future::pending().await,
    }
}

async fn deadline(at: Option<Instant>, event: Event) -> Event {
    match at {
        Some(at) => {
            sleep_until(at).await;
            event
        }
        None => std::future::pending().await,
    }
}

impl Watch {
    fn new(io: ProcessIo, live: &LiveRun) -> Self {
        let mut tick = interval_at(Instant::now() + FLUSH_EVERY, FLUSH_EVERY);
        tick.set_missed_tick_behavior(MissedTickBehavior::Delay);
        Self {
            out: Pipe::new(io.stdout),
            err: Pipe::new(io.stderr),
            exit: Some(io.exit),
            exited: None,
            drain_at: None,
            timeout_at: (live.def.timeout_sec > 0)
                .then(|| Instant::now() + Duration::from_secs(u64::from(live.def.timeout_sec))),
            tick,
        }
    }

    fn pipes_closed(&self) -> bool {
        !self.out.open && !self.err.open
    }

    async fn next_event(&mut self, live: &LiveRun) -> Event {
        let Self {
            out,
            err,
            exit,
            drain_at,
            timeout_at,
            tick,
            ..
        } = self;
        tokio::select! {
            event = stream_event(out, err, exit) => event,
            event = timer_event((*drain_at, *timeout_at), tick, live) => event,
        }
    }
}

async fn stream_event(
    out: &mut Pipe,
    err: &mut Pipe,
    exit: &mut Option<oneshot::Receiver<ExitInfo>>,
) -> Event {
    tokio::select! {
        event = pipe_chunk(out, Stream::Out) => event,
        event = pipe_chunk(err, Stream::Err) => event,
        event = exit_event(exit) => event,
    }
}

async fn timer_event(
    (drain_at, timeout_at): (Option<Instant>, Option<Instant>),
    tick: &mut Interval,
    live: &LiveRun,
) -> Event {
    tokio::select! {
        event = deadline(drain_at, Event::DrainOver) => event,
        event = deadline(timeout_at, Event::Timeout) => event,
        _ = tick.tick() => Event::Tick,
        () = live.force.notified() => Event::Force,
        () = live.release.notified() => Event::Release,
    }
}

impl<H: ProcessHost> RunService<H> {
    /// Projects the run doc when progress was committed and the last doc is old enough.
    fn after_flush(&self, live: &LiveRun, committed: bool) {
        let due = {
            let mut control = live.control();
            control.dirty |= committed;
            let old_enough = control.last_project.elapsed() + TOLERANCE >= FLUSH_EVERY;
            let due = control.dirty && old_enough;
            if due {
                control.dirty = false;
                control.last_project = Instant::now();
            }
            due
        };
        if due && let Some(row) = self.inner.deps.store.get(&live.scope, &live.rid) {
            self.inner.deps.events.project(&row);
        }
    }

    fn flush_and_project(&self, live: &LiveRun) {
        let committed = live.writer().flush();
        self.after_flush(live, committed);
    }

    fn on_output(&self, live: &LiveRun, stream: Stream, text: &str) {
        let due = {
            let mut writer = live.writer();
            writer.push(stream, text);
            writer.flush_due()
        };
        if due {
            self.flush_and_project(live);
        }
    }

    fn on_timeout(&self, live: &LiveRun) {
        live.control().timed_out = true;
        live.sys(&format!(
            "timeout of {}s reached, stopping",
            live.def.timeout_sec
        ));
        if let Some(pid) = live.pid {
            let service = self.clone();
            let grace = Duration::from_millis(self.limits().stop_grace_ms);
            tokio::spawn(async move { service.inner.deps.host.kill_tree(pid, grace).await });
        }
    }

    async fn watch(&self, live: &LiveRun, watch: &mut Watch) -> Ending {
        loop {
            if watch.exited.is_some() && watch.pipes_closed() {
                return Ending::Done(watch.exited.take());
            }
            match watch.next_event(live).await {
                Event::Output(stream, text) => self.on_output(live, stream, &text),
                Event::Exited(info) => {
                    watch.exited = Some(info);
                    watch.exit = None;
                    watch.drain_at = Some(Instant::now() + PIPE_DRAIN);
                }
                Event::Tick => self.flush_and_project(live),
                Event::Timeout => {
                    watch.timeout_at = None;
                    self.on_timeout(live);
                }
                Event::DrainOver | Event::Force => return Ending::Done(watch.exited.take()),
                Event::Release => return Ending::Released,
            }
        }
    }
}

/// Runs a started process to its ending.
pub(crate) async fn supervise<H: ProcessHost>(
    service: RunService<H>,
    live: Arc<LiveRun>,
    io: ProcessIo,
) {
    let mut watch = Watch::new(io, &live);
    let ending = service.watch(&live, &mut watch).await;
    drop(watch);
    service.conclude(&live, ending).await;
}
