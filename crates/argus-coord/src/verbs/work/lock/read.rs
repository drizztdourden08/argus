//! `coord lock list`.

use argus_board::json::{nullish, section};
use serde_json::Value;

use super::holder_of;
use crate::cli::Outcome;
use crate::cli::out::{ago, table};
use crate::verbs::work::keys::js_keys;
use crate::verbs::work::run::Run;

const HEAD: [&str; 5] = ["resource", "holder", "since", "queue", "about"];

fn row(lock: &Value, now: i64) -> Vec<String> {
    let holder = holder_of(lock);
    let queue: Vec<String> = super::queue_of(lock).iter().map(argus_board::json::display).collect();
    vec![
        argus_board::json::display(&lock["resource"]),
        holder.clone().unwrap_or_else(|| "free".to_owned()),
        if holder.is_some() { ago(lock["since"].as_str(), now) } else { String::new() },
        queue.join(" > "),
        nullish(lock, "description").unwrap_or_default(),
    ]
}

/// `coord lock list`.
pub(super) fn list(r: &mut Run<'_>) -> Outcome {
    r.read(|board, r| {
        let locks = section(board, "locks");
        let now = r.now_ms();
        let mut rows: Vec<Vec<String>> = js_keys(locks.keys().cloned().collect())
            .iter()
            .filter_map(|key| locks.get(key))
            .map(|lock| row(lock, now))
            .collect();
        if rows.is_empty() {
            r.say(&["no locks."]);
        } else {
            rows.insert(0, HEAD.iter().map(|h| (*h).to_owned()).collect());
            r.say(&[&table(&rows)]);
        }
        Ok(())
    })
}
