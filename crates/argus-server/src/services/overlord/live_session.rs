//! The live child's session id, kept across restarts of Argus (I-174). The id is saved when the
//! live child reports it, a start with no named session resumes it, and a rollback or a failed
//! resume clears it. The shadow child never touches it.

use argus_store::Db;

use super::live::Origin;

const KEY: &str = "overlord.live-session";

/// The saved id, if any.
pub(super) fn saved(db: &Db) -> Option<String> {
    db.kv_get(KEY)
        .ok()
        .flatten()
        .filter(|id| !id.trim().is_empty())
}

/// Keeps the id of the live child's session.
pub(super) fn save(db: &Db, id: &str) {
    let _ = db.kv_set(KEY, id);
}

/// Forgets the saved id, so the next start is a fresh session.
pub(super) fn clear(db: &Db) {
    let _ = db.kv_set(KEY, "");
}

/// The origin a live start uses: a fresh start resumes the saved session when there is one; a
/// resume or a fork is kept as given.
pub(super) fn origin_for(origin: Origin, saved: Option<&str>) -> Origin {
    match (origin, saved) {
        (Origin::Fresh, Some(id)) => Origin::Resume(id.to_owned()),
        (other, _) => other,
    }
}

#[cfg(test)]
mod specs {
    use super::*;

    // test-tag: critical
    #[test]
    fn a_fresh_start_resumes_the_saved_session() {
        let chosen = origin_for(Origin::Fresh, Some("s-1"));
        assert_eq!(chosen, Origin::Resume("s-1".to_owned()));
    }

    #[test]
    fn a_fresh_start_with_no_saved_session_stays_fresh() {
        assert_eq!(origin_for(Origin::Fresh, None), Origin::Fresh);
    }

    #[test]
    fn a_resume_and_a_fork_are_kept_as_given() {
        let resume = origin_for(Origin::Resume("a".to_owned()), Some("b"));
        assert_eq!(resume, Origin::Resume("a".to_owned()));
        let fork = origin_for(Origin::Fork("a".to_owned()), Some("b"));
        assert_eq!(fork, Origin::Fork("a".to_owned()));
    }
}
