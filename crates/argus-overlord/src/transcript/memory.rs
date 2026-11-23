//! An in-memory [`TranscriptStore`] for tests, here and in the crates that host a transcript.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use super::{Page, Stamp, lines_after};
use crate::ports::{TranscriptFault, TranscriptStore};

/// Transcript files kept in a map, shared between clones.
#[derive(Default, Clone)]
pub struct MemoryTranscripts(pub Arc<Mutex<BTreeMap<String, String>>>);

impl TranscriptStore for MemoryTranscripts {
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
