//! The steps of a sync run, as the `SyncSteps` docs show them, and the sink the job writes them
//! to.

use serde::{Deserialize, Serialize};
use serde_json::{Value, to_value};

/// What a step does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    /// The files a run touches are copied away.
    Backup,
    /// A chat is written into a scope that lacked it.
    CopySession,
    /// A chat record is brought up to date.
    UpdateSession,
    /// Only the sub-thread links of a chat record changed.
    LinkThread,
    /// An archive index follows its records.
    Archive,
    /// A sidebar folder is created in a scope.
    CreateFolder,
    /// A chat moves into a folder.
    AssignFolder,
    /// The sidebar keys are written to the Desktop app's storage.
    WriteStorage,
    /// The re-plan after the writes is empty.
    Verify,
}

/// Where a step is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepState {
    /// Started.
    Running,
    /// Finished.
    Done,
    /// Failed; `detail` says why.
    Failed,
}

/// One step of a run. The doc id is the zero-padded `seq`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStep {
    /// The number in the run, from 1.
    pub seq: u64,
    /// What it does.
    pub kind: StepKind,
    /// The account key (or the scope name when no account owns the scope); empty when none.
    pub account: String,
    /// The chat, empty when the step is not about one.
    pub session_id: String,
    /// The chat's title, empty when the step is not about one.
    pub title: String,
    /// The folder name, empty when the step is not about one.
    pub folder: String,
    /// Where the step is.
    pub status: StepState,
    /// When the step last changed, epoch milliseconds.
    pub at: i64,
    /// A short note, or the reason of a failure.
    pub detail: String,
}

impl SyncStep {
    /// The id of the step's doc, `000001`.
    #[must_use]
    pub fn doc_id(&self) -> String {
        format!("{:06}", self.seq)
    }

    /// The doc body.
    #[must_use]
    pub fn to_doc(&self) -> Value {
        to_value(self).unwrap_or(Value::Null)
    }
}

/// Where the job reports its steps. A step is reported again, with the same `seq`, each time it
/// changes.
pub trait StepSink: Send + Sync {
    /// Forgets the steps of the last run.
    fn clear(&self);
    /// Shows a step, replacing an earlier report of the same `seq`.
    fn put(&self, step: &SyncStep);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_step_reads_as_the_doc_says() {
        let step = SyncStep {
            seq: 7,
            kind: StepKind::LinkThread,
            account: "hotmail".into(),
            session_id: "local_1".into(),
            title: "t".into(),
            folder: String::new(),
            status: StepState::Running,
            at: 5,
            detail: String::new(),
        };
        assert_eq!(step.doc_id(), "000007");
        let doc = step.to_doc();
        assert_eq!(doc["kind"], "link_thread");
        assert_eq!(doc["status"], "running");
        assert_eq!(doc["sessionId"], "local_1");
    }
}
