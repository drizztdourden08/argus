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
    /// Asked for too, but already pending in an earlier add; this add takes them over if that
    /// one is cancelled.
    waiting: Vec<Coll>,
    buf: Vec<DocsEvent>,
}

impl Add {
    /// Takes over a collection of a cancelled add, with the events it buffered, so none between
    /// the two begins is lost.
    fn adopt(&mut self, coll: Coll, buf: &[DocsEvent]) {
        self.waiting.retain(|c| *c != coll);
        self.colls.push(coll);
        for e in buf {
            if !self.buf.iter().any(|b| b.seq == e.seq) {
                self.buf.push(e.clone());
            }
        }
        self.buf.sort_by_key(|e| e.seq);
    }
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

    fn pending(&self, coll: Coll) -> bool {
        self.adds.iter().any(|a| a.colls.contains(&coll))
    }

    /// Starts an add; answers the collections to read, in order: the ones new to this client,
    /// then the ones only pending in another add (read in case that add is cancelled). Empty when
    /// nothing is new. Collections already followed are left alone, so a repeat never sends a
    /// second snapshot.
    pub fn begin_add(&mut self, token: u64, colls: &[Coll]) -> Vec<Coll> {
        let (mut fresh, mut waiting): (Vec<Coll>, Vec<Coll>) = (Vec::new(), Vec::new());
        for c in colls {
            if fresh.contains(c) || waiting.contains(c) {
                continue;
            }
            if !self.follows(*c) {
                fresh.push(*c);
            } else if self.pending(*c) {
                waiting.push(*c);
            }
        }
        if fresh.is_empty() {
            return fresh;
        }
        let read = [fresh.as_slice(), waiting.as_slice()].concat();
        self.adds.push(Add {
            token,
            colls: fresh,
            waiting,
            buf: Vec::new(),
        });
        read
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
            && if matches!(kind, Kind::Mux) {
                replay() && ready()
            } else {
                ready() && replay()
            };
        if sent {
            self.live.extend(add.colls);
        }
        sent
    }

    /// Writes `ready` with no collections, for a mux `sub` that named only collections already
    /// followed, so every `sub` is answered by a `ready`. Legacy streams send nothing.
    pub fn ack(&self, scope: &str, seq: i64) -> bool {
        match self.kind {
            Kind::Mux => frames::ready(self.kind, &self.sender, scope, seq, &[]),
            Kind::Legacy => true,
        }
    }

    /// Ends the stream (shutdown).
    pub fn close(&self) {
        self.sender.close();
    }

    /// Drops an add that failed to read. Another pending add that asked for the same collections
    /// takes them over, so its `ready` is true.
    pub fn cancel_add(&mut self, token: u64) {
        let Some(i) = self.adds.iter().position(|a| a.token == token) else {
            return;
        };
        let gone = self.adds.remove(i);
        for coll in &gone.colls {
            if let Some(heir) = self.adds.iter_mut().find(|a| a.waiting.contains(coll)) {
                heir.adopt(*coll, &gone.buf);
            }
        }
    }

    /// Stops following the collections, live or being added.
    pub fn remove(&mut self, colls: &[Coll]) {
        for c in colls {
            self.live.remove(c);
        }
        for add in &mut self.adds {
            add.colls.retain(|c| !colls.contains(c));
            add.waiting.retain(|c| !colls.contains(c));
        }
        self.adds.retain(|a| !a.colls.is_empty());
    }

    /// Whether it follows nothing and adds nothing.
    pub fn is_idle(&self) -> bool {
        self.live.is_empty() && self.adds.is_empty()
    }
}
