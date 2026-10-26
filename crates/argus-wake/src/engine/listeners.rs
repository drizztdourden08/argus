//! The wake-stream listeners: one per project and thread key, a newer one replaces the older one.
//!
//! A listener is a [`ListenerSink`]. The channel sink ([`channel`]) is what the HTTP stream uses: the
//! engine writes into an unbounded tokio channel and the stream task reads from it.

use serde::Serialize;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::wake::WakeEvent;

/// What came of writing an event to a listener.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delivery {
    /// The listener took it.
    Sent,
    /// The connection is gone; the listener did not take it.
    Gone,
    /// Writing threw; the message says why.
    Failed(String),
}

/// Where a listener's events go.
pub trait ListenerSink: Send {
    /// Write one event.
    fn deliver(&mut self, event: &WakeEvent) -> Delivery;

    /// A newer listener for the same project and thread key took this one's place.
    fn replaced(&mut self);
}

/// A message on a listener's channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListenerMessage {
    /// A wake event.
    Wake(WakeEvent),
    /// A newer listener replaced this one; the stream should say so and close.
    Replaced,
}

/// A sink that writes into a tokio channel.
#[derive(Debug)]
pub struct ChannelSink(UnboundedSender<ListenerMessage>);

impl ListenerSink for ChannelSink {
    fn deliver(&mut self, event: &WakeEvent) -> Delivery {
        match self.0.send(ListenerMessage::Wake(event.clone())) {
            Ok(()) => Delivery::Sent,
            Err(_) => Delivery::Gone,
        }
    }

    fn replaced(&mut self) {
        let _ = self.0.send(ListenerMessage::Replaced);
    }
}

/// A sink and the receiver the stream task reads.
#[must_use]
pub fn channel() -> (ChannelSink, UnboundedReceiver<ListenerMessage>) {
    let (tx, rx) = unbounded_channel();
    (ChannelSink(tx), rx)
}

/// How a listener attaches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachOptions {
    /// 2 is unique per thread key and counted; 1 is the legacy CLI.
    pub v: u8,
    /// A label the listing shows.
    pub label: Option<String>,
}

impl Default for AttachOptions {
    fn default() -> Self {
        Self { v: 2, label: None }
    }
}

/// Handle of an attached listener.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenerId {
    /// Project key.
    pub scope: String,
    n: u64,
}

/// An open listener as the listing shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ListenerInfo {
    /// The thread key (`*` or a thread id).
    pub thread: String,
    /// When it connected (ISO).
    pub since: String,
    /// The label it announced.
    pub label: Option<String>,
    /// 2, or 1 for a legacy one.
    pub v: u8,
}

/// How many listeners a project has for one thread key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ListenerStat {
    /// Project key.
    pub scope: String,
    /// The thread key.
    pub thread: String,
    /// Open listeners for it.
    pub count: usize,
}

struct Entry {
    n: u64,
    thread: String,
    v: u8,
    label: Option<String>,
    since: String,
    sink: Box<dyn ListenerSink>,
}

/// Every open listener, by project.
#[derive(Default)]
pub(super) struct ListenerSet {
    next: u64,
    scopes: Vec<(String, Vec<Entry>)>,
}

pub(super) struct NewListener {
    pub thread: String,
    pub v: u8,
    pub label: Option<String>,
    pub since: String,
    pub sink: Box<dyn ListenerSink>,
}

impl ListenerSet {
    fn entries(&self, scope: &str) -> &[Entry] {
        self.scopes
            .iter()
            .find(|(s, _)| s == scope)
            .map_or(&[], |(_, e)| e.as_slice())
    }

    pub(super) fn add(&mut self, scope: &str, new: NewListener) -> ListenerId {
        self.next += 1;
        let entry = Entry {
            n: self.next,
            thread: new.thread,
            v: new.v,
            label: new.label,
            since: new.since,
            sink: new.sink,
        };
        match self.scopes.iter_mut().find(|(s, _)| s == scope) {
            Some((_, entries)) => entries.push(entry),
            None => self.scopes.push((scope.to_owned(), vec![entry])),
        }
        ListenerId {
            scope: scope.to_owned(),
            n: self.next,
        }
    }

    pub(super) fn replace_older(&mut self, scope: &str, thread: &str) {
        let Some((_, entries)) = self.scopes.iter_mut().find(|(s, _)| s == scope) else {
            return;
        };
        let (older, keep): (Vec<Entry>, Vec<Entry>) = std::mem::take(entries)
            .into_iter()
            .partition(|e| e.v >= 2 && e.thread == thread);
        *entries = keep;
        for mut old in older {
            old.sink.replaced();
        }
    }

    pub(super) fn remove(&mut self, id: &ListenerId) -> Option<(String, u8)> {
        let at = self.scopes.iter().position(|(s, _)| *s == id.scope)?;
        let entries = &mut self.scopes[at].1;
        let pos = entries.iter().position(|e| e.n == id.n)?;
        let gone = entries.remove(pos);
        if entries.is_empty() {
            self.scopes.remove(at);
        }
        Some((gone.thread, gone.v))
    }

    pub(super) fn deliver(
        &mut self,
        scope: &str,
        thread: &str,
        event: &WakeEvent,
    ) -> (u32, Vec<String>) {
        let mut sent = 0;
        let mut errors = Vec::new();
        let Some((_, entries)) = self.scopes.iter_mut().find(|(s, _)| s == scope) else {
            return (sent, errors);
        };
        for entry in entries
            .iter_mut()
            .filter(|e| e.thread == "*" || e.thread == thread)
        {
            match entry.sink.deliver(event) {
                Delivery::Sent if entry.v >= 2 => sent += 1,
                Delivery::Failed(err) => errors.push(err),
                Delivery::Sent | Delivery::Gone => {}
            }
        }
        (sent, errors)
    }

    pub(super) fn total(&self, scope: &str) -> usize {
        self.entries(scope).len()
    }

    pub(super) fn has_v2(&self, scope: &str, thread: &str) -> bool {
        self.entries(scope)
            .iter()
            .any(|e| e.v >= 2 && e.thread == thread)
    }

    pub(super) fn info(&self, scope: &str) -> Vec<ListenerInfo> {
        self.entries(scope)
            .iter()
            .map(|e| ListenerInfo {
                thread: e.thread.clone(),
                since: e.since.clone(),
                label: e.label.clone(),
                v: e.v,
            })
            .collect()
    }

    pub(super) fn stats(&self) -> Vec<ListenerStat> {
        let mut out: Vec<ListenerStat> = Vec::new();
        for (scope, entries) in &self.scopes {
            for entry in entries {
                match out
                    .iter_mut()
                    .find(|s| s.scope == *scope && s.thread == entry.thread)
                {
                    Some(stat) => stat.count += 1,
                    None => out.push(ListenerStat {
                        scope: scope.clone(),
                        thread: entry.thread.clone(),
                        count: 1,
                    }),
                }
            }
        }
        out
    }
}
