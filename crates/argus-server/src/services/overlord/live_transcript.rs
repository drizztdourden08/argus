//! The live child's transcript, written as it happens: every stdout line, every message written to
//! its stdin and every approval change, one redacted line each, into `transcript-<session>.jsonl`.
//! The session id comes from the `init` event (a resumed session is known up front), so lines that
//! come first are held and written once it is known.

use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};

use argus_core::log::Logger;
use argus_overlord::approvals::Approval;
use argus_overlord::ports::{ApiKey, SecretStore, TranscriptStore};
use argus_overlord::protocol::{Event as Wire, decode_line};
use argus_overlord::transcript::Transcript;
use serde_json::json;

use crate::adapters::overlord::key::DpapiKeyStore;
use crate::adapters::overlord::transcript::FileTranscripts;

/// The most lines held while the session id is unknown.
const HELD_MAX: usize = 500;

#[derive(Default)]
struct Cursor {
    session: Option<String>,
    held: Vec<String>,
}

/// Writes the lines of one live child into its session's transcript.
pub(super) struct Recorder<S> {
    transcript: Transcript<S>,
    cursor: Mutex<Cursor>,
}

impl<S: TranscriptStore> Recorder<S> {
    /// A recorder that hides `key`; `session` is the resumed session, when the child has one.
    pub(super) fn new(store: S, key: &ApiKey, session: Option<String>) -> Self {
        Self {
            transcript: Transcript::new(store, key),
            cursor: Mutex::new(Cursor {
                session,
                held: Vec::new(),
            }),
        }
    }

    /// One raw stdout line of the child. An `init` names the session and releases what was held.
    pub(super) fn line(&self, raw: &str) {
        if let Ok(Wire::Init(init)) = decode_line(raw) {
            self.open(&init.session_id);
        }
        self.add(raw);
    }

    /// One line written to the child's stdin or one approval change.
    pub(super) fn add(&self, raw: &str) {
        let mut cursor = self.cursor.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(session) = cursor.session.clone() {
            let _ = self.transcript.record(&session, raw);
        } else {
            if cursor.held.len() == HELD_MAX {
                cursor.held.remove(0);
            }
            cursor.held.push(raw.to_owned());
        }
    }

    fn open(&self, session: &str) {
        let mut cursor = self.cursor.lock().unwrap_or_else(PoisonError::into_inner);
        cursor.session = Some(session.to_owned());
        for raw in std::mem::take(&mut cursor.held) {
            let _ = self.transcript.record(session, &raw);
        }
    }
}

/// The recorder of a live child, with the key read now. Without a key nothing is recorded (the
/// transcript must never hold it), and one warning says so.
pub(super) fn open(
    home: &Path,
    log: &Logger,
    resumed: Option<String>,
) -> Option<Arc<Recorder<FileTranscripts>>> {
    let Ok(Some(key)) = DpapiKeyStore::new(home).load() else {
        log.warn(
            "hosted overlord transcript is off",
            json!({ "why": "no key to redact" }),
        );
        return None;
    };
    Some(Arc::new(Recorder::new(
        FileTranscripts::new(home),
        &key,
        resumed,
    )))
}

/// The transcript line of an approval: `status` is `pending`, `allowed`, `denied` or `expired`.
pub(super) fn approval_line(approval: &Approval) -> String {
    json!({
        "type": "argus",
        "event": "approval",
        "id": approval.id,
        "at": approval.at,
        "tool": approval.tool,
        "summary": approval.input_summary,
        "status": approval.status.as_str(),
        "decidedAt": approval.decided_at,
        "decidedBy": approval.decided_by,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::Arc;

    use argus_overlord::approvals::Status;
    use argus_overlord::ports::TranscriptFault;
    use argus_overlord::transcript::{Page, Stamp, lines_after};

    use super::*;

    const KEY: &str = "sk-ant-api03-SECRETSECRETSECRET-abcd";
    const INIT: &str = r#"{"type":"system","subtype":"init","session_id":"s1","apiKeySource":"ANTHROPIC_API_KEY","model":"sonnet","tools":[],"mcp_servers":[],"cwd":"X:/"}"#;

    #[derive(Default, Clone)]
    struct Memory(Arc<Mutex<BTreeMap<String, String>>>);

    impl TranscriptStore for Memory {
        fn append(&self, session: &str, line: &str) -> Result<(), TranscriptFault> {
            let mut all = self.0.lock().unwrap();
            let file = all.entry(session.to_owned()).or_default();
            file.push_str(line);
            file.push('\n');
            Ok(())
        }

        fn read_after(&self, session: &str, after: usize) -> Result<Page, TranscriptFault> {
            let all = self.0.lock().unwrap();
            Ok(lines_after(
                all.get(session).map_or("", String::as_str),
                after,
            ))
        }

        fn list(&self) -> Result<Vec<Stamp>, TranscriptFault> {
            Ok(Vec::new())
        }

        fn delete(&self, _session: &str) -> Result<(), TranscriptFault> {
            Ok(())
        }
    }

    fn lines(store: &Memory, session: &str) -> Vec<String> {
        store.read_after(session, 0).unwrap().lines
    }

    // test-tag: critical
    #[test]
    fn lines_before_the_init_are_held_then_written_in_order() {
        let store = Memory::default();
        let recorder = Recorder::new(store.clone(), &ApiKey::new(KEY), None);
        recorder.add(r#"{"type":"user","n":1}"#);
        assert_eq!(store.0.lock().unwrap().len(), 0);
        recorder.line(INIT);
        recorder.line(r#"{"type":"assistant","n":2}"#);
        let got = lines(&store, "s1");
        assert_eq!(got.len(), 3);
        assert!(got[0].contains(r#""n":1"#));
        assert!(got[1].contains("init"));
        assert!(got[2].contains(r#""n":2"#));
    }

    #[test]
    fn a_resumed_session_is_written_at_once_and_the_key_is_hidden() {
        let store = Memory::default();
        let recorder = Recorder::new(store.clone(), &ApiKey::new(KEY), Some("s9".to_owned()));
        recorder.add(&format!(r#"{{"type":"user","text":"{KEY}"}}"#));
        let got = lines(&store, "s9");
        assert_eq!(got.len(), 1);
        assert!(!got[0].contains("SECRETSECRET"));
    }

    #[test]
    fn an_approval_line_carries_its_status() {
        let approval = Approval {
            id: "A-1".to_owned(),
            at: "2026-11-22T10:00:00Z".to_owned(),
            tool: "Bash".to_owned(),
            input_summary: "ls".to_owned(),
            status: Status::Pending,
            decided_at: None,
            decided_by: None,
            message: None,
            requested_ms: 0,
        };
        let value: serde_json::Value = serde_json::from_str(&approval_line(&approval)).unwrap();
        assert_eq!(value["type"], "argus");
        assert_eq!(value["event"], "approval");
        assert_eq!(value["status"], "pending");
        assert_eq!(value["tool"], "Bash");
    }
}
