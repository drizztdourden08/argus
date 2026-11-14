//! The approval queue state: what is pending, what was decided, nothing else.

use super::KEEP;
use super::model::{Approval, Decision, Status};

/// Why a decision was not taken.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecideFault {
    /// No approval has this id.
    Unknown,
    /// It was already answered or expired.
    Settled,
}

/// The approvals in memory, oldest first.
#[derive(Debug, Default)]
pub struct Queue {
    items: Vec<Approval>,
    seq: u64,
}

impl Queue {
    /// Adds a pending approval and returns it.
    pub fn submit(&mut self, tool: &str, summary: &str, at: &str, now_ms: i64) -> Approval {
        self.seq += 1;
        let approval = Approval {
            id: format!("ap-{now_ms:x}-{}", self.seq),
            at: at.to_owned(),
            tool: tool.to_owned(),
            input_summary: summary.to_owned(),
            status: Status::Pending,
            decided_at: None,
            decided_by: None,
            message: None,
            requested_ms: now_ms,
        };
        self.items.push(approval.clone());
        self.trim();
        approval
    }

    fn trim(&mut self) {
        while self.items.len() > KEEP {
            let Some(at) = self.items.iter().position(|a| a.status != Status::Pending) else {
                return;
            };
            self.items.remove(at);
        }
    }

    fn settle(
        &mut self,
        id: &str,
        (status, by, message): (Status, &str, Option<&str>),
        at: &str,
    ) -> Result<Approval, DecideFault> {
        let item = self
            .items
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or(DecideFault::Unknown)?;
        if item.status != Status::Pending {
            return Err(DecideFault::Settled);
        }
        item.status = status;
        item.decided_at = Some(at.to_owned());
        item.decided_by = Some(by.to_owned());
        item.message = message.map(str::to_owned);
        Ok(item.clone())
    }

    /// The owner's answer. A denial may carry a message.
    ///
    /// # Errors
    /// `Unknown` for an id nobody asked, `Settled` when it was answered already.
    pub fn decide(
        &mut self,
        id: &str,
        decision: Decision,
        message: Option<&str>,
        at: &str,
    ) -> Result<Approval, DecideFault> {
        let status = match decision {
            Decision::Allow => Status::Allowed,
            Decision::Deny => Status::Denied,
        };
        self.settle(id, (status, "owner", message), at)
    }

    /// The timeout, or the child going away: the approval expires, answered by `by`.
    ///
    /// # Errors
    /// The same faults as `decide`.
    pub fn expire(&mut self, id: &str, by: &str, at: &str) -> Result<Approval, DecideFault> {
        self.settle(id, (Status::Expired, by, None), at)
    }

    /// The approval with this id.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<Approval> {
        self.items.iter().find(|a| a.id == id).cloned()
    }

    /// The pending ids, oldest first.
    #[must_use]
    pub fn pending(&self) -> Vec<String> {
        self.items
            .iter()
            .filter(|a| a.status == Status::Pending)
            .map(|a| a.id.clone())
            .collect()
    }

    /// Up to `limit` approvals, newest first.
    #[must_use]
    pub fn recent(&self, limit: usize) -> Vec<Approval> {
        self.items.iter().rev().take(limit).cloned().collect()
    }
}
