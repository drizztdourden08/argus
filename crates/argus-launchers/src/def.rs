//! A run definition: what a lead defined with `coord run define` (or a launcher's Play),
//! read from the board. Anything stored on a board is untrusted in shape: defaults are filled
//! in and junk is dropped.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::names::is_launch_name;

const DEFAULT_TIMEOUT_SEC: u32 = 1800;
const MAX_TIMEOUT_SEC: i64 = 86_400;
const MAX_PARALLEL: i64 = 8;
const MAX_REV: i64 = 1_000_000_000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunKind {
    Build,
    Test,
    Launch,
    #[default]
    Task,
}

impl RunKind {
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "build" => Some(Self::Build),
            "test" => Some(Self::Test),
            "launch" => Some(Self::Launch),
            "task" => Some(Self::Task),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Test => "test",
            Self::Launch => "launch",
            Self::Task => "task",
        }
    }

    fn default_timeout(self) -> u32 {
        match self {
            Self::Build | Self::Test => DEFAULT_TIMEOUT_SEC,
            Self::Launch | Self::Task => 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunDef {
    pub name: String,
    pub cmd: String,
    pub cwd: String,
    pub kind: RunKind,
    pub env: BTreeMap<String, String>,
    pub hand: bool,
    /// A legacy bundle tag on a stored definition; nothing writes it any more.
    pub bundle: Option<String>,
    pub launcher: Option<String>,
    pub window: bool,
    pub visible: bool,
    pub timeout_sec: u32,
    pub parallel: u32,
    pub defined_by: String,
    pub defined_at: String,
    pub rev: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl Default for RunDef {
    fn default() -> Self {
        Self {
            name: String::new(),
            cmd: String::new(),
            cwd: String::new(),
            kind: RunKind::Task,
            env: BTreeMap::new(),
            hand: false,
            bundle: None,
            launcher: None,
            window: false,
            visible: false,
            timeout_sec: 0,
            parallel: 1,
            defined_by: "unknown".to_owned(),
            defined_at: String::new(),
            rev: 1,
            label: None,
        }
    }
}

impl RunDef {
    /// True for a definition made by a launcher or a legacy bundle: it keeps a stdin pipe and
    /// asks the tool for colour.
    pub fn is_playable(&self) -> bool {
        self.bundle.is_some() || self.launcher.is_some()
    }
}

/// `Number(value)` as an integer inside `min..=max`, else the fallback.
fn int(value: Option<&Value>, fallback: u32, min: i64, max: i64) -> u32 {
    let number = match value {
        Some(Value::Number(n)) => n.as_i64(),
        Some(Value::String(s)) => s.trim().parse::<i64>().ok(),
        _ => None,
    };
    number
        .filter(|n| (min..=max).contains(n))
        .and_then(|n| u32::try_from(n).ok())
        .unwrap_or(fallback)
}

fn text(obj: &Map<String, Value>, key: &str) -> Option<String> {
    obj.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

fn flag(obj: &Map<String, Value>, key: &str) -> bool {
    obj.get(key) == Some(&Value::Bool(true))
}

fn env_of(obj: &Map<String, Value>) -> BTreeMap<String, String> {
    obj.get("env")
        .and_then(Value::as_object)
        .map(|env| {
            env.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_owned())))
                .collect()
        })
        .unwrap_or_default()
}

fn timeout_of(obj: &Map<String, Value>, kind: RunKind) -> u32 {
    match obj.get("timeoutSec") {
        None | Some(Value::Null) => kind.default_timeout(),
        other => int(other, 0, 0, MAX_TIMEOUT_SEC),
    }
}

/// The definition `raw` describes, or `None` when it has no command. A launcher's Play
/// definition and a legacy bundle's run with the console hidden and the output in the
/// dashboard's terminal; only `window: true` gives one a visible console.
pub fn coerce_def(name: &str, raw: &Value) -> Option<RunDef> {
    let obj = raw.as_object()?;
    let cmd = obj.get("cmd")?.as_str().filter(|c| !c.trim().is_empty())?;
    let kind = obj
        .get("kind")
        .and_then(Value::as_str)
        .and_then(RunKind::parse)
        .unwrap_or_default();
    let hand = flag(obj, "hand");
    let bundle = text(obj, "bundle");
    let launcher = text(obj, "launcher");
    let window = flag(obj, "window");
    let wants_visible = if bundle.is_some() || launcher.is_some() {
        window
    } else {
        flag(obj, "visible")
    };
    Some(RunDef {
        name: name.to_owned(),
        cmd: cmd.to_owned(),
        cwd: obj
            .get("cwd")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        kind,
        env: env_of(obj),
        hand,
        bundle,
        launcher,
        window,
        visible: hand && wants_visible,
        timeout_sec: timeout_of(obj, kind),
        parallel: int(obj.get("parallel"), 1, 1, MAX_PARALLEL),
        defined_by: text(obj, "definedBy").unwrap_or_else(|| "unknown".to_owned()),
        defined_at: obj
            .get("definedAt")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        rev: int(obj.get("rev"), 1, 1, MAX_REV),
        label: text(obj, "label"),
    })
}

/// Every usable definition of a `runDefs` map, sorted by name.
pub fn defs_from_map(map: &Map<String, Value>) -> Vec<RunDef> {
    let mut defs: Vec<RunDef> = map
        .iter()
        .filter(|(name, _)| is_launch_name(name))
        .filter_map(|(name, raw)| coerce_def(name, raw))
        .collect();
    defs.sort_by(|a, b| a.name.cmp(&b.name));
    defs
}

/// What a page or the API may see of a definition: the env values and the board's label stay on the board.
pub fn public_def(def: &RunDef) -> RunDef {
    RunDef {
        env: BTreeMap::new(),
        label: None,
        ..def.clone()
    }
}
