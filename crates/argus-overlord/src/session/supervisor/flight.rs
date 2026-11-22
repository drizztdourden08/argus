//! The tool calls in flight: a `tool_use` block the child printed with no `tool_result` yet. The
//! owner's header reads it to tell `working` (a tool runs) from `thinking` (the model writes).

use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::protocol::{Block, Event};

/// The ids of the tool calls that wait for their result. Cheap to clone; every clone sees the same set.
#[derive(Debug, Clone, Default)]
pub struct Flight {
    open: Arc<Mutex<HashSet<String>>>,
}

impl Flight {
    fn open(&self) -> MutexGuard<'_, HashSet<String>> {
        self.open.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Reads one decoded line: a `tool_use` opens a call, a `tool_result` closes it, and the end of
    /// the turn closes every call.
    pub fn see(&self, event: &Event) {
        match event {
            Event::Assistant(message) => {
                for block in &message.blocks {
                    if let Block::ToolUse { id, .. } = block {
                        self.open().insert(id.clone());
                    }
                }
            }
            Event::User(message) => {
                for block in &message.blocks {
                    if let Block::ToolResult { tool_use_id, .. } = block {
                        self.open().remove(tool_use_id);
                    }
                }
            }
            Event::Result(_) => self.clear(),
            _ => {}
        }
    }

    /// Forgets every call: the turn ended or the child is gone.
    pub fn clear(&self) {
        self.open().clear();
    }

    /// Is a tool call waiting for its result?
    #[must_use]
    pub fn in_flight(&self) -> bool {
        !self.open().is_empty()
    }
}

// test-tag: critical
#[cfg(test)]
mod specs {
    use super::*;
    use crate::protocol::decode_line;

    const USE: &str = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"t1","name":"Bash","input":{}}]}}"#;
    const DONE: &str = r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t1","content":"ok"}]}}"#;
    const END: &str = r#"{"type":"result","subtype":"success","session_id":"s1"}"#;

    fn see(flight: &Flight, line: &str) {
        flight.see(&decode_line(line).expect("line"));
    }

    #[test]
    fn a_tool_use_is_in_flight_until_its_result() {
        let flight = Flight::default();
        assert!(!flight.in_flight());
        see(&flight, USE);
        assert!(flight.in_flight());
        see(&flight, DONE);
        assert!(!flight.in_flight());
    }

    #[test]
    fn the_end_of_the_turn_closes_every_call() {
        let flight = Flight::default();
        see(&flight, USE);
        see(&flight, END);
        assert!(!flight.in_flight());
    }

    #[test]
    fn a_result_for_another_call_leaves_the_call_open() {
        let flight = Flight::default();
        see(&flight, USE);
        see(&flight, &DONE.replace("t1", "t2"));
        assert!(flight.in_flight());
    }
}
