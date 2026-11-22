//! What `tools/list` says about the Overlord's own tools.

use serde_json::{Value, json};

fn tool(name: &str, description: &str, properties: &Value, required: &[&str]) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": { "type": "object", "properties": properties, "required": required }
    })
}

/// The tool entries, in the order `tools/list` shows them after `permission_prompt`.
#[must_use]
pub fn specs() -> Vec<Value> {
    let project = json!({ "type": "string", "description": "A board key or a project name." });
    let thread = json!({ "type": "string", "description": "A thread id of that board." });
    vec![
        tool(
            "thread_message",
            "Queues a note from Overlord in a thread's inbox and wakes it, so a sleeping thread is resumed.",
            &json!({ "project": project, "thread": thread, "text": { "type": "string" } }),
            &["project", "thread", "text"],
        ),
        tool(
            "thread_wake",
            "Wakes a thread with a reason, without a note.",
            &json!({ "project": project, "thread": thread, "reason": { "type": "string" } }),
            &["project", "thread", "reason"],
        ),
        tool(
            "sessions_list",
            "Lists the desktop sessions, newest first: localId, title, cwd, archived, cliSessionId, updatedAt.",
            &json!({}),
            &[],
        ),
        tool(
            "session_message",
            "Resumes a desktop session headless with a text and waits for the outcome: ok, sign-in, failed, timed out or skipped.",
            &json!({ "localId": { "type": "string" }, "text": { "type": "string" } }),
            &["localId", "text"],
        ),
    ]
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tool_has_an_object_schema_with_its_required_names() {
        let all = specs();
        let names: Vec<&str> = all.iter().filter_map(|t| t["name"].as_str()).collect();
        assert_eq!(
            names,
            [
                "thread_message",
                "thread_wake",
                "sessions_list",
                "session_message"
            ]
        );
        for entry in &all {
            assert_eq!(entry["inputSchema"]["type"], "object");
        }
        assert_eq!(all[3]["inputSchema"]["required"], json!(["localId", "text"]));
    }
}
