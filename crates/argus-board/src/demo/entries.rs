//! The owner-facing entries of the demo board: asks with options, a decision, notices, notes,
//! a to-do, a lock, a log line, an inbox message, and a blocker on an ask.

use super::{Step, step};

const CONTEXT_Q1: &str = "The lobby and the seating rules touch the same store. Shipping them together is riskier but saves one release. If unanswered the lobby ships first.";
const CONTEXT_Q2: &str = "A guest who leaves mid-join leaves a half made request behind. We can drop it quietly or tell the host. If unanswered it is dropped quietly.";
const CONTEXT_D1: &str = "Only the host approves join requests. A room-wide vote was considered and set aside because rooms are small. Recorded so the next change does not reopen it.";

pub(super) const STEPS: &[Step] = &[
    step(
        "lead",
        "q1",
        &[
            "ask",
            "{epic}",
            "Ship the lobby before the seating rules?",
            "--context",
            CONTEXT_Q1,
            "--option",
            "Ship the lobby first",
            "--option",
            "Ship both together",
            "--option",
            "Seating rules first",
        ],
    ),
    step(
        "lead",
        "q2",
        &[
            "ask",
            "{s2}",
            "Tell the host when a guest cancels a join?",
            "--context",
            CONTEXT_Q2,
        ],
    ),
    step(
        "lead",
        "d1",
        &[
            "decide",
            "{s2}",
            "Only the host approves join requests.",
            "--context",
            CONTEXT_D1,
            "--option",
            "Host only",
            "--option",
            "Room vote",
        ],
    ),
    step("lead", "", &["answer", "{q2}", "Drop it quietly."]),
    step("lead", "", &["item", "block", "{t1}", "--on", "{q1}"]),
    step(
        "lead",
        "",
        &[
            "notify",
            "The gate is green on the lobby branch.",
            "--item",
            "{s3}",
            "--level",
            "info",
        ],
    ),
    step(
        "lead",
        "",
        &[
            "notify",
            "The seat count test fails on the last run.",
            "--item",
            "{bug}",
            "--level",
            "warn",
        ],
    ),
    step(
        "lead",
        "",
        &[
            "message",
            "The demo board is ready to look at: every card kind has an example.",
            "--context",
            "A plain information message for the owner. Nothing is asked and nothing waits on it.",
        ],
    ),
    step(
        "ui",
        "",
        &[
            "note",
            "{s1}",
            "Design review done, the list needs a filter later.",
        ],
    ),
    step(
        "api",
        "",
        &[
            "note",
            "{ta3}",
            "Flaky on the first run, passes when run again.",
        ],
    ),
    step(
        "lead",
        "",
        &[
            "action",
            "add",
            "Try the lobby on a phone",
            "--item",
            "{s3}",
            "--who",
            "owner",
        ],
    ),
    step(
        "lead",
        "",
        &[
            "lock",
            "claim",
            "lobby-db",
            "--desc",
            "Migrating the lobby tables",
        ],
    ),
    step(
        "lead",
        "",
        &[
            "log",
            "Seeded the demo board.",
            "--subject",
            "demo",
            "--item",
            "{epic}",
        ],
    ),
    step(
        "lead",
        "",
        &["inbox", "send", "ui", "Pick up the join button next."],
    ),
];
