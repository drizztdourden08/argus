//! KEEP: K3, K4.
//! When git cannot delete a worktree's folder (Windows: pnpm `node_modules` junctions, a handle held by a process) Argus
//! finishes the removal itself, and the leftovers a removal made are exempt from the age rule. `git worktree remove` is
//! simulated by a scripted git that deletes what git would have deleted and fails with git's own sentence. Every link
//! target must survive: a junction is removed as a link, never followed.

// test-tag: standard

use crate::batch::BatchInput;
use crate::board::Caller;
use crate::clock::DAY_MS;
use crate::doc::Status;
use crate::error::ErrorCode;
use crate::jobs::JobState;
use crate::request::RemoveOutcome;
use crate::testkit::{
    Fixture, Harness, RemoveMode, backdate, board_with, branches, exists_itself, listed, make_link,
    put, records, root_of, rule_for, unplugged,
};
use std::fs;
use std::sync::Arc;
use tempfile::{TempDir, tempdir};

pub(super) struct Shape {
    pub(super) fx: Fixture,
    pub(super) h: Harness,
    pub(super) outside: String,
    pub(super) outside2: String,
    _dirs: (TempDir, TempDir),
}

fn ensure_links(old_merged: &str, outside: &str, outside2: &str) {
    // node_modules is ignored by git, so Argus clears it before it calls git: the package folder is rebuilt here with the links.
    put(
        old_merged,
        "node_modules/.pnpm/pkg@1/node_modules/pkg/index.js",
        "module.exports = 1",
    );
    let nm = format!("{old_merged}/node_modules");
    for (target, link) in [
        (
            format!("{nm}/.pnpm/pkg@1/node_modules/pkg"),
            format!("{nm}/pkg"),
        ),
        (outside.to_string(), format!("{nm}/ext")),
        (outside2.to_string(), format!("{old_merged}/lnk-top")),
    ] {
        if !exists_itself(&link) {
            make_link(&link, &target);
        }
    }
}

pub(super) fn shape(mode: RemoveMode, prep: impl FnOnce(&Fixture)) -> Shape {
    shape_on(Fixture::fake(), mode, prep)
}

/// [`shape`] on a fixture the caller made (a real git one for the smoke in `remove_real.rs`).
pub(super) fn shape_on(fx: Fixture, mode: RemoveMode, prep: impl FnOnce(&Fixture)) -> Shape {
    fs::write(
        format!("{}/.git/info/exclude", fx.repo),
        "node_modules\nlnk-*\nlock\n",
    )
    .unwrap();
    let dirs = (tempdir().unwrap(), tempdir().unwrap());
    let (outside, outside2) = (root_of(&dirs.0), root_of(&dirs.1));
    put(&outside, "keep.txt", "precious one");
    put(&outside2, "keep.txt", "precious two");
    let merged = fx.path("old-merged");
    put(
        &merged,
        "node_modules/.pnpm/pkg@1/node_modules/pkg/index.js",
        "module.exports = 1",
    );
    ensure_links(&merged, &outside, &outside2);
    prep(&fx);
    let h = Harness::with_mode(&fx.root, board_with(vec![rule_for(&fx)]), mode, |o| {
        o.drop_links_first = true;
    });
    let (m, o1, o2) = (merged.clone(), outside.clone(), outside2.clone());
    *h.exec.after_failure.lock().unwrap() = Some(Box::new(move || ensure_links(&m, &o1, &o2)));
    h.scan();
    Shape {
        fx,
        h,
        outside,
        outside2,
        _dirs: dirs,
    }
}

pub(super) fn add_side(fx: &Fixture) {
    fx.add("side");
}

impl Shape {
    pub(super) fn id(&self, name: &str) -> String {
        self.h.id_of(&self.fx.path(name))
    }

    pub(super) fn keeps_survive(&self) {
        assert_eq!(
            fs::read_to_string(format!("{}/keep.txt", self.outside)).unwrap(),
            "precious one",
            "a junction deep in node_modules never followed"
        );
        assert_eq!(
            fs::read_to_string(format!("{}/keep.txt", self.outside2)).unwrap(),
            "precious two",
            "a junction at the top never followed"
        );
    }

    /// Removes `old-merged` with its branch and a reason: the removal every spec here starts from.
    pub(super) fn remove_old_merged(self) -> (Shape, RemoveOutcome) {
        let out = self
            .h
            .rm_with(&self.id("old-merged"), |i| {
                (i.branch, i.reason) = (true, Some("merged long ago".into()));
            })
            .unwrap();
        (self, out)
    }

    /// Removes `old-merged` with its branch, waits for the worker, and checks git no longer lists it.
    fn remove_old_merged_gone_from_git(&self) -> RemoveOutcome {
        let out = self
            .h
            .rm_with(&self.id("old-merged"), |i| i.branch = true)
            .unwrap();
        assert!(out.removed, "{}", out.text);
        self.h.idle();
        assert!(!listed(&self.fx.repo).contains(&"old-merged".to_string()));
        out
    }

    /// The background job of the removal finished clean.
    pub(super) fn assert_job_done(&self, out: &RemoveOutcome) {
        let job = self
            .h
            .meta()
            .jobs
            .into_iter()
            .find(|j| Some(&j.id) == out.job_id.as_ref())
            .unwrap();
        assert_eq!((job.state, job.error), (JobState::Done, None));
    }

    pub(super) fn made(&self) -> String {
        self.h
            .store
            .kv
            .lock()
            .unwrap()
            .get("worktrees:made:alpha")
            .cloned()
            .unwrap_or_else(|| "{}".to_string())
    }
}

fn stuck_removal() -> (Shape, RemoveOutcome) {
    let s = shape(RemoveMode::Stuck, |_| {});
    assert_eq!(s.h.docs()["old-merged"].status, Status::Merged);
    s.remove_old_merged()
}

#[test]
fn when_git_cannot_delete_the_folder_argus_renames_it_and_a_job_deletes_it() {
    let (_s, out) = stuck_removal();
    assert_eq!(
        (out.removed, out.finished_by_argus, out.pruned_only),
        (true, true, false)
    );
    assert!(out.job_id.as_deref().is_some_and(|j| j.starts_with("L-")));
    assert!(out.text.starts_with("removed worktree "), "{}", out.text);
    assert!(
        out.text
            .contains("old-merged (folder finished by Argus: git could not delete it), job L-"),
        "{}",
        out.text
    );
    assert!(
        out.text
            .contains(" deletes the files in the background, branch old-merged deleted"),
        "{}",
        out.text
    );
}

#[test]
fn the_answer_carries_gits_sentence_and_git_lets_go_of_the_worktree() {
    let (s, out) = stuck_removal();
    let said = out.git_error.clone().unwrap();
    assert!(
        said.contains("failed to delete") && said.contains("Directory not empty"),
        "{said}"
    );
    assert!(
        out.folder
            .clone()
            .unwrap()
            .contains("old-merged.argus-deleting-")
    );
    assert!(
        !exists_itself(&s.fx.path("old-merged")),
        "the name is free at once"
    );
    assert!(
        !listed(&s.fx.repo).contains(&"old-merged".to_string()),
        "git lets go of it (the stale record was pruned)"
    );
    assert!(
        !branches(&s.fx.repo).contains(&"old-merged".to_string()),
        "the merged branch went"
    );
}

#[test]
fn the_job_deletes_the_folder_and_every_junction_target_survives() {
    let (s, out) = stuck_removal();
    s.h.idle();
    assert!(
        !exists_itself(&out.folder.clone().unwrap()),
        "the renamed folder is gone"
    );
    s.keeps_survive();
    assert!(!s.h.docs().contains_key("old-merged"), "the row left");
    s.assert_job_done(&out);
}

#[test]
fn the_activity_lines_say_who_finished_the_folder() {
    let (s, _out) = stuck_removal();
    s.h.idle();
    let first = &s.h.notes.removals("removed worktree")[0];
    assert_eq!(
        (first.level, first.project.as_str(), first.thread.as_deref()),
        ("info", "alpha", Some("worker"))
    );
    assert!(
        first
            .text
            .contains("(folder finished by Argus: git could not delete it)"),
        "{}",
        first.text
    );
    assert!(
        first.text.ends_with(" for worker: merged long ago"),
        "{}",
        first.text
    );
    let last = &s.h.notes.removals("deleted worktree folder")[0];
    assert!(
        last.text.starts_with("alpha: deleted worktree folder "),
        "{}",
        last.text
    );
    assert!(
        last.text.contains(
            "(finished by Argus: git could not delete it, it said: Directory not empty; "
        ),
        "{}",
        last.text
    );
    assert!(last.text.ends_with(" s) for worker"), "{}", last.text);
}

#[test]
fn the_links_inside_the_worktree_are_removed_as_links_before_git_is_called() {
    let s = shape(RemoveMode::Stuck, |_| {});
    let seen = Arc::new(std::sync::Mutex::new(Vec::new()));
    let recorder = seen.clone();
    *s.h.exec.on_remove.lock().unwrap() = Some(Box::new(move |p: &str| {
        let present = |rel: &str| exists_itself(&format!("{p}/{rel}"));
        recorder.lock().unwrap().push([
            present("lnk-top"),
            present("node_modules/ext"),
            present("node_modules/pkg"),
            present("node_modules/.pnpm"),
        ]);
    }));
    s.h.rm(&s.id("old-merged")).unwrap();
    assert_eq!(
        *seen.lock().unwrap(),
        vec![[false, false, false, false]],
        "links gone (node_modules, ignored by git, cleared by Argus first) when git runs"
    );
    assert!(exists_itself(&format!("{}/keep.txt", s.outside)));
    s.h.idle();
}

#[test]
fn a_link_that_cannot_be_removed_stops_the_removal_before_git_is_called() {
    let s = shape(RemoveMode::Stuck, |_| {});
    *s.h.fs.fail_unlink.lock().unwrap() = Some(std::io::ErrorKind::ResourceBusy);
    let err = s.h.refused(&s.id("old-merged"), |_| {});
    *s.h.fs.fail_unlink.lock().unwrap() = None;
    assert_eq!(err.code, ErrorCode::GitFailed);
    assert!(err.message.starts_with("could not remove the junctions or symlinks inside ") && err.message.contains("old-merged first (EBUSY): git for Windows would follow them and delete what they point to, so nothing was removed"), "{}", err.message);
    let called = s.h.exec.calls.lock().unwrap().iter().any(|a| {
        a.first().map(String::as_str) == Some("worktree")
            && a.get(1).map(String::as_str) == Some("remove")
    });
    assert!(!called, "git was never called");
    s.keeps_survive();
}

#[test]
fn the_owners_page_finishes_a_removal_the_same_way() {
    let s = shape(RemoveMode::Stuck, add_side);
    let out =
        s.h.rm_with(&s.id("side"), |i| {
            (i.caller, i.branch) = (Caller::Page, true);
        })
        .unwrap();
    assert!(out.finished_by_argus);
    assert!(
        out.text.starts_with("removed worktree ")
            && out
                .text
                .contains("side (folder finished by Argus: git could not delete it)")
    );
    s.h.idle();
    assert!(!exists_itself(&s.fx.path("side")));
    assert!(
        s.h.notes.removals("deleted worktree folder")[0]
            .text
            .ends_with("for the owner (page)")
    );
}

#[test]
fn git_still_listing_the_worktree_with_its_git_keeps_the_refusal_nothing_is_renamed_or_deleted() {
    let s = shape(RemoveMode::Untouched, |_| {});
    let err = s.h.refused(&s.id("old-merged"), |_| {});
    assert_eq!((err.code, err.code.status()), (ErrorCode::GitFailed, 500));
    assert!(
        err.message
            .starts_with("git worktree remove failed: error: failed to delete")
            && err.message.contains("Permission denied"),
        "{}",
        err.message
    );
    assert!(exists_itself(&format!("{}/.git", s.fx.path("old-merged"))));
    assert!(listed(&s.fx.repo).contains(&"old-merged".to_string()));
    assert_eq!(s.h.svc.jobs("alpha").len(), 0);
    assert_eq!(
        s.h.notes.removals("refused to remove worktree")[0].level,
        "error"
    );
    assert_eq!(s.made(), "{}", "nothing was left by a removal");
}

#[test]
fn another_prunable_record_no_rule_vetted_an_unplugged_drive_stops_the_prune_that_finishes_the_folder()
 {
    let s = shape(RemoveMode::Stuck, unplugged(false));
    let err = s.h.refused(&s.id("old-merged"), |_| {});
    assert_eq!(err.code, ErrorCode::GitFailed);
    assert!(
        err.message.contains("Argus did not prune git's records: ")
            && err
                .message
                .contains("would also forget 1 other worktree record")
            && err.message.contains("drive-x"),
        "{}",
        err.message
    );
    assert!(
        records(&s.fx).contains(&"drive-x".to_string()),
        "the unvetted record is still there"
    );
    assert_eq!(s.h.svc.jobs("alpha").len(), 0);
}

#[test]
fn an_error_that_is_not_a_folder_deletion_one_is_a_git_failure_as_before_whatever_state_the_folder_is_in()
 {
    let s = shape(RemoveMode::Other, |_| {});
    let err = s.h.refused(&s.id("old-merged"), |_| {});
    assert_eq!(err.code, ErrorCode::GitFailed);
    assert!(err.message.contains("fatal: something else went wrong"));
    assert_eq!(s.h.svc.jobs("alpha").len(), 0);
    assert!(exists_itself(&s.fx.path("old-merged")), "no rename");
    assert_eq!(s.made(), "{}");
}

const RENAME_REFUSED: &str = "and Argus could not rename it for deletion either (EBUSY): something has it open or locked. It is a leftover folder now: close what holds it and run coord worktree remove on it again (a leftover this removal made is taken without the age limit). Branch old-merged was deleted.";

fn refused_rename() -> (Shape, crate::error::WtError) {
    let s = shape(RemoveMode::Stuck, |_| {});
    *s.h.fs.rename_error.lock().unwrap() = Some(std::io::ErrorKind::ResourceBusy);
    let err = s.h.refused(&s.id("old-merged"), |i| i.branch = true);
    *s.h.fs.rename_error.lock().unwrap() = None;
    (s, err)
}

#[test]
fn a_refused_rename_is_a_423_with_a_plain_sentence_and_the_branch_went_first() {
    let (s, err) = refused_rename();
    assert_eq!((err.code, err.code.status()), (ErrorCode::InUse, 423));
    assert!(
        err.message
            .starts_with("git removed the worktree record of "),
        "{}",
        err.message
    );
    assert!(
        err.message
            .contains("old-merged but could not delete its folder (error: failed to delete "),
        "{}",
        err.message
    );
    assert!(err.message.ends_with(RENAME_REFUSED), "{}", err.message);
    assert!(
        !branches(&s.fx.repo).contains(&"old-merged".to_string()),
        "the branch went before the rename was tried"
    );
    assert!(exists_itself(&s.fx.path("old-merged")));
    assert!(s.made().contains("old-merged"));
}

#[test]
fn the_folder_a_refused_rename_left_is_a_made_leftover_though_it_is_young() {
    let (s, _err) = refused_rename();
    s.h.scan();
    let row = s.h.docs()["old-merged"].clone();
    let leftover = row.leftover.as_ref().unwrap();
    assert_eq!(row.status, Status::Leftover);
    assert!(
        leftover.made && leftover.age_days.unwrap() < 1,
        "young: it was just written"
    );
    assert!(
        s.h.meta().leftover_old.count >= 1,
        "counted with the old ones, it can go"
    );
    let plain = s.h.refused(&s.id("plain"), |_| {});
    assert!(
        plain.message.contains("less than 1 day ago"),
        "a young leftover nobody made is still refused"
    );
}

#[test]
fn the_next_removal_takes_the_made_leftover_and_the_memory_forgets_it_once_it_is_gone() {
    let (s, _err) = refused_rename();
    s.h.scan();
    let out = s.h.rm(&s.id("old-merged")).unwrap();
    assert!(out.started && out.leftover);
    s.h.idle();
    assert!(!exists_itself(&s.fx.path("old-merged")));
    s.keeps_survive();
    assert_eq!(
        s.h.svc.prune_made("alpha"),
        1,
        "the folder is gone: the memory forgets it"
    );
    assert_eq!(s.made(), "{}");
}

#[test]
fn all_leftovers_by_the_lead_takes_the_removal_made_one_and_skips_the_young_one_nobody_made() {
    let s = shape(RemoveMode::Stuck, |_| {});
    *s.h.fs.rename_error.lock().unwrap() = Some(std::io::ErrorKind::PermissionDenied);
    s.h.refused(&s.id("old-merged"), |_| {});
    *s.h.fs.rename_error.lock().unwrap() = None;
    s.h.scan();
    let input = BatchInput {
        reason: Some("cleanup after the merge".into()),
        ..BatchInput::new("alpha", Caller::Thread("lead".into()))
    };
    let out = s.h.svc.remove_leftovers(&input).unwrap();
    assert_eq!(out.started, 1);
    let skipped: Vec<String> = out
        .skipped
        .iter()
        .map(|k| k.path.clone().unwrap())
        .collect();
    assert_eq!(skipped, [s.fx.path("plain")]);
    assert!(
        out.skipped[0].reason.starts_with("changed ")
            && out.skipped[0].reason.ends_with(", under 1 day"),
        "{}",
        out.skipped[0].reason
    );
    s.h.idle();
    assert!(!exists_itself(&s.fx.path("old-merged")));
    assert!(exists_itself(&s.fx.path("plain")));
}

#[test]
fn git_says_removed_but_the_folder_is_still_there_it_is_remembered_and_taken_without_the_age_limit()
{
    let s = shape(RemoveMode::Leaves, add_side);
    let out = s.h.rm(&s.id("side")).unwrap();
    assert_eq!(
        (out.removed, out.folder_left, out.finished_by_argus),
        (true, true, false)
    );
    assert!(out.text.starts_with("removed worktree ") && out.text.contains("side, git left the folder behind: it is a leftover now, which coord worktree remove --all-leftovers takes without the age limit"), "{}", out.text);
    assert!(exists_itself(&s.fx.path("side")));
    s.h.scan();
    let row = s.h.docs()["side"].clone();
    assert_eq!(
        (row.status, row.leftover.unwrap().made),
        (Status::Leftover, true)
    );
    let input = BatchInput::new("alpha", Caller::Thread("worker".into()));
    assert_eq!(s.h.svc.remove_leftovers(&input).unwrap().started, 1);
    s.h.idle();
    assert!(!exists_itself(&s.fx.path("side")));
}

#[test]
fn the_memory_keeps_14_days_and_only_folders_still_on_disk_an_old_entry_no_longer_exempts() {
    let s = shape(RemoveMode::Stuck, |_| {});
    let path = s.fx.path("plain");
    let key = if cfg!(windows) {
        path.to_lowercase()
    } else {
        path.clone()
    };
    let now = s.h.clock_now();
    let set = |text: String| {
        s.h.store
            .kv
            .lock()
            .unwrap()
            .insert("worktrees:made:alpha".to_string(), text)
    };
    set(format!(
        "{{\"{key}\":{{\"at\":{},\"path\":\"{path}\"}},\"x:/gone/folder\":{{\"at\":{now},\"path\":\"X:/gone/folder-that-does-not-exist\"}},\"junk\":\"not an entry\"}}",
        now - 15 * DAY_MS
    ));
    assert!(!s.h.svc.is_made("alpha", &path), "older than 14 days");
    assert!(
        s.h.refused(&s.id("plain"), |_| {})
            .message
            .contains("less than 1 day ago")
    );
    assert_eq!(
        s.h.svc.prune_made("alpha"),
        3,
        "the old one, the one whose folder is gone and the junk entry"
    );
    assert_eq!(s.made(), "{}");
    set(format!(
        "{{\"{key}\":{{\"at\":{},\"path\":\"{path}\"}}}}",
        now - 13 * DAY_MS
    ));
    assert!(s.h.svc.is_made("alpha", &path));
    assert_eq!(
        s.h.svc.prune_made("alpha"),
        0,
        "a folder that exists, 13 days old, stays"
    );
    assert!(s.h.rm(&s.id("plain")).unwrap().started, "exempt");
    s.h.idle();
    set("not json".to_string());
    assert!(
        !s.h.svc.is_made("alpha", &path),
        "a damaged record exempts nothing"
    );
}

fn ages(fx: &Fixture) {
    for (name, days) in [
        ("age-12h", 0.5),
        ("age-3h", 0.125),
        ("age-2d", 2.0),
        ("age-9d", 9.0),
    ] {
        put(&fx.wts, &format!("{name}/a.txt"), "x\n");
        backdate(&fx.path(name), days);
    }
}

fn thread_batch(
    h: &Harness,
    change: impl FnOnce(&mut BatchInput),
) -> crate::error::Result<crate::batch::BatchOutcome> {
    h.batch(Caller::Thread("worker".into()), change)
}

fn job_names(out: &crate::batch::BatchOutcome) -> Vec<String> {
    let mut names: Vec<String> = out.jobs.iter().map(|j| j.name.clone()).collect();
    names.sort();
    names
}

fn why(out: &crate::batch::BatchOutcome, name: &str) -> String {
    out.skipped
        .iter()
        .find(|s| {
            s.path
                .as_deref()
                .is_some_and(|p| p.ends_with(&format!("/{name}")))
        })
        .unwrap()
        .reason
        .clone()
}

#[test]
fn the_server_default_is_one_day_for_a_thread_with_nothing_else_to_say() {
    let s = shape(RemoveMode::Real, ages);
    assert_eq!(s.h.svc.leftover_age_days(), 1.0);
    assert_eq!(s.h.meta().leftover_old.days, 1.0);
    let out = thread_batch(&s.h, |_| {}).unwrap();
    assert_eq!(out.older_than_days, 1.0);
    assert_eq!(job_names(&out), ["age-2d", "age-9d"]);
    assert_eq!(why(&out, "age-12h"), "changed 12 hours ago, under 1 day");
    assert_eq!(why(&out, "age-3h"), "changed 3 hours ago, under 1 day");
    s.h.idle();
}

#[test]
fn older_than_below_the_default_needs_a_reason_and_not_below_6_hours_each_refusal_a_plain_sentence_and_a_line()
 {
    let s = shape(RemoveMode::Real, ages);
    let no_reason = thread_batch(&s.h, |i| i.older_than_days = Some(0.25)).unwrap_err();
    assert_eq!(
        (no_reason.code, no_reason.code.status()),
        (ErrorCode::BadRequest, 400)
    );
    assert!(no_reason.message.starts_with("--older-than 6 hours is below the server's limit of 1 day: say why it is safe with --reason \"<why>\""), "{}", no_reason.message);
    let too_low = thread_batch(&s.h, |i| {
        (i.older_than_days, i.reason) = (Some(0.1), Some("finished agents".into()));
    })
    .unwrap_err();
    assert_eq!(
        too_low.message,
        "a thread deletes a leftover untouched for 6 hours at the least (--older-than 6h), not 2 hours; the owner's page deletes younger ones"
    );
    let blank = thread_batch(&s.h, |i| {
        (i.older_than_days, i.reason) = (Some(0.5), Some("   ".into()));
    })
    .unwrap_err();
    assert_eq!(
        blank.code,
        ErrorCode::BadRequest,
        "a blank reason is no reason"
    );
    assert_eq!(
        s.h.notes
            .removals("refused to delete leftover folders for worker: --older-than")
            .len(),
        2
    );
    assert_eq!(
        s.h.notes
            .removals("refused to delete leftover folders for worker")[0]
            .level,
        "warn"
    );
    assert!(
        exists_itself(&s.fx.path("age-12h")) && exists_itself(&s.fx.path("age-3h")),
        "nothing was deleted"
    );
}

#[test]
fn with_a_reason_a_thread_goes_down_to_6_hours_the_reason_is_in_the_activity_line() {
    let s = shape(RemoveMode::Real, ages);
    let reason = "the finished agent worktrees of this project, nothing runs in them";
    let out = thread_batch(&s.h, |i| {
        (i.older_than_days, i.reason) = (Some(0.25), Some(reason.into()));
    })
    .unwrap();
    assert_eq!(out.older_than_days, 0.25);
    assert_eq!(job_names(&out), ["age-12h", "age-2d", "age-9d"]);
    assert_eq!(why(&out, "age-3h"), "changed 3 hours ago, under 6 hours");
    s.h.idle();
    assert!(!exists_itself(&s.fx.path("age-12h")) && exists_itself(&s.fx.path("age-3h")));
    let line = &s.h.notes.removals("queued 3 leftover folders")[0];
    assert!(
        line.text
            .contains(&format!(" for worker: {reason}. Skipped: ")),
        "{}",
        line.text
    );
    assert_eq!(
        s.h.notes
            .removals("leftover batch finished: 3 deleted or started, 0 refused for worker")
            .len(),
        1
    );
}

#[test]
fn older_than_above_the_default_needs_no_reason_and_holds_the_single_removal_takes_the_same_rule() {
    let s = shape(RemoveMode::Real, ages);
    let out = thread_batch(&s.h, |i| i.older_than_days = Some(3.0)).unwrap();
    assert_eq!(job_names(&out), ["age-9d"]);
    assert_eq!(why(&out, "age-2d"), "changed 2 days ago, under 3 days");
    s.h.idle();
    let no_reason =
        s.h.refused(&s.id("age-12h"), |i| i.older_than_days = Some(0.25));
    assert_eq!(
        (no_reason.code, no_reason.code.status()),
        (ErrorCode::BadRequest, 400)
    );
    let young = s.h.refused(&s.id("age-12h"), |_| {});
    assert_eq!(young.code, ErrorCode::Forbidden);
    assert!(
        young
            .message
            .contains("changed 12 hours ago, less than 1 day ago"),
        "{}",
        young.message
    );
    let ok =
        s.h.rm_with(&s.id("age-12h"), |i| {
            (i.older_than_days, i.reason) = (Some(0.25), Some("a finished agent".into()));
        })
        .unwrap();
    assert!(ok.started);
    s.h.idle();
    assert!(!exists_itself(&s.fx.path("age-12h")) && exists_itself(&s.fx.path("age-3h")));
    let line =
        s.h.notes
            .removals("deleting leftover folder")
            .into_iter()
            .find(|l| l.text.contains("age-12h"))
            .unwrap();
    assert!(line.text.ends_with("for worker: a finished agent"));
}

#[test]
fn the_page_keeps_its_own_stronger_confirm_whatever_age_it_asks_for_and_the_server_limit_is_configurable()
 {
    let s = shape(RemoveMode::Real, ages);
    let e = s.h.refused(&s.id("age-3h"), |i| i.caller = Caller::Page);
    assert!(
        e.message
            .contains("confirm that you want a young folder deleted")
    );
    let page = thread_batch(&s.h, |i| {
        (i.caller, i.older_than_days, i.ids) =
            (Caller::Page, Some(0.0), Some(vec![s.id("age-3h")]));
    })
    .unwrap();
    assert_eq!(page.started, 1);
    s.h.idle();
    assert!(
        exists_itself(&s.fx.path("age-3h")),
        "refused at its turn: the batch does not carry the confirm"
    );
    let c = Harness::with_mode(
        &s.fx.root,
        board_with(vec![rule_for(&s.fx)]),
        RemoveMode::Real,
        |o| o.leftover_age_days = 3.0,
    );
    c.scan();
    assert_eq!(c.meta().leftover_old.days, 3.0);
    let out = thread_batch(&c, |_| {}).unwrap();
    assert_eq!(job_names(&out), ["age-9d"]);
    c.idle();
}

fn lock_old_merged(reason: &'static str) -> impl FnOnce(&Fixture) {
    move |fx| {
        fx.git(&[
            "worktree",
            "lock",
            "--reason",
            reason,
            &fx.path("old-merged"),
        ]);
    }
}

fn unlock_calls(s: &Shape) -> usize {
    s.h.exec
        .calls
        .lock()
        .unwrap()
        .iter()
        .filter(|c| c.starts_with(&["worktree".to_string(), "unlock".to_string()]))
        .count()
}

#[test]
fn a_brock_locked_clone_with_junctions_is_unlinked_unlocked_and_removed_and_no_target_is_touched() {
    let s = shape(
        RemoveMode::Real,
        lock_old_merged("cloned node_modules from main (brock)"),
    );
    let _ = s.remove_old_merged_gone_from_git();
    assert!(
        !exists_itself(&s.fx.path("old-merged")),
        "the folder is gone"
    );
    s.keeps_survive();
    assert_eq!(unlock_calls(&s), 1);
}

#[test]
fn a_worktree_locked_for_another_reason_is_refused_and_stays_with_its_links() {
    let s = shape(RemoveMode::Real, lock_old_merged("kept for the demo"));
    let err = s.h.refused(&s.id("old-merged"), |_| {});
    assert_eq!(err.code, ErrorCode::Forbidden);
    assert!(err.message.contains("is locked (kept for the demo)"));
    assert!(listed(&s.fx.repo).contains(&"old-merged".to_string()));
    assert!(exists_itself(&s.fx.path("old-merged")));
    assert_eq!(unlock_calls(&s), 0);
    s.keeps_survive();
}
