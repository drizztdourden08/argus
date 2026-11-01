//! KEEP: K3.
//! A slow removal: `git worktree remove` gets its own long timeout, a timeout takes the same fallback as a folder git
//! could not delete, and an ignored, untracked `node_modules` is cleared by Argus before git runs. The timeout is the
//! scripted git's answer ("git timed out after 900 s"), so no test waits. The first case of the Node file (the writer's
//! two timeouts) is `git.rs`'s `worktree_remove_alone_gets_the_long_timeout...` and its sibling.

// test-tag: e2e

use super::fallback::{Shape, add_side, shape};
use crate::error::ErrorCode;
use crate::request::RemoveOutcome;
use crate::testkit::{RemoveMode, exists_itself, git_in, listed, put};
use std::fs;
use std::sync::{Arc, Mutex};

const TIMED_OUT: &str = "git timed out after 900 s";

fn removal_after_timeout() -> (Shape, RemoveOutcome) {
    let s = shape(RemoveMode::Timeout, |_| {});
    s.remove_old_merged()
}

#[test]
#[ignore = "e2e"]
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
    s.assert_job_done(&out);
    let line = &s.h.notes.removals("deleted worktree folder")[0].text;
    assert!(
        line.contains(&format!(
            "finished by Argus: git could not delete it, it said: {TIMED_OUT};"
        )),
        "{line}"
    );
}

#[test]
#[ignore = "e2e"]
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

fn modules_of(path: &str) -> String {
    format!("{path}/node_modules")
}

#[test]
#[ignore = "e2e"]
fn a_timeout_after_node_modules_was_cleared_says_so_in_the_refusal() {
    let s = shape(RemoveMode::TimeoutUntouched, |_| {});
    let err = s.h.refused(&s.id("old-merged"), |_| {});
    assert_eq!(err.code, ErrorCode::GitFailed);
    assert!(
        err.message.contains(&format!(
            "{TIMED_OUT}; Argus deleted nothing but the ignored node_modules"
        )),
        "{}",
        err.message
    );
    assert!(!exists_itself(&modules_of(&s.fx.path("old-merged"))));
    s.keeps_survive();
}

#[test]
#[ignore = "e2e"]
fn node_modules_is_ignored_by_git_argus_clears_it_before_git_runs_and_the_answer_and_the_activity_line_say_so()
 {
    let s = shape(RemoveMode::Stuck, |_| {});
    let seen = Arc::new(Mutex::new(Vec::new()));
    let recorder = seen.clone();
    *s.h.exec.on_remove.lock().unwrap() = Some(Box::new(move |p: &str| {
        recorder.lock().unwrap().push((
            exists_itself(&modules_of(p)),
            exists_itself(&format!("{p}/a.txt")),
        ));
    }));
    let out = s.h.rm(&s.id("old-merged")).unwrap();
    assert_eq!(
        *seen.lock().unwrap(),
        [(false, true)],
        "when git ran, node_modules was gone and the tracked files were still there"
    );
    assert!(out.node_modules_cleared);
    assert!(
        out.text.ends_with("node_modules cleared by Argus first"),
        "{}",
        out.text
    );
    assert_eq!(
        serde_json::to_value(&out).unwrap()["nodeModulesCleared"],
        true
    );
    let line = &s.h.notes.removals("removed worktree")[0].text;
    assert!(
        line.starts_with("alpha: removed worktree ")
            && line.ends_with("node_modules cleared by Argus first for worker"),
        "{line}"
    );
    s.h.idle();
    s.keeps_survive();
}

/// Whether `file` (inside the worktree's `node_modules`) is there each time git is asked to remove the worktree.
fn recorded_modules(s: &Shape, name: &str, file: &str) -> Arc<Mutex<Vec<bool>>> {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let recorder = seen.clone();
    let file = format!("{}/{file}", modules_of(&s.fx.path(name)));
    *s.h.exec.on_remove.lock().unwrap() = Some(Box::new(move |_: &str| {
        recorder.lock().unwrap().push(exists_itself(&file));
    }));
    seen
}

#[test]
#[ignore = "e2e"]
fn node_modules_that_git_does_not_ignore_is_left_to_git_and_says_nothing_about_clearing() {
    let s = shape(RemoveMode::Stuck, |fx| {
        add_side(fx);
        let side = fx.path("side");
        put(&side, ".gitignore", "!node_modules\n");
        git_in(&side, &["add", ".gitignore"], &[]);
        git_in(&side, &["commit", "-q", "-m", "unignore node_modules"], &[]);
        fs::create_dir_all(modules_of(&side)).unwrap(); // empty: git calls the worktree clean
    });
    assert_eq!(
        git_in(&s.fx.path("side"), &["status", "--porcelain"], &[]).trim(),
        ""
    );
    let seen = recorded_modules(&s, "side", "");
    let out = s.h.rm(&s.id("side")).unwrap();
    assert_eq!(
        *seen.lock().unwrap(),
        [true],
        "node_modules was still there when git ran"
    );
    assert!(!out.node_modules_cleared);
    assert!(!out.text.contains("node_modules cleared"));
    assert!(
        serde_json::to_value(&out)
            .unwrap()
            .get("nodeModulesCleared")
            .is_none()
    );
    s.h.idle();
}

#[test]
#[ignore = "e2e"]
fn node_modules_with_a_tracked_file_in_it_is_left_to_git_even_when_a_pattern_ignores_the_name() {
    let s = shape(RemoveMode::Stuck, |fx| {
        add_side(fx);
        let side = fx.path("side");
        put(&side, "node_modules/t.js", "tracked\n");
        git_in(&side, &["add", "-f", "node_modules/t.js"], &[]);
        git_in(
            &side,
            &["commit", "-q", "-m", "track a file in node_modules"],
            &[],
        );
    });
    let seen = recorded_modules(&s, "side", "t.js");
    let out = s.h.rm(&s.id("side")).unwrap();
    assert_eq!(*seen.lock().unwrap(), [true]);
    assert!(!out.node_modules_cleared);
    s.h.idle();
}

#[test]
#[ignore = "e2e"]
fn the_guards_run_first_a_worktree_with_uncommitted_work_is_refused_before_node_modules_is_touched()
{
    let s = shape(RemoveMode::Stuck, |fx| {
        put(&fx.path("dirty-old"), "node_modules/x.js", "regenerable\n");
    });
    let err = s.h.refused(&s.id("dirty-old"), |_| {});
    assert_eq!(err.code, ErrorCode::Forbidden);
    assert!(
        err.message.contains("uncommitted change"),
        "{}",
        err.message
    );
    assert!(exists_itself(&format!(
        "{}/x.js",
        modules_of(&s.fx.path("dirty-old"))
    )));
    let asked = s.h.exec.calls.lock().unwrap().iter().any(|a| {
        a.first().map(String::as_str) == Some("check-ignore")
            || a.starts_with(&["worktree".to_string(), "remove".to_string()])
    });
    assert!(!asked, "git was not asked to remove or check anything");
}
