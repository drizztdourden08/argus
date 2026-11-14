//! `thread register`: a session joins the board as a thread, or a registered one updates its
//! root, session title, link, subject and note. A placeholder the hooks made for this session is
//! folded into the real thread, on this board and on any other.

use argus_board::Board;
use argus_board::json::{doc, set, truthy, unset};
use argus_board::limits::{optional_text, require_text};
use argus_board::roles::is_overlord;
use argus_board::store::stamp;
use argus_board::text::slug;
use argus_core::clock::now_iso;
use argus_core::config::peek_config;
use argus_overlord::mode::Mode;
use argus_overlord::seat::{SEAT, may_take_seat};
use serde_json::{Value, json};

use super::fold;
use super::naming;
use crate::cli::exit::{CoordError, Outcome};
use crate::verbs::thread::docs::{list, thread_mut};
use crate::verbs::thread::frame::Frame;
use crate::verbs::thread::newthread::new_thread;
use crate::who::git_root;

const LINK_PREFIX: &str = "claude://";
const LINK_HELP: &str = "--link takes the session link from get_session (claude://...).";

fn word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn is_link(link: &str) -> bool {
    link.strip_prefix(LINK_PREFIX).is_some_and(|rest| {
        !rest.is_empty() && rest.chars().all(|c| word_char(c) || "./-".contains(c))
    })
}

fn is_host_id(host: &str) -> bool {
    !host.is_empty() && host.chars().all(|c| word_char(c) || c == '-')
}

/// The session link: `--link` when given, else the one made from the host session id.
fn link_of(frame: &Frame<'_>) -> Result<Option<String>, CoordError> {
    if let Some(link) = frame.options.text("link") {
        if !is_link(link) {
            return Err(CoordError::refuse(LINK_HELP));
        }
        return Ok(Some(link.to_owned()));
    }
    Ok(frame
        .env
        .get("CLAUDE_CODE_HOST_SESSION_ID")
        .filter(|host| is_host_id(host))
        .map(|host| format!("claude://claude.ai/epitaxy/{host}")))
}

fn record_session(
    board: &mut Board,
    frame: &Frame<'_>,
    name: &str,
    sid: Option<&str>,
) -> Result<(), CoordError> {
    let root = frame.options.text("root").map_or_else(
        || git_root(frame.cwd).to_string_lossy().into_owned(),
        str::to_owned,
    );
    let title = optional_text("title", &frame.value("session"))?;
    let link = link_of(frame)?;
    let thread = thread_mut(board, name)?;
    set(thread, "root", root);
    let kept = thread["session"].clone();
    set(thread, "session", title.map_or(kept, Value::from));
    if let Some(sid) = sid {
        set(thread, "sessionId", sid);
    }
    if let Some(link) = link {
        set(thread, "link", link);
    }
    Ok(())
}

fn record_texts(board: &mut Board, frame: &Frame<'_>, name: &str) -> Result<(), CoordError> {
    let subject = optional_text("subject", &frame.value("subject"))?;
    let note = optional_text("note", &frame.value("note"))?;
    let thread = thread_mut(board, name)?;
    for (key, text) in [("subject", subject), ("note", note)] {
        if let Some(text) = text {
            set(thread, key, text);
        }
    }
    if thread["status"] != "paused" {
        set(thread, "status", "active");
    }
    for key in ["placeholder", "mergedInto", "archivedAt", "archivedBy"] {
        unset(thread, key);
    }
    set(thread, "lastSeen", now_iso());
    Ok(())
}

fn tips(frame: &mut Frame<'_>, board: &Board, name: &str) {
    let thread = &board["threads"][name];
    if !truthy(thread, "session") {
        frame.say(&[
            "tip: pass --session \"<your session title>\" so the coordinator can wake you.",
        ]);
    }
    if !truthy(thread, "link") {
        frame.say(&["tip: no session link found; pass --link \"<link from get_session self>\" so the owner can open you from the board."]);
    }
    let waiting = list(&board["inbox"][name]).len();
    if waiting > 0 {
        frame.say(&[&format!("{waiting} message(s) waiting: run `coord inbox`.")]);
    }
}

fn apply(board: &mut Board, frame: &mut Frame<'_>, name: &str, sid: Option<&str>) -> Outcome {
    if let Some(sid) = sid {
        fold::placeholders(board, name, sid)?;
    }
    let existing = doc(board, "threads", name).is_some();
    if !existing {
        let role = if frame.options.text("role") == Some("coordinator") {
            "coordinator"
        } else {
            "thread"
        };
        let fields = vec![
            ("role", json!(role)),
            ("flags", json!({"needsHuman": false, "blocked": false})),
            ("note", Value::Null),
            ("subject", Value::Null),
            ("brief", Value::Null),
        ];
        new_thread(board, name, fields, name);
    }
    record_session(board, frame, name, sid)?;
    if let Some(problem) = naming::problem(&frame.paths.coord_home, board, &board["threads"][name])
    {
        frame.say(&[&format!("naming: {problem}")]);
    }
    record_texts(board, frame, name)?;
    stamp(board, "threads", name, name)?;
    let root = argus_board::json::display(&board["threads"][name]["root"]);
    let verb = if existing { "updated" } else { "registered" };
    frame.say(&[&format!("{verb} thread {name} (root {root})")]);
    tips(frame, board, name);
    Ok(())
}

/// K4: the Overlord's seat goes to the session the mode puts there, so two live coordinators
/// never exist at once (I-173).
fn check_seat(frame: &Frame<'_>, name: &str) -> Result<(), CoordError> {
    if name != SEAT || !is_overlord(&frame.paths.coord_home, frame.project) {
        return Ok(());
    }
    let config = peek_config(frame.paths, None).unwrap_or_default();
    let mode = Mode::of_config(config["overlord"]["mode"].as_str());
    let hosted = frame.env.get("ARGUS_HOSTED") == Some("1");
    if let Err(fault) = may_take_seat(mode, hosted) {
        return Err(CoordError::refuse(fault.to_string()));
    }
    Ok(())
}

/// The verb `thread register <name>`.
///
/// # Errors
/// Refused for a missing or too long name, a bad `--link`, texts over their limits, no board, and
/// the Overlord's seat taken by the wrong session for the mode.
pub fn run(frame: &mut Frame<'_>) -> Outcome {
    let name = slug(&require_text("name", &json!(frame.arg(0)))?);
    if name.is_empty() {
        return Err(CoordError::refuse(
            "thread name must contain letters or digits.",
        ));
    }
    check_seat(frame, &name)?;
    let sid = frame.env.session_id().map(str::to_owned);
    frame.mutate(false, |board, fr| apply(board, fr, &name, sid.as_deref()))?;
    if let Some(sid) = sid {
        fold::elsewhere(frame, &name, &sid)?;
    }
    Ok(())
}
