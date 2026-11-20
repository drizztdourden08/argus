//! The sentences of a pending keep: the note and the wake to the Overlord, and the notes to the asking thread.

use crate::status::MAX_TEXT;

/// The reply to the thread that filed a pending keep.
pub(super) fn filed_reply(path: &str) -> String {
    format!(
        "pending keep filed: the hygiene holds {path} until the Overlord answers (at most 24 h)"
    )
}

/// The Overlord's inbox note: who asks, why, and the two answers.
pub(super) fn overlord_note(project: &str, id: &str, path: &str, asker: &str, why: &str) -> String {
    format!(
        "Argus: {asker} of {project} asks you to keep the worktree {id} at {path}.\nReason: {why}\nThe hygiene holds it for at most 24 hours while you have not answered; after that it judges the worktree again.\nConfirm: coord worktree keep {id} --project {project} --asked-by {asker} --note \"<why>\"\nDecline: coord worktree keep {id} --project {project} --clear"
    )
}

/// The Overlord's wake.
pub(super) fn overlord_wake(project: &str, id: &str, asker: &str) -> String {
    let text = format!(
        "{asker} of {project} asks to keep worktree {id}: confirm or decline (commands in your inbox)."
    );
    text.chars().take(MAX_TEXT).collect()
}

/// The asking thread's note when the Overlord answered.
pub(super) fn answer_note(path: &str, confirmed: bool) -> String {
    if confirmed {
        format!(
            "Argus: the Overlord confirmed your keep of {path}: it stays on the exception list."
        )
    } else {
        format!("Argus: the Overlord declined your keep of {path}: the hygiene judges it again.")
    }
}

/// The asking thread's note when its pending keep lapsed unanswered.
pub(super) fn lapsed_note(path: &str) -> String {
    format!(
        "Argus: your pending keep of {path} lapsed after 24 hours with no answer from the Overlord: the hygiene judges it again."
    )
}

/// The asking thread's note when a confirmed keep ended: it holds one more hour.
pub(super) fn ended_note(path: &str) -> String {
    format!(
        "Argus: the keep of {path} ended. It is removed in an hour unless it is kept again with a later --until (coord worktree keep <id> --until <date>)."
    )
}
