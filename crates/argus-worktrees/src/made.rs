//! The folders a removal left behind (git could not delete them, or left them): leftovers exempt from the age rule.
//!
//! Kept as `worktrees:made:<project>` = `{<path key>: {at, path}}`, for 14 days and while the folder is on disk.

use crate::clock::DAY_MS;
use crate::paths::{clean_path, path_key};
use crate::service::Service;
use serde_json::{Map, Value, json};

/// How long a made leftover stays exempt.
pub const MADE_DAYS: i64 = 14;

fn made_key(project: &str) -> String {
    format!("worktrees:made:{project}")
}

fn entry_at(entry: &Value) -> Option<i64> {
    entry.get("at")?.as_f64().map(|at| {
        #[allow(clippy::cast_possible_truncation)]
        let ms = at as i64;
        ms
    })
}

fn entry_path(entry: &Value) -> Option<&str> {
    entry.get("path")?.as_str()
}

impl Service {
    fn load_made(&self, project: &str) -> Map<String, Value> {
        let text = self
            .deps
            .store
            .kv_get(&made_key(project))
            .unwrap_or_else(|| "{}".to_string());
        match serde_json::from_str::<Value>(&text) {
            Ok(Value::Object(map)) => map,
            _ => Map::new(),
        }
    }

    fn save_made(&self, project: &str, map: Map<String, Value>) {
        let _ = self
            .deps
            .store
            .kv_set(&made_key(project), &Value::Object(map).to_string());
    }

    /// Remember a folder a removal left: the next removal takes it without the age limit.
    pub(crate) fn remember_made(&self, project: &str, path: &str) {
        let mut map = self.load_made(project);
        map.insert(
            path_key(path, self.ci()),
            json!({"at": self.now(), "path": clean_path(path)}),
        );
        self.save_made(project, map);
    }

    /// Did a removal make this leftover, less than 14 days ago?
    pub fn is_made(&self, project: &str, path: &str) -> bool {
        let map = self.load_made(project);
        map.get(&path_key(path, self.ci()))
            .and_then(entry_at)
            .is_some_and(|at| self.now() - at < MADE_DAYS * DAY_MS)
    }

    /// Forget the folders that are gone or older than 14 days; the number forgotten.
    pub fn prune_made(&self, project: &str) -> usize {
        let map = self.load_made(project);
        let total = map.len();
        let kept: Map<String, Value> = map
            .into_iter()
            .filter(|(_, e)| {
                entry_at(e).is_some_and(|at| self.now() - at < MADE_DAYS * DAY_MS)
                    && entry_path(e).is_some_and(|p| self.deps.fs.exists(p))
            })
            .collect();
        let dropped = total - kept.len();
        if dropped > 0 {
            self.save_made(project, kept);
        }
        dropped
    }
}
