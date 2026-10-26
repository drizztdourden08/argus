//! A [`RunPatch`] as the columns it sets.

use argus_launchers::RunPatch;
use argus_store::rusqlite::types::Value;

type Assignments = Vec<(&'static str, Value)>;

fn nullable<T: Into<Value>>(value: Option<T>) -> Value {
    value.map_or(Value::Null, Into::into)
}

fn state(patch: &RunPatch, out: &mut Assignments) {
    if let Some(status) = patch.status {
        out.push(("status", status.as_str().to_owned().into()));
    }
    if let Some(code) = patch.exit_code {
        out.push(("exit_code", nullable(code)));
    }
    if let Some(signal) = &patch.signal {
        out.push(("signal", nullable(signal.clone())));
    }
    if let Some(pid) = patch.pid {
        out.push(("pid", nullable(pid.map(i64::from))));
    }
    if let Some(stdin) = patch.stdin {
        out.push(("stdin", stdin.into()));
    }
}

fn story(patch: &RunPatch, out: &mut Assignments) {
    if let Some(by) = &patch.by {
        out.push(("\"by\"", by.clone().into()));
    }
    if let Some(at) = &patch.started_at {
        out.push(("started_at", nullable(at.clone())));
    }
    if let Some(at) = &patch.ended_at {
        out.push(("ended_at", nullable(at.clone())));
    }
    if let Some(orphan) = patch.orphan {
        out.push(("orphan", orphan.into()));
    }
    if let Some(error) = &patch.error {
        out.push(("error", nullable(error.clone())));
    }
}

/// The `column = value` pairs of the fields the patch sets.
pub(super) fn assignments(patch: &RunPatch) -> Assignments {
    let mut out = Vec::new();
    state(patch, &mut out);
    story(patch, &mut out);
    out
}
