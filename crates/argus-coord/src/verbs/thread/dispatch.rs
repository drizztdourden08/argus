//! The coordinator adding a new thread (`coord thread add`, with `coord dispatch` as its alias):
//! it is recorded on the board as `dispatched` (so the checker notices when it never registers)
//! and the prompt to hand to the session chip (or to paste into a new session) is printed.

use argus_board::json::display;
use argus_board::limits::{optional_text, require_text};
use argus_board::names::session_name;
use argus_board::text::slug;
use serde_json::{Value, json};

use super::frame::{Frame, is_coordinator, with_frame};
use super::newthread::new_thread;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;

const TITLE_WORDS: usize = 4;

/// What the prompt of a dispatched thread says.
struct Prompt<'a> {
    project: &'a str,
    display: &'a str,
    title: &'a str,
    name: &'a str,
    subject: &'a str,
    brief: &'a str,
    worktree: Option<&'a str>,
}

fn prompt(p: &Prompt<'_>) -> String {
    let (project, display, title, name) = (p.project, p.display, p.title, p.name);
    let from = p
        .worktree
        .map_or_else(String::new, |w| format!("  (from the {w} worktree)"));
    [
        format!(
            "You are \"{title}\", the \"{name}\" thread of the {display} coordination board (subject: {}).",
            p.subject
        ),
        String::new(),
        "Before anything else, load the `coord` skill and register on the board:".to_owned(),
        format!("  1. Rename this session to \"{title}\" (set_session_title, session_id \"self\")."),
        format!(
            "  2. Run: coord thread register {name} --subject {} --session \"{title}\" --project {project}{from}",
            p.subject
        ),
        "  3. Run: coord inbox   and act on anything waiting. From then on, hooks hand you new"
            .to_owned(),
        "     messages on their own; run coord inbox --ack once you have handled them.".to_owned(),
        String::new(),
        "Then work the brief below. Enter each thing you are asked to do as a work item with"
            .to_owned(),
        "short acceptance criteria (coord item add), keep your note current (coord thread note),"
            .to_owned(),
        "log what you finish (coord log), claim shared resources before using them (coord lock"
            .to_owned(),
        "claim), ask the human through the board (coord ask) and record decisions (coord decide)."
            .to_owned(),
        "Register any sub-agent you spawn (coord agent start/stop). Document every test on the board"
            .to_owned(),
        "(coord test add, steps, expectations, result with --proof screenshots), never in a separate"
            .to_owned(),
        "page or file. The owner sees the board live on the dashboard (coord board open). Run coord"
            .to_owned(),
        "thread done at the end.".to_owned(),
        String::new(),
        "## Brief".to_owned(),
        p.brief.to_owned(),
    ]
    .join("\n")
}

/// `charAt(0).toUpperCase() + slice(1)`.
fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) if first.len_utf16() == 1 => {
            format!("{}{}", first.to_uppercase(), chars.as_str())
        }
        _ => word.to_owned(),
    }
}

/// The chip title: "<Project> - <Subject>", the subject at most four words, each capitalised.
fn chip_title(display: &str, typed: Option<&str>, subject: &str) -> String {
    let source = typed.map_or_else(|| subject.replace('-', " "), str::to_owned);
    let trimmed = argus_core::js::trim(&source);
    let words: Vec<&str> = if trimmed.is_empty() {
        vec![""]
    } else {
        super::docs::words(trimmed).take(TITLE_WORDS).collect()
    };
    let shown: Vec<String> = words.into_iter().map(capitalize).collect();
    format!("{display} - {}", shown.join(" "))
}

/// Adds the thread to the board as `dispatched`, for the coordinator only.
fn add(
    frame: &mut Frame<'_>,
    (name, subject, brief): (&str, &str, &str),
    worktree: Option<&str>,
) -> Outcome {
    frame.mutate(false, |board, fr| {
        let who = fr.caller(board)?;
        if !is_coordinator(&who) {
            return Err(CoordError::refuse(
                "only the coordinator adds a thread (register with --role coordinator).",
            ));
        }
        let known = &board["threads"][name];
        if !known.is_null() && known["status"] != "done" {
            return Err(CoordError::refuse(format!(
                "thread \"{name}\" already exists ({}).",
                display(&known["status"])
            )));
        }
        let fields = vec![
            ("status", json!("dispatched")),
            ("subject", json!(subject)),
            ("brief", json!(brief)),
            ("worktree", json!(worktree)),
            ("root", Value::Null),
            ("session", Value::Null),
            (
                "note",
                json!("dispatched, waiting for the session to register"),
            ),
            ("flags", json!({"needsHuman": false, "blocked": false})),
            ("lastSeen", Value::Null),
        ];
        new_thread(board, name, fields, &who.actor);
        Ok(())
    })
}

/// The verb `coord dispatch <name> --subject <s> --brief "<text>"`, as `thread add`.
///
/// # Errors
/// Refused for a missing or too long name, subject or brief, a caller that is not the
/// coordinator, a thread that already exists and no board.
pub fn run_frame(frame: &mut Frame<'_>) -> Outcome {
    let name = slug(&require_text("name", &json!(frame.arg(0)))?);
    let subject = slug(&require_text("subject", &frame.value("subject"))?);
    let brief = require_text("brief", &frame.value("brief"))?;
    let worktree = optional_text("name", &frame.value("worktree"))?;
    add(frame, (&name, &subject, &brief), worktree.as_deref())?;
    let board = frame.read_board()?;
    let shown = session_name(&frame.paths.coord_home, frame.project, board["name"].as_str());
    let title = chip_title(&shown, frame.options.text("title"), &subject);
    let text = prompt(&Prompt {
        project: frame.project,
        display: &shown,
        title: &title,
        name: &name,
        subject: &subject,
        brief: &brief,
        worktree: worktree.as_deref(),
    });
    let first = format!(
        "dispatched {name} ({subject}). Chip title: \"{title}\". Prompt for the session chip:"
    );
    frame.say(&[&first, "", &text]);
    Ok(())
}

/// The verb `dispatch`.
///
/// # Errors
/// As [`run_frame`].
pub fn run(call: &mut Call<'_>) -> Outcome {
    with_frame(call, run_frame)
}
