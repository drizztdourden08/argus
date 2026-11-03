//! Whether a sub-agent's stop only waits on its own background command. Such an agent ends its
//! turn to wait, which fires `SubagentStop`, but it is not finished: its real end is the
//! `SubagentHandback` call that delivers the report.
//!
//! The transcript is read as the JSON lines it is, never as a flat string: Claude Code also
//! writes the API request (its tool list names `SubagentHandback` and describes
//! `run_in_background`) into the transcript, so a text search sees a handback in every early stop.

use std::collections::HashSet;
use std::path::Path;

use argus_agents::fs::{FileSystem, StdFs};
use serde_json::Value;

/// How much of the end of the transcript is looked at.
const TAIL_BYTES: u64 = 512 * 1024;
const HANDBACK: &str = "SubagentHandback";
const NOTICE: &str = "<task-notification>";

/// The text between `open` and `close` in `block`, if both are there.
fn tag<'a>(block: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let from = block.find(open)? + open.len();
    let len = block[from..].find(close)?;
    Some(block[from..from + len].trim())
}

/// The agent's own background commands still running, read line by line in transcript order.
#[derive(Default)]
struct Runs {
    open: HashSet<String>,
}

impl Runs {
    fn read(&mut self, line: &Value) {
        let content = &line["message"]["content"];
        match line["type"].as_str() {
            Some("assistant") => self.calls(content),
            Some("user") => self.results(content),
            _ => {}
        }
    }

    /// A background call opens a run; the handback ends every run before it.
    fn calls(&mut self, content: &Value) {
        let calls = content.as_array().into_iter().flatten();
        for call in calls.filter(|b| b["type"] == "tool_use") {
            if call["name"] == HANDBACK {
                self.open.clear();
            } else if call["input"]["run_in_background"] == true {
                self.open
                    .insert(call["id"].as_str().unwrap_or_default().to_owned());
            }
        }
    }

    /// A task notification other than `running` ends its run; so does a refused background call.
    fn results(&mut self, content: &Value) {
        if let Some(text) = content.as_str() {
            self.notices(text);
            return;
        }
        for block in content.as_array().into_iter().flatten() {
            if let Some(text) = block["text"].as_str().or_else(|| block["content"].as_str()) {
                self.notices(text);
            }
            if block["type"] == "tool_result" && block["is_error"] == true {
                self.open
                    .remove(block["tool_use_id"].as_str().unwrap_or_default());
            }
        }
    }

    fn notices(&mut self, text: &str) {
        for block in text.split(NOTICE).skip(1) {
            let status = tag(block, "<status>", "</status>");
            if status == Some("running") {
                continue;
            }
            if let Some(id) = tag(block, "<tool-use-id>", "</tool-use-id>") {
                self.open.remove(id);
            }
        }
    }
}

/// Whether the transcript lines in `tail` leave a background command of the agent running with
/// no handback after it. A cut first line, or any line that is not JSON, is skipped.
fn still_running(tail: &str) -> bool {
    let mut runs = Runs::default();
    for line in tail.lines() {
        if let Ok(value) = serde_json::from_str::<Value>(line) {
            runs.read(&value);
        }
    }
    !runs.open.is_empty()
}

/// The last 512 KB of the transcript, `None` when it cannot be read.
fn tail_of(file: &Path) -> Option<String> {
    let size = StdFs.stat(file)?.size;
    let len = size.min(TAIL_BYTES);
    let bytes = StdFs
        .read_at(file, size - len, usize::try_from(len).ok()?)
        .ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

/// True when the tail of the transcript at `file` shows a background command of the agent
/// still running and no handback after it; a missing or unreadable transcript, or one without
/// such a command, is never waiting.
pub fn on_background(file: &Value) -> bool {
    let Some(file) = file.as_str().filter(|f| !f.is_empty()) else {
        return false;
    };
    tail_of(Path::new(file)).is_some_and(|tail| still_running(&tail))
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::still_running;

    fn call(id: &str, name: &str, input: &Value) -> Value {
        json!({ "type": "assistant", "message": { "role": "assistant", "content": [
            { "type": "tool_use", "id": id, "name": name, "input": input }
        ] } })
    }

    fn notice(id: &str, status: &str) -> Value {
        let text = format!(
            "[SYSTEM NOTIFICATION]\n<task-notification>\n<task-id>b1</task-id>\n\
             <tool-use-id>{id}</tool-use-id>\n<status>{status}</status>\n</task-notification>"
        );
        json!({ "type": "user", "message": { "role": "user", "content": text } })
    }

    /// The API request Claude Code writes into the transcript: its tool list names the handback.
    fn request_shape() -> Value {
        json!({ "type": "api-request-shape", "tools": [
            { "name": "Bash", "input_schema": { "properties": { "run_in_background": { "type": "boolean" } } } },
            { "name": "SubagentHandback", "description": "Deliver your final report" }
        ] })
    }

    fn tail(lines: &[Value]) -> String {
        lines.iter().fold(String::new(), |mut text, line| {
            text.push_str(&line.to_string());
            text.push('\n');
            text
        })
    }

    fn background(id: &str) -> Value {
        call(
            id,
            "Bash",
            &json!({ "command": "cargo test", "run_in_background": true }),
        )
    }

    #[test]
    fn the_handback_tool_in_the_request_shape_is_not_a_handback() {
        assert!(still_running(&tail(&[request_shape(), background("t1")])));
    }

    #[test]
    fn a_background_run_ends_with_its_notification_or_the_handback() {
        let both = [request_shape(), background("t1"), background("t2")];
        let one_done = [&both[..], &[notice("t1", "completed")]].concat();
        assert!(still_running(&tail(&one_done)));
        let all_done = [&one_done[..], &[notice("t2", "failed")]].concat();
        assert!(!still_running(&tail(&all_done)));
        let handed = [&both[..], &[call("t3", "SubagentHandback", &json!({}))]].concat();
        assert!(!still_running(&tail(&handed)));
    }

    #[test]
    fn a_running_notice_and_a_cut_line_keep_a_run_open_and_a_foreground_call_opens_none() {
        let lines = [background("t1"), notice("t1", "running")];
        let cut = format!("ype\":\"x\"}}\n{}", tail(&lines));
        assert!(still_running(&cut));
        let foreground = call("t2", "Bash", &json!({ "command": "ls" }));
        assert!(!still_running(&tail(&[request_shape(), foreground])));
    }

    #[test]
    fn a_refused_background_call_is_not_running() {
        let refused = json!({ "type": "user", "message": { "role": "user", "content": [
            { "type": "tool_result", "tool_use_id": "t1", "is_error": true, "content": "denied" }
        ] } });
        assert!(!still_running(&tail(&[background("t1"), refused])));
    }
}
