//! Unit tests of the fan-out. The rig is a real store whose bus feeds a real fan-out, and a real
//! sender whose body the tests read, so the frames checked are the frames on the wire.
// test-tag: critical

mod add;
mod shapes;

use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::time::Duration;

use argus_core::{Bus, Coll, DocChange, Subscription};
use argus_store::Db;
use axum::body::Body;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::time::timeout;

use super::*;

pub const ALPHA: &str = "alpha";

pub struct Rig {
    pub fanout: Arc<Fanout>,
    pub db: Db,
    _dir: TempDir,
    _links: Vec<Subscription>,
}

pub fn rig() -> Rig {
    let bus = Bus::new();
    let fanout = Arc::new(Fanout::default());
    let links = fanout.link(&bus);
    let dir = tempfile::tempdir().expect("a temp folder");
    let db = Db::open(&dir.path().join("argus.db"), bus).expect("the db opens");
    Rig {
        fanout,
        db,
        _dir: dir,
        _links: links,
    }
}

impl Rig {
    /// A mux client of `scope` that follows `meta`, its opening frames not yet read.
    pub fn follow_meta(&self, scope: &str, w: &Wire) -> Handle {
        let target = Target::New(Kind::Mux, &w.sender);
        let got = self
            .fanout
            .subscribe(&self.db, scope, target, &[Coll::Meta], None);
        got.expect("subscribed")
    }

    /// Writes one doc; the bus carries the event to the fan-out before this returns.
    pub fn set_in(&self, scope: &str, coll: Coll, id: &str) -> i64 {
        let change = DocChange::set(coll, id, json!({ "id": id }));
        self.db.docs().apply(scope, &[change]).expect("apply").seq
    }

    pub fn set(&self, coll: Coll, id: &str) -> i64 {
        self.set_in(ALPHA, coll, id)
    }
}

/// A sender and the body that carries what it writes.
pub struct Wire {
    pub sender: SseSender,
    body: Body,
}

pub fn wire() -> Wire {
    let open = Arc::new(AtomicUsize::new(0));
    let (sender, res) = open_stream(Duration::from_secs(60), &open, None);
    Wire {
        sender,
        body: res.into_body(),
    }
}

fn parse(text: &str) -> Option<(String, Value)> {
    let name = text.lines().find_map(|l| l.strip_prefix("event: "))?;
    let data = text.lines().find_map(|l| l.strip_prefix("data: "))?;
    Some((name.to_owned(), serde_json::from_str(data).ok()?))
}

impl Wire {
    /// Every frame queued so far, as written.
    pub async fn raw(&mut self) -> Vec<String> {
        let mut out = Vec::new();
        while let Ok(Some(Ok(f))) = timeout(Duration::from_millis(30), self.body.frame()).await {
            let bytes = f.into_data().unwrap_or_default();
            out.push(String::from_utf8_lossy(&bytes).into_owned());
        }
        out
    }

    /// Every frame queued so far, as (event, data).
    pub async fn frames(&mut self) -> Vec<(String, Value)> {
        self.raw().await.iter().filter_map(|t| parse(t)).collect()
    }
}

pub fn names(got: &[(String, Value)]) -> Vec<&str> {
    got.iter().map(|(n, _)| n.as_str()).collect()
}

/// The doc ids of a `snapshot` or `changes` frame.
pub fn ids(data: &Value) -> Vec<String> {
    let list = data.get("docs").or_else(|| data.get("changes"));
    let rows = list.and_then(Value::as_array).cloned().unwrap_or_default();
    let id = |r: &Value| r.get("id")?.as_str().map(str::to_owned);
    rows.iter().filter_map(id).collect()
}
