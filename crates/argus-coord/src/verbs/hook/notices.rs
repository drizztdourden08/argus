//! The notice digest of the prompt hook: one line about the passive notices the wake policy kept
//! off the thread's listener since its last turn (count by kind, the newest few), read from the
//! dashboard (`GET /api/p/:key/wakes/notices`). A lead counts every thread of its project, as it
//! hears every thread's wakes. Best effort: a dashboard that is down or slow gives no line.

use std::time::Duration;

use argus_board::json::truthy;
use argus_core::js;

use super::ctx::Hook;
use super::visit::Hit;
use super::wake_line::is_lead;
use crate::client::dashboard_client;

/// The longest the prompt waits for the digest.
const NOTICE_TIMEOUT: Duration = Duration::from_millis(1500);

fn notices_path(project: &str, thread: &str, since: Option<&str>) -> String {
    let mut path = format!(
        "/api/p/{project}/wakes/notices?thread={}",
        js::encode_uri_component(thread)
    );
    if let Some(since) = since {
        path.push_str("&since=");
        path.push_str(&js::encode_uri_component(since));
    }
    path
}

/// The digest line of a registered thread, when it has notices since its last turn.
pub fn notice_line(hook: &Hook<'_>, project: &str, hit: &Hit) -> Option<String> {
    if hit.adopted || truthy(&hit.thread, "placeholder") {
        return None;
    }
    let id = hit.thread["id"].as_str()?;
    let thread = if is_lead(&hit.thread) { "*" } else { id };
    let path = notices_path(project, thread, hit.seen_before.as_deref());
    let client = dashboard_client(hook.env, &hook.paths, false).ok()?;
    let body = client
        .request_within(NOTICE_TIMEOUT, "GET", &path, None)
        .ok()?;
    body["line"]
        .as_str()
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
}
