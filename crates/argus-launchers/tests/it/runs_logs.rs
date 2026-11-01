//! Logs: lines and partial lines, carriage returns, long lines, the head and tail cap,
//! cursors, redaction and the live tail (runs.logs).

use std::time::Duration;

use argus_launchers::{LogOptions, RunLimits, RunRow, Stream};

use crate::common::{Env, EnvOptions, until};

/// Starts the one-job project and returns its pid.
fn job(env: &Env) -> u32 {
    env.define("alpha", vec![env.def("job", "job")]);
    env.start("alpha", "job");
    env.pid(0)
}

async fn finish(env: &Env, pid: u32) -> RunRow {
    env.host.exit(pid, Some(0)).await;
    env.finished("alpha", "R-1").await
}

#[tokio::test(start_paused = true)]
async fn a_carriage_return_progress_line_shows_its_last_state_and_multibyte_text_survives() {
    let env = Env::new();
    let pid = job(&env);
    env.host.say(pid, "10%\r50%\r100%\n").await;
    let bytes = "h\u{e9}llo w\u{f6}rld \u{2713}\n".as_bytes().to_vec();
    env.host.say_bytes(pid, &bytes[..2]).await;
    tokio::time::sleep(Duration::from_millis(60)).await;
    env.host.say_bytes(pid, &bytes[2..]).await;
    finish(&env, pid).await;
    let out = env.texts("alpha", "R-1", Some("out"));
    assert_eq!(out, ["100%", "h\u{e9}llo w\u{f6}rld \u{2713}"]);
}

#[tokio::test(start_paused = true)]
async fn a_prompt_without_a_newline_is_released_after_a_pause() {
    let env = Env::new();
    let pid = job(&env);
    env.host.say(pid, "Continue? ").await;
    until("the partial line", || {
        env.texts("alpha", "R-1", Some("out"))
            .iter()
            .any(|t| t == "Continue? ")
            .then_some(())
    })
    .await;
    env.svc.stop("alpha", "R-1", "tester").await.unwrap();
}

/// A run that printed 3000 numbered lines under a 20 KB cap with a 4 KB head.
async fn capped_run() -> (Env, RunRow) {
    let env = Env::with_limits(RunLimits {
        log_cap_bytes: 20_000,
        log_head_bytes: 4000,
        ..RunLimits::default()
    });
    let pid = job(&env);
    let text: Vec<String> = (1..=3000)
        .map(|i| format!("line {i:05} {}\n", "x".repeat(14)))
        .collect();
    env.host.say(pid, &text.concat()).await;
    let row = finish(&env, pid).await;
    (env, row)
}

#[tokio::test(start_paused = true)]
async fn the_cap_keeps_the_head_and_the_newest_lines_and_counts_the_dropped_ones() {
    let (env, row) = capped_run().await;
    let out = env.texts("alpha", "R-1", Some("out"));
    assert!(out[0].starts_with("line 00001"), "the head survives");
    assert!(
        out.last().unwrap().starts_with("line 03000"),
        "the newest survives"
    );
    assert!(row.log_dropped > 0, "something was dropped");
    assert!(row.log_bytes <= 20_000, "kept {} bytes", row.log_bytes);
    assert_eq!(
        env.svc
            .logs("alpha", "R-1", &LogOptions::default())
            .unwrap()
            .dropped,
        row.log_dropped
    );
}

#[tokio::test(start_paused = true)]
async fn the_kept_part_is_the_head_a_gap_then_a_contiguous_tail() {
    let (env, row) = capped_run().await;
    let lines = env.lines("alpha", "R-1", None);
    let gap = (1..lines.len())
        .find(|i| lines[*i].seq != lines[i - 1].seq + 1)
        .unwrap();
    let head: usize = lines[..gap].iter().map(|l| l.text.len() + 1).sum();
    assert!((4000..4300).contains(&head), "head is {head} bytes");
    assert!(lines[gap..].windows(2).all(|w| w[1].seq == w[0].seq + 1));
    let stored: usize = lines.iter().map(|l| l.text.len() + 1).sum();
    assert_eq!(stored as u64, row.log_bytes, "log_bytes is what is stored");
}

#[tokio::test(start_paused = true)]
async fn cursors_after_limit_tail_and_the_stream_filter() {
    let env = Env::new();
    let pid = job(&env);
    let numbers = (1..=10)
        .map(|n| format!("n{n}\n"))
        .collect::<Vec<_>>()
        .concat();
    env.host.say(pid, &numbers).await;
    env.host.complain(pid, "e\n").await;
    finish(&env, pid).await;
    let page = |options: LogOptions| env.svc.logs("alpha", "R-1", &options).unwrap();
    let first = page(LogOptions {
        after: Some(0),
        limit: Some(4),
        ..LogOptions::default()
    });
    assert_eq!(
        first.lines.iter().map(|l| l.seq).collect::<Vec<_>>(),
        [1, 2, 3, 4]
    );
    assert_eq!(first.next, 4);
    let rest = page(LogOptions {
        after: Some(4),
        limit: Some(100),
        ..LogOptions::default()
    });
    assert_eq!(rest.lines[0].seq, 5);
    assert_eq!(rest.next, rest.lines.last().unwrap().seq);
    let empty = page(LogOptions {
        after: i64::try_from(rest.next).ok(),
        ..LogOptions::default()
    });
    assert!(empty.lines.is_empty());
    assert_eq!(
        empty.next, rest.next,
        "the cursor does not move without news"
    );
    let tail = page(LogOptions {
        tail: Some(3),
        stream: Some("out".to_owned()),
        ..LogOptions::default()
    });
    assert_eq!(
        tail.lines
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>(),
        ["n8", "n9", "n10"]
    );
    assert!(tail.next >= tail.lines.last().unwrap().seq);
    assert_eq!(env.texts("alpha", "R-1", Some("err")), ["e"]);
    assert_eq!(
        env.svc
            .logs("alpha", "R-9", &LogOptions::default())
            .unwrap_err()
            .code,
        "unknown_run"
    );
}

#[tokio::test(start_paused = true)]
async fn secrets_never_reach_the_log() {
    let token = "0123456789abcdef0123456789abcdef";
    let env = Env::build(EnvOptions {
        token: Some(token.to_owned()),
        ..EnvOptions::default()
    });
    let mut def = env.def("job", "job");
    def.env
        .insert("MY_SECRET".to_owned(), "zebra-crossing-42".to_owned());
    def.env
        .insert("PLAIN_NAME".to_owned(), "visible-value".to_owned());
    env.define("alpha", vec![def]);
    env.start("alpha", "job");
    let pid = env.pid(0);
    let leaks = format!(
        "token is {token}\nAuthorization: Bearer abc.def.ghi\napi_key=hunter2hunter2 and password: swordfish\nkey sk-abcdefghijklmnopqrstuvwxyz0123 and ghp_abcdefghijklmnopqrstuvwxyz0123456789\nfrom env zebra-crossing-42 and visible-value\n"
    );
    env.host.say(pid, &leaks).await;
    env.host.complain(pid, "err zebra-crossing-42\n").await;
    finish(&env, pid).await;
    let text = env.text("alpha", "R-1", None);
    for secret in [
        token,
        "abc.def.ghi",
        "hunter2hunter2",
        "swordfish",
        "sk-abcdefghijklmnopqrstuvwxyz0123",
        "ghp_abcdefghijklmnopqrstuvwxyz0123456789",
        "zebra-crossing-42",
    ] {
        assert!(!text.contains(secret), "{secret} leaked");
    }
    assert!(text.contains("from env [redacted] and visible-value"));
    assert!(text.contains("Authorization: Bearer [redacted]"));
    let last = env.events.doc("alpha", "R-1").unwrap()["lastLine"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    assert!(!last.contains("zebra") && !last.contains("swordfish"));
}

#[tokio::test(start_paused = true)]
async fn the_live_tail_carries_every_line_once_in_order_with_the_run_scope() {
    let env = Env::new();
    let pid = job(&env);
    for n in 1..=5 {
        env.host.say(pid, &format!("tick {n}\n")).await;
        tokio::time::sleep(Duration::from_millis(80)).await;
    }
    finish(&env, pid).await;
    let mine: Vec<_> = env
        .events
        .lines()
        .into_iter()
        .filter(|e| e.scope == "alpha" && e.rid == "R-1")
        .collect();
    let seqs: Vec<u64> = mine.iter().map(|e| e.seq).collect();
    assert_eq!(seqs, (1..=seqs.len() as u64).collect::<Vec<_>>());
    let ticks: Vec<&str> = mine
        .iter()
        .filter(|e| e.stream == Stream::Out)
        .map(|e| e.text.as_str())
        .collect();
    assert_eq!(ticks, ["tick 1", "tick 2", "tick 3", "tick 4", "tick 5"]);
    assert!(mine.iter().all(|e| !e.at.is_empty()));
}
