//! The live child's transcript, written as it happens: every stdout line, every message written to
//! its stdin and every approval change, one redacted line each, into `transcript-<session>.jsonl`.
//! The session id comes from the `init` event (a resumed session is known up front), so lines that
//! come first are held and written once it is known. A `stream_event` (a partial message of the
//! live thinking) goes to the open pages only: the whole event that follows is what is stored.

use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};

use argus_core::clock::now_iso;
use argus_core::log::Logger;
use argus_overlord::approvals::Approval;
use argus_overlord::ports::{ApiKey, SecretStore, TranscriptStore};
use argus_overlord::protocol::{Event as Wire, decode_line};
use argus_overlord::transcript::Transcript;
use serde_json::{Value, json};

use crate::adapters::overlord::feed::{FedLine, TranscriptFeed};
use crate::adapters::overlord::key::DpapiKeyStore;
use crate::adapters::overlord::transcript::FileTranscripts;

/// The most lines held while the session id is unknown.
const HELD_MAX: usize = 500;

#[derive(Default)]
struct Cursor {
    session: Option<String>,
    held: Vec<String>,
    /// The lines the session file holds, read once when first needed.
    stored: Option<usize>,
}

/// Writes the lines of one live child into its session's transcript.
pub(super) struct Recorder<S> {
    transcript: Transcript<S>,
    cursor: Mutex<Cursor>,
    feed: Option<Arc<TranscriptFeed>>,
}

impl<S: TranscriptStore> Recorder<S> {
    /// A recorder that hides `key`; `session` is the resumed session, when the child has one.
    /// Every line it handles is also told to `feed`, for the pages open now.
    pub(super) fn new(
        store: S,
        key: &ApiKey,
        session: Option<String>,
        feed: Option<Arc<TranscriptFeed>>,
    ) -> Self {
        Self {
            transcript: Transcript::new(store, key),
            cursor: Mutex::new(Cursor {
                session,
                ..Cursor::default()
            }),
            feed,
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
        let stamped = stamped(raw, &now_iso());
        let raw = stamped.as_str();
        let mut cursor = self.cursor.lock().unwrap_or_else(PoisonError::into_inner);
        let partial = is_stream_event(raw);
        if let Some(session) = cursor.session.clone() {
            if partial {
                self.tell(&mut cursor, &session, raw, false);
            } else {
                self.store(&mut cursor, &session, raw);
            }
        } else if !partial {
            if cursor.held.len() == HELD_MAX {
                cursor.held.remove(0);
            }
            cursor.held.push(raw.to_owned());
        }
    }

    fn open(&self, session: &str) {
        let mut cursor = self.cursor.lock().unwrap_or_else(PoisonError::into_inner);
        cursor.session = Some(session.to_owned());
        cursor.stored = None;
        for raw in std::mem::take(&mut cursor.held) {
            self.store(&mut cursor, session, &raw);
        }
    }

    fn store(&self, cursor: &mut Cursor, session: &str, raw: &str) {
        let count = self.count(cursor, session);
        if self.transcript.record(session, raw).is_ok() {
            cursor.stored = Some(count + 1);
            self.tell(cursor, session, raw, true);
        }
    }

    fn count(&self, cursor: &mut Cursor, session: &str) -> usize {
        *cursor.stored.get_or_insert_with(|| {
            self.transcript
                .read_after(session, 0)
                .map_or(0, |page| page.next)
        })
    }

    fn tell(&self, cursor: &mut Cursor, session: &str, raw: &str, stored: bool) {
        let Some(feed) = &self.feed else { return };
        feed.publish(&FedLine {
            session: session.to_owned(),
            index: self.count(cursor, session),
            line: self.transcript.redact(raw),
            stored,
        });
    }
}

/// Whether the line is a partial message of the stream (`--include-partial-messages`).
fn is_stream_event(raw: &str) -> bool {
    serde_json::from_str::<Value>(raw)
        .is_ok_and(|value| value.get("type").and_then(Value::as_str) == Some("stream_event"))
}

/// The line with the time it arrived in `argusAt`, so the chat can show it. A line that is not a
/// JSON object, or already has the field, is kept as it is.
fn stamped(raw: &str, at: &str) -> String {
    match serde_json::from_str::<Value>(raw) {
        Ok(Value::Object(mut map)) if !map.contains_key("argusAt") => {
            map.insert("argusAt".to_owned(), Value::String(at.to_owned()));
            Value::Object(map).to_string()
        }
        _ => raw.to_owned(),
    }
}

/// The recorder of a live child, with the key read now. Without a key nothing is recorded (the
/// transcript must never hold it), and one warning says so.
pub(super) fn open(
    home: &Path,
    log: &Logger,
    resumed: Option<String>,
    feed: &Arc<TranscriptFeed>,
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
        Some(Arc::clone(feed)),
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
    use argus_overlord::approvals::Status;
    use argus_overlord::transcript::MemoryTranscripts;

    use super::*;

    const KEY: &str = "sk-ant-api03-SECRETSECRETSECRET-abcd";
    const INIT: &str = r#"{"type":"system","subtype":"init","session_id":"s1","apiKeySource":"ANTHROPIC_API_KEY","model":"sonnet","tools":[],"mcp_servers":[],"cwd":"X:/"}"#;

    type Memory = MemoryTranscripts;

    fn lines(store: &Memory, session: &str) -> Vec<String> {
        store.read_after(session, 0).unwrap().lines
    }

    // test-tag: critical
    #[test]
    fn lines_before_the_init_are_held_then_written_in_order() {
        let store = Memory::default();
        let recorder = Recorder::new(store.clone(), &ApiKey::new(KEY), None, None);
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
        let recorder = Recorder::new(
            store.clone(),
            &ApiKey::new(KEY),
            Some("s9".to_owned()),
            None,
        );
        recorder.add(&format!(r#"{{"type":"user","text":"{KEY}"}}"#));
        let got = lines(&store, "s9");
        assert_eq!(got.len(), 1);
        assert!(!got[0].contains("SECRETSECRET"));
    }

    // test-tag: critical
    #[test]
    fn a_stream_event_reaches_the_pages_and_is_not_stored() {
        let store = Memory::default();
        let feed = Arc::new(TranscriptFeed::default());
        let heard: Arc<Mutex<Vec<FedLine>>> = Arc::default();
        let sink = Arc::clone(&heard);
        let _listening = feed.listen(move |fed| sink.lock().unwrap().push(fed.clone()));
        let recorder = Recorder::new(
            store.clone(),
            &ApiKey::new(KEY),
            Some("s9".to_owned()),
            Some(Arc::clone(&feed)),
        );
        recorder.add(r#"{"type":"user","n":1}"#);
        recorder.add(&format!(
            r#"{{"type":"stream_event","event":{{"delta":{{"thinking":"{KEY}"}}}}}}"#
        ));
        recorder.add(r#"{"type":"assistant","n":2}"#);
        let got = lines(&store, "s9");
        assert_eq!(got.len(), 2);
        assert!(!got.iter().any(|line| line.contains("stream_event")));
        let heard = heard.lock().unwrap();
        assert_eq!(heard.len(), 3);
        assert!(heard[1].line.contains("stream_event"));
        assert!(!heard[1].line.contains("SECRETSECRET"));
        assert!(!heard[1].stored);
        assert_eq!(heard[1].index, 1);
        assert!(heard[2].stored);
        assert_eq!(heard[2].index, 2);
    }

    #[test]
    fn a_line_keeps_the_time_it_arrived() {
        let got: Value = serde_json::from_str(&stamped(r#"{"type":"assistant"}"#, "T1")).unwrap();
        assert_eq!(got["argusAt"], "T1");
        assert_eq!(got["type"], "assistant");
        assert_eq!(stamped("not json", "T1"), "not json");
        let known = r#"{"argusAt":"T0"}"#;
        assert_eq!(stamped(known, "T1"), known);
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
