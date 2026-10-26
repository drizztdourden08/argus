//! The launcher and bundle rules over a board (launcher.verbs and build.verbs, the pure
//! parts): the usage window, the three kinds of launcher, Play definitions, who may write,
//! retire and restore with a bundle, keep, the status of a bundle and the hand rules.

use argus_launchers::RunKind;
use argus_launchers::launcher::BundleStatus::{Draft, Ready};
use argus_launchers::launcher::{
    Bundle, BundleStatus, EVERY_HELP, Launcher, LauncherBoard, LauncherPatch, LinkState, Platform,
    Role, ThreadInfo, Who, WriteContext, authorize_launcher, bundle_status, command_warnings,
    duration_text, has_master, is_standalone, joined_bundle, keep_launcher, launcher_slug,
    listed_status, parse_every, require_launcher_name, resolve_bundle, resolve_path,
    retire_launcher, shell_of, usage_of, write_launcher,
};

pub const AT: &str = "2026-10-24T00:00:00.000Z";
const DAY: i64 = 86_400_000;
const HOUR: i64 = 3_600_000;

pub fn thread(id: &str, role: &str) -> ThreadInfo {
    ThreadInfo {
        id: id.to_owned(),
        role: role.to_owned(),
        status: "active".to_owned(),
        ..ThreadInfo::default()
    }
}

pub fn lead() -> Who {
    Who {
        actor: "coordinator".to_owned(),
        role: Role::Coordinator,
        thread: Some(thread("coordinator", "coordinator")),
    }
}

pub fn worker(id: &str) -> Who {
    Who {
        actor: id.to_owned(),
        role: Role::Thread,
        thread: Some(thread(id, "thread")),
    }
}

pub fn launcher(name: &str) -> Launcher {
    Launcher {
        name: name.to_owned(),
        cmd: Some(format!("run {name}")),
        created_at: AT.to_owned(),
        updated_at: AT.to_owned(),
        created_by: Some("worker".to_owned()),
        by: Some("worker".to_owned()),
        use_every: Some("7d".to_owned()),
        ..Launcher::default()
    }
}

pub fn board() -> LauncherBoard {
    let mut board = LauncherBoard {
        project: Some("alpha".to_owned()),
        created_at: Some(AT.to_owned()),
        ..LauncherBoard::default()
    };
    for (id, role) in [
        ("coordinator", "coordinator"),
        ("worker", "thread"),
        ("other", "thread"),
    ] {
        board.threads.insert(id.to_owned(), thread(id, role));
    }
    board
        .launchers
        .insert("master".to_owned(), launcher("master"));
    board
}

fn bundle(tag: &str) -> Bundle {
    Bundle {
        tag: tag.to_owned(),
        ..Bundle::default()
    }
}

fn ctx() -> WriteContext<'static> {
    WriteContext {
        cwd: "/somewhere",
        bundle: None,
        at: AT,
    }
}

fn set_cmd(cmd: &str) -> LauncherPatch {
    LauncherPatch {
        cmd: Some(Some(cmd.to_owned())),
        ..LauncherPatch::default()
    }
}

fn with_every(cmd: &str) -> LauncherPatch {
    LauncherPatch {
        use_every: Some(Some("7d".to_owned())),
        ..set_cmd(cmd)
    }
}

#[test]
fn parse_every_takes_hours_days_weeks_and_months_of_30_days_from_1h_to_365d() {
    for (text, ms) in [
        ("12h", 12 * HOUR),
        (" 3D ", 3 * DAY),
        ("2w", 14 * DAY),
        ("1mo", 30 * DAY),
        ("1h", HOUR),
        ("365d", 365 * DAY),
        ("12mo", 360 * DAY),
    ] {
        let every = parse_every(text).unwrap();
        assert_eq!(
            (every.text, every.ms),
            (text.trim().to_lowercase(), ms),
            "{text}"
        );
    }
    for bad in [
        "0h", "366d", "13mo", "53w", "9999h", "7", "d", "1.5d", "-2d", "2 d", "", "weekly", "1y",
        "30m",
    ] {
        let message = parse_every(bad).unwrap_err().to_string();
        assert!(message.starts_with("--use-every "), "{bad}: {message}");
        assert!(
            message.contains("is not a duration") || message.contains("is out of range"),
            "{bad}"
        );
        assert!(message.ends_with(&format!("{EVERY_HELP}.")), "{bad}");
    }
}

#[test]
fn durations_read_in_minutes_hours_or_days() {
    assert_eq!(duration_text(0), "1m");
    assert_eq!(duration_text(40 * 60_000), "40m");
    assert_eq!(duration_text(9 * HOUR), "9h");
    assert_eq!(duration_text(47 * HOUR), "47h");
    assert_eq!(duration_text(48 * HOUR), "2d");
    assert_eq!(duration_text(9 * DAY), "9d");
}

fn aged(t0: i64, change: impl FnOnce(&mut Launcher)) -> Launcher {
    let mut rec = launcher("tool");
    rec.created_at = argus_launchers::format_iso(t0);
    rec.updated_at = argus_launchers::format_iso(t0 + DAY);
    change(&mut rec);
    rec
}

#[test]
fn usage_last_used_is_the_newest_of_play_update_create_and_keep_and_stale_is_past_the_window() {
    let t0 = 1_790_000_000_000_i64;
    let iso = argus_launchers::format_iso;
    let board = LauncherBoard::default();
    let at = |hours: i64, rec: &Launcher, played: i64| {
        usage_of(&board, rec, t0 + hours * HOUR, played).unwrap()
    };
    assert!(
        !at(8 * 24, &aged(t0, |_| {}), 0).stale,
        "updated on day 1: 7 days of use end on day 8"
    );
    let late = at(204, &aged(t0, |_| {}), 0);
    assert!(late.stale);
    assert_eq!(late.last_used_at, iso(t0 + DAY));
    assert!(
        !at(204, &aged(t0, |_| {}), t0 + 5 * DAY).stale,
        "a Play run on day 5 counts"
    );
    assert!(at(300, &aged(t0, |_| {}), t0 + 5 * DAY).stale);
    let kept = |r: &mut Launcher| r.kept_at = Some(iso(t0 + 11 * DAY));
    assert!(!at(300, &aged(t0, kept), 0).stale, "a keep counts as a use");
    let held = |r: &mut Launcher| {
        kept(r);
        r.kept_until = Some(iso(t0 + 40 * DAY));
    };
    assert!(
        !at(720, &aged(t0, held), 0).stale,
        "kept for days: fresh until then"
    );
    assert!(at(984, &aged(t0, held), 0).stale);
    assert!(
        !at(
            2400,
            &aged(t0, |r| r.use_every = Some("365d".to_owned())),
            0
        )
        .stale
    );
}

#[test]
fn usage_a_launcher_without_the_field_counts_as_7d_and_master_joined_and_retired_have_none() {
    let t0 = 1_790_000_000_000_i64;
    let board = LauncherBoard::default();
    let mut legacy = launcher("tool");
    legacy.use_every = None;
    legacy.created_at = argus_launchers::format_iso(t0);
    legacy.updated_at = legacy.created_at.clone();
    let usage = usage_of(&board, &legacy, t0 + 9 * DAY, 0).unwrap();
    assert_eq!(
        (usage.use_every.as_str(), usage.legacy, usage.stale),
        ("7d", true, true)
    );
    assert!(
        usage_of(&board, &launcher("master"), t0, 0).is_none(),
        "master has no window"
    );
    let mut retired = launcher("tool");
    retired.retired_at = Some(AT.to_owned());
    assert!(usage_of(&board, &retired, t0, 0).is_none());
    let made = Launcher {
        bundle: Some("b1".to_owned()),
        ..launcher("tool")
    };
    assert!(
        usage_of(&board, &made, t0, 0).is_none(),
        "made for a bundle"
    );
}

#[test]
fn usage_a_launcher_linked_by_an_open_bundle_is_joined_and_standalone_again_when_it_closes() {
    let mut board = LauncherBoard::default();
    board.builds.insert(
        "b2".to_owned(),
        Bundle {
            launcher: Some("tool".to_owned()),
            ..bundle("b2")
        },
    );
    let tool = launcher("tool");
    assert_eq!(joined_bundle(&board, &tool).as_deref(), Some("b2"));
    assert!(
        usage_of(&board, &tool, i64::MAX / 2, 0).is_none(),
        "linked by an open bundle"
    );
    board.builds.get_mut("b2").unwrap().closed_at = Some(AT.to_owned());
    assert!(is_standalone(&board, &tool));
    assert!(usage_of(&board, &tool, i64::MAX / 2, 0).unwrap().stale);
    assert_eq!(
        joined_bundle(&board, &launcher("master")),
        None,
        "master is never joined"
    );
}

#[test]
fn master_is_mandatory_a_live_master_counts_a_retired_one_does_not() {
    let mut board = board();
    assert!(has_master(&board));
    board.launchers.get_mut("master").unwrap().retired_at = Some(AT.to_owned());
    assert!(!has_master(&board));
    board.launchers.clear();
    assert!(!has_master(&board));
}

#[test]
fn the_status_of_a_bundle_without_one_reads_ready_when_handed_or_starting_something() {
    let mut board = board();
    board.handed_build = Some("old-handed".to_owned());
    let records = [
        (
            "old-launcher",
            Bundle {
                launcher: Some("master".to_owned()),
                ..bundle("old-launcher")
            },
        ),
        (
            "old-cmd",
            Bundle {
                cmd: Some("rotp launch".to_owned()),
                ..bundle("old-cmd")
            },
        ),
        ("old-handed", bundle("old-handed")),
        ("old-bare", bundle("old-bare")),
        (
            "was-handed",
            Bundle {
                last_handed_at: Some(AT.to_owned()),
                ..bundle("was-handed")
            },
        ),
        (
            "explicit",
            Bundle {
                status: Some("draft".to_owned()),
                launcher: Some("master".to_owned()),
                ..bundle("explicit")
            },
        ),
    ];
    let statuses: Vec<BundleStatus> = records
        .iter()
        .map(|(tag, rec)| bundle_status(&board, tag, Some(rec)))
        .collect();
    assert_eq!(statuses, [Ready, Ready, Ready, Draft, Ready, Draft]);
    assert_eq!(bundle_status(&board, "missing", None), Draft);
    let closed = Bundle {
        closed_at: Some(AT.to_owned()),
        ..bundle("old-bare")
    };
    assert_eq!(
        listed_status(&board, "old-bare", &closed),
        BundleStatus::Closed
    );
    assert_eq!(BundleStatus::Closed.as_str(), "closed");
}

#[test]
fn a_bundle_starts_its_linked_launcher_or_its_legacy_command() {
    let mut board = board();
    board.launchers.insert(
        "gone".to_owned(),
        Launcher {
            retired_at: Some(AT.to_owned()),
            ..launcher("gone")
        },
    );
    let link = |name: &str| Bundle {
        launcher: Some(name.to_owned()),
        ..bundle("b")
    };
    let linked = resolve_bundle(&board, Some(&link("master")));
    assert_eq!(
        (linked.state, linked.cmd.as_deref()),
        (LinkState::Linked, Some("run master"))
    );
    assert_eq!(
        resolve_bundle(&board, Some(&link("gone"))).state,
        LinkState::Retired
    );
    assert_eq!(
        resolve_bundle(&board, Some(&link("nope"))).state,
        LinkState::Missing
    );
    let legacy = Bundle {
        cmd: Some("rotp launch".to_owned()),
        cwd: Some("sub".to_owned()),
        ..bundle("b")
    };
    let resolved = resolve_bundle(&board, Some(&legacy));
    assert_eq!(
        (resolved.state, resolved.cwd.as_deref()),
        (LinkState::Legacy, Some("sub"))
    );
    assert_eq!(
        resolve_bundle(&board, Some(&bundle("b"))).state,
        LinkState::None
    );
    assert_eq!(resolve_bundle(&board, None).state, LinkState::None);
    assert_eq!(LinkState::Missing.as_str(), "missing");
}

#[test]
fn a_launcher_name_is_a_short_slug_and_a_bundle_tag_becomes_one() {
    assert_eq!(require_launcher_name(" build-2 ").unwrap(), "build-2");
    for bad in ["Build_2".to_owned(), "x".repeat(32)] {
        assert!(
            require_launcher_name(&bad)
                .unwrap_err()
                .to_string()
                .contains("is not a launcher name")
        );
    }
    assert!(require_launcher_name("x".repeat(31).as_str()).is_ok());
    assert!(
        require_launcher_name("  ")
            .unwrap_err()
            .to_string()
            .starts_with("name the launcher")
    );
    assert_eq!(launcher_slug("Bundle 4!").as_deref(), Some("bundle-4"));
    assert_eq!(launcher_slug("---"), None);
    let long = launcher_slug(&format!("{}-tail", "a".repeat(30))).unwrap();
    assert_eq!(long, "a".repeat(30), "cut to 31 and the dash trimmed");
}

#[test]
fn the_shell_is_named_and_habits_from_another_shell_get_a_warning_never_a_refusal() {
    assert_eq!(shell_of(Platform::Windows), "cmd.exe /d /s /c");
    assert_eq!(shell_of(Platform::Other), "sh -c");
    assert!(
        argus_launchers::launcher::runs_note(Platform::Windows)
            .contains("cmd.exe /d /s /c from the launcher's folder (--cwd), hidden")
    );
    assert!(command_warnings("bxel build --if-stale && bxel start", Platform::Windows).is_empty());
    let bad = command_warnings(
        "export NODE_ENV=dev; cd apps && echo '$HOME'",
        Platform::Windows,
    );
    assert_eq!(bad.len(), 5);
    for expected in [
        "$VAR",
        "export and source are bash",
        "single quotes",
        "\";\" does not chain",
        "do not cd",
    ] {
        assert!(bad.iter().any(|w| w.contains(expected)), "{expected}");
    }
    assert_eq!(
        command_warnings("echo %HOME%", Platform::Other),
        ["sh does not expand %NAME%: write $NAME."]
    );
    assert!(
        command_warnings("echo \"a;b\"", Platform::Windows).is_empty(),
        "a quoted semicolon is fine"
    );
}

#[test]
fn who_may_change_a_launcher_the_lead_its_maker_or_a_manager_of_its_bundle() {
    let mut board = board();
    board.builds.insert(
        "b1".to_owned(),
        Bundle {
            thread: Some("other".to_owned()),
            ..bundle("b1")
        },
    );
    let rec = launcher("build-2");
    assert!(authorize_launcher(&board, &lead(), Some(&rec)).is_ok());
    assert!(authorize_launcher(&board, &worker("worker"), Some(&rec)).is_ok());
    assert!(
        authorize_launcher(&board, &worker("other"), None).is_ok(),
        "anyone may make one"
    );
    let refused = authorize_launcher(&board, &worker("other"), Some(&rec))
        .unwrap_err()
        .to_string();
    assert!(refused.starts_with(
        "launcher build-2 was set by worker; only worker or the coordinator can change it."
    ));
    let made = Launcher {
        bundle: Some("b1".to_owned()),
        ..launcher("build-2")
    };
    assert!(authorize_launcher(&board, &worker("other"), Some(&made)).is_ok());
    let nobody = Who {
        actor: "x".to_owned(),
        role: Role::Thread,
        thread: None,
    };
    assert!(
        authorize_launcher(&board, &nobody, Some(&rec))
            .unwrap_err()
            .to_string()
            .starts_with("no thread is calling")
    );
    let by_lead = Launcher {
        created_by: Some("coordinator".to_owned()),
        ..launcher("c")
    };
    assert!(
        authorize_launcher(&board, &worker("other"), Some(&by_lead))
            .unwrap_err()
            .to_string()
            .contains("only the coordinator can change it")
    );
}

#[test]
fn set_makes_a_launcher_with_its_play_definition_and_the_same_set_again_changes_nothing() {
    let mut board = board();
    let patch = LauncherPatch {
        title: Some(Some("Build two".to_owned())),
        kind: Some("dev".to_owned()),
        current: Some(true),
        ..with_every("bxel launch build-2")
    };
    let made = write_launcher(&mut board, &worker("worker"), "build-2", &patch, &ctx()).unwrap();
    assert!(made.created && made.changed.is_empty());
    assert!(
        made.play.starts_with("Play ready (launcher-build-2, in "),
        "{}",
        made.play
    );
    let rec = &board.launchers["build-2"];
    assert_eq!(
        (rec.created_by.as_deref(), rec.kind.as_str(), rec.current),
        (Some("worker"), "dev", true)
    );
    let def = &board.run_defs["launcher-build-2"];
    assert_eq!(
        (def.rev, def.kind, def.hand, def.visible, def.window),
        (1, RunKind::Launch, true, false, false)
    );
    assert_eq!(
        (def.label.as_deref(), def.launcher.as_deref()),
        (Some("Play build-2"), Some("build-2"))
    );
    assert_eq!(board.audit.last().unwrap().text, "added launcher build-2");
    let again = write_launcher(
        &mut board,
        &worker("worker"),
        "build-2",
        &set_cmd("bxel launch build-2"),
        &ctx(),
    )
    .unwrap();
    assert!(!again.created && again.changed.is_empty() && again.play.is_empty());
    assert_eq!(board.run_defs["launcher-build-2"].rev, 1);
    assert_eq!(
        board.audit.len(),
        1,
        "an unchanged set leaves no audit line"
    );
}

#[test]
fn set_changes_the_flags_given_and_keeps_the_play_definition_in_step() {
    let mut board = board();
    write_launcher(
        &mut board,
        &worker("worker"),
        "tool",
        &with_every("run tool"),
        &ctx(),
    )
    .unwrap();
    let patch = LauncherPatch {
        note: Some(None),
        cwd: Some(Some("sub".to_owned())),
        window: Some(true),
        ..LauncherPatch::default()
    };
    let changed = write_launcher(&mut board, &worker("worker"), "tool", &patch, &ctx()).unwrap();
    assert_eq!(changed.changed, ["cwd", "window"]);
    assert!(
        changed.play.starts_with("Play updated (launcher-tool, in ")
            && changed.play.contains(", with a console window")
    );
    assert!(
        changed.play.contains("; that folder does not exist yet"),
        "a missing folder is said, not refused"
    );
    let def = &board.run_defs["launcher-tool"];
    assert_eq!((def.rev, def.visible, def.window), (2, true, true));
    assert_eq!(def.cwd, resolve_path("/somewhere", "sub"));
    let off = LauncherPatch {
        window: Some(false),
        cwd: Some(Some(String::new())),
        ..LauncherPatch::default()
    };
    write_launcher(&mut board, &worker("worker"), "tool", &off, &ctx()).unwrap();
    assert!(!board.launchers["tool"].window);
    assert_eq!(
        board.run_defs["launcher-tool"].cwd,
        resolve_path("/somewhere", "")
    );
}

#[test]
fn set_refuses_a_new_launcher_without_a_command_and_never_empties_one() {
    let mut board = board();
    let no_cmd = LauncherPatch {
        title: Some(Some("x".to_owned())),
        ..LauncherPatch::default()
    };
    let refused = write_launcher(&mut board, &lead(), "fresh-1", &no_cmd, &ctx())
        .unwrap_err()
        .to_string();
    assert!(refused.starts_with("launcher fresh-1 is new: give it the command"));
    assert!(!board.launchers.contains_key("fresh-1"));
    write_launcher(&mut board, &lead(), "long-1", &with_every("x"), &ctx()).unwrap();
    let empty = write_launcher(&mut board, &lead(), "long-1", &set_cmd(""), &ctx())
        .unwrap_err()
        .to_string();
    assert!(empty.starts_with("a launcher always has a command"));
    let cleared = LauncherPatch {
        cmd: Some(None),
        ..LauncherPatch::default()
    };
    assert!(write_launcher(&mut board, &lead(), "long-1", &cleared, &ctx()).is_err());
}

#[test]
fn a_standalone_launcher_needs_use_every_once_master_and_joined_ones_take_none() {
    let mut board = board();
    let none = write_launcher(
        &mut board,
        &worker("worker"),
        "tool-1",
        &set_cmd("node tool.js"),
        &ctx(),
    )
    .unwrap_err()
    .to_string();
    assert!(
        none.starts_with(
            "launcher tool-1 is a standalone build (not master, not made for a bundle)"
        )
    );
    assert!(none.contains("add --use-every <window> (12h, 3d, 2w or 1mo, from 1h to 365d)"));
    assert!(
        !board.launchers.contains_key("tool-1"),
        "nothing is written"
    );
    let made = write_launcher(
        &mut board,
        &worker("worker"),
        "tool-1",
        &with_every("node tool.js"),
        &ctx(),
    )
    .unwrap();
    assert!(made.ignored.is_empty());
    let later = write_launcher(
        &mut board,
        &worker("worker"),
        "tool-1",
        &LauncherPatch {
            current: Some(true),
            ..LauncherPatch::default()
        },
        &ctx(),
    );
    assert!(later.is_ok(), "once stored, a later save needs no flag");
}

#[test]
fn master_and_a_launcher_made_for_a_bundle_take_no_use_every() {
    let mut board = board();
    let master = write_launcher(
        &mut board,
        &lead(),
        "master",
        &with_every("game run"),
        &ctx(),
    )
    .unwrap();
    assert_eq!(master.ignored, ["useEvery"]);
    let for_bundle = WriteContext {
        bundle: Some("b1"),
        ..ctx()
    };
    let joined = write_launcher(
        &mut board,
        &lead(),
        "b1",
        &with_every("node b1.js"),
        &for_bundle,
    )
    .unwrap();
    assert_eq!(joined.ignored, ["useEvery"]);
    assert_eq!(board.launchers["b1"].use_every, None);
    assert_eq!(board.launchers["b1"].bundle.as_deref(), Some("b1"));
}

#[test]
fn the_play_name_made_by_hand_is_never_taken_and_a_multi_line_command_gets_no_play() {
    let mut board = board();
    let taken = argus_launchers::RunDef {
        name: "launcher-taken".to_owned(),
        cmd: "node mine.js".to_owned(),
        ..argus_launchers::RunDef::default()
    };
    board.run_defs.insert("launcher-taken".to_owned(), taken);
    let refused = write_launcher(
        &mut board,
        &lead(),
        "taken",
        &with_every("node theirs.js"),
        &ctx(),
    )
    .unwrap_err()
    .to_string();
    assert!(refused.contains(
        "the run name launcher-taken is already taken by a launch defined with coord run define"
    ));
    assert!(!board.launchers.contains_key("taken"), "nothing is written");
    assert_eq!(board.run_defs["launcher-taken"].cmd, "node mine.js");
    let multi = write_launcher(
        &mut board,
        &lead(),
        "multi-1",
        &with_every("line one\nline two"),
        &ctx(),
    )
    .unwrap();
    assert!(
        multi
            .play
            .starts_with("no Play button: the command spans several lines")
    );
    assert!(!board.run_defs.contains_key("launcher-multi-1"));
}

#[test]
fn retire_removes_the_play_and_set_brings_it_back_master_is_never_retired() {
    let mut board = board();
    write_launcher(
        &mut board,
        &worker("worker"),
        "build-2",
        &LauncherPatch {
            current: Some(true),
            ..with_every("go")
        },
        &ctx(),
    )
    .unwrap();
    let note =
        retire_launcher(&mut board, &lead(), "build-2", Some("merged into main"), AT).unwrap();
    assert_eq!(note, "Play removed (launcher-build-2).");
    let rec = &board.launchers["build-2"];
    assert_eq!(
        (
            rec.retired_by.as_deref(),
            rec.retire_note.as_deref(),
            rec.current
        ),
        (Some("coordinator"), Some("merged into main"), false)
    );
    assert!(!board.run_defs.contains_key("launcher-build-2"));
    assert!(
        board
            .audit
            .last()
            .unwrap()
            .text
            .contains("retired launcher build-2: merged into main")
    );
    let back = write_launcher(
        &mut board,
        &lead(),
        "build-2",
        &LauncherPatch::default(),
        &ctx(),
    )
    .unwrap();
    assert!(back.revived && back.play.starts_with("Play ready"));
    assert!(board.launchers["build-2"].retired_at.is_none());
    let master = retire_launcher(&mut board, &lead(), "master", None, AT)
        .unwrap_err()
        .to_string();
    assert!(master.starts_with("master is never retired"));
    assert!(
        retire_launcher(&mut board, &worker("other"), "build-2", None, AT)
            .unwrap_err()
            .to_string()
            .contains("set by worker")
    );
    assert!(
        retire_launcher(&mut board, &lead(), "nope", None, AT)
            .unwrap_err()
            .to_string()
            .contains("no launcher \"nope\"")
    );
}

#[test]
fn keep_counts_as_a_use_with_who_and_why_and_days_hold_it_fresh() {
    let mut board = board();
    write_launcher(
        &mut board,
        &worker("worker"),
        "keep-1",
        &with_every("node keep.js"),
        &ctx(),
    )
    .unwrap();
    let before = board.launchers["keep-1"].updated_at.clone();
    keep_launcher(
        &mut board,
        &worker("worker"),
        "keep-1",
        (None, Some("still the nightly check")),
        AT,
    )
    .unwrap();
    let rec = &board.launchers["keep-1"];
    assert_eq!(
        (
            rec.kept_by.as_deref(),
            rec.kept_note.as_deref(),
            rec.kept_until.clone()
        ),
        (Some("worker"), Some("still the nightly check"), None)
    );
    assert_eq!(rec.updated_at, before, "a keep leaves updatedAt alone");
    keep_launcher(&mut board, &lead(), "keep-1", (Some(30), None), AT).unwrap();
    let held = &board.launchers["keep-1"];
    assert_eq!(held.kept_until.as_deref(), Some("2026-11-23T00:00:00.000Z"));
    assert_eq!(held.kept_by.as_deref(), Some("coordinator"));
    assert!(
        board
            .audit
            .last()
            .unwrap()
            .text
            .contains("kept launcher keep-1 for 30d")
    );
    let refuse = |board: &mut LauncherBoard, who: &Who, name: &str| {
        keep_launcher(board, who, name, (None, None), AT)
            .unwrap_err()
            .to_string()
    };
    assert!(
        refuse(&mut board, &worker("other"), "keep-1")
            .contains("launcher keep-1 was set by worker")
    );
    assert!(refuse(&mut board, &lead(), "master").contains("master has no usage window"));
    assert!(refuse(&mut board, &lead(), "nope").contains("no launcher \"nope\""));
    retire_launcher(&mut board, &worker("worker"), "keep-1", None, AT).unwrap();
    assert!(refuse(&mut board, &worker("worker"), "keep-1").contains("launcher keep-1 is retired"));
}
