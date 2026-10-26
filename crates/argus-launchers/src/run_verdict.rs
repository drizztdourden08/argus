//! How a run ended: its final status, the error it keeps, and the dashboard's line about it.

use crate::host::ExitInfo;
use crate::record::RunStatus;

/// What the supervisor knows when the run is over.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EndInput {
    pub stop_by: Option<String>,
    pub timed_out: bool,
    pub timeout_sec: u32,
    pub exit: Option<ExitInfo>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verdict {
    pub status: RunStatus,
    pub error: Option<String>,
    pub note: String,
}

fn exit_note(exit: Option<&ExitInfo>, exited: bool) -> String {
    if exited {
        return "exited with code 0".to_owned();
    }
    match exit {
        Some(ExitInfo {
            signal: Some(signal),
            ..
        }) => format!("ended by signal {signal}"),
        Some(ExitInfo {
            code: Some(code), ..
        }) => format!("exited with code {code}"),
        _ => "exited with code null".to_owned(),
    }
}

/// A stop wins over a timeout, which wins over the exit.
pub fn verdict(input: &EndInput) -> Verdict {
    if let Some(by) = &input.stop_by {
        return Verdict {
            status: RunStatus::Stopped,
            error: None,
            note: format!("stopped by {by}"),
        };
    }
    if input.timed_out {
        return Verdict {
            status: RunStatus::Failed,
            error: Some("timeout".to_owned()),
            note: format!("timed out after {}s and was killed", input.timeout_sec),
        };
    }
    let exited = input.exit.as_ref().is_some_and(|e| e.code == Some(0));
    Verdict {
        status: if exited {
            RunStatus::Exited
        } else {
            RunStatus::Failed
        },
        error: None,
        note: exit_note(input.exit.as_ref(), exited),
    }
}
