//! The wait itself: `GET /api/p/<key>/wakes/stream?thread=<t>&v=2&id=<label>` on the running
//! dashboard, read as server-sent events until a `wake` or a `replaced` event. A refused or
//! broken stream starts the dashboard; any end of a stream is retried 5 s later, until the
//! deadline.

use std::cmp::{max, min};
use std::io::Read;
use std::thread::sleep;
use std::time::Duration;

use argus_core::clock::now_ms;
use argus_core::js::encode_uri_component as enc;

use super::super::Said;
use super::super::autostart::ensure_dashboard;
use super::super::ctx::Hook;
use super::super::read_stamp::stamp_inbox;
use super::Target;
use super::line::{all_from, wake_line};
use crate::client::{Client, ClientError, DashboardConfig, SseReader, server_port};

const RETRY_MS: i64 = 5000;
const MIN_WAIT_MS: i64 = 1000;

/// How one stream ended.
enum Heard {
    /// A newer listener of the thread took over: end silently.
    Replaced,
    /// A wake, with its line.
    Wake(String),
    /// The stream ended or the server refused it: retry.
    Ended,
    /// The dashboard did not answer or the stream broke: start it, then retry.
    Broken,
}

/// A lead (`*`) skips the wakes its own thread sent; everyone else hears every wake.
fn own_wake(target: &Target, data: &str) -> bool {
    target.thread == "*" && all_from(data, &target.id)
}

fn read_events(body: Box<dyn Read + Send>, target: &Target) -> Heard {
    let mut reader = SseReader::new(body);
    loop {
        match reader.next_event() {
            Ok(Some(event)) if event.name == "replaced" => return Heard::Replaced,
            Ok(Some(event)) if event.name == "wake" && own_wake(target, &event.data) => {}
            Ok(Some(event)) if event.name == "wake" && !event.data.is_empty() => {
                return Heard::Wake(wake_line(&event.data, &target.thread));
            }
            Ok(Some(_)) => {}
            Ok(None) => return Heard::Ended,
            Err(_) => return Heard::Broken,
        }
    }
}

fn listen_once(hook: &Hook<'_>, target: &Target, deadline: i64) -> Heard {
    let client = Client::new(&DashboardConfig {
        port: server_port(&hook.paths),
        token: None,
        install_dir: None,
    });
    let path = format!(
        "/api/p/{}/wakes/stream?thread={}&v=2&id={}",
        enc(&target.project),
        enc(&target.thread),
        enc(&target.label)
    );
    let wait = max(MIN_WAIT_MS, deadline - now_ms()).unsigned_abs();
    match client.stream(&path, Duration::from_millis(wait)) {
        Ok(body) => read_events(body, target),
        Err(ClientError::Http { .. } | ClientError::Timeout(_)) => Heard::Ended,
        Err(ClientError::Down(_)) => Heard::Broken,
    }
}

/// Waits for a wake of `target` until `deadline` (epoch ms).
pub fn wait(hook: &Hook<'_>, target: &Target, deadline: i64) -> Said {
    while now_ms() < deadline {
        match listen_once(hook, target, deadline) {
            Heard::Replaced => return Said::Nothing,
            Heard::Wake(line) => {
                stamp_inbox(hook, &target.project, &target.id);
                return Said::Wake(line);
            }
            Heard::Ended => {}
            Heard::Broken => ensure_dashboard(hook.env, &hook.paths),
        }
        let left = min(RETRY_MS, deadline - now_ms());
        if left > 0 {
            sleep(Duration::from_millis(left.unsigned_abs()));
        }
    }
    Said::Nothing
}
