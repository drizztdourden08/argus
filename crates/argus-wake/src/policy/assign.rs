//! A move of an item to another thread wakes the new thread; resuming a paused thread wakes it.

use super::ApplyCtx;
use crate::board::EntryView;
use crate::text::clip;
use crate::wake::WakeInput;

pub(super) fn from_resume(ctx: &ApplyCtx<'_>, entry: &EntryView) -> Vec<WakeInput> {
    let Some(id) = entry.reply_to.some() else {
        return Vec::new();
    };
    let was_paused = ctx
        .before
        .and_then(|b| b.threads.get(id))
        .is_some_and(|t| t.status.is("paused"));
    let still_paused = match ctx.board.and_then(|b| b.threads.get(id)) {
        Some(thread) => thread.status.is("paused"),
        None => true,
    };
    if !was_paused || still_paused {
        return Vec::new();
    }
    vec![WakeInput::about(
        ctx.scope,
        id,
        "resume",
        id,
        format!("{id} was resumed by the owner"),
    )]
}

fn applied(entry: &EntryView, index: usize) -> bool {
    if entry.kind.is("bulk") {
        entry
            .results
            .get(index)
            .is_some_and(|r| r.status.is("applied"))
    } else {
        true
    }
}

fn moved_items(entry: &EntryView) -> Vec<String> {
    let ops = if entry.kind.is("bulk") {
        entry.ops.clone()
    } else {
        vec![entry.as_op()]
    };
    let mut ids: Vec<String> = Vec::new();
    for (index, op) in ops.iter().enumerate() {
        let id = op.target.as_ref().map(|t| t.id.as_str().to_owned());
        if let Some(id) = id.filter(|_| op.moves_to_thread() && applied(entry, index)) {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    ids
}

pub(super) fn from_move(ctx: &ApplyCtx<'_>, entry: &EntryView) -> Vec<WakeInput> {
    let mut out = Vec::new();
    for id in moved_items(entry) {
        let Some(item) = ctx.board.and_then(|b| b.items.get(&id)) else {
            continue;
        };
        let Some(thread) = item.thread.some() else {
            continue;
        };
        let before = ctx.before.and_then(|b| b.items.get(&id));
        if before.is_some_and(|b| b.thread.some() == Some(thread)) {
            continue;
        }
        let text = format!(
            "{id} ({}) was assigned to you by the owner",
            clip(item.title.as_str(), 60)
        );
        out.push(WakeInput::about(
            ctx.scope,
            thread,
            "item-assigned",
            &id,
            text,
        ));
    }
    out
}
