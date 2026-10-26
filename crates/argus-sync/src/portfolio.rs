//! The Argus scope (`_portfolio`): one card per project, computed live from the boards the
//! exporter holds plus the runs each has going, the owner's portfolio posts, and the deliveries
//! still waiting for a board. Cards live only in the doc mirror.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use argus_board::portfolio::{
    dispatch_entry, import_portfolio_entries, project_keys, shape, summarize,
};
use argus_board::store::{LockWait, with_board_async};
use argus_board::{Board, BoardError};
use argus_core::ApiError;
use argus_core::ids::PORTFOLIO;
use serde_json::{Map, Value, json};

use crate::exporter::Exported;
use crate::failure::board_error;
use crate::read::read_board_file;
use crate::state::{State, clear_marks};

const DEBOUNCE: Duration = Duration::from_millis(500);

fn blank_portfolio() -> Board {
    json!({ "project": PORTFOLIO, "kind": "portfolio", "name": "Argus", "projects": {}, "entries": {}, "counters": {}, "threads": {}, "items": {}, "tests": {}, "locks": {}, "actions": {}, "agents": {}, "log": {}, "runDefs": {} })
}

fn pending(e: &Value) -> Vec<String> {
    e["pendingFor"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn delivered(e: &Value) -> Vec<String> {
    e["deliveredTo"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn entries(p: &Board) -> impl Iterator<Item = (&String, &Value)> {
    p["entries"].as_object().into_iter().flatten()
}

/// Every pending entry handed down again; answers the keys newly delivered.
fn retry_all(home: &std::path::Path, p: &mut Board) -> Result<Vec<String>, BoardError> {
    shape(p);
    let ids: Vec<String> = entries(p)
        .filter(|(_, e)| !pending(e).is_empty())
        .map(|(id, _)| id.clone())
        .collect();
    let mut newly = Vec::new();
    for id in ids {
        let before = delivered(&p["entries"][&id]);
        dispatch_entry(home, p, &id)?;
        newly.extend(
            delivered(&p["entries"][&id])
                .into_iter()
                .filter(|k| !before.contains(k)),
        );
    }
    clear_marks(p);
    Ok(newly)
}

impl State {
    fn card_with_runs(&self, key: &str, p: &Board) -> Value {
        let board = self.board(key);
        let mut card = summarize(self.coord_home(), key, p, board.as_ref());
        let runs = self.opts.runs.as_ref();
        let play = card["master"]["play"].as_str().map(str::to_owned);
        if let (Some(play), Some(master)) =
            (play, card.get_mut("master").and_then(Value::as_object_mut))
        {
            master.insert(
                "lastRun".into(),
                runs.and_then(|r| r.last_run(key, &play))
                    .unwrap_or(Value::Null),
            );
        }
        if let Some(o) = card.as_object_mut() {
            o.insert(
                "runs".into(),
                json!({ "running": runs.map_or(0, |r| r.running(key)) }),
            );
        }
        card
    }

    fn cards_for(&self, p: &Board) -> Vec<Value> {
        project_keys(self.coord_home(), p)
            .iter()
            .map(|k| self.card_with_runs(k, p))
            .collect()
    }

    fn export_with_cards(self: &Arc<Self>, p: Board) -> Result<Exported, ApiError> {
        let cards = self.cards_for(&p);
        let mut full = p;
        let by_key: Map<String, Value> = cards
            .iter()
            .map(|c| (c["key"].as_str().unwrap_or_default().to_owned(), c.clone()))
            .collect();
        if let Some(o) = full.as_object_mut() {
            o.insert("cards".into(), Value::Object(by_key));
        }
        self.set_cards(cards);
        self.export_board(PORTFOLIO, full)
    }

    async fn retry_pending(self: &Arc<Self>, p: Board) -> Result<(Board, Vec<String>), ApiError> {
        let waiting = entries(&p).any(|(_, e)| {
            pending(e).iter().any(|k| {
                self.board(k).is_some()
                    || argus_board::store::board_path(self.coord_home(), k).exists()
            })
        });
        if !waiting {
            return Ok((p, Vec::new()));
        }
        let home = self.coord_home().to_path_buf();
        with_board_async(self.coord_home(), PORTFOLIO, LockWait::default(), |pp| {
            let newly = retry_all(&home, pp)?;
            Ok((pp.clone(), newly))
        })
        .await
        .map_err(board_error)
    }

    /// Recomputes every card and exports the portfolio scope.
    pub async fn refresh_portfolio(self: &Arc<Self>) -> Result<Option<Exported>, ApiError> {
        if self.stopped() {
            return Ok(None);
        }
        let read = read_board_file(self.coord_home(), PORTFOLIO)
            .await
            .map_err(board_error)?;
        let mut p = read.unwrap_or_else(blank_portfolio);
        shape(&mut p);
        let (p, newly) = self.retry_pending(p).await?;
        let out = self.export_with_cards(p)?;
        for key in newly {
            self.export_key(&key);
        }
        Ok(Some(out))
    }

    /// [`Self::refresh_portfolio`] on the portfolio queue.
    pub async fn refresh_portfolio_queued(self: &Arc<Self>) -> Result<Option<Exported>, ApiError> {
        let me = Arc::clone(self);
        self.opts
            .queue
            .run(PORTFOLIO, async move { me.refresh_portfolio().await })
            .await
    }

    /// A portfolio refresh 500 ms from now, unless one is already coming.
    pub fn schedule_portfolio(self: &Arc<Self>) {
        if self.stopped() || self.portfolio_pending.swap(true, Ordering::SeqCst) {
            return;
        }
        let me = Arc::clone(self);
        tokio::spawn(async move {
            tokio::time::sleep(DEBOUNCE).await;
            me.portfolio_pending.store(false, Ordering::SeqCst);
            if !me.stopped() {
                me.export_now(PORTFOLIO).await;
            }
        });
    }

    /// One owner post from the Argus page: handed down at once. Answers the stored entry, the
    /// import status and the export seq.
    pub async fn post_portfolio(
        self: &Arc<Self>,
        entry: Value,
    ) -> Result<(Option<Value>, String, i64), ApiError> {
        let me = Arc::clone(self);
        self.opts
            .queue
            .run(PORTFOLIO, async move { me.post_portfolio_now(entry).await })
            .await
    }

    async fn post_portfolio_now(
        self: &Arc<Self>,
        entry: Value,
    ) -> Result<(Option<Value>, String, i64), ApiError> {
        let id = entry["id"].as_str().unwrap_or_default().to_owned();
        let home = self.coord_home().to_path_buf();
        let opts = LockWait {
            create: true,
            ..LockWait::default()
        };
        let (saved, status) = with_board_async(self.coord_home(), PORTFOLIO, opts, |p| {
            shape(p);
            let r = import_portfolio_entries(&home, p, vec![entry])?;
            clear_marks(p);
            Ok((p.clone(), r.results.into_iter().next().map(|(_, s)| s)))
        })
        .await
        .map_err(board_error)?;
        let stored = saved["entries"].get(&id).cloned();
        let out = self.export_with_cards(saved)?;
        for key in stored.as_ref().map(delivered).unwrap_or_default() {
            self.export_key(&key);
        }
        let status = stored
            .as_ref()
            .and_then(|s| s["status"].as_str().map(str::to_owned))
            .or(status)
            .unwrap_or_else(|| "refused".into());
        Ok((stored, status, out.seq))
    }
}
