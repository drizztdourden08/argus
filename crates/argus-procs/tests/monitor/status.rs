//! K1: Status, protection and docs from fake samples.
// test-tag: critical

use super::harness::{Harness, MIN, Setup, Spec, T0, proc};
use argus_procs::{Flag, Guard, ProcRow, RuleSummary, Status};
use serde_json::{Value, json};

fn rules() -> Value {
    json!([
        {"name": "dev", "match": {"name": "^node", "cmd": "vite"}, "maxMemMB": 1000, "maxAgeMin": 60, "idleMin": 1, "orphan": true, "hung": true, "killable": true, "maxGroupMemMB": 2500},
        {"name": "shy", "match": {"name": "^cmd"}, "onlyFlagged": true, "orphan": true, "killable": true},
        {"name": "big", "match": {}, "minMemMB": 2000, "maxMemMB": 2000, "killable": true}
    ])
}

fn with_rules() -> Harness {
    Harness::new(Setup {
        rules: Some(rules()),
        ..Setup::default()
    })
}

fn mixed(t: i64) -> Vec<ProcRow> {
    vec![
        proc(Spec {
            mem: 1500,
            ..Spec::new(201)
        }),
        proc(Spec {
            ppid: 9999,
            ..Spec::new(202)
        }),
        proc(Spec {
            start: T0 - 120 * MIN,
            cpu: t - T0,
            ..Spec::new(203)
        }),
        proc(Spec {
            responding: Some(false),
            cpu: t - T0,
            ..Spec::new(204)
        }),
        proc(Spec {
            name: "cmd.exe",
            cmd: "cmd /c x",
            ..Spec::new(205)
        }),
        proc(Spec {
            name: "cmd.exe",
            cmd: "cmd /c y",
            ppid: 9999,
            ..Spec::new(206)
        }),
        proc(Spec {
            name: "game.exe",
            cmd: "game",
            path: "D:\\g.exe",
            mem: 1500,
            ..Spec::new(207)
        }),
    ]
}

#[test]
fn hog_orphan_old_hung_and_the_listing_filters() {
    let h = with_rules();
    h.at(T0, mixed(T0));
    let mut pids: Vec<u32> = h.service.list(None).iter().map(|d| d.pid).collect();
    pids.sort_unstable();
    assert_eq!(pids, [201, 202, 203, 204, 206]);
    assert_eq!(h.doc(201).status, Status::Hog);
    assert_eq!(h.doc(202).status, Status::Orphan);
    assert!(!h.doc(202).parent_alive);
    assert_eq!(h.doc(203).status, Status::Old);
    assert_eq!(h.doc(204).status, Status::Hung);
    assert_eq!(h.doc(201).cpu_pct, None);
    assert!(!h.doc(201).flags.contains(&Flag::Group));
}

#[test]
fn cpu_between_two_samples_and_stale_after_idle_min() {
    let h = with_rules();
    h.at(T0, mixed(T0));
    h.at(T0 + 30_000, mixed(T0 + 30_000));
    assert_eq!(h.doc(203).cpu_pct, Some(100.0));
    assert_eq!(h.doc(201).cpu_pct, Some(0.0));
    assert_eq!(h.doc(201).idle_min, 0);
    h.at(T0 + 90_000, mixed(T0 + 90_000));
    assert_eq!(h.doc(201).idle_min, 1);
    let flags: Vec<Flag> = h
        .doc(201)
        .flags
        .into_iter()
        .filter(|f| *f != Flag::Group)
        .collect();
    assert_eq!(flags, [Flag::Hog, Flag::Stale]);
    assert!(!h.doc(203).flags.contains(&Flag::Stale));
}

#[test]
fn a_doc_carries_its_id_times_rule_hint_project_and_status_start() {
    let h = with_rules();
    h.at(T0, mixed(T0));
    h.at(T0 + 30_000, mixed(T0 + 30_000));
    h.at(T0 + 90_000, mixed(T0 + 90_000));
    let d = h.doc(201);
    assert_eq!(d.id, format!("201-{}", T0 - 10 * MIN));
    assert_eq!(d.started_at.as_deref(), Some("2026-10-25T11:50:00.000Z"));
    assert_eq!(d.age_min, 11);
    assert_eq!(d.rule, "dev");
    assert_eq!(d.hint.as_deref(), Some("vite"));
    assert_eq!(d.project.as_deref(), Some("alpha"));
    assert_eq!(d.mem_mb, 1500);
    assert!(d.killable);
    assert!(!d.protected);
    assert_eq!(d.since, "2026-10-25T12:00:00.000Z");
}

#[test]
fn the_group_flag_the_meta_and_the_published_docs() {
    let h = with_rules();
    let three = |pid| {
        proc(Spec {
            mem: 900,
            ..Spec::new(pid)
        })
    };
    let meta = h.at(T0, vec![three(301), three(302), three(303)]);
    assert!(
        h.service
            .list(None)
            .iter()
            .all(|d| d.flags.contains(&Flag::Group))
    );
    assert_eq!(meta.count, 3);
    assert_eq!(meta.total_mem_mb, 2700);
    let summary = |rule: &str, count, matched, mem_mb, max, over| RuleSummary {
        rule: rule.to_string(),
        count,
        matched,
        mem_mb,
        max_group_mem_mb: max,
        over,
    };
    assert_eq!(
        meta.by_rule,
        [
            summary("dev", 3, 3, 2700, Some(2500.0), true),
            summary("big", 0, 1, 100, None, false)
        ]
    );
    assert_eq!(meta.platform, "win32");
    assert_eq!(meta.error, None);
    assert_eq!(meta.rules.len(), 3);

    let (docs, stored) = h.fake.published().unwrap();
    let mut pids: Vec<u32> = docs.iter().map(|d| d.pid).collect();
    pids.sort_unstable();
    assert_eq!(pids, [301, 302, 303]);
    assert_eq!(stored.unwrap().count, 3);

    h.at(T0 + 30_000, vec![three(301)]);
    let (docs, _) = h.fake.published().unwrap();
    assert_eq!(docs.iter().map(|d| d.pid).collect::<Vec<_>>(), [301]);

    h.fake.wipe();
    h.service.start();
    h.service.republish();
    assert_eq!(h.fake.published().unwrap().0.len(), 1);
    h.service.stop();
}

fn guard_fixture() -> Vec<ProcRow> {
    let ps = |pid, name, cmd| Spec {
        name,
        cmd,
        ..Spec::new(pid)
    };
    vec![
        proc(Spec {
            path: "C:\\Windows\\System32\\svchost.exe",
            ..ps(401, "svchost.exe", "svchost -k x")
        }),
        proc(Spec {
            ppid: 403,
            ..ps(
                402,
                "node.exe",
                "node x:/ai-config/dashboard/bin/dashboard.mjs start",
            )
        }),
        proc(Spec {
            start: T0 - 20 * MIN,
            ..ps(403, "cmd.exe", "cmd /c start")
        }),
        proc(ps(404, "claude.exe", "claude")),
        proc(ps(405, "powershell.exe", "powershell -File tray.ps1")),
        proc(Spec {
            path: "C:\\Windows\\thing.exe",
            ..ps(406, "thing.exe", "thing")
        }),
        proc(ps(407, "node.exe", "node x:/a/coord.mjs hook listen")),
        proc(Spec {
            path: "D:\\free.exe",
            ..ps(408, "free.exe", "free")
        }),
        proc(ps(409, "powershell.exe", "powershell -EncodedCommand ...")),
        proc(Spec {
            ppid: 409,
            start: T0 - 5 * MIN,
            ..ps(410, "conhost.exe", "conhost")
        }),
    ]
}

#[test]
fn protected_system_names_windows_claude_the_dashboard_chain_the_tray_and_command_lines() {
    let all = json!([{"name": "all", "match": {}, "killable": true}]);
    let h = Harness::new(Setup {
        rules: Some(all),
        self_pid: Some(402),
        tray: Some(405),
        sampler_pid: Some(409),
        ..Setup::default()
    });
    h.at(T0, guard_fixture());
    let mut why: Vec<(u32, Option<Guard>)> = h
        .service
        .list(None)
        .iter()
        .map(|d| (d.pid, d.protected_why))
        .collect();
    why.sort_by_key(|(pid, _)| *pid);
    assert_eq!(
        why,
        [
            (100, Some(Guard::System)),
            (401, Some(Guard::System)),
            (402, Some(Guard::Dashboard)),
            (403, Some(Guard::Dashboard)),
            (404, Some(Guard::Claude)),
            (405, Some(Guard::Tray)),
            (406, Some(Guard::Windows)),
            (407, Some(Guard::Dashboard)),
            (408, None),
        ]
    );
    assert!(h.doc(408).killable);
    assert!(!h.doc(401).killable);
}

#[test]
fn a_failed_sample_is_recorded_in_the_meta_warned_once_and_the_listing_stays() {
    let h = with_rules();
    h.at(T0, vec![proc(Spec::new(501))]);
    h.fake.fail("powershell.exe failed: boom");
    let _ = h.service.sample_now();
    let meta = h.service.sample_now();
    assert_eq!(meta.error.as_deref(), Some("powershell.exe failed: boom"));
    assert_eq!(
        h.fake
            .warns()
            .iter()
            .filter(|w| *w == "process sample failed")
            .count(),
        1
    );
    assert_eq!(h.service.list(None).len(), 1);
    let stored = h.fake.published().unwrap().1.unwrap();
    assert_eq!(stored.error.as_deref(), Some("powershell.exe failed: boom"));
}
