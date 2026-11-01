//! The automatic pings: a build handed and a question asked, and their de-duplication.
//! Keep rule: S2.
// test-tag: standard

use argus_notify::{
    AutoConfig, BoardFacts, Bundle, By, EntryFact, Level, NotifyConfig, TestFact, ToastState, iso,
};

use crate::harness::{Harness, ROTP, T0};

fn before(ms: i64) -> String {
    iso(T0 - ms)
}

fn handed(tag: &str, ms: i64) -> BoardFacts {
    BoardFacts {
        handed_build: Some(tag.into()),
        handed_at: Some(before(ms)),
        ..BoardFacts::default()
    }
}

fn question(id: &str, text: &str) -> EntryFact {
    EntryFact {
        id: id.into(),
        kind: "question".into(),
        status: "open".into(),
        author: "lead".into(),
        thread: Some("lead".into()),
        text: text.into(),
        created_at: Some(before(60_000)),
        ..EntryFact::default()
    }
}

fn asking(entries: Vec<EntryFact>) -> BoardFacts {
    BoardFacts {
        entries,
        ..BoardFacts::default()
    }
}

fn texts(h: &Harness) -> Vec<String> {
    h.store
        .rows()
        .into_iter()
        .map(|note| note.text.clone())
        .collect()
}

fn started() -> Harness {
    let h = Harness::named(NotifyConfig::default());
    h.notify.start();
    h
}

#[test]
fn the_first_sight_of_a_project_only_sets_the_markers_then_a_handed_bundle_pings_once() {
    let h = started();
    let old = || vec![question("Q-1", "old question")];
    assert_eq!(h.notify.on_board("rotp", &asking(old())).len(), 0);
    assert_eq!(h.store.rows().len(), 0, "nothing old is told");
    let facts = BoardFacts {
        bundle: Some(Bundle {
            title: Some("Save slots".into()),
            thread: None,
        }),
        tests: vec![
            TestFact {
                build: Some("b12".into()),
                removed: false,
            },
            TestFact {
                build: Some("b12".into()),
                removed: false,
            },
            TestFact {
                build: Some("b12".into()),
                removed: true,
            },
            TestFact {
                build: Some("other".into()),
                removed: false,
            },
        ],
        entries: old(),
        ..handed("b12", 1000)
    };
    assert_eq!(h.notify.on_board("rotp", &facts).len(), 1);
    assert_eq!(h.notify.on_board("rotp", &facts).len(), 0, "once");
    let rows = h.store.rows();
    assert_eq!(rows.len(), 1);
    let note = &rows[0];
    assert_eq!(
        (note.level, note.by, note.toast),
        (Level::Ping, By::Auto, ToastState::Shown)
    );
    assert_eq!(note.project.as_deref(), Some("rotp"));
    assert_eq!(note.title, "Rotp: ready to test");
    assert_eq!(note.text, "Rotp: Save slots is ready to test (2 tests)");
    assert_eq!(h.shown()[0].url.as_deref(), Some(ROTP));
}

#[test]
fn a_build_handed_without_a_title_says_the_tag_handing_another_or_again_pings_unhanding_does_not() {
    let h = started();
    h.notify.on_board("rotp", &BoardFacts::default());
    h.notify.on_board("rotp", &handed("b1", 3000));
    h.notify.on_board("rotp", &BoardFacts::default());
    h.notify.on_board("rotp", &handed("b2", 2000));
    h.notify.on_board("rotp", &handed("b2", 1000));
    assert_eq!(
        texts(&h),
        [
            "Rotp: b1 is ready to test",
            "Rotp: b2 is ready to test",
            "Rotp: b2 is ready to test"
        ]
    );
}

#[test]
fn a_question_asked_pings_once_with_its_first_100_characters() {
    let h = started();
    h.notify.on_board("rotp", &BoardFacts::default());
    let long = format!("Should the save slots be per profile {}", "x".repeat(120));
    let stale = EntryFact {
        created_at: Some(before(7 * 3_600_000)),
        ..question("Q-6", "stale")
    };
    let facts = asking(vec![
        EntryFact {
            needs_decision: true,
            ..question("Q-2", &long)
        },
        question("Q-3", "plain question"),
        EntryFact {
            status: "done".into(),
            ..question("Q-4", "answered")
        },
        EntryFact {
            author: "human".into(),
            ..question("Q-5", "the owner wrote this")
        },
        stale,
        EntryFact {
            removed: true,
            ..question("Q-7", "trashed")
        },
    ]);
    assert_eq!(h.notify.on_board("rotp", &facts).len(), 2);
    assert_eq!(h.notify.on_board("rotp", &facts).len(), 0);
    let rows = h.store.rows();
    let head: String = long.chars().take(100).collect();
    assert_eq!(rows[0].text, format!("Rotp asks: {head}\u{2026}"));
    assert_eq!(rows[0].title, "Rotp needs a decision");
    assert_eq!(rows[0].thread.as_deref(), Some("lead"));
    assert_eq!(rows[1].text, "Rotp asks: plain question");
    assert_eq!(rows[1].title, "Rotp has a question");
    assert!(
        rows.iter()
            .all(|n| (n.by, n.level) == (By::Auto, Level::Ping))
    );
}

#[test]
fn nothing_is_told_again_after_a_restart_or_a_resync_and_news_from_the_downtime_is_told_once() {
    let first = started();
    first.notify.on_board("rotp", &BoardFacts::default());
    let both = BoardFacts {
        entries: vec![question("Q-1", "which?")],
        ..handed("b1", 1000)
    };
    assert_eq!(first.notify.on_board("rotp", &both).len(), 2);
    first.notify.stop();
    let second = first.restart();
    second.notify.start();
    assert_eq!(second.notify.on_board("rotp", &both).len(), 0);
    assert_eq!(second.notify.on_board("rotp", &both).len(), 0);
    assert_eq!(second.store.rows().len(), 2, "nothing new");
    assert_eq!(second.shown().len(), 0);
    let news = BoardFacts {
        entries: vec![question("Q-1", "which?"), question("Q-2", "and this?")],
        ..handed("b2", 500)
    };
    assert_eq!(second.notify.on_board("rotp", &news).len(), 2);
    assert_eq!(
        texts(&second)[2..],
        ["Rotp: b2 is ready to test", "Rotp asks: and this?"]
    );
}

#[test]
fn config_auto_switches_each_one_off_and_the_markers_still_move() {
    let config = |handed, ask| NotifyConfig {
        auto: AutoConfig { handed, ask },
        ..NotifyConfig::default()
    };
    let h = Harness::named(config(false, true));
    h.notify.start();
    h.notify.on_board("rotp", &BoardFacts::default());
    let facts = BoardFacts {
        entries: vec![question("Q-1", "which?")],
        ..handed("b1", 0)
    };
    h.notify.on_board("rotp", &facts);
    assert_eq!(texts(&h), ["Rotp asks: which?"]);
    h.config.set(config(true, true));
    h.notify.on_board("rotp", &facts);
    assert_eq!(h.store.rows().len(), 1, "b1 is not replayed");
    let none = Harness::named(config(false, false));
    none.notify.start();
    none.notify.on_board("rotp", &BoardFacts::default());
    none.notify.on_board("rotp", &facts);
    assert_eq!(none.store.rows().len(), 0);
}

#[test]
fn a_retired_project_and_a_stopped_service_tell_nothing() {
    let h = started();
    let retired = |facts: BoardFacts| BoardFacts {
        retired: true,
        ..facts
    };
    h.notify.on_board("rotp", &retired(BoardFacts::default()));
    h.notify.on_board("rotp", &retired(handed("b1", 0)));
    assert_eq!(h.store.rows().len(), 0, "retired");
    h.notify.on_board("rotp", &BoardFacts::default());
    h.notify.stop();
    assert_eq!(h.notify.on_board("rotp", &handed("b1", 0)).len(), 0);
    assert_eq!(h.store.rows().len(), 0, "stopped");
}

#[test]
fn markers_that_cannot_be_saved_hold_the_ping_back_and_say_so() {
    let h = started();
    h.notify.on_board("rotp", &BoardFacts::default());
    h.store.set_broken(true);
    assert_eq!(h.notify.on_board("rotp", &handed("b1", 0)).len(), 0);
    let lines = h.activity.lines();
    assert_eq!(
        lines.last().map(|line| line.text.as_str()),
        Some("automatic ping failed")
    );
}
