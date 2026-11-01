//! The requests of every project, kept in memory and in the key-value table (`worktrees:requests:<project>`).

use super::KEPT;
use super::record::{Brief, RequestRec};
use crate::service::Service;
use crate::sync::locked;
use serde::Serialize;
use serde_json::Value;

/// The key-value key of a project's requests.
pub fn requests_key(project: &str) -> String {
    format!("worktrees:requests:{project}")
}

/// A project's requests, oldest first, with the counter of the next id.
#[derive(Debug, Default, Serialize)]
pub(crate) struct Store {
    pub seq: u64,
    pub list: Vec<RequestRec>,
}

fn number_of(id: &str) -> u64 {
    id.strip_prefix("R-")
        .filter(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}

impl Store {
    /// What the table holds; an unreadable record starts a new list.
    pub(crate) fn parse(text: &str) -> Self {
        let Ok(value) = serde_json::from_str::<Value>(text) else {
            return Self::default();
        };
        let Some(items) = value.get("list").and_then(Value::as_array) else {
            return Self::default();
        };
        let list: Vec<RequestRec> = items
            .iter()
            .filter_map(|item| serde_json::from_value::<RequestRec>(item.clone()).ok())
            .filter(|rec| !rec.id.is_empty())
            .collect();
        let kept = value.get("seq").and_then(Value::as_u64).unwrap_or(0);
        let highest = list.iter().map(|r| number_of(&r.id)).max().unwrap_or(0);
        Self {
            seq: kept.max(highest),
            list,
        }
    }

    pub(crate) fn active(&self) -> usize {
        self.list.iter().filter(|r| r.status.active()).count()
    }

    /// The list is cut to the last 100: finished ones go oldest first, an active one is never dropped.
    pub(crate) fn cut(&mut self) {
        let room = KEPT.saturating_sub(self.active());
        let finished = self.list.len() - self.active();
        let mut drop = finished.saturating_sub(room);
        self.list.retain(|rec| {
            if rec.status.active() || drop == 0 {
                return true;
            }
            drop -= 1;
            false
        });
    }
}

impl Service {
    /// Runs `f` on the project's requests, loading them first when this is the first look. Locked for short
    /// stretches only: never call git, the removal or a derive from `f`.
    pub(crate) fn with_requests<T>(&self, project: &str, f: impl FnOnce(&mut Store) -> T) -> T {
        let mut state = locked(&self.requests.state);
        let store = state
            .stores
            .entry(project.to_string())
            .or_insert_with(|| self.load_requests(project));
        f(store)
    }

    fn load_requests(&self, project: &str) -> Store {
        let text = self
            .deps
            .store
            .kv_get(&requests_key(project))
            .unwrap_or_default();
        Store::parse(&text)
    }

    /// Cut the list and write it down. A failed write is not fatal (the next save writes again), only logged.
    pub(crate) fn save_requests(&self, project: &str) {
        let written = self.with_requests(project, |store| {
            store.cut();
            let text = serde_json::to_string(store).unwrap_or_default();
            self.deps.store.kv_set(&requests_key(project), &text)
        });
        if let Err(err) = written {
            self.deps.notifier.problem(&format!(
                "{project}: the worktree requests were not saved: {err}"
            ));
        }
    }

    /// Change one request, found by id.
    pub(crate) fn update_request(&self, project: &str, id: &str, f: impl FnOnce(&mut RequestRec)) {
        self.with_requests(project, |store| {
            if let Some(rec) = store.list.iter_mut().find(|r| r.id == id) {
                f(rec);
            }
        });
    }

    /// A copy of one request.
    pub(crate) fn request_of(&self, project: &str, id: &str) -> Option<RequestRec> {
        self.with_requests(project, |store| {
            store.list.iter().find(|r| r.id == id).cloned()
        })
    }

    /// The project's requests, newest first.
    pub fn request_list(&self, project: &str) -> Vec<RequestRec> {
        let now = self.now();
        let mut list: Vec<RequestRec> =
            self.with_requests(project, |store| store.list.iter().rev().cloned().collect());
        for current in list.iter_mut().filter_map(|r| r.current.as_mut()) {
            current.for_s = (now - current.since_ms).max(0) / 1000;
            current.step = self.step_of(project, &current.id);
        }
        list
    }

    /// The newest `n` requests in their brief form, newest first: what the page's summary carries.
    pub fn requests_recent(&self, project: &str, n: usize) -> Vec<Brief> {
        self.with_requests(project, |store| {
            store
                .list
                .iter()
                .rev()
                .take(n)
                .map(RequestRec::brief)
                .collect()
        })
    }

    /// Block until every worker has run what was queued, and told what it finished.
    pub fn requests_idle(&self) {
        self.requests.queues.wait_all_idle();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::requests::record::RequestStatus;

    fn rec(id: &str, status: RequestStatus) -> RequestRec {
        RequestRec {
            id: id.into(),
            status,
            ..RequestRec::default()
        }
    }

    #[test]
    fn the_counter_never_falls_below_the_ids_in_the_list() {
        let text = r#"{"seq":1,"list":[{"id":"R-7"},{"id":"R-2"},{"nothing":true}]}"#;
        let store = Store::parse(text);
        assert_eq!((store.seq, store.list.len()), (7, 2));
        assert_eq!(Store::parse("not json").list.len(), 0);
        assert_eq!(Store::parse(r#"{"seq":5,"list":"no"}"#).seq, 0);
    }

    #[test]
    fn the_cut_keeps_the_newest_hundred_and_every_active_one() {
        let mut store = Store::default();
        for n in 0..105 {
            store.list.push(rec(&format!("R-{n}"), RequestStatus::Done));
        }
        store.list.insert(3, rec("R-live", RequestStatus::Running));
        store.cut();
        assert_eq!(store.list.len(), 100);
        assert!(store.list.iter().any(|r| r.id == "R-live"));
        assert_eq!(store.list.get(1).map(|r| r.id.as_str()), Some("R-6"));
    }
}
