//! The words of a relay note and of the headless wake.

/// A thread the note names: project, thread and its desktop session id (`local_...`).
pub struct Line<'a> {
    /// Project key.
    pub project: &'a str,
    /// The thread.
    pub thread: &'a str,
    /// The desktop session id.
    pub local_id: &'a str,
}

/// The inbox note to the relay: the instruction, then one line per target.
#[must_use]
pub fn note_text(lines: &[Line<'_>]) -> String {
    let head = "RELAY (owner rule D-107 on the Argus board): wake these paused threads with the \
                desktop's send-to-session tool (ToolSearch \
                \"select:mcp__ccd_session_mgmt__send_message\"), one call each, message \
                \"WAKE <thread>: run coord inbox (relayed by Argus)\". No reply needed.";
    let rows = lines.iter().map(|line| {
        format!(
            "- {} {}: session_id {}",
            line.project, line.thread, line.local_id
        )
    });
    std::iter::once(head.to_owned())
        .chain(rows)
        .collect::<Vec<_>>()
        .join("\n")
}

/// The message a headless resume carries.
#[must_use]
pub fn wake_text(thread: &str) -> String {
    format!("WAKE {thread}: run coord inbox (relayed by Argus)")
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_note_has_the_instruction_and_one_line_per_target() {
        let lines = [
            Line {
                project: "ui",
                thread: "lead",
                local_id: "local_a",
            },
            Line {
                project: "api",
                thread: "worker",
                local_id: "local_b",
            },
        ];
        let text = note_text(&lines);
        let rows: Vec<&str> = text.lines().collect();
        assert_eq!(rows.len(), 3);
        assert!(rows[0].starts_with("RELAY (owner rule D-107 on the Argus board): wake these"));
        assert!(rows[0].ends_with("(relayed by Argus)\". No reply needed."));
        assert_eq!(rows[1], "- ui lead: session_id local_a");
        assert_eq!(rows[2], "- api worker: session_id local_b");
    }

    #[test]
    fn the_headless_wake_names_the_thread() {
        assert_eq!(
            wake_text("lead"),
            "WAKE lead: run coord inbox (relayed by Argus)"
        );
    }
}
