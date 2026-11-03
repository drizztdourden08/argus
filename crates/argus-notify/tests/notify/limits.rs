//! The rate limits and the one summary toast a minute per thread.
//! Keep rule: S2.
// test-tag: standard

use argus_notify::{
    GLOBAL_PER_MINUTE, NotifyConfig, Outcome, PingRequest, THREAD_PER_HOUR, ToastState,
};

use crate::harness::{Harness, Shown, TAB};

fn from(h: &Harness, thread: &str) -> Outcome {
    h.notify.send(PingRequest {
        project: Some("rotp".into()),
        thread: Some(thread.into()),
        ..PingRequest::new(format!("from {thread}"))
    })
}

fn summaries(h: &Harness) -> Vec<String> {
    h.shown()
        .into_iter()
        .filter(Shown::is_summary)
        .map(|toast| toast.texts[1].clone())
        .collect()
}

#[test]
fn a_thread_gets_3_toasts_a_minute_the_4th_is_recorded_as_limited_with_one_summary_toast() {
    let h = Harness::new();
    let out: Vec<Outcome> = (0..5).map(|_| from(&h, "lead")).collect();
    let states: Vec<_> = out.iter().map(|o| o.toast).collect();
    let (shown, limited) = (ToastState::Shown, ToastState::Limited);
    assert_eq!(states, [shown, shown, shown, limited, limited]);
    assert!(out[3].is_recorded());
    assert_eq!(
        out[3].why.as_deref(),
        Some("you pinged 4 times in a minute")
    );
    assert_eq!(
        out[4].why.as_deref(),
        Some("you pinged 4 times in a minute"),
        "limited ones are not counted"
    );
    assert_eq!(h.store.rows().len(), 5, "all of them are recorded");
    let toasts = h.shown();
    assert_eq!(toasts.len(), 4, "3 toasts and the one summary");
    assert_eq!(
        toasts[3].texts,
        ["Argus", "1 more notification from rotp/lead: see Argus"]
    );
    assert_eq!(
        toasts[3].tag, toasts[0].tag,
        "the summary replaces the thread's older toast"
    );
    assert_eq!(toasts[3].url.as_deref(), Some(TAB));
    let line = h.activity.lines()[3].text.clone();
    assert!(
        line.ends_with("(no toast: limited, you pinged 4 times in a minute)"),
        "{line}"
    );
    h.minute();
    assert_eq!(from(&h, "lead").toast, ToastState::Shown);
}

#[test]
fn one_summary_toast_a_minute_per_thread_counting_what_it_summarises() {
    let h = Harness::new();
    for _ in 0..3 {
        from(&h, "lead");
    }
    for _ in 0..4 {
        from(&h, "lead");
    }
    assert_eq!(summaries(&h).len(), 1);
    h.clock.advance(30_000);
    from(&h, "lead");
    assert_eq!(summaries(&h).len(), 1, "still the same minute");
    h.clock.advance(31_000);
    for _ in 0..3 {
        from(&h, "lead");
    }
    assert_eq!(from(&h, "lead").toast, ToastState::Limited);
    let all = summaries(&h);
    assert_eq!(all.len(), 2);
    assert_eq!(all[1], "5 more notifications from rotp/lead: see Argus");
    assert_eq!(from(&h, "other").toast, ToastState::Shown);
}

#[test]
fn twenty_an_hour_per_thread_and_eight_a_minute_for_everyone() {
    let h = Harness::new();
    let mut shown = 0;
    for _ in 0..25 {
        h.clock.advance(21_000);
        if from(&h, "lead").toast == ToastState::Shown {
            shown += 1;
        }
    }
    assert_eq!(shown, THREAD_PER_HOUR);
    let hour = from(&h, "lead");
    assert_eq!(hour.toast, ToastState::Limited);
    assert_eq!(hour.why.as_deref(), Some("you pinged 21 times in an hour"));
    let g = Harness::new();
    let results: Vec<_> = (0..10).map(|i| from(&g, &format!("t{i}")).toast).collect();
    let mut expected = vec![ToastState::Shown; GLOBAL_PER_MINUTE];
    expected.extend([ToastState::Limited; 2]);
    assert_eq!(results, expected);
    let again = from(&g, "t0");
    assert_eq!(
        again.why.as_deref(),
        Some("9 notifications in a minute across all threads")
    );
    assert_eq!(g.store.rows().len(), 11, "all recorded");
}

#[test]
fn info_os_off_and_quiet_levels_never_use_up_the_limits() {
    let h = Harness::new();
    for _ in 0..10 {
        h.notify.send(PingRequest {
            level: argus_notify::Level::Info,
            thread: Some("lead".into()),
            ..PingRequest::new("i")
        });
    }
    let out = h.notify.send(PingRequest {
        thread: Some("lead".into()),
        ..PingRequest::new("p")
    });
    assert_eq!(out.toast, ToastState::Shown);
    let off = Harness::with(NotifyConfig {
        os: false,
        ..NotifyConfig::default()
    });
    for _ in 0..10 {
        assert_eq!(from(&off, "lead").toast, ToastState::Skipped);
    }
}
