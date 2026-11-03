//! One client of the fan-out: a sender, the collections it follows, and the collections being
//! added. An added collection is pending: its events are buffered, not written, until its first
//! frames are out, so the live collections never wait for it.

use std::collections::HashSet;

use argus_core::{Coll, DocsEvent};

use super::first_frames::First;
use super::frames::{self, Kind};
use crate::sse::SseSender;

/// One add in flight: its collections and the events that touched them since it began.
struct Add {
    token: u64,
    colls: Vec<Coll>,
    buf: Vec<DocsEvent>,
}

/// One scope of one stream.
pub struct Client {
    /// Its id in the scope.
    pub id: u64,
    /// Which frame shapes it gets.
    pub kind: Kind,
    sender: SseSender,
    live: HashSet<Coll>,
    adds: Vec<Add>,
}

impl Client {
    /// A client that follows nothing yet.
    pub fn new(id: u64, kind: Kind, sender: SseSender) -> Self {
        Client {
            id,
            kind,
            sender,
            live: HashSet::new(),
            adds: Vec::new(),
        }
    }

    /// Whether the collection is followed or being added.
    fn follows(&self, coll: Coll) -> bool {
        self.live.contains(&coll) || self.adds.iter().any(|a| a.colls.contains(&coll))
    }

    /// Starts an add; answers the collections that are new to this client, in order. Collections
    /// it already follows are left alone, so a repeat never sends a second snapshot.
    pub fn begin_add(&mut self, token: u64, colls: &[Coll]) -> Vec<Coll> {
        let mut fresh: Vec<Coll> = Vec::new();
        for c in colls {
            if !self.follows(*c) && !fresh.contains(c) {
                fresh.push(*c);
            }
        }
        if !fresh.is_empty() {
            self.adds.push(Add {
                token,
                colls: fresh.clone(),
                buf: Vec::new(),
            });
        }
        fresh
    }

    /// Delivers one doc apply: live collections get their frame now, collections being added get
    /// the event buffered. `false` once the stream is closed.
    pub fn deliver(&mut self, scope: &str, e: &DocsEvent) -> bool {
        for add in &mut self.adds {
            if e.changes.iter().any(|c| add.colls.contains(&c.change.coll)) {
                add.buf.push(e.clone());
            }
        }
        frames::changes(self.kind, &self.sender, scope, e, |c| {
            self.live.contains(&c)
        })
    }

    /// Writes a reset.
    pub fn reset(&self, scope: &str, reason: &str) -> bool {
        frames::reset(self.kind, &self.sender, scope, reason)
    }

    /// Ends an add: the first frames, the buffered events above the read seq, `ready`, then the
    /// collections join the live set. A legacy client gets `ready` before the buffered events (the
    /// order of the Node server); a mux client after them, so its `ready` means complete. `false`
    /// once the stream is closed.
    pub fn finish_add(&mut self, scope: &str, token: u64, first: &First) -> bool {
        let Some(i) = self.adds.iter().position(|a| a.token == token) else {
            return true;
        };
        let add = self.adds.remove(i);
        let (kind, sender) = (self.kind, &self.sender);
        let keep = |c: Coll| add.colls.contains(&c);
        let ready = || frames::ready(kind, sender, scope, first.seq, &add.colls);
        let replay = || {
            add.buf
                .iter()
                .filter(|e| e.seq > first.seq)
                .all(|e| frames::changes(kind, sender, scope, e, keep))
        };
        let sent = frames::opening(kind, sender, scope, first, &add.colls)
            && match kind {
                Kind::Legacy => ready() && replay(),
                Kind::Mux => replay() && ready(),
            };
        if sent {
            self.live.extend(add.colls);
        }
        sent
    }

    /// Drops an add that failed to read.
    pub fn cancel_add(&mut self, token: u64) {
        self.adds.retain(|a| a.token != token);
    }

    /// Stops following the collections, live or being added.
    pub fn remove(&mut self, colls: &[Coll]) {
        for c in colls {
            self.live.remove(c);
        }
        for add in &mut self.adds {
            add.colls.retain(|c| !colls.contains(c));
        }
        self.adds.retain(|a| !a.colls.is_empty());
    }

    /// Whether it follows nothing and adds nothing.
    pub fn is_idle(&self) -> bool {
        self.live.is_empty() && self.adds.is_empty()
    }
}
