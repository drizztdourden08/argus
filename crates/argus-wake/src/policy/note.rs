//! A followup or a decision of the owner: it answers an entry or is a note on an item or thread.

use super::ApplyCtx;
use crate::board::EntryView;
use crate::text::{clip, clip_full, one_line};
use crate::wake::WakeInput;

fn thread_on(ctx: &ApplyCtx<'_>, id: Option<&str>) -> Option<String> {
    let known = id.filter(|id| ctx.board.is_some_and(|b| b.threads.contains_key(*id)));
    match known {
        Some(id) => Some(id.to_owned()),
        None => (ctx.coordinator_of)(ctx.scope).or_else(|| id.map(str::to_owned)),
    }
}

fn worktree_line(entry: &EntryView) -> Option<String> {
    let w = entry.worktree.as_ref()?;
    let message = one_line(w.message.as_str());
    let tail = if message.is_empty() {
        String::new()
    } else {
        format!(": {}", clip(&message, 80))
    };
    let target = if w.path.some().is_some() {
        w.path.as_str()
    } else {
        w.id.as_str()
    };
    let verb = if w.kind.is("delete") {
        "delete"
    } else {
        "check"
    };
    let id = entry.rec.id.as_str();
    Some(format!(
        "owner asks to {verb} worktree {}{tail}. Answer: coord worktree done {id} --note \"...\"",
        clip(target, 90)
    ))
}

pub(super) fn from_note(ctx: &ApplyCtx<'_>, entry: &EntryView) -> Vec<WakeInput> {
    let asked = entry
        .reply_to
        .some()
        .and_then(|id| ctx.board?.entries.get(id));
    let about = entry
        .thread
        .some()
        .or_else(|| asked.and_then(|a| a.thread.some().or_else(|| a.author.some())));
    let Some(thread) = thread_on(ctx, about) else {
        return Vec::new();
    };
    if entry.text.as_str().trim().is_empty() {
        return Vec::new();
    }
    let decision = entry.kind.is("decision");
    if asked.is_none() && entry.kind.is("followup") {
        if let Some(line) = worktree_line(entry) {
            return vec![WakeInput::about(
                ctx.scope,
                &thread,
                "owner-note",
                entry.rec.id.as_str(),
                line,
            )];
        }
    }
    let text = clip_full(entry.rec.id.as_str(), entry.text.as_str());
    if let Some(asked) = asked {
        let verb = if decision { "decided on" } else { "answered" };
        let id = asked.rec.id.as_str();
        return vec![WakeInput::about(
            ctx.scope,
            &thread,
            "owner-answer",
            id,
            format!("owner {verb} {id}: {text}"),
        )];
    }
    let item = entry.rec.item.some();
    let on = item.map_or(String::new(), |i| format!(" on {i}"));
    let (reason, label) = if decision {
        ("owner-decision", "decision")
    } else {
        ("owner-note", "note")
    };
    let subject = item.unwrap_or(&thread);
    vec![WakeInput::about(
        ctx.scope,
        &thread,
        reason,
        subject,
        format!("owner {label}{on}: {text}"),
    )]
}
