//! The coordinator's side: `coord lock free <resource> [--to T]` and `rm <resource>`.

use argus_board::inbox::post;
use argus_board::json::doc;
use argus_board::store::remove_doc;

use super::{find_lock, hand_on, holder_of, on_lock, queue_of};
use crate::cli::Outcome;
use crate::verbs::work::crud::thread_arg;
use crate::verbs::work::run::Run;

/// `coord lock free`: frees a lock whoever holds it, optionally straight to a named thread.
pub(super) fn free(r: &mut Run<'_>) -> Outcome {
    on_lock(r, "free", |board, r, (who, resource, lock)| {
        let was = holder_of(lock);
        let next = match r.text_opt("to") {
            Some(name) => Some(thread_arg(board, name)?),
            None => None,
        };
        hand_on(board, resource, next.as_deref(), &who.actor);
        let now = holder_of(&find_lock(board, resource)?);
        if let Some(was) = was.as_ref().filter(|w| Some(*w) != now.as_ref()) {
            let text = format!(
                "{resource} was freed by {}; you no longer hold it.",
                who.actor
            );
            post(board, was, &who.actor, ("note", &text), Some(resource));
        }
        let from = was.map_or_else(String::new, |w| format!(" from {w}"));
        let after = now.map_or_else(
            || "nobody waiting".to_owned(),
            |h| format!("now held by {h}"),
        );
        r.say(&[&format!("{resource}: freed{from}, {after}.")]);
        Ok(())
    })
}

/// `coord lock rm`: deletes the lock itself; the holder and the queue are told.
pub(super) fn rm(r: &mut Run<'_>) -> Outcome {
    on_lock(r, "rm", |board, r, (who, resource, lock)| {
        let mut waiting: Vec<String> = holder_of(lock).into_iter().collect();
        waiting.extend(
            queue_of(lock)
                .iter()
                .filter_map(|id| id.as_str().map(str::to_owned)),
        );
        let text = format!("{resource} was removed by {}.", who.actor);
        for id in waiting {
            if doc(board, "threads", &id).is_some_and(|t| !t.is_null()) {
                post(board, &id, &who.actor, ("note", &text), Some(resource));
            }
        }
        remove_doc(board, "locks", resource);
        r.say(&[&format!("{resource}: removed.")]);
        Ok(())
    })
}
