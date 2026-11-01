//! The launchers' wakes (launchers.stale): a project with no master launcher wakes its lead
//! once (after 10 minutes on a board), again every 24 hours; a standalone launcher unused
//! past its window wakes the thread that made it, one wake a day that lists them; a launcher
//! without the field counts as 7d and is asked for it. A manual clock drives every tick and
//! the wake store's last wake stands in for the persisted state.
//!
//! Keep rule: K1, K2.
// test-tag: critical

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use argus_launchers::launcher::{
    Bundle, Launcher, LauncherBoard, LauncherStale, MASTER, REASON_MASTER, REASON_STALE,
    StaleConfig, StaleEnv, StaleItem, StaleNote, ThreadInfo, WakeEvent, master_text, stale_text,
};
use argus_launchers::{ManualClock, format_iso};
use serde_json::{Value, json};

const START: i64 = 1_792_800_000_000;
const MIN: i64 = 60_000;
const H: i64 = 3_600_000;
const DAY: i64 = 24 * H;

type LastSent = Box<dyn Fn(&str, &str, &str, &str) -> i64 + Send + Sync>;

#[derive(Default)]
struct Fixture {
    boards: Mutex<BTreeMap<String, LauncherBoard>>,
    lead: Mutex<Option<String>>,
    played: Mutex<HashMap<String, i64>>,
    last_sent: Mutex<Option<LastSent>>,
    events: Mutex<Vec<WakeEvent>>,
    notes: Mutex<Vec<StaleNote>>,
}

impl StaleEnv for Fixture {
    fn scopes(&self) -> Vec<String> {
        self.boards.lock().unwrap().keys().cloned().collect()
    }
    fn board(&self, scope: &str) -> Option<LauncherBoard> {
        self.boards.lock().unwrap().get(scope).cloned()
    }
    fn coordinator(&self, _scope: &str) -> Option<String> {
        self.lead.lock().unwrap().clone()
    }
    fn played_ms(&self, scope: &str, name: &str) -> i64 {
        self.played
            .lock()
            .unwrap()
            .get(&format!("{scope}/{name}"))
            .copied()
            .unwrap_or(0)
    }
    fn last_sent_ms(&self, scope: &str, thread: &str, reason: &str, subject: &str) -> i64 {
        self.last_sent
            .lock()
            .unwrap()
            .as_ref()
            .map_or(0, |f| f(scope, thread, reason, subject))
    }
    fn submit(&self, events: Vec<WakeEvent>) -> Result<(), String> {
        self.events.lock().unwrap().extend(events);
        Ok(())
    }
    fn note(&self, note: StaleNote) {
        self.notes.lock().unwrap().push(note);
    }
}

struct World {
    fixture: Arc<Fixture>,
    clock: Arc<ManualClock>,
    stale: LauncherStale,
}

fn launcher(name: &str, edit: impl FnOnce(&mut Launcher)) -> Launcher {
    let mut rec = Launcher {
        name: name.to_owned(),
        cmd: Some(format!("run {name}")),
        created_at: format_iso(START - 30 * DAY),
        created_by: Some("worker".to_owned()),
        updated_at: format_iso(START - 20 * DAY),
        by: Some("worker".to_owned()),
        use_every: Some("7d".to_owned()),
        ..Launcher::default()
    };
    edit(&mut rec);
    rec
}

fn board() -> LauncherBoard {
    let mut board = LauncherBoard {
        project: Some("alpha".to_owned()),
        created_at: Some(format_iso(START - 5 * DAY)),
        ..LauncherBoard::default()
    };
    for (id, role, status) in [
        ("lead", "coordinator", "active"),
        ("worker", "thread", "active"),
        ("other", "thread", "active"),
        ("gone", "thread", "done"),
    ] {
        board.threads.insert(
            id.to_owned(),
            ThreadInfo {
                id: id.to_owned(),
                role: role.to_owned(),
                status: status.to_owned(),
                ..ThreadInfo::default()
            },
        );
    }
    board
        .launchers
        .insert(MASTER.to_owned(), launcher(MASTER, |l| l.use_every = None));
    board
}

fn make(fixture: &Arc<Fixture>, clock: &Arc<ManualClock>) -> LauncherStale {
    LauncherStale::new(
        Arc::clone(fixture) as _,
        Arc::clone(clock) as _,
        StaleConfig::default(),
    )
}

fn world() -> World {
    let fixture = Arc::new(Fixture::default());
    fixture
        .boards
        .lock()
        .unwrap()
        .insert("alpha".to_owned(), board());
    *fixture.lead.lock().unwrap() = Some("lead".to_owned());
    let clock = Arc::new(ManualClock::at(START));
    let stale = make(&fixture, &clock);
    World {
        fixture,
        clock,
        stale,
    }
}

impl World {
    fn edit(&self, change: impl FnOnce(&mut LauncherBoard)) {
        change(
            self.fixture
                .boards
                .lock()
                .unwrap()
                .get_mut("alpha")
                .unwrap(),
        );
    }

    fn add(&self, launchers: Vec<Launcher>) {
        self.edit(|b| {
            b.launchers
                .extend(launchers.into_iter().map(|l| (l.name.clone(), l)));
        });
    }

    fn at(&self, offset_ms: i64) -> usize {
        self.clock.set(START + offset_ms);
        self.stale.tick()
    }

    fn events(&self) -> Vec<WakeEvent> {
        self.fixture.events.lock().unwrap().clone()
    }

    fn restart(&mut self, last_sent: LastSent) {
        *self.fixture.last_sent.lock().unwrap() = Some(last_sent);
        self.stale = make(&self.fixture, &self.clock);
        self.fixture.events.lock().unwrap().clear();
    }
}

/// Where a wake goes: scope, thread, reason and subject.
fn route(event: &WakeEvent) -> (&str, &str, &str, &str) {
    (
        event.scope.as_str(),
        event.thread.as_str(),
        event.reason,
        event.subject.as_str(),
    )
}

fn names(world: &World) -> Vec<String> {
    let mut found: Vec<String> = world
        .stale
        .stale_launchers()
        .into_iter()
        .map(|u| u.name)
        .collect();
    found.sort();
    found
}

#[test]
fn no_master_the_lead_is_woken_once_after_the_board_is_10_minutes_old() {
    let world = world();
    world.edit(|b| {
        b.launchers.remove(MASTER);
        b.created_at = Some(format_iso(START - 3 * MIN));
    });
    assert_eq!(world.stale.tick(), 0, "registered 3 minutes ago: too soon");
    assert_eq!(world.at(6 * MIN), 0, "9 minutes");
    assert_eq!(world.at(8 * MIN), 1, "11 minutes");
    let event = world.events().remove(0);
    assert_eq!(
        route(&event),
        ("alpha", "lead", "launcher-master-missing", "alpha")
    );
    assert_eq!(event.text, master_text());
    assert!(
        event
            .text
            .starts_with("No master launcher: every project must have one")
    );
    assert!(event.text.contains("coord launcher set master --cmd \"<command that starts it>\". Until then coord build hand is refused"));
    assert!(event.text.chars().count() <= 300);
    assert_eq!(world.at(9 * MIN), 0, "once");
    world.edit(|b| {
        b.launchers
            .insert(MASTER.to_owned(), launcher(MASTER, |_| {}));
    });
    assert_eq!(world.at(3 * DAY), 0, "a master: nothing");
}

#[test]
fn no_master_repeats_every_24_hours_and_a_restart_does_not_send_it_again() {
    let mut world = world();
    world.edit(|b| {
        b.launchers.remove(MASTER);
    });
    assert_eq!(world.stale.tick(), 1);
    assert_eq!(world.at(23 * H), 0);
    assert_eq!(world.at(25 * H), 1, "a day later");
    let sent_at = START + 25 * H;
    world.restart(Box::new(move |scope, thread, reason, subject| {
        i64::from(
            reason == REASON_MASTER && scope == "alpha" && thread == "lead" && subject == "alpha",
        ) * sent_at
    }));
    assert_eq!(world.at(26 * H), 0, "restarted, one hour after the wake");
    assert_eq!(world.at(50 * H), 1, "a day after the stored one");
}

#[test]
fn no_master_without_a_lead_or_on_the_portfolio_board_sends_nothing_and_a_retired_master_is_missing()
 {
    let world = world();
    world.edit(|b| {
        b.launchers.remove(MASTER);
    });
    *world.fixture.lead.lock().unwrap() = None;
    assert_eq!(world.stale.tick(), 0, "nobody to tell");
    *world.fixture.lead.lock().unwrap() = Some("lead".to_owned());
    world.fixture.boards.lock().unwrap().insert(
        "_portfolio".to_owned(),
        LauncherBoard {
            created_at: Some(format_iso(START - DAY)),
            ..LauncherBoard::default()
        },
    );
    world.edit(|b| {
        b.launchers.insert(
            MASTER.to_owned(),
            launcher(MASTER, |l| l.retired_at = Some(format_iso(START))),
        );
    });
    assert_eq!(world.stale.tick(), 1);
    assert_eq!(world.events()[0].scope, "alpha");
}

#[test]
fn a_standalone_launcher_unused_past_its_window_wakes_its_maker_with_one_line() {
    let world = world();
    world.add(vec![
        launcher("build-2", |l| l.updated_at = format_iso(START - 9 * DAY)),
        launcher("try-engine", |l| {
            l.updated_at = format_iso(START - 15 * DAY);
            l.use_every = Some("2w".to_owned());
        }),
        launcher("fresh-1", |l| l.updated_at = format_iso(START - 2 * DAY)),
    ]);
    assert_eq!(world.stale.tick(), 1);
    let event = world.events().remove(0);
    assert_eq!(
        route(&event),
        ("alpha", "worker", "launcher-stale", "worker")
    );
    assert_eq!(
        event.text,
        "2 launchers not used within their window: try-engine (unused 15d, expected every 2w), build-2 (unused 9d, expected every 7d). Analyse: still needed? keep it (coord launcher keep <name> --days n), retire it, or change its --use-every."
    );
}

#[test]
fn the_window_is_a_strict_past_and_the_newest_of_play_record_and_keep_counts() {
    let world = world();
    let old = |l: &mut Launcher| l.updated_at = format_iso(START - 30 * DAY);
    world.add(vec![
        launcher("edge-1", |l| l.updated_at = format_iso(START - 7 * DAY)),
        launcher("played-1", old),
        launcher("kept-1", |l| {
            old(l);
            l.kept_at = Some(format_iso(START - DAY));
        }),
        launcher("hold-1", |l| {
            old(l);
            l.kept_at = Some(format_iso(START - 20 * DAY));
            l.kept_until = Some(format_iso(START + 5 * DAY));
        }),
    ]);
    world
        .fixture
        .played
        .lock()
        .unwrap()
        .insert("alpha/played-1".to_owned(), START - 2 * DAY);
    assert!(names(&world).is_empty(), "edge-1 is at 7d exactly");
    assert_eq!(world.stale.tick(), 0);
    world.clock.set(START + 2 * H);
    assert_eq!(names(&world), ["edge-1"]);
    world.clock.set(START + 4 * DAY);
    assert_eq!(
        names(&world),
        ["edge-1"],
        "hold-1 is held until day 5; played-1 ran 6 days ago; kept-1 was kept 5 days ago"
    );
    world.clock.set(START + 6 * DAY);
    assert_eq!(
        names(&world),
        ["edge-1", "hold-1", "played-1"],
        "the hold ended; kept-1 is at exactly 7d"
    );
    world.clock.set(START + 6 * DAY + H);
    assert_eq!(names(&world), ["edge-1", "hold-1", "kept-1", "played-1"]);
}

#[test]
fn master_a_launcher_made_for_a_bundle_one_linked_by_an_open_bundle_and_a_retired_one_have_no_window()
 {
    let world = world();
    world.add(vec![
        launcher("made-1", |l| l.bundle = Some("b1".to_owned())),
        launcher("linked-1", |_| {}),
        launcher("closed-link", |_| {}),
        launcher("old-1", |l| l.retired_at = Some(format_iso(START - DAY))),
    ]);
    world.edit(|b| {
        let bundle = |tag: &str, launcher: Option<&str>, closed: bool| Bundle {
            tag: tag.to_owned(),
            launcher: launcher.map(str::to_owned),
            closed_at: closed.then(|| format_iso(START)),
            ..Bundle::default()
        };
        b.builds.insert("b1".to_owned(), bundle("b1", None, false));
        b.builds
            .insert("b2".to_owned(), bundle("b2", Some("linked-1"), false));
        b.builds
            .insert("b3".to_owned(), bundle("b3", Some("closed-link"), true));
    });
    assert_eq!(
        names(&world),
        ["closed-link"],
        "its only bundle is closed: standalone again"
    );
}

#[test]
fn the_wake_goes_once_a_day_per_thread_and_a_restart_does_not_send_it_again() {
    let mut world = world();
    world.add(vec![launcher("build-2", |l| {
        l.updated_at = format_iso(START - 9 * DAY);
    })]);
    assert_eq!(world.stale.tick(), 1);
    assert_eq!(world.at(10 * MIN), 0);
    assert_eq!(world.at(23 * H), 0);
    assert_eq!(world.at(25 * H), 1, "the next day");
    let sent_at = START + 25 * H;
    world.restart(Box::new(move |scope, thread, reason, subject| {
        i64::from(
            reason == REASON_STALE && scope == "alpha" && thread == "worker" && subject == "worker",
        ) * sent_at
    }));
    assert_eq!(world.at(26 * H), 0, "after a restart");
    assert_eq!(world.at(50 * H), 1);
}

#[test]
fn each_thread_hears_about_its_own_launchers_an_ended_thread_or_the_owner_hands_them_to_the_lead() {
    let world = world();
    let by = |name: &str, maker: &str| {
        launcher(name, |l| {
            l.updated_at = format_iso(START - 9 * DAY);
            l.created_by = Some(maker.to_owned());
        })
    };
    world.add(vec![
        by("a-1", "worker"),
        by("b-1", "other"),
        by("c-1", "gone"),
        by("d-1", "human"),
        by("e-1", "nobody-known"),
    ]);
    assert_eq!(world.stale.tick(), 3);
    let events = world.events();
    let to = |thread: &str| {
        events
            .iter()
            .find(|e| e.thread == thread)
            .unwrap()
            .text
            .clone()
    };
    assert!(to("worker").starts_with("1 launcher not used within their window: a-1 ("));
    assert!(to("other").contains("b-1 ("));
    let lead = to("lead");
    assert!(lead.starts_with("3 launchers not used"));
    assert!(["c-1", "d-1", "e-1"].iter().all(|n| lead.contains(n)));
}

#[test]
fn a_launcher_with_no_use_every_counts_as_7d_and_its_thread_is_asked_for_the_field() {
    let world = world();
    world.add(vec![
        launcher("legacy-1", |l| {
            l.use_every = None;
            l.updated_at = format_iso(START - 12 * DAY);
        }),
        launcher("legacy-2", |l| {
            l.use_every = None;
            l.updated_at = format_iso(START - DAY);
            l.created_by = Some("other".to_owned());
        }),
        launcher("stale-1", |l| l.updated_at = format_iso(START - 9 * DAY)),
    ]);
    assert_eq!(world.stale.tick(), 2);
    let events = world.events();
    let mine = &events.iter().find(|e| e.thread == "worker").unwrap().text;
    assert!(mine.starts_with("2 launchers not used within their window: legacy-1 (unused 12d, expected every 7d by default), stale-1 (unused 9d, expected every 7d)."));
    assert!(mine.chars().count() <= 300);
    let other = &events.iter().find(|e| e.thread == "other").unwrap().text;
    assert_eq!(
        other,
        "1 launcher without a usage window. No --use-every on legacy-2: set it (coord launcher set <name> --use-every 7d); 7d is assumed. Reported when unused longer."
    );
}

#[test]
fn stale_text_lists_the_worst_first_shrinks_to_300_characters_and_keeps_the_instruction() {
    let stale: Vec<StaleItem> = (0..12)
        .map(|i| StaleItem {
            name: format!("build-{}", i + 1),
            use_every: "7d".to_owned(),
            legacy: false,
            unused_ms: (40 - i) * DAY,
        })
        .collect();
    let text = stale_text(&stale, &[]);
    assert!(text.chars().count() <= 300, "{}", text.chars().count());
    assert!(text.starts_with(
        "12 launchers not used within their window: build-1 (unused 40d, expected every 7d), "
    ));
    assert!(text.contains("(+") && text.ends_with(" more). Analyse: still needed? keep it (coord launcher keep <name> --days n), retire it, or change its --use-every."));
    assert_eq!(stale_text(&[], &[]), "");
    let long = StaleItem {
        name: "x".repeat(31),
        use_every: "365d".to_owned(),
        legacy: true,
        unused_ms: 400 * DAY,
    };
    let letters: Vec<String> = "abcdefg".chars().map(String::from).collect();
    let one = stale_text(&[long], &letters);
    assert!(one.chars().count() <= 300);
    assert!(one.starts_with(&format!(
        "1 launcher not used within their window: {} (unused 400d, expected every 365d by default)",
        "x".repeat(31)
    )));
}

#[test]
fn overlay_the_newest_play_run_is_a_use_on_the_launchers_docs_of_an_export() {
    let world = world();
    let doc = |name: &str, standalone: bool, last: i64, stale: bool| json!({"coll": "launchers", "id": name, "op": "set", "data": {"name": name, "standalone": standalone, "retiredAt": null, "lastUsedAt": format_iso(last), "useEveryMs": 7 * DAY, "keptUntil": null, "stale": stale}});
    let mut docs: Vec<Value> = vec![
        doc("a", true, START - 9 * DAY, true),
        doc("b", true, START - 9 * DAY, true),
        doc("master", false, START - 9 * DAY, false),
        json!({"coll": "threads", "id": "a", "op": "set", "data": {"id": "a"}}),
    ];
    let mut played = world.fixture.played.lock().unwrap();
    played.insert("alpha/a".to_owned(), START - DAY);
    played.insert("alpha/b".to_owned(), START - 20 * DAY);
    drop(played);
    world.stale.overlay("alpha", &mut docs);
    assert_eq!(
        (
            docs[0]["data"]["lastUsedAt"].as_str(),
            docs[0]["data"]["stale"].as_bool()
        ),
        (Some(format_iso(START - DAY).as_str()), Some(false))
    );
    assert_eq!(
        (
            docs[1]["data"]["lastUsedAt"].as_str(),
            docs[1]["data"]["stale"].as_bool()
        ),
        (Some(format_iso(START - 9 * DAY).as_str()), Some(true)),
        "an older run changes nothing"
    );
    assert_eq!(
        docs[2]["data"]["lastUsedAt"].as_str(),
        Some(format_iso(START - 9 * DAY).as_str()),
        "master is left alone"
    );
    assert_eq!(docs[3]["data"], json!({"id": "a"}));
}

#[test]
fn what_the_wakes_note_for_the_activity_log() {
    let world = world();
    world.add(vec![launcher("build-2", |l| {
        l.updated_at = format_iso(START - 9 * DAY);
    })]);
    world.edit(|b| {
        b.launchers.remove(MASTER);
    });
    world.stale.tick();
    let notes = world.fixture.notes.lock().unwrap().clone();
    assert!(
        notes
            .iter()
            .any(|n| n.level == "warn"
                && n.text == "alpha has no master launcher: its lead was told")
    );
    assert!(notes.iter().any(|n| {
        n.level == "info"
            && n.text
                .contains("alpha/worker: 1 launcher not used within their window (build-2)")
    }));
}
