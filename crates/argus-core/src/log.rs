//! The server log: JSON lines in `logs/server.log`, rotated at 5 MB with five files kept
//! (`server.log.1` is the newest). String values pass the redactor first; a log that cannot be
//! written never takes the server down.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use serde_json::{Map, Value, json};

use crate::clock::now_iso;
use crate::redact::Redactor;

/// Rotation size.
pub const LOG_MAX_BYTES: u64 = 5 * 1024 * 1024;
/// Rotated files kept.
pub const LOG_KEEP: u32 = 5;

struct State {
    size: Option<u64>,
    redactor: Redactor,
}

/// A JSON-lines logger.
pub struct Logger {
    file: PathBuf,
    max_bytes: u64,
    echo: bool,
    state: Mutex<State>,
}

impl Logger {
    /// A logger writing `file`, rotating past `max_bytes`.
    #[must_use]
    pub fn new(file: impl Into<PathBuf>, redactor: Redactor, max_bytes: u64) -> Self {
        Logger {
            file: file.into(),
            max_bytes,
            echo: false,
            state: Mutex::new(State {
                size: None,
                redactor,
            }),
        }
    }

    /// The same logger, also writing each line to stderr.
    #[must_use]
    pub fn echo(mut self, on: bool) -> Self {
        self.echo = on;
        self
    }

    /// The log file.
    #[must_use]
    pub fn file(&self) -> &Path {
        &self.file
    }

    /// Replaces the redactor.
    pub fn set_redactor(&self, r: Redactor) {
        self.state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .redactor = r;
    }

    /// An `info` line.
    pub fn info(&self, msg: &str, fields: Value) {
        self.write("info", msg, fields);
    }

    /// A `warn` line.
    pub fn warn(&self, msg: &str, fields: Value) {
        self.write("warn", msg, fields);
    }

    /// An `error` line.
    pub fn error(&self, msg: &str, fields: Value) {
        self.write("error", msg, fields);
    }

    fn write(&self, level: &str, msg: &str, fields: Value) {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        let mut rec = Map::new();
        rec.insert("t".into(), json!(now_iso()));
        rec.insert("level".into(), json!(level));
        rec.insert("msg".into(), json!(msg));
        if let Value::Object(f) = fields {
            rec.extend(f);
        }
        let clean = scrub(&state.redactor, Value::Object(rec), 0);
        let line = format!("{clean}\n");
        let _ = self.append(&mut state, &line);
        if self.echo {
            eprint!("{line}");
        }
    }

    fn append(&self, state: &mut State, line: &str) -> std::io::Result<()> {
        let size = if let Some(s) = state.size {
            s
        } else {
            if let Some(dir) = self.file.parent() {
                fs::create_dir_all(dir)?;
            }
            fs::metadata(&self.file).map_or(0, |m| m.len())
        };
        let len = line.len() as u64;
        let size = if size + len > self.max_bytes && size > 0 {
            self.rotate().map(|()| 0)?
        } else {
            size
        };
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file)?
            .write_all(line.as_bytes())?;
        state.size = Some(size + len);
        Ok(())
    }

    fn rotate(&self) -> std::io::Result<()> {
        let nth = |i: u32| PathBuf::from(format!("{}.{i}", self.file.display()));
        let _ = fs::remove_file(nth(LOG_KEEP));
        for i in (1..LOG_KEEP).rev() {
            if nth(i).exists() {
                fs::rename(nth(i), nth(i + 1))?;
            }
        }
        fs::rename(&self.file, nth(1))
    }
}

fn scrub(r: &Redactor, v: Value, depth: u32) -> Value {
    match v {
        Value::String(s) => Value::String(r.redact(&s)),
        _ if depth > 3 => v,
        Value::Array(a) => Value::Array(a.into_iter().map(|x| scrub(r, x, depth + 1)).collect()),
        Value::Object(o) => Value::Object(
            o.into_iter()
                .map(|(k, x)| (k, scrub(r, x, depth + 1)))
                .collect(),
        ),
        other => other,
    }
}

/// An error as a log field: its message.
#[must_use]
pub fn err_field(e: &dyn std::error::Error) -> Value {
    json!({ "message": e.to_string() })
}
