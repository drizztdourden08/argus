//! A launcher as `list` and `show` print it (and `--json` returns it): the record plus its run
//! name, bundles and, for a standalone one, its usage (`useEvery`, `lastUsedAt`, `stale`: by the
//! board record; the dashboard also counts a Play run as a use).

use argus_board::Board;
use argus_board::json::{display, doc};
use argus_launchers::launcher::{
    LauncherBoard, Platform, Usage, is_standalone, joined_bundle, launcher_def_name, shell_of,
    usage_of,
};
use serde_json::{Map, Value};

use crate::verbs::agent::base::links::linked;

/// One launcher, read.
pub struct Row {
    /// The record as the board holds it.
    pub rec: Value,
    /// The name of its Play definition, when it has one.
    pub play: Option<String>,
    /// The bundles that link it.
    pub bundles: Vec<String>,
    /// The bundle it goes with.
    pub joined: Option<String>,
    /// Not master and not joined: it has a usage window.
    pub standalone: bool,
    /// Its usage, for a live standalone launcher.
    pub usage: Option<Usage>,
}

impl Row {
    /// The row as `--json` prints it: the record, then what was worked out.
    #[must_use]
    pub fn to_json(&self) -> Value {
        let mut map: Map<String, Value> = self.rec.as_object().cloned().unwrap_or_default();
        map.insert("runsWith".to_owned(), Value::from(shell_of(Platform::current())));
        map.insert("play".to_owned(), Value::from(self.play.clone()));
        map.insert("bundles".to_owned(), Value::from(self.bundles.clone()));
        map.insert("joined".to_owned(), Value::from(self.joined.clone()));
        map.insert("standalone".to_owned(), Value::from(self.standalone));
        if let Some(u) = &self.usage {
            map.insert("useEveryEffective".to_owned(), Value::from(u.use_every.as_str()));
            map.insert("useEveryLegacy".to_owned(), Value::from(u.legacy));
            map.insert("lastUsedAt".to_owned(), Value::from(u.last_used_at.as_str()));
            map.insert("stale".to_owned(), Value::from(u.stale));
        }
        Value::Object(map)
    }
}

/// The row of one launcher record.
#[must_use]
pub fn row_of(board: &Board, typed: &LauncherBoard, rec: &Value, now_ms: i64) -> Row {
    let name = display(&rec["name"]);
    let def = launcher_def_name(&name);
    let own = doc(board, "runDefs", &def).is_some_and(|d| d["launcher"].as_str() == Some(name.as_str()));
    let known = typed.launchers.get(&name);
    Row {
        rec: rec.clone(),
        play: own.then_some(def),
        bundles: linked(board, &name, None),
        joined: known.and_then(|l| joined_bundle(typed, l)),
        standalone: known.is_some_and(|l| is_standalone(typed, l)),
        usage: known.and_then(|l| usage_of(typed, l, now_ms, 0)),
    }
}
