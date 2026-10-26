//! A listener connects: it replaces an older one for the same key and takes the events nobody received.

use tracing::warn;

use super::WakeEngine;
use super::listeners::{AttachOptions, Delivery, ListenerId, ListenerSink, NewListener};
use super::notes::{EngineNote, ReplayedEvent};
use crate::time::iso;
use crate::wake::WakeEvent;

/// The most listeners one project accepts.
pub const MAX_LISTENERS_PER_SCOPE: usize = 64;

/// An attached listener.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attached {
    /// Hand it to [`WakeEngine::detach`] when the connection closes.
    pub id: ListenerId,
    /// How many queued events it took on the way in.
    pub replayed: usize,
}

impl WakeEngine {
    /// Register a listener (thread `*` hears every thread of the project) and replay what nobody received.
    ///
    /// A v2 listener replaces the older v2 listener of the same project and thread key: that one's sink
    /// is told so and dropped. A legacy listener (v 1) is served live but never replaces or is replaced,
    /// is not counted as received, and gets no replay.
    pub fn attach(
        &mut self,
        scope: &str,
        thread: &str,
        mut sink: Box<dyn ListenerSink>,
        options: AttachOptions,
    ) -> Attached {
        let v2 = options.v >= 2;
        if v2 {
            self.listeners.replace_older(scope, thread);
        }
        let events = if v2 {
            self.replay(scope, thread, sink.as_mut())
        } else {
            Vec::new()
        };
        let replayed = events.len();
        let new = NewListener {
            thread: thread.to_owned(),
            v: options.v,
            label: options.label,
            since: iso(self.clock.now_ms()),
            sink,
        };
        let id = self.listeners.add(scope, new);
        self.note(EngineNote::Attached {
            scope: scope.to_owned(),
        });
        if replayed > 0 {
            self.note(EngineNote::Replayed {
                scope: scope.to_owned(),
                events,
            });
        }
        Attached { id, replayed }
    }

    /// A listener's connection closed.
    pub fn detach(&mut self, id: &ListenerId) {
        if self.listeners.remove(id).is_some() {
            self.note(EngineNote::Detached {
                scope: id.scope.clone(),
            });
        }
    }

    /// Would a new listener push the project over its cap? A v2 listener that replaces an open one does not.
    #[must_use]
    pub fn listener_cap_reached(&self, scope: &str, thread: &str, v: u8) -> bool {
        let replacing = v >= 2 && self.has_listener(scope, thread);
        !replacing && self.listener_total(scope) >= MAX_LISTENERS_PER_SCOPE
    }

    fn replay(
        &mut self,
        scope: &str,
        thread: &str,
        sink: &mut dyn ListenerSink,
    ) -> Vec<ReplayedEvent> {
        let since = iso(self.clock.now_ms() - self.config.replay_ms);
        let rows = match self.store.queued_events(scope, thread, &since) {
            Ok(rows) => rows,
            Err(err) => {
                warn!(scope, thread, %err, "wake replay failed");
                return Vec::new();
            }
        };
        let mut replayed = Vec::new();
        for row in rows {
            match sink.deliver(&WakeEvent::of_row(&row)) {
                Delivery::Sent => {}
                Delivery::Gone => break,
                Delivery::Failed(err) => {
                    let note = EngineNote::DeliveryError {
                        scope: scope.to_owned(),
                        thread: row.thread.clone(),
                        id: row.id.clone(),
                        err,
                    };
                    self.note(note);
                    break;
                }
            }
            if let Err(err) = self
                .store
                .mark_delivered(&row.id, 1, &iso(self.clock.now_ms()))
            {
                warn!(scope, thread, %err, "wake replay not recorded");
            }
            replayed.push(ReplayedEvent {
                id: row.id,
                thread: row.thread,
                at: row.at,
                reason: row.reason,
                text: row.text,
            });
        }
        if !replayed.is_empty() {
            self.mirror(scope);
        }
        replayed
    }
}
