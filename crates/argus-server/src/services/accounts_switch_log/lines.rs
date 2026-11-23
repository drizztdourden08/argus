//! The plain lines of the step log, as pure functions: what changed in the Desktop steps of a
//! switch, and what changed in the handover cycle.

use argus_accounts::desktop::{Step, StepStatus};
use argus_accounts::handover::{Cycle, CycleMode, CyclePhase, Handover, HandoverState};
use argus_accounts::sync::Summary;

/// The lines a change of the handover cycle makes: the ones of the process, and the resume
/// lines that come after the switch ended.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct CycleLines {
    /// Cycle started, leads asked, handovers received, every handover in.
    pub process: Vec<String>,
    /// Leads that resumed, or did not, after the switch.
    pub resume: Vec<String>,
}

fn with_detail(label: &str, word: &str, detail: &str) -> String {
    if detail.is_empty() {
        format!("{label}: {word}.")
    } else {
        format!("{label}: {word} ({detail}).")
    }
}

fn step_line(step: &Step) -> Option<String> {
    let word = match step.status {
        StepStatus::Pending => return None,
        StepStatus::Running => return Some(format!("{}: started.", step.label)),
        StepStatus::Done => "done",
        StepStatus::Failed => "failed",
        StepStatus::Skipped => "skipped",
    };
    Some(with_detail(&step.label, word, &step.detail))
}

/// One line per Desktop step whose status changed between `before` and `after`.
pub(crate) fn step_lines(before: &[Step], after: &[Step]) -> Vec<String> {
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

fn lead_line(h: &Handover) -> String {
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

fn phase_line(phase: CyclePhase) -> Option<&'static str> {
    match phase {
        CyclePhase::Ready => Some("Every handover is in."),
        CyclePhase::Cancelled => Some("The handover cycle was called off."),
        CyclePhase::Estimating | CyclePhase::Asked | CyclePhase::Done => None,
    }
}

/// The lines between two reads of the handover cycle.
pub(crate) fn cycle_lines(before: Option<&Cycle>, after: Option<&Cycle>) -> CycleLines {
    let mut out = CycleLines::default();
    let Some(after) = after else { return out };
    let before = before.filter(|b| b.id == after.id);
    if before.is_none() {
        out.process.push(format!(
            "Handover cycle {} started for {}: {}.",
            after.id,
            after.account,
            mode_text(after.mode)
        ));
        if after.leads.is_empty() {
            out.process
                .push("No lead is working, so there is nothing to hand over.".to_owned());
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
        if let Some(line) = phase_line(after.phase) {
            out.process.push(line.to_owned());
        }
    }
    out
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
pub(crate) fn sidebar_check_line(left: &Result<usize, String>, to: &str) -> String {
    match left {
        Ok(0) => "Sidebar check: the chats and folders are the same on every account.".to_owned(),
        Ok(n) => format!(
            "Sidebar check failed: {n} sidebar changes are missing again after Claude started on {to}. \
             Claude rewrote its sidebar. Close Claude from its tray and run the sync from the Accounts tab."
        ),
        Err(why) => format!("Sidebar check failed: the sidebars could not be read ({why})."),
    }
}
