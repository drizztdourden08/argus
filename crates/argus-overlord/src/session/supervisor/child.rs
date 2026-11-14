//! One life of the child: spawn, read its lines, write what is held, end.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use argus_core::clock::now_iso;
use tokio::sync::mpsc::UnboundedReceiver;

use super::super::guard::{GuardFault, check_init};
use super::super::state::Event as Change;
use super::run::Run;
use super::{Command, Config, Link, Observer};
use crate::ports::{ChildHost, LineStream, SpawnedChild};
use crate::protocol::{Event as Wire, Init, decode_line};

/// Everything the supervisor reads from and tells.
pub(super) struct Wiring {
    pub(super) rx: UnboundedReceiver<Command>,
    pub(super) config: Config,
    pub(super) observer: Arc<dyn Observer>,
    pub(super) alive: Arc<AtomicBool>,
    pub(super) link: Link,
}

/// How a life ended.
pub(super) enum Ended {
    /// The child went away by itself, or never started: a crash.
    Exited,
    /// Argus asked for it.
    Requested,
    /// The billing guard refused the child.
    Guard,
}

impl<H: ChildHost> Run<H> {
    /// Starts a child and serves it until it ends.
    pub(super) async fn life(&mut self, w: &mut Wiring) -> Ended {
        let Ok(spec) = (w.config.spec)(self.resume.as_deref()) else {
            return Ended::Exited;
        };
        let Ok(child) = self.host.spawn(&spec) else {
            return Ended::Exited;
        };
        self.apply(&Change::Spawned {
            pid: child.pid,
            started_at: now_iso(),
            resume_from: self.resume.clone(),
        });
        w.alive.store(true, Ordering::SeqCst);
        w.observer.started(&w.link);
        let (ended, fault) = self
            .serve(&mut w.rx, w.observer.as_ref(), child)
            .await;
        w.alive.store(false, Ordering::SeqCst);
        w.observer.stopped();
        if let Some(fault) = fault {
            w.observer.guard_failed(&fault);
        }
        self.child_gone(matches!(ended, Ended::Requested));
        ended
    }

    async fn serve(
        &mut self,
        rx: &mut UnboundedReceiver<Command>,
        observer: &dyn Observer,
        mut child: SpawnedChild<H::Lines>,
    ) -> (Ended, Option<GuardFault>) {
        let pid = child.pid;
        self.flush(pid).await;
        loop {
            tokio::select! {
                line = child.stdout.next_line() => {
                    let Some(line) = line else {
                        return (Ended::Exited, None);
                    };
                    observer.line(&line);
                    if let Some(fault) = self.on_line(pid, &line).await {
                        return (Ended::Guard, Some(fault));
                    }
                }
                command = rx.recv() => match command {
                    Some(Command::Message(text)) => {
                        self.inbox.push(text);
                        self.flush(pid).await;
                    }
                    Some(Command::Stop) | None => {
                        self.host.stop(pid).await;
                        return (Ended::Requested, None);
                    }
                },
            }
        }
    }

    /// Reads one stdout line. A fault means the child was refused and ended.
    async fn on_line(&mut self, pid: u32, line: &str) -> Option<GuardFault> {
        match decode_line(line) {
            Ok(Wire::Init(init)) => return self.on_init(pid, &init).await,
            Ok(Wire::Result(outcome)) => {
                self.turn_done(outcome.total_cost_usd);
                self.flush(pid).await;
            }
            _ => {}
        }
        None
    }

    async fn on_init(&mut self, pid: u32, init: &Init) -> Option<GuardFault> {
        if let Err(fault) = check_init(init) {
            self.apply(&Change::InitGuardFailed);
            self.host.kill_tree(pid).await;
            return Some(fault);
        }
        self.resume = Some(init.session_id.clone());
        self.apply(&Change::InitOk {
            session_id: init.session_id.clone(),
        });
        if self.busy {
            self.apply(&Change::TurnStarted);
        }
        None
    }
}
