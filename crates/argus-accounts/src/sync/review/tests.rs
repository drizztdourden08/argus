//! The review on temp fixtures: two accounts (`h` on `a1/o1`, `s` on `a2/o2`) and a cross scope
//! `a1/o2` the old tool made, a fake sidebar store, a fixed clock.

// test-tag: critical

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use serde_json::{Map, Value, json};
use tempfile::TempDir;

use super::*;
use crate::catalog::fixtures::{Fixture, record, scope, session_id};
use crate::catalog::plan::ChangeKind;
use crate::config::ArchiveRule;
use crate::fixtures::account_in;
use crate::ports::{Catalog, Clock, SidebarStore};
use crate::sidebar::store::CHANGED;
use crate::sidebar::{FolderStep, SidebarKeys, chat_folder};
use crate::sync::backup;
use crate::sync::job::{Env, Planned, Summary, scan, scan_and_plan};
use crate::sync::onto::{Route, scan_and_plan_onto};
use crate::sync::steps::{StepKind, StepSink, SyncStep};
use crate::types::Scope;

const H: &str = "a1/o1";
const S: &str = "a2/o2";
const NOW: i64 = 100 * 24 * 60 * 60 * 1000;

struct Fixed;

impl Clock for Fixed {
    fn now_ms(&self) -> i64 {
        NOW
    }
}

struct FakeSidebar(Mutex<Value>);

impl SidebarStore for FakeSidebar {
    fn read(&self) -> Result<Value, String> {
        Ok(self.0.lock().unwrap().clone())
    }

    fn write(&self, keys: &Value, scanned: &Value, _target: Option<&Scope>) -> Result<(), String> {
        let mut live = self.0.lock().unwrap();
        if *live != *scanned {
            return Err(CHANGED.to_owned());
        }
        *live = keys.clone();
        Ok(())
    }
}

#[derive(Default)]
struct Steps(Mutex<Vec<SyncStep>>);

impl StepSink for Steps {
    fn clear(&self) {
        self.0.lock().unwrap().clear();
    }

    fn put(&self, step: &SyncStep) {
        let mut all = self.0.lock().unwrap();
        all.retain(|s| s.seq != step.seq);
        all.push(step.clone());
    }
}

/// A folder: id, name, chat numbers in order.
type Folder<'a> = (&'a str, &'a str, &'a [u32]);

fn section(id: &str, kind: &str, name: &str, order: i64) -> Value {
    json!({"id": id, "kind": kind, "v": 2, "name": name, "order": order, "members": []})
}

fn keys(scopes: &[(&str, &[Folder])]) -> SidebarKeys {
    let (mut code, mut groups, mut sidebar) = (Map::new(), Map::new(), Map::new());
    for (scope, folders) in scopes {
        let mut sections = vec![section("pinned", "pinned", "Pinned", 0)];
        let (mut list, mut assignments, mut order) = (Vec::new(), Map::new(), Map::new());
        for (i, (id, name, chats)) in folders.iter().enumerate() {
            let mut s = section(id, "manual", name, i64::try_from(i).unwrap() + 1);
            let members: Vec<String> = chats
                .iter()
                .map(|n| format!("code:{}", session_id(*n)))
                .collect();
            s["members"] = json!(members);
            sections.push(s);
            list.push(json!({"id": id, "name": name, "mode": "code"}));
            for m in &members {
                assignments.insert(m.clone(), json!(id));
            }
            order.insert((*id).to_owned(), json!(members));
        }
        sections.push(section("sessions", "sessions", "Sessions", 9));
        let entry = json!({"groups": list, "assignments": assignments, "order": order});
        code.insert((*scope).to_owned(), json!({"sections": sections.clone()}));
        groups.insert((*scope).to_owned(), entry);
        sidebar.insert((*scope).to_owned(), json!({"sections": sections}));
    }
    SidebarKeys {
        code_sections: json!({"value": code, "tabId": "", "timestamp": 1}),
        group_scopes: json!({"value": groups.clone(), "tabId": "", "timestamp": 1}),
        dframe_store: json!({"state": {"customGroupsByScope": groups, "codeSidebarByScope": sidebar}, "version": 3}),
        encodings: [1, 1, 1],
    }
}

struct World {
    fx: Fixture,
    backups: TempDir,
    sidebar: FakeSidebar,
    accounts: Vec<crate::types::Account>,
}

fn rec(n: u32, activity: i64, focused: i64) -> Value {
    let mut r = record(n, activity);
    r["lastFocusedAt"] = json!(focused);
    r
}

fn h() -> Scope {
    scope("a1", "o1")
}

fn s() -> Scope {
    scope("a2", "o2")
}

fn cross() -> Scope {
    scope("a1", "o2")
}

impl World {
    /// Chat 1 only on h; chat 2 newer on s; chat 3 archived on h last; chat 4 in Alpha on h
    /// and in Beta on s, focused last on s; chat 5 a sub-thread of 4 on h only; folder `OnlyH` on
    /// h holding chat 1; chat 6 archived on both and old; chat 7 the same everywhere.
    fn new() -> Self {
        let fx = Fixture::new();
        let (h, s, c) = (h(), s(), cross());
        fx.put(&h, 1, &rec(1, 10, 10));
        let mut newer = rec(2, 20, 20);
        newer["title"] = json!("newer");
        fx.put(&h, 2, &rec(2, 10, 10));
        fx.put(&s, 2, &newer);
        let mut archived = rec(3, 10, 50);
        archived["isArchived"] = json!(true);
        fx.put(&h, 3, &archived);
        fx.put(&s, 3, &rec(3, 10, 10));
        fx.put(&c, 3, &rec(3, 10, 5));
        fx.put(&h, 4, &rec(4, 10, 10));
        fx.put(&s, 4, &rec(4, 10, 30));
        let mut child = rec(5, 10, 10);
        child["spawnedFrom"] = json!({"sessionId": session_id(4)});
        fx.put(&h, 5, &child);
        fx.put(&s, 5, &rec(5, 10, 10));
        let mut old = rec(6, 1000, 1000);
        old["isArchived"] = json!(true);
        fx.put(&h, 6, &old);
        fx.put(&s, 6, &old);
        fx.put(&c, 6, &old);
        for scope in [&h, &s, &c] {
            fx.put(scope, 7, &rec(7, 10, 10));
        }
        fx.write_index(&h, &[session_id(3), session_id(6)]);
        fx.write_index(&s, &[session_id(6)]);
        fx.write_index(&c, &[session_id(6)]);
        for n in 1..=7 {
            fx.transcript(n);
        }
        let k = keys(&[
            (
                H,
                &[
                    ("cg-ha", "Alpha", &[4]),
                    ("cg-hb", "Beta", &[]),
                    ("cg-ho", "OnlyH", &[1]),
                ],
            ),
            (S, &[("cg-sa", "Alpha", &[]), ("cg-sb", "Beta", &[4])]),
        ]);
        World {
            fx,
            backups: TempDir::new().unwrap(),
            sidebar: FakeSidebar(Mutex::new(k.to_value())),
            accounts: vec![
                account_in("h", "Hotmail", Some("a1".into()), Some("o1".into())),
                account_in("s", "Second", Some("a2".into()), Some("o2".into())),
            ],
        }
    }

    fn with_env<T>(&self, f: impl FnOnce(&Env) -> T) -> T {
        let catalog = self.fx.catalog();
        let transcripts = self.fx.transcripts();
        let env = Env {
            app: self.fx.root(),
            backups: self.backups.path(),
            accounts: &self.accounts,
            rule: ArchiveRule::Newest,
            catalog: &catalog,
            transcripts: &transcripts,
            sidebar: &self.sidebar,
            clock: &Fixed,
            running: &|| false,
        };
        f(&env)
    }

    fn auto(&self) -> Planned {
        self.with_env(|env| scan_and_plan(env).unwrap())
    }

    fn review(&self) -> Review {
        let auto = self.auto();
        build_review(&auto, &self.accounts)
    }

    fn plan(&self, request: &Request) -> Result<Planned, String> {
        let mut n = 0;
        self.with_env(|env| {
            plan_review(env, scan(env).unwrap(), request, &mut || {
                n += 1;
                format!("cg-new-{n}")
            })
        })
    }

    fn apply(&self, request: &Request) -> Result<Summary, String> {
        let steps = Steps::default();
        self.with_env(|env| run_review(env, request, &steps, &mut |_| {}))
    }

    fn apply_steps(&self, request: &Request) -> Vec<SyncStep> {
        let steps = Steps::default();
        self.with_env(|env| run_review(env, request, &steps, &mut |_| {}))
            .unwrap();
        steps.0.into_inner().unwrap()
    }

    fn file(&self, scope: &Scope, name: &str) -> PathBuf {
        self.fx
            .root()
            .join("claude-code-sessions")
            .join(&scope.account)
            .join(&scope.org)
            .join(name)
    }

    fn record(&self, scope: &Scope, n: u32) -> Option<Value> {
        let bytes = fs::read(self.file(scope, &format!("{}.json", session_id(n)))).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    fn keys(&self) -> SidebarKeys {
        SidebarKeys::from_value(&self.sidebar.read().unwrap()).unwrap()
    }

    fn folder(&self, scope: &str, n: u32) -> Option<String> {
        chat_folder(&self.keys(), scope, &session_id(n))
    }

    fn folders(&self, scope: &str) -> Vec<String> {
        group_names(&self.keys(), scope)
    }
}

fn decide(pairs: &[(&str, &str)]) -> Request {
    Request {
        decisions: pairs
            .iter()
            .map(|(r, c)| ((*r).to_owned(), (*c).to_owned()))
            .collect(),
        edits: Vec::new(),
    }
}

fn edits(list: Value) -> Request {
    Request {
        decisions: BTreeMap::new(),
        edits: serde_json::from_value(list).unwrap(),
    }
}

fn row<'a>(review: &'a Review, id: &str) -> &'a Row {
    review.rows.iter().find(|r| r.id == id).unwrap()
}

fn default_of(review: &Review, id: &str) -> (DiffKind, String) {
    let d = row(review, id).diff.as_ref().unwrap();
    (d.kind, d.default.clone())
}

fn changes_for(planned: &Planned, n: u32) -> usize {
    let id = session_id(n);
    let chats = planned
        .catalog
        .changes
        .iter()
        .filter(|c| c.session_id == id)
        .count();
    let folders = planned
        .folders
        .steps
        .iter()
        .filter(|s| matches!(s, FolderStep::AssignChat { session_id, .. } if *session_id == id))
        .count();
    chats + folders
}

/// The folder names of a scope, in order.
fn group_names(keys: &SidebarKeys, scope: &str) -> Vec<String> {
    keys.group_scopes["value"][scope]["groups"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g["name"].as_str().unwrap().to_owned())
        .collect()
}

/// Per scope: the folder names in order, and each chat's folder name.
fn shape(keys: &SidebarKeys) -> Vec<(Vec<String>, Vec<Option<String>>)> {
    [H, S]
        .iter()
        .map(|scope| {
            let names = group_names(keys, scope);
            let chats = (1..=7)
                .map(|n| chat_folder(keys, scope, &session_id(n)))
                .collect();
            (names, chats)
        })
        .collect()
}

fn creates(planned: &Planned, name: &str) -> bool {
    planned
        .folders
        .steps
        .iter()
        .any(|s| matches!(s, FolderStep::CreateFolder { name: n, .. } if n == name))
}

#[test]
fn every_row_is_listed_and_the_defaults_are_the_automatic_sync() {
    let world = World::new();
    let review = world.review();
    assert_eq!(review.rows.len(), 11, "4 folder rows and 7 chat rows");
    assert_eq!(review.counts.chats["h"], 7);
    assert_eq!(review.counts.chats["s"], 6);
    let missing = (DiffKind::ChatMissing, "copy_to:s".to_owned());
    assert_eq!(default_of(&review, &session_id(1)), missing);
    assert_eq!(
        default_of(&review, &session_id(2)),
        (DiffKind::History, "newest".into())
    );
    assert_eq!(
        default_of(&review, &session_id(3)),
        (DiffKind::Archive, "archive_all".into())
    );
    assert_eq!(
        default_of(&review, &session_id(5)),
        (DiffKind::Thread, "link_all".into())
    );
    let folder = (DiffKind::Membership, "folder_of:s".to_owned());
    assert_eq!(default_of(&review, &session_id(4)), folder);
    let only = (DiffKind::FolderMissing, "create_on:s".to_owned());
    assert_eq!(default_of(&review, "folder:OnlyH"), only);
    assert!(row(&review, &session_id(6)).diff.is_none());
    assert!(row(&review, "folder:Alpha").diff.is_none());
    let chat = row(&review, &session_id(4));
    assert_eq!(chat.folder_id.as_deref(), Some("folder:Beta"));
    assert_eq!(chat.cells["s"].last_focused_at, Some(30));
    assert_eq!(chat.cells["h"].last_activity_at, Some(10));
    assert_eq!(review.counts.differences, 6);
}

#[test]
fn no_choice_or_every_default_plans_exactly_the_automatic_sync() {
    let world = World::new();
    let auto = world.auto();
    let review = build_review(&auto, &world.accounts);
    let defaults: Vec<(String, String)> = review
        .rows
        .iter()
        .filter_map(|r| Some((r.id.clone(), r.diff.as_ref()?.default.clone())))
        .collect();
    let all = Request {
        decisions: defaults.into_iter().collect(),
        edits: Vec::new(),
    };
    for request in [Request::default(), all] {
        let planned = world.plan(&request).unwrap();
        assert_eq!(planned.catalog.changes, auto.catalog.changes);
        assert_eq!(planned.kinds, auto.kinds);
        // New folders get fresh ids each plan, so the folders compare by name.
        assert_eq!(shape(&planned.folders.keys), shape(&auto.folders.keys));
        assert_eq!(planned.folders.steps.len(), auto.folders.steps.len());
    }
}

#[test]
fn applying_the_defaults_leaves_nothing_for_the_automatic_sync() {
    let world = World::new();
    let summary = world.apply(&Request::default()).unwrap();
    assert!(summary.verified);
    assert!(summary.backup_dir.is_some());
    assert!(world.auto().is_empty());
}

#[test]
fn skip_leaves_the_row_untouched_and_the_next_automatic_plan_offers_it_again() {
    let world = World::new();
    let before = [h(), s(), cross()].map(|sc| world.record(&sc, 3));
    let request = decide(&[
        (&session_id(3), "skip"),
        (&session_id(1), "skip"),
        (&session_id(4), "skip"),
        ("folder:OnlyH", "skip"),
    ]);
    world.apply(&request).unwrap();
    assert_eq!([h(), s(), cross()].map(|sc| world.record(&sc, 3)), before);
    assert!(world.record(&s(), 1).is_none(), "not copied");
    assert_eq!(world.folder(H, 4).as_deref(), Some("Alpha"));
    assert_eq!(world.folder(S, 4).as_deref(), Some("Beta"));
    assert!(!world.folders(S).contains(&"OnlyH".to_owned()));
    // The copies still differ, so the automatic sync plans the same changes again.
    let again = world.auto();
    assert!(changes_for(&again, 3) > 0);
    assert!(changes_for(&again, 1) > 0);
    assert!(changes_for(&again, 4) > 0);
    assert!(creates(&again, "OnlyH"));
    // What the owner did not skip was written.
    assert_eq!(world.record(&h(), 2).unwrap()["title"], "newer");
}

#[test]
fn a_choice_that_makes_the_copies_equal_holds_on_the_next_automatic_plan() {
    let world = World::new();
    let request = decide(&[
        (&session_id(3), "unarchive_all"),
        (&session_id(2), "take:h"),
        (&session_id(4), "folder_of:h"),
        (&session_id(5), "unlink_all"),
        ("folder:OnlyH", "delete_on:h"),
    ]);
    world.apply(&request).unwrap();
    for sc in [h(), s(), cross()] {
        assert_eq!(world.record(&sc, 3).unwrap()["isArchived"], false);
        assert_eq!(world.record(&sc, 2).unwrap()["title"], "chat 2");
        assert!(world.record(&sc, 5).unwrap().get("spawnedFrom").is_none());
    }
    assert_eq!(world.folder(S, 4).as_deref(), Some("Alpha"));
    assert!(!world.folders(H).contains(&"OnlyH".to_owned()));
    assert_eq!(world.folder(H, 1), None, "its chats are unfiled");
    let again = world.auto();
    for n in [2, 3, 4, 5] {
        assert_eq!(changes_for(&again, n), 0, "chat {n}");
    }
    assert!(!creates(&again, "OnlyH"));
}

#[test]
fn link_all_makes_every_copy_a_sub_thread() {
    let world = World::new();
    let mut detached = rec(5, 10, 10);
    detached["spawnedFrom"] = json!({"sessionId": session_id(4)});
    detached["lineageDetached"] = json!(true);
    world.fx.put(&s(), 5, &detached);
    let review = world.review();
    assert_eq!(default_of(&review, &session_id(5)).1, "unlink_all");
    world
        .apply(&decide(&[(&session_id(5), "link_all")]))
        .unwrap();
    for sc in [h(), s(), cross()] {
        let r = world.record(&sc, 5).unwrap();
        assert_eq!(r["spawnedFrom"]["sessionId"], session_id(4));
        assert!(r.get("lineageDetached").is_none());
    }
    assert_eq!(changes_for(&world.auto(), 5), 0);
}

#[test]
fn unknown_rows_and_choices_are_refused() {
    let world = World::new();
    let err = world.plan(&decide(&[("local_nope", "skip")])).unwrap_err();
    assert!(err.contains("local_nope"), "{err}");
    let err = world
        .plan(&decide(&[(&session_id(3), "take:h")]))
        .unwrap_err();
    assert!(err.contains("take:h"), "{err}");
    let err = world
        .plan(&decide(&[(&session_id(7), "skip")]))
        .unwrap_err();
    assert!(err.contains("nothing to decide"), "{err}");
}

#[test]
fn move_chat_files_it_on_that_account_and_its_cross_scopes() {
    let world = World::new();
    let request = edits(json!([
        {"op": "move_chat", "account": "s", "sessionId": session_id(7), "folder": "Alpha"},
        {"op": "move_chat", "account": "h", "sessionId": session_id(4), "folder": null},
    ]));
    let steps = world.apply_steps(&request);
    assert_eq!(world.folder(S, 7).as_deref(), Some("Alpha"));
    assert_eq!(world.folder(H, 4), None);
    let moves = steps
        .iter()
        .filter(|s| s.kind == StepKind::MoveChat)
        .count();
    assert_eq!(moves, 2);
    let store = &world.keys().dframe_store["state"]["customGroupsByScope"][H]["assignments"];
    assert!(store.get(format!("code:{}", session_id(4))).is_none());
}

#[test]
fn archive_and_unarchive_follow_records_and_index() {
    let world = World::new();
    let request = edits(json!([
        {"op": "archive", "account": "s", "sessionId": session_id(7)},
        {"op": "unarchive", "account": "h", "sessionId": session_id(6)},
    ]));
    world.apply(&request).unwrap();
    assert_eq!(world.record(&s(), 7).unwrap()["isArchived"], true);
    assert_eq!(world.record(&h(), 6).unwrap()["isArchived"], false);
    assert_eq!(
        world.record(&cross(), 6).unwrap()["isArchived"],
        false,
        "cross scope"
    );
    assert_eq!(
        world.record(&s(), 6).unwrap()["isArchived"],
        true,
        "other account"
    );
    let catalog = world.fx.catalog();
    assert!(catalog.archived(&s()).unwrap().contains(&session_id(7)));
    assert!(!catalog.archived(&h()).unwrap().contains(&session_id(6)));
}

#[test]
fn delete_chat_writes_the_apps_tombstone_and_undo_brings_the_record_back() {
    let world = World::new();
    let original = fs::read(world.file(&h(), &format!("{}.json", session_id(6)))).unwrap();
    let request = edits(json!([{"op": "delete_chat", "account": "h", "sessionId": session_id(6)}]));
    let summary = world.apply(&request).unwrap();
    let uuid = session_id(6).replace("local_", "");
    for sc in [h(), cross()] {
        assert!(world.record(&sc, 6).is_none());
        let tomb = fs::read_to_string(world.file(&sc, &format!("deleted_{uuid}"))).unwrap();
        assert_eq!(tomb, NOW.to_string());
    }
    assert!(
        world.record(&s(), 6).is_some(),
        "the other account keeps it"
    );
    assert!(
        !world
            .fx
            .catalog()
            .archived(&h())
            .unwrap()
            .contains(&session_id(6))
    );
    assert_eq!(
        changes_for(&world.auto(), 6),
        0,
        "the tombstone blocks a copy back"
    );
    let dir = PathBuf::from(summary.backup_dir.unwrap());
    let report = backup::undo(&dir, false).unwrap();
    assert!(report.kept.is_empty(), "{:?}", report.kept);
    let back = fs::read(world.file(&h(), &format!("{}.json", session_id(6)))).unwrap();
    assert_eq!(back, original);
    assert!(!world.file(&h(), &format!("deleted_{uuid}")).exists());
    assert!(
        world
            .fx
            .catalog()
            .archived(&h())
            .unwrap()
            .contains(&session_id(6))
    );
}

#[test]
fn delete_chat_takes_active_chats_too_but_only_known_ones() {
    let world = World::new();
    let active = edits(json!([{"op": "delete_chat", "account": "h", "sessionId": session_id(7)}]));
    assert!(world.plan(&active).is_ok());
    let unknown = edits(json!([{"op": "delete_chat", "account": "s", "sessionId": session_id(1)}]));
    assert!(world.plan(&unknown).unwrap_err().contains(&session_id(1)));
    let nobody = edits(json!([{"op": "archive", "account": "x", "sessionId": session_id(1)}]));
    assert!(
        world
            .plan(&nobody)
            .unwrap_err()
            .contains("unknown account x")
    );
    let folder = edits(
        json!([{"op": "move_chat", "account": "h", "sessionId": session_id(7), "folder": "Nope"}]),
    );
    assert!(
        world
            .plan(&folder)
            .unwrap_err()
            .contains("unknown folder Nope")
    );
    let gone = edits(json!([{"op": "delete_folder", "account": "s", "folder": "Nope"}]));
    assert!(
        world
            .plan(&gone)
            .unwrap_err()
            .contains("unknown folder Nope")
    );
}

#[test]
fn folder_edits_rename_delete_and_create() {
    let world = World::new();
    let request = Request {
        decisions: BTreeMap::from([("folder:OnlyH".to_owned(), "skip".to_owned())]),
        edits: serde_json::from_value(json!([
            {"op": "rename_folder", "account": "s", "folder": "Alpha", "name": "First"},
            {"op": "delete_folder", "account": "s", "folder": "Beta"},
            {"op": "create_folder", "account": "h", "name": "Fresh"},
            {"op": "move_chat", "account": "h", "sessionId": session_id(1), "folder": "Fresh"},
        ]))
        .unwrap(),
    };
    let steps = world.apply_steps(&request);
    assert_eq!(world.folders(S), ["First"]);
    assert_eq!(world.folder(S, 4), None, "Beta's chats are unfiled");
    assert_eq!(world.folders(H), ["Alpha", "Beta", "OnlyH", "Fresh"]);
    assert_eq!(world.folder(H, 1).as_deref(), Some("Fresh"));
    let keys = world.keys();
    let fresh = keys.group_scopes["value"][H]["groups"][3]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(fresh.starts_with("cg-") && fresh.len() == 39, "{fresh}");
    let sections = keys.code_sections["value"][H]["sections"]
        .as_array()
        .unwrap();
    assert_eq!(sections.last().unwrap()["order"], 10);
    let mirror = &keys.dframe_store["state"]["codeSidebarByScope"][S]["sections"];
    assert!(
        mirror
            .as_array()
            .unwrap()
            .iter()
            .all(|s| s["name"] != "Beta")
    );
    let kinds: Vec<StepKind> = steps.iter().map(|s| s.kind).collect();
    for kind in [
        StepKind::RenameFolder,
        StepKind::DeleteFolder,
        StepKind::CreateFolder,
    ] {
        assert!(kinds.contains(&kind), "{kind:?}");
    }
}

#[test]
fn a_review_with_nothing_to_write_does_not_back_up() {
    let world = World::new();
    world.apply(&Request::default()).unwrap();
    let summary = world.apply(&Request::default()).unwrap();
    assert!(summary.verified);
    assert!(summary.backup_dir.is_none());
}

#[test]
fn cleanup_lists_archived_chats_older_than_the_limit() {
    let world = World::new();
    let scanned = world.with_env(|env| scan(env).unwrap());
    let found = cleanup(&scanned, &world.accounts, NOW, 30);
    let ids: Vec<(&str, &str)> = found
        .candidates
        .iter()
        .map(|c| (c.account.as_str(), c.session_id.as_str()))
        .collect();
    let (three, six) = (session_id(3), session_id(6));
    let want = [
        ("h", three.as_str()),
        ("h", six.as_str()),
        ("s", six.as_str()),
    ];
    assert_eq!(
        ids, want,
        "archived, oldest first per account; chat 7 is active"
    );
    assert_eq!(found.candidates[1].last_activity_at, 1000);
    assert!(
        cleanup(&scanned, &world.accounts, 2000, 30)
            .candidates
            .is_empty()
    );
    assert_eq!(found.days, 30);
}

#[test]
fn a_run_refuses_while_claude_runs() {
    let world = World::new();
    let catalog = world.fx.catalog();
    let transcripts = world.fx.transcripts();
    let env = Env {
        app: world.fx.root(),
        backups: world.backups.path(),
        accounts: &world.accounts,
        rule: ArchiveRule::Newest,
        catalog: &catalog,
        transcripts: &transcripts,
        sidebar: &world.sidebar,
        clock: &Fixed,
        running: &|| true,
    };
    let steps = Steps::default();
    let err = run_review(&env, &Request::default(), &steps, &mut |_| {}).unwrap_err();
    assert!(err.contains("running"), "{err}");
    assert!(world.record(&s(), 1).is_none());
}

#[test]
fn delete_chat_changes_are_planned_as_deletes() {
    let world = World::new();
    let request = edits(json!([{"op": "delete_chat", "account": "h", "sessionId": session_id(6)}]));
    let planned = world.plan(&request).unwrap();
    let deletes: Vec<&Scope> = planned
        .catalog
        .changes
        .iter()
        .filter(|c| c.kind == ChangeKind::DeleteSession)
        .map(|c| &c.scope)
        .collect();
    assert_eq!(deletes, [&h(), &cross()]);
    assert!(planned.kinds.contains(&StepKind::DeleteChat));
}

#[test]
fn a_copy_passes_its_target_to_the_sidebar_write_and_a_merge_passes_none() {
    let world = World::new();
    let route = Route {
        source: h(),
        target: s(),
    };
    let onto = world.with_env(|env| scan_and_plan_onto(env, &route).unwrap());
    assert_eq!(onto.target, Some(s()));
    assert_eq!(world.auto().target, None);
}
