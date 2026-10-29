//! A slow removal: `git worktree remove` gets its own long timeout, a timeout takes the same fallback as a folder git
//! could not delete, and an ignored, untracked `node_modules` is cleared by Argus before git runs. The timeout is the
//! scripted git's answer ("git timed out after 900 s"), so no test waits. The first case of the Node file (the writer's
//! two timeouts) is `git.rs`'s `worktree_remove_alone_gets_the_long_timeout...` and its sibling.

use super::fallback::{Shape, add_side, shape};
use crate::error::ErrorCode;
use crate::jobs::JobState;
use crate::testkit::{RemoveMode, exists_itself, listed};

const TIMED_OUT: &str = "git timed out after 900 s";

fn removal_after_timeout() -> (Shape, crate::request::RemoveOutcome) {
    let s = shape(RemoveMode::Timeout, |_| {});
    let out =
        s.h.rm_with(&s.id("old-merged"), |i| {
            (i.branch, i.reason) = (true, Some("merged long ago".into()));
        })
        .unwrap();
    (s, out)
}

#[test]
fn git_times_out_after_deleting_the_files_the_same_fallback_as_directory_not_empty_and_the_junction_targets_survive()
 {
    let (s, out) = removal_after_timeout();
    assert_eq!(
        (out.removed, out.finished_by_argus, out.git_error.as_deref()),
        (true, true, Some(TIMED_OUT))
    );
    assert!(
        out.text.starts_with("removed worktree ")
            && out
                .text
                .contains("old-merged (folder finished by Argus: git could not delete it), job L-")
            && out
                .text
                .contains(" deletes the files in the background, branch old-merged deleted"),
        "{}",
        out.text
    );
    assert!(
        !exists_itself(&s.fx.path("old-merged")),
        "the name is free at once"
    );
    assert!(
        !listed(&s.fx.repo).contains(&"old-merged".to_string()),
        "the stale record was pruned"
    );
    s.h.idle();
    assert!(!exists_itself(&out.folder.clone().unwrap()));
    s.keeps_survive();
    let job =
        s.h.meta()
            .jobs
            .into_iter()
            .find(|j| Some(&j.id) == out.job_id.as_ref())
            .unwrap();
    assert_eq!((job.state, job.error), (JobState::Done, None));
    let line = &s.h.notes.removals("deleted worktree folder")[0].text;
    assert!(
        line.contains(&format!(
            "finished by Argus: git could not delete it, it said: {TIMED_OUT};"
        )),
        "{line}"
    );
}

#[test]
fn git_times_out_and_still_lists_the_worktree_with_its_git_the_refusal_says_so_nothing_is_renamed()
{
    let s = shape(RemoveMode::TimeoutUntouched, add_side);
    let err = s.h.refused(&s.id("side"), |_| {});
    assert_eq!((err.code, err.code.status()), (ErrorCode::GitFailed, 500));
    assert!(
        err.message.starts_with(&format!(
            "git worktree remove failed: {TIMED_OUT}; nothing was deleted by Argus"
        )),
        "{}",
        err.message
    );
    assert!(exists_itself(&format!("{}/.git", s.fx.path("side"))));
    assert!(listed(&s.fx.repo).contains(&"side".to_string()));
    assert_eq!(s.h.svc.jobs("alpha").len(), 0);
    assert_eq!(s.made(), "{}");
}
