//! Removing a worktree on real throwaway repositories: every guard, the safe removals, discarding refused or allowed
//! by the owner's entry, the backup written first and restored from the bundle, the live check and the kept state.

use crate::board::{Caller, EntryWorktree, Keep, OwnerEntry};
use crate::clock::DAY_MS;
use crate::doc::Status;
use crate::error::ErrorCode;
use crate::request::{Discarded, RemoveInput};
use crate::rule::Rule;
use crate::service::Options;
use crate::testkit::{
    Fixture, Harness, backdate_index, branches, commit, days_ago, exists_itself, git_in, listed,
    owner_now, put, rule_for, set_mtime,
};
use crate::waking::sent_key;
use std::fs;
use std::path::Path;
use std::time::Duration;

fn setup(prep: impl FnOnce(&Fixture), options: impl FnOnce(&mut Options)) -> (Fixture, Harness) {
    let fx = Fixture::new();
    prep(&fx);
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], options);
    h.scan();
    (fx, h)
}

fn std_setup() -> (Fixture, Harness) {
    setup(|_| {}, |_| {})
}

fn id(h: &Harness, fx: &Fixture, name: &str) -> String {
    h.id_of(&fx.path(name))
}

fn owner(id: &str, text: &str) -> OwnerEntry {
    owner_now(id, text)
}

fn add_entries(h: &Harness, entries: Vec<OwnerEntry>) {
    for e in entries {
        h.board().entries.insert(e.id.clone(), e);
    }
}

#[test]
fn only_what_is_in_the_live_listing_an_unknown_id_or_path_is_404_and_the_refusal_is_an_activity_line()
 {
    let (_fx, h) = std_setup();
    let err = h.refused("ffffffffff", |_| {});
    assert_eq!(err.code, ErrorCode::NotFound);
    assert!(err.message.contains("not in the live listing of alpha"));
    assert_eq!(
        h.refused("X:/somewhere/else", |_| {}).code,
        ErrorCode::NotFound
    );
    let line = &h.notes.removals("refused to remove worktree")[0];
    assert_eq!(
        (line.level, line.project.as_str(), line.thread.as_deref()),
        ("warn", "alpha", Some("worker"))
    );
    assert!(line.text.starts_with("alpha: refused to remove worktree ffffffffff for worker: ffffffffff is not in the live listing"), "{}", line.text);
}

#[test]
fn a_rule_that_no_longer_covers_the_folder_is_a_403_never_an_arbitrary_path() {
    let (fx, h) = std_setup();
    h.board().rules[0].path = format!("{}/elsewhere", fx.root);
    let err = h.refused(&id(&h, &fx, "old-merged"), |_| {});
    assert_eq!(err.code, ErrorCode::Forbidden);
    assert!(err.message.contains("outside the project's worktree rules"));
    assert!(exists_itself(&fx.path("old-merged")), "nothing was removed");
}

#[test]
fn never_a_main_checkout_a_folder_rule_that_reaches_the_repository_itself() {
    let fx = Fixture::new();
    let h = Harness::new(
        &fx.root,
        vec![Rule::folder("W-1", &fx.root, "worker")],
        |_| {},
    );
    h.scan();
    let err = h.refused(&h.id_of(&fx.repo), |_| {});
    assert_eq!(err.code, ErrorCode::Forbidden);
    assert!(
        err.message
            .contains("main checkout: Argus never removes one")
    );
    assert!(exists_itself(&format!("{}/.git", fx.repo)));
}

fn lock_old_merged(fx: &Fixture) {
    fx.git(&[
        "worktree",
        "lock",
        "--reason",
        "kept for the demo",
        &fx.path("old-merged"),
    ]);
}

fn detach_at_main(fx: &Fixture) {
    fx.git(&[
        "worktree",
        "add",
        "-q",
        "--detach",
        &fx.path("detached"),
        "main",
    ]);
}

#[test]
fn never_a_locked_worktree() {
    let (fx, h) = setup(lock_old_merged, |_| {});
    let err = h.refused(&id(&h, &fx, "old-merged"), |_| {});
    assert_eq!(err.code, ErrorCode::Forbidden);
    assert!(err.message.contains("is locked (kept for the demo)"));
    assert!(listed(&fx.repo).contains(&"old-merged".to_string()));
}

#[test]
fn a_live_process_in_the_folder_is_a_423_and_the_coord_request_that_names_it_is_not_a_user() {
    let (fx, h) = std_setup();
    let path = fx.path("old-merged");
    let node = crate::ports::ProcInfo {
        pid: 4242,
        name: "node.exe".into(),
        cmd: format!(
            "node {}\\node_modules\\vite\\bin\\vite.js",
            path.replace('/', "\\")
        ),
        path: Some("C:/Program Files/nodejs/node.exe".into()),
        cwd: None,
    };
    *h.procs.0.lock().unwrap() = vec![node];
    let err = h.refused(&id(&h, &fx, "old-merged"), |_| {});
    assert_eq!((err.code, err.code.status()), (ErrorCode::InUse, 423));
    assert!(err.message.contains("is in use by node.exe 4242"));
    let requester = crate::ports::ProcInfo {
        pid: 5151,
        name: "bash.exe".into(),
        cmd: format!("bash -c \"coord worktree remove {path}\""),
        ..Default::default()
    };
    let sibling = crate::ports::ProcInfo {
        pid: 6161,
        name: "node.exe".into(),
        cmd: format!("node {path}-other/x.js"),
        ..Default::default()
    };
    *h.procs.0.lock().unwrap() = vec![requester, sibling];
    assert!(
        h.rm(&id(&h, &fx, "old-merged")).unwrap().removed,
        "the requester and a sibling folder with a longer name are not users"
    );
}

#[test]
fn a_commit_after_the_listing_or_a_head_other_than_the_one_the_caller_saw_is_a_409() {
    let (fx, h) = std_setup();
    let target = id(&h, &fx, "old-merged");
    let head = h.docs()["old-merged"].head.clone().unwrap();
    commit(&fx.path("old-merged"), "late.txt", "late work\n", 0.0);
    let err = h.refused(&target, |i| i.head = Some(head.clone()));
    assert_eq!(err.code, ErrorCode::Conflict);
    assert!(
        err.message.contains("changed since the listing (HEAD ")
            && err.message.contains(
                ", 0 -> 0 uncommitted): nothing was removed, the listing is being refreshed"
            ),
        "{}",
        err.message
    );
    assert!(exists_itself(&fx.path("old-merged")));
    assert_eq!(h.svc.take_rescans(), ["alpha"]);
    h.scan();
    assert_ne!(h.docs()["old-merged"].head.as_deref(), Some(head.as_str()));
    let stale = h.refused(&target, |i| i.head = Some(head.clone()));
    assert!(
        stale
            .message
            .contains("changed since you looked: its HEAD is ")
            && stale.message.contains(&format!(", not {head}")),
        "{}",
        stale.message
    );
    let new_head = h.docs()["old-merged"].head.clone();
    let dirty = h.refused(&target, |i| (i.head, i.dirty) = (new_head, Some(3)));
    assert!(
        dirty.message.contains("0 uncommitted changes, not 3"),
        "{}",
        dirty.message
    );
    assert!(exists_itself(&fx.path("old-merged")));
}

#[test]
fn the_rules_thread_and_the_lead_may_remove_another_thread_or_an_ended_one_may_not() {
    let (fx, h) = std_setup();
    let target = id(&h, &fx, "old-merged");
    let ask = |who: &str| h.refused(&target, |i| i.caller = Caller::Thread(who.into()));
    assert!(ask("other").message.contains(
        "other may not remove it: only the project's lead or the thread its rule tells (worker)"
    ));
    assert!(
        ask("ended")
            .message
            .contains("\"ended\" is not a live thread here")
    );
    assert_eq!(ask("").code, ErrorCode::Forbidden);
    assert!(
        h.rm_with(&target, |i| i.caller = Caller::Thread("lead".into()))
            .unwrap()
            .removed,
        "the lead may"
    );
}

#[test]
fn merged_and_clean_goes_with_no_backup_the_row_leaves_at_once_and_branch_deletes_the_merged_branch()
 {
    let (fx, h) = std_setup();
    let out = h
        .rm_with(&id(&h, &fx, "old-merged"), |i| {
            (i.branch, i.reason) = (true, Some("merged last week".into()));
        })
        .unwrap();
    assert_eq!(
        (
            out.removed,
            out.pruned_only,
            out.branch.as_deref(),
            out.deleted_branch,
            out.backup.is_none()
        ),
        (true, false, Some("old-merged"), true, true)
    );
    assert!(!exists_itself(&fx.path("old-merged")), "the folder is gone");
    assert!(
        !listed(&fx.repo).contains(&"old-merged".to_string()),
        "git no longer lists it"
    );
    assert!(
        !branches(&fx.repo).contains(&"old-merged".to_string()),
        "the merged branch was deleted"
    );
    assert!(
        !h.docs().contains_key("old-merged"),
        "the row left the docs"
    );
    let line = &h.notes.removals("removed worktree")[0];
    assert_eq!(line.level, "info");
    assert!(line.text.starts_with("alpha: removed worktree ") && line.text.ends_with("old-merged, branch old-merged deleted: merged into local main (not pushed) for worker: merged last week"), "{}", line.text);
}

#[test]
fn a_clean_worktree_whose_head_is_reachable_from_the_default_branch_goes_with_no_branch_to_delete()
{
    let (fx, h) = setup(detach_at_main, |_| {});
    let out = h
        .rm_with(&id(&h, &fx, "detached"), |i| i.branch = true)
        .unwrap();
    assert_eq!(
        (out.removed, out.deleted_branch, out.branch_note.as_deref()),
        (true, false, Some("no branch (detached HEAD)"))
    );
}

#[test]
fn a_gone_folder_is_pruned_by_id_a_folder_that_exists_is_not_and_prune_with_no_id_prunes_every_gone_one()
 {
    let (fx, h) = std_setup();
    let gone = id(&h, &fx, "gone");
    assert_eq!(h.meta().gone, 1);
    let mut not_gone = Harness::input(&id(&h, &fx, "old-merged"));
    not_gone.prune = true;
    assert!(
        h.svc
            .prune(&not_gone)
            .unwrap_err()
            .message
            .contains("is not gone: remove it with coord worktree remove")
    );
    let out = h.rm(&gone).unwrap();
    assert_eq!(
        (out.removed, out.pruned_only, out.backup.is_none()),
        (true, true, true)
    );
    assert!(!listed(&fx.repo).contains(&"gone".to_string()));
    assert_eq!(h.meta().gone, 0);
    fs::remove_dir_all(fx.path("fresh")).unwrap();
    h.scan();
    assert_eq!(h.meta().gone, 1);
    let all = h
        .svc
        .prune(&RemoveInput::new(
            "alpha",
            "",
            Caller::Thread("lead".into()),
        ))
        .unwrap();
    assert_eq!((all.pruned_only, all.pruned.len()), (true, 1));
    assert!(!listed(&fx.repo).contains(&"fresh".to_string()));
}

fn loose(fx: &Fixture) {
    let path = fx.path("loose");
    git_in(
        &fx.repo,
        &["worktree", "add", "-q", "--detach", &path, "main"],
        &[],
    );
    commit(&path, "loose.txt", "work on no branch\n", 10.0);
    backdate_index(&path, 10.0);
}

fn bad_entries() -> Vec<OwnerEntry> {
    let old = crate::clock::Clock::now_ms(&crate::clock::SystemClock) - 15 * DAY_MS;
    vec![
        OwnerEntry {
            author: "lead".into(),
            ..owner("O-a", "delete loose")
        },
        OwnerEntry {
            via: Some("mcp".into()),
            ..owner("O-b", "delete loose")
        },
        OwnerEntry {
            kind: "pause".into(),
            ..owner("O-c", "delete loose")
        },
        OwnerEntry {
            created_ms: Some(old),
            ..owner("O-d", "delete loose")
        },
        owner("O-e", "delete the fresh one"),
        OwnerEntry {
            kind: "decision".into(),
            reply_to: Some("E-9".into()),
            ..owner("O-f", "Delete both")
        },
    ]
}

fn assert_not_the_owners_say(h: &Harness, target: &str, entry: &str, reason: &str) {
    let err = h.refused(target, |i| {
        (i.discard, i.owner_entry) = (true, Some(entry.into()));
    });
    assert_eq!(err.code, ErrorCode::Forbidden, "{entry}");
    assert!(
        err.message
            .starts_with("discarding 1 commit on a detached HEAD that no branch keeps in "),
        "{entry}: {}",
        err.message
    );
    assert!(
        err.message.contains("needs the owner: ") && err.message.contains(reason),
        "{entry}: {}",
        err.message
    );
}

#[test]
fn work_no_ref_keeps_is_refused_without_discard() {
    let (fx, h) = setup(loose, |_| {});
    let row = &h.docs()["loose"];
    assert_eq!(
        (row.detached, row.referenced, row.unmerged),
        (true, Some(false), Some(1)),
        "a detached HEAD that no ref keeps"
    );
    let plain = h.refused(&id(&h, &fx, "loose"), |_| {});
    assert_eq!(plain.code, ErrorCode::Forbidden);
    assert!(plain.message.contains("has 1 commit on a detached HEAD that no branch keeps: removing it discards them. The owner can do it from the page (Delete now), or send discard"));
}

#[test]
fn discard_without_a_valid_owner_entry_is_refused_for_every_way_an_entry_can_fail() {
    let (fx, h) = setup(loose, |_| {});
    add_entries(&h, bad_entries());
    let target = id(&h, &fx, "loose");
    let want = [
        ("", "no owner entry was given"),
        ("O-zz", "there is no entry O-zz on this board"),
        ("O-a", "O-a was not posted by the owner from the page"),
        ("O-b", "O-b was not posted by the owner from the page"),
        (
            "O-c",
            "O-c is a pause, not the owner's answer to a decision or a note",
        ),
        ("O-d", "O-d is older than 14 days"),
        (
            "O-e",
            "O-e does not name this worktree (its branch (detached), its folder loose or its path)",
        ),
        ("O-f", "O-f does not name this worktree"),
    ];
    for (entry, reason) in want {
        assert_not_the_owners_say(&h, &target, entry, reason);
    }
    assert!(exists_itself(&fx.path("loose")), "nothing was removed");
    assert!(!Path::new(&h.backups).exists(), "no backup for a refusal");
    let warns = h
        .notes
        .lines
        .lock()
        .unwrap()
        .iter()
        .filter(|l| l.kind == "worktree-remove" && l.level == "warn")
        .count();
    assert!(warns >= 8, "{warns}");
}

fn dirty_old_unmerged(fx: &Fixture) {
    put(&fx.path("old-unmerged"), "old.txt", "older work, edited\n");
    put(&fx.path("old-unmerged"), "notes/todo.md", "not committed\n");
}

fn answered_by_decision() -> Vec<OwnerEntry> {
    let question = OwnerEntry {
        id: "E-1".into(),
        kind: "question".into(),
        author: "lead".into(),
        text: "Delete the two old worktrees old-unmerged and dirty-old?".into(),
        options: vec!["Delete both".into(), "Keep them".into()],
        ..OwnerEntry::default()
    };
    vec![
        question,
        OwnerEntry {
            kind: "decision".into(),
            reply_to: Some("E-1".into()),
            ..owner("O-ok", "Delete both")
        },
    ]
}

fn discard_by_decision() -> (Fixture, Harness, crate::request::RemoveOutcome, String) {
    let (fx, h) = setup(dirty_old_unmerged, |_| {});
    add_entries(&h, answered_by_decision());
    let head = h.docs()["old-unmerged"].head.clone().unwrap();
    let out = h
        .rm_with(&id(&h, &fx, "old-unmerged"), |i| {
            (i.discard, i.owner_entry, i.branch) = (true, Some("O-ok".into()), true);
            (i.head, i.reason) = (
                Some(head.clone()),
                Some("the owner picked Delete both".into()),
            );
        })
        .unwrap();
    (fx, h, out, head)
}

#[test]
fn the_owners_decision_answer_under_the_question_naming_the_folder_lets_the_discard_through() {
    let (fx, _h, out, _head) = discard_by_decision();
    assert_eq!((out.removed, out.deleted_branch), (true, true));
    assert_eq!(
        out.discarded,
        Some(Discarded {
            dirty: 2,
            unmerged: Some(1)
        })
    );
    assert!(!exists_itself(&fx.path("old-unmerged")));
    assert!(
        !branches(&fx.repo).contains(&"old-unmerged".to_string()),
        "-D deleted the unmerged branch"
    );
}

#[test]
fn the_bundle_the_patch_and_the_untracked_files_are_written_first() {
    let (_fx, _h, out, head) = discard_by_decision();
    let b = out.backup.clone().unwrap();
    assert!(
        b.base.starts_with(&format!("alpha-old-unmerged-{head}-")),
        "{}",
        b.base
    );
    assert_eq!(
        b.base.len(),
        "alpha-old-unmerged-".len() + head.len() + 1 + 12
    );
    let (bundle, patch, untracked) = (
        b.bundle.clone().unwrap(),
        b.patch.clone().unwrap(),
        b.untracked.clone().unwrap(),
    );
    assert!(bundle.ends_with(".bundle") && untracked.ends_with(".untracked.txt"));
    assert_eq!(
        Path::new(&patch).extension().and_then(|e| e.to_str()),
        Some("patch")
    );
    assert!(
        [&bundle, &patch, &untracked]
            .iter()
            .all(|f| exists_itself(f))
    );
    assert!(
        fs::read_to_string(&patch)
            .unwrap()
            .contains("older work, edited")
    );
    assert_eq!(
        fs::read_to_string(&untracked)
            .unwrap()
            .trim()
            .lines()
            .collect::<Vec<_>>(),
        ["notes/todo.md"]
    );
    let copy = format!("{}/notes/todo.md", b.untracked_copy.unwrap());
    assert_eq!(
        fs::read_to_string(copy).unwrap(),
        "not committed\n",
        "the untracked files are copied"
    );
}

#[test]
fn the_bundle_gives_the_commit_back_and_the_activity_line_says_how_to_restore() {
    let (fx, h, out, _head) = discard_by_decision();
    let b = out.backup.unwrap();
    assert!(
        b.restore[0].ends_with("old-unmerged:old-unmerged"),
        "{}",
        b.restore[0]
    );
    assert!(
        b.restore[0].contains(".bundle\" old-unmerged"),
        "{}",
        b.restore[0]
    );
    git_in(
        &fx.repo,
        &[
            "fetch",
            "-q",
            b.bundle.as_deref().unwrap(),
            "old-unmerged:old-unmerged",
        ],
        &[],
    );
    let subject = git_in(&fx.repo, &["log", "-1", "--format=%s", "old-unmerged"], &[]);
    assert_eq!(
        subject.trim(),
        "add old.txt",
        "the bundle gives the commit back"
    );
    let line = h
        .notes
        .removals("removed worktree")
        .into_iter()
        .find(|l| l.level == "info")
        .unwrap();
    assert!(line.text.contains("branch old-unmerged deleted: discarded with the owner's say (it was not merged), backup "), "{}", line.text);
    assert!(
        line.text.contains("(restore: git -C ")
            && line
                .text
                .ends_with(" for worker: the owner picked Delete both"),
        "{}",
        line.text
    );
}

#[test]
fn a_followup_that_names_the_folder_is_enough_and_the_page_itself_needs_no_entry() {
    let (fx, h) = std_setup();
    add_entries(
        &h,
        vec![owner("O-n", "you can remove **old-unmerged** now")],
    );
    let out = h
        .rm_with(&id(&h, &fx, "old-unmerged"), |i| {
            (i.discard, i.owner_entry, i.branch) = (true, Some("O-n".into()), true);
        })
        .unwrap();
    assert_eq!((out.removed, out.deleted_branch), (true, true));
    let b = out.backup.unwrap();
    assert!(
        b.bundle.is_some() && b.patch.is_none(),
        "clean: a bundle, no patch"
    );
    let target = id(&h, &fx, "dirty-old");
    let without = h.refused(&target, |i| i.caller = Caller::Page);
    assert!(
        without
            .message
            .contains("has 1 uncommitted change: removing it discards them"),
        "even the page sends discard"
    );
    let page = h
        .rm_with(&target, |i| (i.discard, i.caller) = (true, Caller::Page))
        .unwrap();
    let b = page.backup.unwrap();
    assert!(
        b.patch.is_some() && b.bundle.is_none(),
        "merged: no bundle, the patch and the untracked list"
    );
    assert_eq!(
        fs::read_to_string(b.untracked.unwrap())
            .unwrap()
            .trim()
            .lines()
            .collect::<Vec<_>>(),
        ["scratch.txt"]
    );
    assert!(
        h.notes
            .removals("removed worktree")
            .last()
            .unwrap()
            .text
            .ends_with("for the owner (page)")
    );
}

#[test]
fn a_backup_that_cannot_be_written_refuses_the_removal() {
    let (fx, h) = setup(loose, |_| {});
    add_entries(&h, vec![owner("O-n", "delete loose")]);
    put(
        &fx.root,
        "worktree-backups",
        "a file where the folder should be\n",
    );
    let err = h.refused(&id(&h, &fx, "loose"), |i| {
        (i.discard, i.owner_entry) = (true, Some("O-n".into()));
    });
    assert_eq!(
        (err.code, err.code.status()),
        (ErrorCode::BackupFailed, 500)
    );
    assert!(
        err.message.contains("the backup could not be written")
            && err.message.contains("so nothing was removed"),
        "{}",
        err.message
    );
    assert!(exists_itself(&fx.path("loose")) && listed(&fx.repo).contains(&"loose".to_string()));
    assert_eq!(
        h.notes.removals("refused to remove worktree")[0].level,
        "error"
    );
}

fn old_file(dir: &str, name: &str, ago_days: f64, offset_secs: u64) {
    let path = put(dir, name, "x");
    let when = days_ago(ago_days) + Duration::from_secs(offset_secs);
    set_mtime(Path::new(&path), when);
}

#[test]
fn backups_kept_are_the_last_30_or_the_last_30_days_whichever_is_more() {
    let (_fx, h) = std_setup();
    let dir = h.backups.clone();
    fs::create_dir_all(&dir).unwrap();
    for i in 0..34u64 {
        old_file(
            &dir,
            &format!("alpha-b{i}-abcdef0-2026010100{i:02}.bundle"),
            40.0,
            i * 60,
        );
    }
    for i in 0..3 {
        put(
            &dir,
            &format!("alpha-new{i}-abcdef0-202610250000.bundle"),
            "x",
        );
    }
    h.svc.prune_backups();
    let names = |needle: &str| {
        fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().contains(needle))
            .count()
    };
    assert_eq!(names(""), 30, "the 30 newest stay");
    assert_eq!(names("-new"), 3);
    for i in 0..40 {
        put(
            &dir,
            &format!("alpha-young{i}-abcdef0-202610250000.patch"),
            "x",
        );
    }
    h.svc.prune_backups();
    assert_eq!(names("young"), 40, "nothing younger than 30 days goes");
    assert_eq!(names("-b"), 0, "the old ones beyond 30 went");
}

#[test]
fn a_deleted_folder_leaves_the_docs_at_the_live_check_without_git_and_a_new_folder_joins_them() {
    let (fx, h) = std_setup();
    assert_eq!(h.docs()["fresh"].status, Status::Active);
    let before = h.svc.deps.git.stats().started;
    fs::remove_dir_all(fx.path("fresh")).unwrap();
    fs::remove_dir_all(fx.path("plain")).unwrap();
    let out = h.svc.live_step("alpha");
    assert_eq!(out.dropped, 2);
    assert_eq!(h.svc.deps.git.stats().started, before, "no git ran");
    assert!(!h.docs().contains_key("fresh") && !h.docs().contains_key("plain"));
    let meta = h.meta();
    assert_eq!(
        (meta.gone, meta.count),
        (2, 3),
        "the gone worktree is counted (git still lists it), the plain folder is simply forgotten"
    );
    fs::create_dir(fx.path("brand-new")).unwrap();
    let next = h.svc.live_step("alpha");
    assert_eq!(next.added, [fx.path("brand-new")]);
    assert_eq!(h.docs()["brand-new"].status, Status::Leftover);
    assert!(h.svc.live_step("alpha").added.is_empty(), "known now");
}

fn day_keep(h: &Harness) -> Keep {
    Keep {
        until_ms: h.clock_now() + DAY_MS,
        note: Some("n".into()),
        by: Some("worker".into()),
    }
}

fn sent_text(h: &Harness) -> String {
    h.store
        .kv
        .lock()
        .unwrap()
        .get(&sent_key("alpha"))
        .cloned()
        .unwrap_or_default()
}

#[test]
fn keeps_and_answers_of_worktrees_that_no_longer_exist_leave_the_board() {
    let fx = Fixture::new();
    let gone_id = "abcdef0123";
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], |_| {});
    h.board().keeps.insert(gone_id.into(), day_keep(&h));
    let about = EntryWorktree {
        id: gone_id.into(),
        path: "X:/nowhere".into(),
        kind: "check".into(),
        message: None,
    };
    add_entries(
        &h,
        vec![OwnerEntry {
            worktree: Some(about),
            ..owner("O-g", "")
        }],
    );
    h.board().answers.insert(
        "O-g".into(),
        crate::board::Answer {
            note: "gone".into(),
            at_ms: 1,
            by: None,
        },
    );
    h.scan();
    assert_eq!(
        *h.store.dropped.lock().unwrap(),
        vec![(vec![gone_id.to_string()], vec!["O-g".to_string()])]
    );
    assert!(h.board().keeps.is_empty() && h.board().answers.is_empty());
    assert!(
        h.board().entries.contains_key("O-g"),
        "the owner's entry itself stays on the board"
    );
}

#[test]
fn a_live_worktree_keeps_its_keep_and_the_sent_map_keeps_live_ones_only() {
    let (fx, h) = std_setup();
    let live = id(&h, &fx, "old-unmerged");
    h.board().keeps.insert(live.clone(), day_keep(&h));
    assert!(sent_text(&h).contains(&id(&h, &fx, "old-merged")));
    h.rm(&id(&h, &fx, "old-merged")).unwrap();
    h.scan();
    assert!(
        h.board().keeps.contains_key(&live),
        "a live worktree keeps its keep"
    );
    assert!(
        !sent_text(&h).contains(&id(&h, &fx, "old-merged")),
        "the removed one left the sent map"
    );
}

#[test]
fn a_scan_with_a_rule_error_prunes_nothing() {
    let fx = Fixture::new();
    let rules = vec![rule_for(&fx), Rule::folder("W-2", &fx.wts, "worker").repo()];
    let h = Harness::new(&fx.root, rules, |_| {});
    h.board().keeps.insert(
        "abcdef0123".into(),
        Keep {
            until_ms: h.clock_now() + DAY_MS,
            note: None,
            by: None,
        },
    );
    h.scan();
    assert!(h.store.dropped.lock().unwrap().is_empty());
    assert!(h.board().keeps.contains_key("abcdef0123"));
}

#[test]
fn over_many_at_live_folders_the_summary_says_so_and_one_sentence_goes_first_in_the_leads_wake_not_again_before_repeat_ms()
 {
    let (fx, h) = setup(|_| {}, |o| o.many_at = 2);
    let meta = h.meta();
    assert_eq!(
        (meta.count, meta.many_at, meta.too_many, meta.name.as_str()),
        (5, 2, true, "Alpha")
    );
    assert_eq!(
        h.lead_stale_text(),
        "Alpha has 5 folders: 4 git worktrees and 1 leftover: `coord worktree remove --all-leftovers` clears the old ones.",
        "git worktrees and leftovers are counted apart"
    );
    assert!(!h.wake_text("worker").contains("that is a lot"));
    h.notes.wakes.lock().unwrap().clear();
    h.scan();
    assert_eq!(h.notes.wake_count(), 0, "not again before repeat_ms");
    let one = Harness::new(&fx.root, vec![Rule::folder("W-1", &fx.wts, "lead")], |o| {
        o.many_at = 2;
    });
    one.scan();
    let text = one
        .wake_texts()
        .into_iter()
        .find(|t| t.starts_with("Alpha has"))
        .unwrap();
    assert!(text.starts_with("Alpha has 5 folders: 4 git worktrees and 1 leftover: `coord worktree remove --all-leftovers` clears the old ones. 3 worktrees need a decision: "), "first in the same wake line: {text}");
    assert!(text.chars().count() <= 300);
}
