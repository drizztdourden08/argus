//! The texts the hooks hand a session: the join text of an adopted session, the placeholder
//! reminder, the thread line, the board pane line (with the shortcut sentence on a session
//! start), the wake line, and the new messages.

use argus_board::json::{shown, truthy};
use argus_core::js;
use serde_json::Value;

use super::ctx::Hook;
use super::visit::Hit;
use super::wake_line::wake_line;
use crate::client::server_port;
use crate::shortcut::{is_overlord, needs_shortcut};
use crate::who::git_root;

/// The new messages of a thread and what to do with them.
pub fn render(thread: &Value, messages: &[Value]) -> String {
    let mut lines = vec![format!(
        "Coordination board: {} new message(s) for thread \"{}\".",
        messages.len(),
        shown(thread.get("id"))
    )];
    lines.extend(messages.iter().map(|m| {
        format!(
            "- [{}] from {}: {}",
            shown(m.get("kind")),
            shown(m.get("from")),
            shown(m.get("text"))
        )
    }));
    lines.push("Act on them, then run `coord inbox --ack`.".to_owned());
    lines.join("\n")
}

/// The board url of a thread: `?thread=` is the presence claim the dashboard reads.
pub fn board_url(hook: &Hook<'_>, project: &str, thread: &str) -> String {
    format!(
        "http://127.0.0.1:{}/p/{project}/?thread={}",
        server_port(&hook.paths),
        js::encode_uri_component(thread)
    )
}

fn join_text(hit: &Hit, project: &str) -> String {
    let (name, id) = (&hit.name, shown(hit.thread.get("id")));
    format!(
        "Coordination board: this session is part of {name}, run by its lead session \"{name}\". It is on the board as a placeholder thread \"{id}\". Now: load the coord skill; once you know what this session works on, rename it \"{name} - <Subject>\" (set_session_title on \"self\", subject four words at most) and run coord thread register <short-name> --subject <subject> --session \"{name} - <Subject>\". If the work belongs to another project, register there with --project <key> instead of {project}."
    )
}

fn reminder(hit: &Hit) -> String {
    format!(
        "Coordination board: this session is still the unnamed placeholder \"{}\". Name and register it (coord skill, SKILL.md step 1).",
        shown(hit.thread.get("id"))
    )
}

fn thread_line(hit: &Hit) -> String {
    let subject = match hit.thread.get("subject") {
        None | Some(Value::Null) => "none".to_owned(),
        Some(v) => shown(Some(v)),
    };
    format!(
        "You are thread \"{}\" on the {} coordination board (subject {subject}). Load the coord skill before working; keep your note current with coord thread note.",
        shown(hit.thread.get("id")),
        hit.name
    )
}

/// The sentence that asks a thread to run `coord board shortcut`; the Overlord's makes two rows.
fn shortcut_line(project: &str, thread: &str) -> &'static str {
    if is_overlord(project, thread) {
        " This checkout's .claude/launch.json lacks the Overlord's shortcuts: run `coord board shortcut` once (idempotent); it creates two rows, \"Argus\" (the overview of every project) and \"Overlord\" (your own board), so the Browser pane's new tab offers both."
    } else {
        " This checkout's .claude/launch.json lists no shortcut to your board: run `coord board shortcut` once (idempotent), so the Browser pane's new tab offers it."
    }
}

fn pane_line(hook: &Hook<'_>, hit: &Hit, project: &str, start: bool) -> String {
    let id = shown(hit.thread.get("id"));
    let wants = start && needs_shortcut(&git_root(&hook.cwd), project, &id);
    let shortcut = if wants {
        shortcut_line(project, &id)
    } else {
        ""
    };
    format!(
        "Dashboard: open your board now in this thread's built-in browser pane and keep it open for the whole session: preview_start with url {} (no sign-in; the ?thread= part is how the dashboard knows your board is open). Then call mcp__ccd_view__show_pane with pane \"preview\": that is what makes the Browser pane DISPLAY beside your chat, so the owner sees the board (an open page in a hidden pane is invisible to him). Call it again whenever the owner writes to you; it does nothing, harmlessly, while your session is not open in a window. Never close that tab; browse other pages in a new tab. A registered thread whose board is not open for 30 seconds is reported to the Overlord, who resolves it with you at once.{shortcut}",
        board_url(hook, project, &id)
    )
}

/// The context of a visit. `start` adds the shortcut sentence and the wake line.
pub fn context(hook: &Hook<'_>, hit: &Hit, project: &str, start: bool) -> String {
    let placeholder = truthy(&hit.thread, "placeholder");
    let registered = !hit.adopted && !placeholder;
    let mut parts = vec![if hit.adopted {
        join_text(hit, project)
    } else if placeholder {
        reminder(hit)
    } else {
        thread_line(hit)
    }];
    if registered {
        parts.push(pane_line(hook, hit, project, start));
    }
    if start && registered {
        parts.push(wake_line(hook, hit, project));
    }
    if !hit.fresh.is_empty() {
        parts.push(render(&hit.thread, &hit.fresh));
    }
    parts.join("\n")
}
