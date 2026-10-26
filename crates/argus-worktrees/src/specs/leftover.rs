//! Leftover folders: finding them in a scan, every guard of their deletion, the rename-then-delete job, the batch, the
//! folders a crash left half deleted, and the wake sentence that counts git worktrees and leftovers apart.

use crate::batch::BatchInput;
use crate::board::Caller;
use crate::clock::DAY_MS;
use crate::deleting::is_deleting;
use crate::doc::{Status, WorktreeDoc};
use crate::error::ErrorCode;
use crate::fact::Fact;
use crate::fs::RealFs;
use crate::git::Git;
use crate::jobs::JobState;
use crate::paths::{base_name, path_key};
use crate::ports::ProcInfo;
use crate::rule::Rule;
use crate::scan::{ScanEnv, scan_rules};
use crate::service::Options;
use crate::status::{many_text, wake_text};
use crate::testkit::{
    Fixture, Gate, Harness, NotRepo, backdate, exists_itself, git_in, make_link, put, real_measure,
    root_of, rule_for,
};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tempfile::{TempDir, tempdir};

const MB: usize = 1024 * 1024;

fn facts_by_name(facts: &[Fact]) -> HashMap<String, &Fact> {
    facts.iter().map(|f| (base_name(&f.path), f)).collect()
}

fn scan_no_repo(wts: &str, ci: bool) -> crate::scan::ScanResult {
    let git = Git::new(Arc::new(NotRepo));
    let env = ScanEnv {
        git: &git,
        fs: &RealFs,
        case_insensitive: ci,
        measure: &real_measure,
    };
    scan_rules(&[Rule::folder("W-1", wts, "lead")], &env)
}

#[test]
fn with_no_repository_only_the_git_check_decides_a_link_is_a_leaf() {
    let (dir, outside) = (tempdir().unwrap(), tempdir().unwrap());
    let (root, out_root) = (root_of(&dir), root_of(&outside));
    let wts = format!("{root}/proj/.claude/worktrees");
    put(&wts, "plain/a.txt", "x\n");
    put(&wts, "has-git-file/.git", "gitdir: X:/nowhere\n");
    fs::create_dir_all(format!("{wts}/has-git-dir/.git")).unwrap();
    put(&wts, "nested/inner/.git/HEAD", "x");
    put(&out_root, "keep.txt", "x");
    make_link(&format!("{wts}/link"), &out_root);
    put(&wts, "half.argus-deleting-1730000000000/x.txt", "x");
    let out = scan_no_repo(&wts, false);
    let f = facts_by_name(&out.facts);
    let mut names: Vec<&String> = f.keys().collect();
    names.sort();
    assert_eq!(
        names,
        ["has-git-dir", "has-git-file", "link", "nested", "plain"],
        "the half-deleted folder is not a candidate"
    );
    let plain = f["plain"].leftover.as_ref().unwrap();
    assert_eq!(
        (plain.files, plain.has_nested_git, f["plain"].is_git),
        (Some(1), false, false)
    );
    assert!(f["has-git-file"].leftover.is_none() && f["has-git-dir"].leftover.is_none());
    let nested = f["nested"].leftover.as_ref().unwrap();
    assert_eq!(
        (nested.has_nested_git, nested.nested_git.as_deref()),
        (true, Some("inner/.git"))
    );
    assert!(f["link"].leftover.as_ref().unwrap().is_link);
    let deleting: Vec<String> = out.deleting.iter().map(|d| base_name(&d.path)).collect();
    assert_eq!(deleting, ["half.argus-deleting-1730000000000"]);
    assert!(exists_itself(&format!("{out_root}/keep.txt")));
}

#[test]
fn with_a_repository_a_folder_git_still_lists_stays_not_git_one_it_does_not_list_is_a_leftover() {
    let fx = Fixture::new();
    let orphan = fx.path("orphan");
    git_in(
        &fx.repo,
        &["worktree", "add", "-q", "-b", "orphan", &orphan],
        &[],
    );
    fs::remove_file(format!("{orphan}/.git")).unwrap();
    let ci = cfg!(windows);
    let git = Git::system();
    let env = ScanEnv {
        git: &git,
        fs: &RealFs,
        case_insensitive: ci,
        measure: &real_measure,
    };
    let out = scan_rules(&[Rule::folder("W-1", &fx.wts, "lead")], &env);
    let f = facts_by_name(&out.facts);
    assert!(!f["orphan"].is_git);
    assert!(
        f["orphan"].leftover.is_none(),
        "git lists it: not a leftover"
    );
    assert_eq!(f["plain"].leftover.as_ref().unwrap().files, Some(1));
    assert!(f["fresh"].leftover.is_none());
    assert!(out.listed.contains(&path_key(&orphan, ci)));
}

fn lo_with(
    spec: &[(&str, f64)],
    extra: impl FnOnce(&Fixture),
    options: impl FnOnce(&mut Options),
) -> (Fixture, Harness) {
    let fx = Fixture::new();
    for (name, _) in spec {
        put(&fx.wts, &format!("{name}/a.txt"), "hello\n");
    }
    extra(&fx);
    for (name, days) in spec.iter().filter(|(_, days)| *days > 0.0) {
        backdate(&fx.path(name), *days);
    }
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], options);
    h.scan();
    (fx, h)
}

fn lo(names: &[&str], extra: impl FnOnce(&Fixture)) -> (Fixture, Harness) {
    let spec: Vec<(&str, f64)> = names.iter().map(|n| (*n, 10.0)).collect();
    lo_with(&spec, extra, |_| {})
}

fn id(h: &Harness, fx: &Fixture, name: &str) -> String {
    h.id_of(&fx.path(name))
}

fn rows_setup() -> (Harness, TempDir) {
    let outside = tempdir().unwrap();
    let spec = [
        ("junk-old", 10.0),
        ("junk-nested", 10.0),
        ("junk-young", 0.0),
    ];
    let target = root_of(&outside);
    let extra = |fx: &Fixture| {
        put(&fx.wts, "junk-old/build/out.bin", &"x".repeat(2 * MB));
        put(
            &fx.wts,
            "junk-nested/inner/.git/HEAD",
            "ref: refs/heads/main\n",
        );
        make_link(&fx.path("junk-link"), &target);
    };
    let (_fx, h) = lo_with(&spec, extra, |_| {});
    (h, outside)
}

#[test]
fn a_leftover_row_carries_size_files_age_top_entries_and_flags() {
    let (h, _outside) = rows_setup();
    let d = h.docs();
    let old = &d["junk-old"];
    let l = old.leftover.as_ref().unwrap();
    assert_eq!(old.status, Status::Leftover);
    assert_eq!(
        (l.files, l.size_mb, l.age_days),
        (Some(2), Some(2.0), Some(10))
    );
    assert_eq!(
        (l.has_nested_git, l.partial, l.pending),
        (false, false, false)
    );
    assert_eq!(l.top_entries, ["a.txt", "build/"]);
    assert_eq!(old.last_touched_at, l.newest_at);
    let nested = d["junk-nested"].leftover.as_ref().unwrap();
    assert_eq!(nested.nested_git.as_deref(), Some("inner/.git"));
    assert!(d["junk-link"].leftover.as_ref().unwrap().is_link);
    assert!(d["fresh"].leftover.is_none() && d["fresh"].status == Status::Active);
}

#[test]
fn the_summary_counts_git_worktrees_and_leftovers_apart() {
    let (h, _outside) = rows_setup();
    let d = h.docs();
    let meta = h.meta();
    let left: Vec<&WorktreeDoc> = d
        .values()
        .filter(|x| x.status == Status::Leftover)
        .collect();
    assert_eq!(
        (meta.leftovers, meta.git, meta.count),
        (left.len(), 4, left.len() + 4)
    );
    assert_eq!(meta.by_status["leftover"], left.len());
    let old_enough = |l: &crate::doc::LeftoverDoc| {
        l.age_days.unwrap_or(0) >= 1 && !l.has_nested_git && !l.is_link
    };
    let old_ones = left
        .iter()
        .filter(|x| x.leftover.as_ref().is_some_and(old_enough))
        .count();
    assert_eq!(
        (meta.leftover_old.days, meta.leftover_old.count),
        (1.0, old_ones),
        "the default limit is one day"
    );
    assert!(meta.leftover_old.size_mb >= 2.0);
    assert_eq!((meta.deleting, meta.jobs.len()), (0, 0));
    assert!(
        left.iter().all(|x| !x.status.needs_decision()),
        "a leftover never needs a decision"
    );
}

#[test]
fn another_thread_and_an_ended_one_are_refused_the_lead_and_the_rules_thread_may() {
    let (fx, h) = lo(&["junk-old"], |_| {});
    let target = id(&h, &fx, "junk-old");
    let other = h
        .svc
        .remove(&crate::request::RemoveInput::new(
            "alpha",
            &target,
            Caller::Thread("other".into()),
        ))
        .unwrap_err();
    assert_eq!(other.code, ErrorCode::Forbidden);
    assert!(other.message.contains("other may not remove it"));
    let ended = h
        .svc
        .remove(&crate::request::RemoveInput::new(
            "alpha",
            &target,
            Caller::Thread("ended".into()),
        ))
        .unwrap_err();
    assert!(
        ended
            .message
            .contains("\"ended\" is not a live thread here")
    );
    assert!(exists_itself(&fx.path("junk-old")));
    let line = &h.notes.removals("refused to delete leftover folder")[0];
    assert_eq!(line.level, "warn");
    assert!(line.text.starts_with(&format!(
        "alpha: refused to delete leftover folder {target} for other: other may not remove it"
    )));
}

#[test]
fn a_leftover_has_no_branch_and_no_discard_a_400_with_the_reason_and_no_backup_is_promised() {
    let (fx, h) = lo(&["junk-flags"], |_| {});
    let target = id(&h, &fx, "junk-flags");
    let err = h.refused(&target, |i| i.branch = true);
    assert_eq!((err.code, err.code.status()), (ErrorCode::BadRequest, 400));
    assert!(err.message.contains("is a leftover folder: it has no branch and nothing git knows, so --branch and --discard do not apply"));
    assert!(err.message.contains("no backup is possible"));
    assert_eq!(
        h.refused(&target, |i| i.discard = true).code,
        ErrorCode::BadRequest
    );
    assert!(exists_itself(&fx.path("junk-flags")));
}

#[test]
fn outside_the_rules_the_rule_folder_changed_or_a_rule_whose_folder_it_is() {
    let (fx, h) = lo(&["junk-x"], |_| {});
    let target = id(&h, &fx, "junk-x");
    h.board().rules[0].path = format!("{}/elsewhere", fx.root);
    assert!(
        h.refused(&target, |_| {})
            .message
            .contains("outside the project's worktree rules")
    );
    h.board().rules[0].path = fx.wts.clone();
    h.scan();
    h.board()
        .rules
        .push(Rule::folder("W-2", &fx.path("junk-x"), "worker"));
    let own = h.refused(&target, |_| {});
    assert_eq!(own.code, ErrorCode::Forbidden);
    assert!(
        own.message
            .contains("is itself the folder of a worktree rule")
    );
    assert!(exists_itself(&fx.path("junk-x")));
}

#[test]
fn a_git_of_its_own_now_or_git_listing_it_again_is_a_409_and_nothing_is_deleted() {
    let (fx, h) = lo(&["junk-gitnow"], |fx| {
        fs::create_dir_all(fx.path("junk-listed")).unwrap();
    });
    put(&fx.path("junk-gitnow"), ".git", "gitdir: X:/nowhere\n");
    let own = h.refused(&id(&h, &fx, "junk-gitnow"), |_| {});
    assert_eq!(own.code, ErrorCode::Conflict);
    assert!(
        own.message
            .contains("has a .git of its own now: it is not a leftover, nothing was deleted")
    );
    fs::remove_file(format!("{}/.git", fx.path("junk-gitnow"))).unwrap();
    git_in(
        &fx.repo,
        &["worktree", "add", "-q", "--detach", &fx.path("junk-listed")],
        &[],
    );
    fs::remove_file(format!("{}/.git", fx.path("junk-listed"))).unwrap();
    backdate(&fx.path("junk-listed"), 10.0);
    let listed = h.refused(&id(&h, &fx, "junk-listed"), |_| {});
    assert_eq!(listed.code, ErrorCode::Conflict);
    assert!(
        listed.message.contains("again (a worktree of")
            && listed
                .message
                .contains("it is not a leftover any more, nothing was deleted"),
        "{}",
        listed.message
    );
    assert!(exists_itself(&fx.path("junk-listed")) && exists_itself(&fx.path("junk-gitnow")));
}

#[test]
fn a_nested_repository_is_refused_with_its_name_for_a_thread_and_the_page_and_a_link_too() {
    let outside = tempdir().unwrap();
    let (fx, h) = lo(&["junk-nested"], |fx| {
        put(
            &fx.wts,
            "junk-nested/inner/.git/HEAD",
            "ref: refs/heads/main\n",
        );
        put(&root_of(&outside), "keep.txt", "precious");
        make_link(&fx.path("junk-link"), &root_of(&outside));
    });
    for caller in [Caller::Thread("worker".into()), Caller::Page] {
        let err = h.refused(&id(&h, &fx, "junk-nested"), |i| {
            (i.caller, i.confirm_young) = (caller, true);
        });
        assert_eq!(err.code, ErrorCode::Forbidden);
        assert!(err.message.contains("contains a nested git repository (inner/.git), which may hold real work: Argus does not delete it"));
    }
    let link = h.refused(&id(&h, &fx, "junk-link"), |i| {
        (i.caller, i.confirm_young) = (Caller::Page, true);
    });
    assert!(
        link.message
            .contains("is a link or junction, not a folder: Argus never follows or deletes one")
    );
    assert!(
        exists_itself(&format!("{}/inner/.git", fx.path("junk-nested")))
            && exists_itself(&format!("{}/keep.txt", root_of(&outside)))
    );
}

fn node_on(path: &str, pid: u32) -> ProcInfo {
    ProcInfo {
        pid,
        name: "node.exe".into(),
        cmd: format!("node {}\\server.js", path.replace('/', "\\")),
        path: Some("C:/Program Files/nodejs/node.exe".into()),
        cwd: None,
    }
}

#[test]
fn a_live_process_in_the_folder_is_a_423_the_requester_is_not_a_user_then_the_deletion_starts() {
    let (fx, h) = lo(&["junk-proc"], |_| {});
    let (path, target) = (fx.path("junk-proc"), id(&h, &fx, "junk-proc"));
    *h.procs.0.lock().unwrap() = vec![node_on(&path, 4242)];
    let err = h.refused(&target, |_| {});
    assert_eq!((err.code, err.code.status()), (ErrorCode::InUse, 423));
    assert!(err.message.contains("is in use by node.exe 4242"));
    assert!(exists_itself(&path), "not even renamed");
    let requester = ProcInfo {
        pid: 5151,
        name: "bash.exe".into(),
        cmd: format!("bash -c \"coord worktree remove {path}\""),
        ..ProcInfo::default()
    };
    let sibling = ProcInfo {
        pid: 6161,
        name: "node.exe".into(),
        cmd: format!("node {path}-other/x.js"),
        ..ProcInfo::default()
    };
    *h.procs.0.lock().unwrap() = vec![requester, sibling];
    assert!(h.rm(&target).unwrap().started);
    h.idle();
    assert!(!exists_itself(&path));
}

#[test]
fn a_young_folder_a_thread_is_refused_the_page_is_refused_until_it_confirms_then_it_starts() {
    let (fx, h) = lo_with(&[("junk-young", 0.0)], |_| {}, |_| {});
    let target = id(&h, &fx, "junk-young");
    let thread = h.refused(&target, |_| {});
    assert_eq!(thread.code, ErrorCode::Forbidden);
    assert!(thread.message.contains("less than 1 day ago: a thread only deletes a leftover nothing touched for 1 day (or, with --older-than and --reason, for at least 6 hours). The owner can delete it from the page"), "{}", thread.message);
    let page = h.refused(&target, |i| i.caller = Caller::Page);
    assert!(
        page.message
            .contains("confirm that you want a young folder deleted")
    );
    assert!(exists_itself(&fx.path("junk-young")));
    let out = h
        .rm_with(&target, |i| {
            (i.caller, i.confirm_young) = (Caller::Page, true);
        })
        .unwrap();
    assert!(out.started);
    h.idle();
    assert!(!exists_itself(&fx.path("junk-young")));
    assert!(
        h.notes.removals("deleting leftover folder")[0]
            .text
            .ends_with("for the owner (page)")
    );
}

#[test]
fn the_token_a_wrong_newest_change_is_a_409_and_a_file_written_since_the_listing_is_a_409_even_for_the_page()
 {
    let (fx, h) = lo(&["junk-token"], |_| {});
    let target = id(&h, &fx, "junk-token");
    let wrong = h.refused(&target, |i| i.newest_at_ms = Some(1_577_836_800_000));
    assert_eq!(wrong.code, ErrorCode::Conflict);
    assert!(
        wrong
            .message
            .contains("changed since you looked (its newest change is 20")
            && wrong.message.contains("not 2020-01-01T00:00:00.000Z)"),
        "{}",
        wrong.message
    );
    put(
        &fx.path("junk-token"),
        "late.txt",
        "written after the scan\n",
    );
    let late = h.refused(&target, |i| {
        (i.caller, i.confirm_young) = (Caller::Page, true);
    });
    assert_eq!(late.code, ErrorCode::Conflict);
    assert!(
        late.message
            .contains("changed since the listing (a file was written at 20"),
        "{}",
        late.message
    );
    assert!(exists_itself(&fx.path("junk-token")));
}

#[test]
fn the_rename_is_refused_a_423_nothing_deleted_no_job_and_the_row_stays() {
    let (fx, h) = lo(&["junk-lock"], |_| {});
    *h.fs.rename_error.lock().unwrap() = Some(std::io::ErrorKind::ResourceBusy);
    let err = h.refused(&id(&h, &fx, "junk-lock"), |_| {});
    assert_eq!((err.code, err.code.status()), (ErrorCode::InUse, 423));
    assert!(err.message.contains("could not be renamed for deletion (EBUSY): something has it open or locked. Close it and try again; nothing was deleted"));
    assert!(exists_itself(&fx.path("junk-lock")));
    assert_eq!(h.docs()["junk-lock"].status, Status::Leftover);
    assert_eq!(h.svc.jobs("alpha").len(), 0);
}

fn newest_of(h: &Harness, name: &str) -> Option<i64> {
    h.fact(name).leftover.and_then(|l| l.newest_ms)
}

struct Gated {
    fx: Fixture,
    h: Harness,
    out_root: String,
    gate: Arc<Gate>,
    _outside: TempDir,
}

fn gated_job() -> Gated {
    let outside = tempdir().unwrap();
    let out_root = root_of(&outside);
    put(&out_root, "keep.txt", "precious");
    let (fx, h) = lo(&["junk-inner-link"], |fx| {
        make_link(&format!("{}/junk-inner-link/lnk", fx.wts), &out_root);
    });
    let gate = Arc::new(Gate::default());
    *h.fs.gate.lock().unwrap() = Some(gate.clone());
    Gated {
        fx,
        h,
        out_root,
        gate,
        _outside: outside,
    }
}

fn start_gated(g: &Gated) -> crate::request::RemoveOutcome {
    let target = id(&g.h, &g.fx, "junk-inner-link");
    let newest = newest_of(&g.h, "junk-inner-link");
    g.h.rm_with(&target, |i| {
        (i.reason, i.newest_at_ms) = (Some("old agent folder".into()), newest);
    })
    .unwrap()
}

#[test]
fn the_job_answers_at_once_with_the_renamed_folder_and_what_it_will_delete() {
    let g = gated_job();
    let out = start_gated(&g);
    assert!(out.started && !out.removed && out.leftover);
    assert!(out.job_id.as_deref().is_some_and(|j| j.starts_with("L-")));
    assert!(is_deleting(&base_name(&out.folder.clone().unwrap())));
    assert!(
        out.text.starts_with("deleting leftover folder "),
        "{}",
        out.text
    );
    assert!(
        out.text.contains(
            ", 2 files, newest change 10 days ago; git does not know it, so there is no backup)"
        ),
        "{}",
        out.text
    );
    g.gate.open();
    g.h.idle();
}

#[test]
fn the_row_shows_the_deletion_while_the_job_runs_and_the_name_is_free_at_once() {
    let g = gated_job();
    let out = start_gated(&g);
    assert!(
        !exists_itself(&g.fx.path("junk-inner-link")),
        "the name is free at once"
    );
    assert!(exists_itself(&out.folder.unwrap()));
    let row = g.h.docs()["junk-inner-link"].clone();
    assert_eq!(
        row.status,
        Status::Deleting,
        "the row stays while the job runs"
    );
    assert!(matches!(
        row.job.as_ref().unwrap().state,
        JobState::Queued | JobState::Running
    ));
    assert_eq!(g.h.meta().deleting, 1);
    g.gate.open();
    g.h.idle();
}

#[test]
fn a_second_request_for_a_folder_being_deleted_is_refused() {
    let g = gated_job();
    let target = id(&g.h, &g.fx, "junk-inner-link");
    g.h.rm(&target).unwrap();
    let again = g.h.refused(&target, |_| {});
    assert_eq!(again.code, ErrorCode::Conflict);
    assert!(
        again.message.contains("is already being deleted (job L-"),
        "{}",
        again.message
    );
    g.gate.open();
    g.h.idle();
}

#[test]
fn a_junction_inside_is_never_followed_and_the_row_the_folder_and_the_lines_end_well() {
    let g = gated_job();
    g.gate.open();
    let out =
        g.h.rm_with(&id(&g.h, &g.fx, "junk-inner-link"), |i| {
            i.reason = Some("old agent folder".into());
        })
        .unwrap();
    g.h.idle();
    assert!(
        !exists_itself(&out.folder.unwrap()),
        "the renamed folder is gone"
    );
    assert_eq!(
        fs::read_to_string(format!("{}/keep.txt", g.out_root)).unwrap(),
        "precious",
        "the junction's target was never followed"
    );
    assert!(!g.h.docs().contains_key("junk-inner-link"), "the row left");
    let job = g.h.svc.jobs("alpha").remove(0);
    assert_eq!(
        (job.state, job.error.clone(), Some(job.removed) == job.total),
        (JobState::Done, None, true)
    );
    let start = &g.h.notes.removals("deleting leftover folder")[0];
    assert_eq!(
        (start.level, start.project.as_str(), start.thread.as_deref()),
        ("info", "alpha", Some("worker"))
    );
    assert!(start.text.ends_with("for worker: old agent folder"));
    let end = &g.h.notes.removals("deleted leftover folder")[0];
    assert!(
        end.text.starts_with("alpha: deleted leftover folder "),
        "{}",
        end.text
    );
    assert!(
        end.text.contains("2 top-level entries, 2 files") && end.text.ends_with(" s) for worker"),
        "{}",
        end.text
    );
}

#[test]
fn a_failed_entry_fails_the_job_the_renamed_folder_stays_and_an_error_line_says_why() {
    let (fx, h) = lo(&["junk-fail"], |fx| {
        put(&fx.wts, "junk-fail/build/out.bin", "x");
        put(&fx.wts, "junk-fail/other/o.txt", "x");
    });
    *h.fs.fail_remove_containing.lock().unwrap() = Some("/build".to_string());
    let out = h.rm(&id(&h, &fx, "junk-fail")).unwrap();
    h.idle();
    *h.fs.fail_remove_containing.lock().unwrap() = None;
    let folder = out.folder.unwrap();
    let job = h.svc.jobs("alpha").pop().unwrap();
    assert_eq!(
        (job.state, job.error.as_deref()),
        (JobState::Failed, Some("build: EBUSY"))
    );
    assert!(exists_itself(&folder), "kept for the next start");
    assert!(
        !exists_itself(&format!("{folder}/other")),
        "the other entries went"
    );
    let row = &h.docs()["junk-fail"];
    assert_eq!(
        (row.status, row.job.as_ref().unwrap().state),
        (Status::Deleting, JobState::Failed)
    );
    let line = &h.notes.removals("failed after")[0];
    assert_eq!(line.level, "error");
    assert!(
        line.text.contains(
            "failed after 2 of 3 entries: build: EBUSY. It is renamed junk-fail.argus-deleting-"
        ) && line.text.contains("and is finished at the next start"),
        "{}",
        line.text
    );
}

#[test]
fn an_id_that_is_not_in_the_last_scan_is_a_404() {
    let (_fx, h) = lo(&["junk-x"], |_| {});
    assert_eq!(h.refused("ffffffffff", |_| {}).code, ErrorCode::NotFound);
}

fn batch(
    h: &Harness,
    caller: Caller,
    change: impl FnOnce(&mut BatchInput),
) -> crate::error::Result<crate::batch::BatchOutcome> {
    h.batch(caller, change)
}

fn worker() -> Caller {
    Caller::Thread("worker".into())
}

fn batch_setup() -> (Fixture, Harness) {
    lo_with(
        &[
            ("old-1", 8.0),
            ("old-2", 8.0),
            ("old-3", 8.0),
            ("nested", 8.0),
            ("young", 0.0),
        ],
        |fx| {
            put(&fx.wts, "nested/inner/.git/HEAD", "x");
            backdate(&fx.path("plain"), 8.0);
        },
        |_| {},
    )
}

fn assert_batch_lines(h: &Harness) {
    assert_eq!(h.notes.removals("queued 4 leftover folders").len(), 1);
    assert!(
        h.notes.removals("queued 4 leftover folders")[0]
            .text
            .contains(" for worker: cleanup. Skipped: ")
    );
    assert_eq!(
        h.notes
            .removals("leftover batch finished: 4 deleted or started, 0 refused for worker")
            .len(),
        1
    );
    assert_eq!(h.notes.removals("deleted leftover folder").len(), 4);
    assert!(
        h.svc
            .jobs("alpha")
            .iter()
            .all(|j| j.state == JobState::Done)
    );
}

fn cleanup_batch(h: &Harness) -> crate::batch::BatchOutcome {
    batch(h, worker(), |i| {
        (i.older_than_days, i.reason) = (Some(1.0), Some("cleanup".into()));
    })
    .unwrap()
}

#[test]
fn a_thread_that_is_neither_lead_nor_rule_thread_is_refused_as_a_whole() {
    let (_fx, h) = batch_setup();
    let bad = batch(&h, Caller::Thread("other".into()), |_| {}).unwrap_err();
    assert_eq!(bad.code, ErrorCode::Forbidden);
}

#[test]
fn the_rules_thread_queues_the_old_ones_and_is_told_what_was_skipped_and_why() {
    let (_fx, h) = batch_setup();
    let out = cleanup_batch(&h);
    assert_eq!(
        (out.started, out.older_than_days),
        (4, 1.0),
        "old-1, old-2, old-3 and plain"
    );
    let why: HashMap<String, String> = out
        .skipped
        .iter()
        .map(|s| (base_name(s.path.as_deref().unwrap()), s.reason.clone()))
        .collect();
    assert!(why["nested"].contains("holds a nested git repository (inner/.git)"));
    assert_eq!(why["young"], "changed 1 hour ago, under 1 day");
    assert_eq!((out.skipped.len(), out.jobs.len()), (2, 4));
    assert!(
        out.text
            .starts_with("queued 4 leftover folders for deletion ("),
        "{}",
        out.text
    );
    assert!(
        out.text
            .ends_with(" files; no backup: git does not know them), skipped 2"),
        "{}",
        out.text
    );
    h.idle();
}

#[test]
fn the_old_ones_are_gone_after_the_batch_the_others_stay_and_the_lines_say_so() {
    let (fx, h) = batch_setup();
    cleanup_batch(&h);
    h.idle();
    for name in ["old-1", "old-2", "old-3", "plain"] {
        assert!(!exists_itself(&fx.path(name)), "{name}");
    }
    assert!(exists_itself(&fx.path("nested")) && exists_itself(&fx.path("young")));
    assert!(
        !sibling_names(&fx.wts)
            .iter()
            .any(|n| n.contains("argus-deleting"))
    );
    assert_batch_lines(&h);
}

fn page_batch(h: &Harness, change: impl FnOnce(&mut BatchInput)) -> crate::batch::BatchOutcome {
    batch(h, Caller::Page, change).unwrap()
}

#[test]
fn the_page_may_go_younger_but_a_young_folder_is_refused_at_its_turn_without_the_confirm() {
    let (fx, h) = batch_setup();
    cleanup_batch(&h);
    h.idle();
    let out = page_batch(&h, |i| i.older_than_days = Some(0.0));
    assert_eq!(
        out.started, 1,
        "young is the only plain leftover left besides the nested one (skipped)"
    );
    h.idle();
    assert!(
        exists_itself(&fx.path("young")),
        "refused at its turn: needs the stronger confirm"
    );
    let refused = h
        .meta()
        .jobs
        .into_iter()
        .find(|j| j.state == JobState::Refused)
        .unwrap();
    assert!(
        refused
            .error
            .unwrap()
            .contains("confirm that you want a young folder deleted")
    );
    assert_eq!(
        h.notes
            .removals("leftover batch finished: 0 deleted or started, 1 refused")
            .len(),
        1
    );
}

#[test]
fn ids_pick_folders_the_confirm_lets_a_young_one_go_and_a_nested_repository_never_goes() {
    let (fx, h) = batch_setup();
    let ids = vec![
        id(&h, &fx, "young"),
        id(&h, &fx, "nested"),
        "ffffffffff".to_string(),
    ];
    let ok = page_batch(&h, |i| {
        (i.older_than_days, i.confirm_young, i.ids) = (Some(0.0), true, Some(ids));
    });
    assert_eq!(ok.started, 1);
    let mut reasons: Vec<&str> = ok.skipped.iter().map(|s| s.reason.as_str()).collect();
    reasons.sort_unstable();
    assert_eq!(
        reasons,
        [
            "holds a nested git repository (inner/.git)",
            "not a leftover row in the listing"
        ]
    );
    h.idle();
    assert!(!exists_itself(&fx.path("young")));
    assert!(
        exists_itself(&fx.path("nested")),
        "a nested repository is never deleted"
    );
}

#[test]
fn nothing_to_delete_says_so_in_the_activity_log() {
    let (_fx, h) = batch_setup();
    page_batch(&h, |i| {
        (i.older_than_days, i.confirm_young) = (Some(0.0), true);
    });
    h.idle();
    let none = batch(&h, Caller::Thread("lead".into()), |_| {}).unwrap();
    assert_eq!((none.started, none.skipped.len()), (0, 1));
    assert_eq!(none.text, "no leftover folder to delete (1 skipped)");
    assert_eq!(
        h.notes.removals("no leftover folder to delete")[0].level,
        "warn"
    );
}

fn sibling_names(wts: &str) -> Vec<String> {
    let mut out: Vec<String> = fs::read_dir(wts)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    out.sort();
    out
}

#[test]
fn a_folder_a_crash_left_half_deleted_is_finished_at_the_next_start_once_other_names_and_places_are_left()
 {
    let (fx, h) = lo_with(
        &[],
        |fx| {
            put(&fx.wts, "crashed.argus-deleting-1730000000001/a/b.txt", "x");
            put(&fx.wts, "weird.argus-deleting-abc/c.txt", "x");
            put(
                &fx.root,
                "elsewhere.argus-deleting-1730000000002/d.txt",
                "x",
            );
        },
        |_| {},
    );
    h.idle();
    assert!(
        !exists_itself(&fx.path("crashed.argus-deleting-1730000000001")),
        "finished"
    );
    assert!(
        exists_itself(&fx.path("weird.argus-deleting-abc")),
        "not a time stamp: an ordinary leftover"
    );
    assert_eq!(
        h.docs()["weird.argus-deleting-abc"].status,
        Status::Leftover
    );
    assert!(
        exists_itself(&format!(
            "{}/elsewhere.argus-deleting-1730000000002",
            fx.root
        )),
        "outside the rules"
    );
    assert_eq!(
        h.notes
            .removals("finishing a deletion a crash cut short: ")
            .len(),
        1
    );
    assert_eq!(h.notes.removals("deleted leftover folder").len(), 1);
    assert!(
        !h.docs()
            .contains_key("crashed.argus-deleting-1730000000001")
    );
    put(&fx.wts, "again.argus-deleting-1730000000003/e.txt", "x");
    h.scan();
    h.idle();
    assert!(
        sibling_names(&fx.wts).contains(&"again.argus-deleting-1730000000003".to_string()),
        "only at the start of a run"
    );
}

#[test]
fn a_scan_with_no_time_left_to_measure_shows_the_leftovers_as_pending_and_a_batch_skips_them() {
    let (_fx, h) = lo_with(&[("old", 9.0)], |_| {}, |o| o.budget_ms = 0);
    let row = &h.docs()["old"];
    let l = row.leftover.as_ref().unwrap();
    assert_eq!(
        (row.status, l.pending, l.size_mb, l.files),
        (Status::Leftover, true, None, None)
    );
    assert!(h.meta().leftover_pending >= 2);
    assert_eq!(h.svc.pending_count("alpha"), h.meta().leftover_pending);
    let out = batch(&h, Caller::Thread("lead".into()), |_| {}).unwrap();
    assert_eq!(out.started, 0);
    assert!(
        out.skipped
            .iter()
            .all(|s| s.reason.contains("not measured yet"))
    );
}

fn doc_with(path: &str, age: i64) -> WorktreeDoc {
    let rule = Rule::folder("W-1", "X:/Bxel/.claude/worktrees", "lead");
    let mut fact = crate::fact::blank_fact(path, &rule, true);
    (fact.is_git, fact.branch, fact.commit_ms, fact.ahead) = (
        true,
        Some(format!("feature/some-long-branch-name-{age}")),
        Some(1_792_929_600_000 - age * DAY_MS),
        2,
    );
    crate::status::doc_of(
        &fact,
        &crate::status::Judging {
            rule: Some(&rule),
            now_ms: 1_792_929_600_000,
            ..crate::status::Judging::default()
        },
        None,
    )
}

#[test]
fn many_text_counts_git_worktrees_and_leftovers_apart_and_the_wake_keeps_the_sentence_first_within_300_characters()
 {
    let sentence = many_text("Bxel", 100, 8, 92, 6144.0);
    assert_eq!(
        sentence,
        "Bxel has 100 folders: 8 git worktrees and 92 leftovers (about 6 GB): `coord worktree remove --all-leftovers` clears the old ones."
    );
    assert_eq!(
        many_text("Bxel", 13, 12, 1, 0.2),
        "Bxel has 13 folders: 12 git worktrees and 1 leftover: `coord worktree remove --all-leftovers` clears the old ones.",
        "under 1 MB says no size"
    );
    assert_eq!(
        many_text("Alpha", 14, 14, 0, 0.0),
        "Alpha has 14 worktrees: that is a lot, remove the ones you no longer need (coord worktree list, coord worktree remove)."
    );
    let docs: Vec<WorktreeDoc> = (5..10)
        .map(|i| {
            doc_with(
                &format!("X:/Bxel/.claude/worktrees/some-long-worktree-name-{i}"),
                i,
            )
        })
        .collect();
    let refs: Vec<&WorktreeDoc> = docs.iter().collect();
    let text = wake_text(&refs, &sentence);
    assert!(
        text.chars().count() <= 300,
        "{} characters",
        text.chars().count()
    );
    assert!(text.starts_with(&format!("{sentence} ")));
}

#[test]
fn the_leads_wake_counts_folders_once_and_not_again_before_repeat_ms() {
    let (_fx, h) = lo_with(
        &[],
        |fx| {
            for n in ["l1", "l2"] {
                put(&fx.wts, &format!("{n}/a.bin"), &"x".repeat(3 * MB));
            }
        },
        |o| o.many_at = 2,
    );
    let wakes = h.notes.wakes.lock().unwrap().clone();
    let lead = wakes.iter().find(|w| w.thread == "lead").unwrap();
    assert_eq!(lead.reason, "worktree-stale");
    assert_eq!(
        lead.text,
        "Alpha has 7 folders: 4 git worktrees and 3 leftovers (about 6 MB): `coord worktree remove --all-leftovers` clears the old ones."
    );
    h.notes.wakes.lock().unwrap().clear();
    h.scan();
    assert_eq!(h.notes.wake_count(), 0, "once per repeat_ms (24 h)");
    let worker = wakes.iter().find(|w| w.thread == "worker").unwrap();
    assert!(!worker.text.contains("folders:"));
}
