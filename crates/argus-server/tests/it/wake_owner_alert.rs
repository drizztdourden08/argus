//! A wake fault that stays open for an hour reaches the owner as a toast and a notification, once.
//! The wake clock is manual, so the hour passes in one step.
// test-tag: critical

use std::sync::Arc;

use crate::toasts::Toasts;
use crate::wait::until;
use crate::wake_rig::{fault_config, post_wake};
use argus_native::Notifier;
use argus_server::ServeOptions;
use argus_testkit::{TestOptions, TestServer, Wakes, alpha_board, make_home, write_board};
use serde_json::json;
use tempfile::TempDir;

const HOUR_MS: i64 = 3_600_000;
const TITLE: &str = "Wake fault open for";

async fn faulted_server(toasts: &Arc<Toasts>) -> (TestServer, TempDir) {
    let home = tempfile::tempdir().expect("coord home");
    make_home(home.path());
    write_board(home.path(), "alpha", &alpha_board(home.path()));
    let sink: Arc<dyn Notifier> = toasts.clone();
    let config = fault_config();
    let t = TestServer::start(TestOptions {
        coord_home: Some(home.path().to_path_buf()),
        config,
        wakes: Wakes::Manual,
        tweak: Some(Arc::new(move |opts: &mut ServeOptions| {
            opts.notify.notifier = Some(Arc::clone(&sink));
        })),
        ..TestOptions::default()
    })
    .await;
    (t, home)
}

fn fault_toasts(toasts: &Toasts) -> usize {
    toasts
        .shown()
        .iter()
        .filter(|toast| toast.title.starts_with(TITLE))
        .count()
}

#[tokio::test(flavor = "multi_thread")]
async fn a_fault_open_past_an_hour_reaches_the_owner_once() {
    let toasts = Arc::new(Toasts::default());
    let (t, _home) = faulted_server(&toasts).await;
    let body = json!({ "thread": "ui", "reason": "thread-message", "text": "lead: look at ui" });
    assert_eq!(post_wake(&t, "alpha", &body).await.json()["queued"], true);
    t.advance_wakes(1000);
    t.advance_wakes(1000);
    assert_eq!(fault_toasts(&toasts), 0, "not before the hour");
    t.advance_wakes(HOUR_MS);
    until("the owner toast", || async {
        (fault_toasts(&toasts) == 1).then_some(())
    })
    .await;
    t.advance_wakes(HOUR_MS);
    assert_eq!(fault_toasts(&toasts), 1, "once per fault");
}
