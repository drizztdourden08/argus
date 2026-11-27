//! The lines of the step log, as pure functions: what changed in the Desktop steps of a switch,
//! what changed in the handover cycle, and the leads the cycle found.

use argus_accounts::desktop::{Step, StepStatus};
use argus_accounts::handover::{Cycle, CycleMode, CyclePhase, Handover, HandoverState};
use argus_accounts::sync::Summary;
use serde_json::{Value, json};

use super::line::{Line, LineState, Stage};

/// The lines a change of the handover cycle makes: the ones of the process, and the resume
/// lines that come after the switch ended.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct CycleLines {
    /// Cycle started, leads asked, handovers received, every handover in.
    pub process: Vec<Line>,
    /// Leads that resumed, or did not, after the switch.
    pub resume: Vec<Line>,
}

/// The stage a Desktop step belongs to.
pub(crate) fn stage_of(step_id: &str) -> Stage {
    match step_id {
        "handover" => Stage::Handover,
        "sync" => Stage::Sync,
        "confirm" => Stage::Verify,
        _ => Stage::Switch,
    }
}

fn step_line(step: &Step) -> Option<Line> {
    let (state, word) = match step.status {
        StepStatus::Pending => return None,
        StepStatus::Running => (LineState::Running, "started"),
        StepStatus::Done => (LineState::Done, "done"),
        StepStatus::Failed => (LineState::Failed, "failed"),
        StepStatus::Skipped => (LineState::Skipped, "skipped"),
    };
    let text = format!("{}: {word}.", step.label);
    Some(Line::new(stage_of(step.id), state, text).with_detail(&step.detail))
}

/// One line per Desktop step whose status changed between `before` and `after`.
pub(crate) fn step_lines(before: &[Step], after: &[Step]) -> Vec<Line> {
    after
        .iter()
        .filter(|step| {
            before
                .iter()
                .find(|b| b.id == step.id)
                .is_none_or(|b| b.status != step.status)
        })
        .filter_map(step_line)
        .collect()
}

fn mode_text(mode: CycleMode) -> &'static str {
    match mode {
        CycleMode::Auto => "the leads are asked for an estimate first",
        CycleMode::Fixed => "every lead is asked at the fixed percent",
        CycleMode::Now => "every lead is asked at once",
    }
}

fn lead_text(h: &Handover) -> String {
    let name = format!("{}/{}", h.board, h.thread);
    match h.state {
        HandoverState::EstimateAsked => format!("{name} was asked for an estimate."),
        HandoverState::Estimated => format!("{name} sent its estimate."),
        HandoverState::Asked => format!("{name} was asked for its handover."),
        HandoverState::Writing => format!("{name} is writing its handover."),
        HandoverState::Ready => format!("Handover received from {name}."),
        HandoverState::CutOff => format!("{name} was cut off before its handover."),
        HandoverState::Resumed => format!("{name} resumed on the new account."),
        HandoverState::NotResumed => format!("{name} did not resume."),
    }
}

fn lead_line(h: &Handover) -> Line {
    use LineState::{Done, Failed, Running, Skipped, Waiting};
    let (stage, how) = match h.state {
        HandoverState::EstimateAsked | HandoverState::Asked => (Stage::Handover, Waiting),
        HandoverState::Estimated | HandoverState::Writing => (Stage::Handover, Running),
        HandoverState::Ready => (Stage::Handover, Done),
        HandoverState::CutOff => (Stage::Handover, Skipped),
        HandoverState::Resumed => (Stage::Verify, Done),
        HandoverState::NotResumed => (Stage::Verify, Failed),
    };
    Line::new(stage, how, lead_text(h))
}

fn phase_line(phase: CyclePhase) -> Option<Line> {
    match phase {
        CyclePhase::Ready => Some(Line::new(
            Stage::Handover,
            LineState::Done,
            "Every handover is in.",
        )),
        CyclePhase::Cancelled => Some(Line::new(
            Stage::Handover,
            LineState::Skipped,
            "The handover cycle was called off.",
        )),
        CyclePhase::Estimating | CyclePhase::Asked | CyclePhase::Done => None,
    }
}

/// The lines between two reads of the handover cycle.
pub(crate) fn cycle_lines(before: Option<&Cycle>, after: Option<&Cycle>) -> CycleLines {
    let mut out = CycleLines::default();
    let Some(after) = after else { return out };
    let before = before.filter(|b| b.id == after.id);
    if before.is_none() {
        out.process.push(Line::new(
            Stage::Handover,
            LineState::Running,
            format!(
                "Handover cycle {} started for {}: {}.",
                after.id,
                after.account,
                mode_text(after.mode)
            ),
        ));
        if after.leads.is_empty() {
            out.process.push(Line::new(
                Stage::Handover,
                LineState::Skipped,
                "No lead is working, so there is nothing to hand over.",
            ));
        }
    }
    for h in &after.leads {
        let was = before.and_then(|b| b.handover_of(&h.board, &h.thread));
        if was.is_some_and(|w| w.state == h.state) {
            continue;
        }
        let line = lead_line(h);
        if matches!(h.state, HandoverState::Resumed | HandoverState::NotResumed) {
            out.resume.push(line);
        } else {
            out.process.push(line);
        }
    }
    if before.is_none_or(|b| b.phase != after.phase) {
        out.process.extend(phase_line(after.phase));
    }
    out
}

/// How a lead of the cycle stands, in the words of the doc, with a note when it did not go well.
fn thread_state(state: HandoverState) -> (&'static str, Option<&'static str>) {
    match state {
        HandoverState::EstimateAsked | HandoverState::Estimated => ("found", None),
        HandoverState::Asked | HandoverState::Writing => ("asked", None),
        HandoverState::Ready => ("ready", None),
        HandoverState::Resumed => ("ready", Some("resumed on the new account")),
        HandoverState::CutOff => ("skipped", Some("cut off before its handover")),
        HandoverState::NotResumed => ("failed", Some("did not resume on the new account")),
    }
}

/// The title of a lead's row: its project's name and the thread, or the thread alone when the
/// project has no known name.
fn lead_title(h: &Handover, name_of: &dyn Fn(&str) -> Option<String>) -> String {
    match name_of(&h.board) {
        Some(name) => format!("{name} {}", h.thread),
        None => h.thread.clone(),
    }
}

/// The leads the cycle found for the handover, as the `threads` of the doc.
pub(crate) fn threads_of(
    cycle: &Cycle,
    name_of: &dyn Fn(&str) -> Option<String>,
) -> Vec<Value> {
    cycle
        .leads
        .iter()
        .map(|h| {
            let (state, note) = thread_state(h.state);
            let mut out = json!({
                "board": h.board,
                "thread": h.thread,
                "title": lead_title(h, name_of),
                "sessionId": h.session_id,
                "state": state,
            });
            if let Some(note) = note {
                out["note"] = json!(note);
            }
            out
        })
        .collect()
}

/// What the sync step of a switch did, in words: the changes by kind and what it left alone.
pub(crate) fn sync_note(summary: &Summary) -> String {
    let k = &summary.drift.by_kind;
    let wrote = if summary.drift.total == 0 {
        "nothing to change".to_owned()
    } else {
        format!(
            "{} changes written ({} chats, {} folders, {} memberships, {} archive, {} threads)",
            summary.drift.total, k.chat, k.folder, k.membership, k.archive, k.thread
        )
    };
    let check = if summary.verified {
        "checked"
    } else {
        "not checked"
    };
    format!(
        "{wrote}, {check}; {} conflicts and {} chats without a transcript left alone",
        summary.conflicts, summary.missing_transcripts
    )
}

/// The step-log line of the sidebar check after Claude started again.
pub(crate) fn sidebar_check_line(left: &Result<usize, String>, to: &str) -> Line {
    let (state, text) = match left {
        Ok(0) => (
            LineState::Done,
            "Sidebar check: the chats and folders of the previous account are on the new one."
                .to_owned(),
        ),
        Ok(n) => (
            LineState::Failed,
            format!(
                "Sidebar check failed: {n} sidebar changes are missing again after Claude started on {to}. \
                 Claude rewrote its sidebar. Close Claude from its tray and copy the sidebar again from the Accounts tab."
            ),
        ),
        Err(why) => (
            LineState::Failed,
            format!("Sidebar check failed: the sidebars could not be read ({why})."),
        ),
    };
    Line::new(Stage::Verify, state, text)
}
