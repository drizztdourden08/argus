//! A portfolio send (the Overlord's decision or note) wakes the coordinator of every project it reached.

use super::ApplyCtx;
use crate::board::EntryView;
use crate::text::clip_full;
use crate::wake::WakeInput;

pub(super) fn from_portfolio(ctx: &ApplyCtx<'_>, entry: &EntryView) -> Vec<WakeInput> {
    let decision = entry.kind.is("decision");
    if !(decision || entry.kind.is("followup")) || entry.text.as_str().trim().is_empty() {
        return Vec::new();
    }
    let (reason, word) = if decision {
        ("portfolio-decision", "decision")
    } else {
        ("portfolio-note", "note")
    };
    let text = format!(
        "owner {word} from the Overlord: {}",
        clip_full(entry.rec.id.as_str(), entry.text.as_str())
    );
    entry
        .delivered_to
        .iter()
        .filter_map(|key| {
            let key = key.some()?;
            let thread = (ctx.coordinator_of)(key)?;
            Some(WakeInput::about(
                key,
                &thread,
                reason,
                entry.rec.id.as_str(),
                text.clone(),
            ))
        })
        .collect()
}
