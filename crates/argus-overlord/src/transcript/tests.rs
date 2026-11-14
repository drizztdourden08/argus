use std::collections::BTreeMap;
use std::sync::Mutex;

use super::*;
use crate::ports::{ApiKey, TranscriptFault, TranscriptStore};

const KEY: &str = "sk-ant-api03-SECRETSECRETSECRET-abcd";

#[derive(Default)]
struct Memory(Mutex<BTreeMap<String, String>>);

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
        let all = self.0.lock().unwrap();
        let stamps = all.keys().zip(0..).map(|(k, ms)| Stamp {
            session: k.clone(),
            modified_ms: ms,
        });
        Ok(stamps.collect())
    }

    fn delete(&self, session: &str) -> Result<(), TranscriptFault> {
        self.0.lock().unwrap().remove(session);
        Ok(())
    }
}

fn stamps(count: u64) -> Vec<Stamp> {
    (0..count)
        .map(|i| Stamp {
            session: format!("s{i:02}"),
            modified_ms: 1000 + i,
        })
        .collect()
}

// test-tag: critical
#[test]
fn an_event_holding_the_key_is_stored_without_it() {
    let transcript = Transcript::new(Memory::default(), &ApiKey::new(KEY));
    let nested = format!(r#"{{"type":"user","content":"{{\"stdout\":\"echo {KEY}\"}}"}}"#);
    let plain = format!(r#"{{"type":"assistant","text":"my key is {KEY}","{KEY}":1}}"#);
    let loose = format!("not json {KEY}");
    for raw in [&nested, &plain, &loose] {
        let stored = transcript.record("s1", raw).unwrap();
        assert!(!stored.contains("SECRETSECRET"), "{stored}");
        assert!(stored.contains("[redacted]"));
    }
    let page = transcript.read_after("s1", 0).unwrap();
    assert_eq!(page.lines.len(), 3);
    assert!(page.lines.iter().all(|l| !l.contains("SECRETSECRET")));
    let last: serde_json::Value = serde_json::from_str(&page.lines[2]).unwrap();
    assert_eq!(last["type"], "raw");
}

#[test]
fn reading_after_an_index_skips_what_was_read() {
    let transcript = Transcript::new(Memory::default(), &ApiKey::new(KEY));
    for n in 0..4 {
        transcript.record("s1", &format!(r#"{{"n":{n}}}"#)).unwrap();
    }
    let page = transcript.read_after("s1", 2).unwrap();
    assert_eq!(page.lines, vec![r#"{"n":2}"#, r#"{"n":3}"#]);
    assert_eq!(page.next, 4);
    assert_eq!(transcript.read_after("s1", 9).unwrap().lines.len(), 0);
}

#[test]
fn a_line_without_its_newline_is_not_read_yet() {
    let page = lines_after("{\"a\":1}\n{\"b\"", 0);
    assert_eq!(page.lines, vec![r#"{"a":1}"#]);
    assert_eq!(page.next, 1);
}

#[test]
fn retention_keeps_the_ten_newest_of_twelve() {
    let doomed = stale(&stamps(12), KEEP_SESSIONS);
    assert_eq!(doomed, vec!["s01", "s00"]);
    assert_eq!(stale(&stamps(10), KEEP_SESSIONS).len(), 0);
    assert_eq!(newest(&stamps(12)).as_deref(), Some("s11"));
}

#[test]
fn prune_deletes_the_oldest_files() {
    let transcript = Transcript::new(Memory::default(), &ApiKey::new(KEY));
    for s in stamps(12) {
        transcript.record(&s.session, "{}").unwrap();
    }
    assert_eq!(transcript.prune().unwrap(), 2);
    assert_eq!(transcript.read_after("s00", 0).unwrap().next, 0);
    assert_eq!(transcript.read_after("s11", 0).unwrap().next, 1);
}

#[test]
fn file_names_round_trip_and_unsafe_ids_are_refused() {
    assert_eq!(
        file_name("abc-1_2").as_deref(),
        Some("transcript-abc-1_2.jsonl")
    );
    assert_eq!(
        session_of("transcript-abc-1_2.jsonl").as_deref(),
        Some("abc-1_2")
    );
    assert_eq!(file_name("../x"), None);
    assert_eq!(file_name(""), None);
    assert_eq!(session_of("state.json"), None);
}
