//! Checks on the stages and statuses of one type.

use std::collections::HashSet;

use super::super::model::{Collection, ItemType, Registry};
use super::vocab::GATE_NEEDS;

/// Every problem in the stages of `t`.
pub(super) fn verify(reg: &Registry, t: &ItemType, out: &mut Vec<String>) {
    let mut say = |text: &str| out.push(format!("types.{}: {text}", t.id));
    match t.collection {
        Collection::Tests => verify_statuses(t, &mut say),
        Collection::Items => {
            verify_ids(t, &mut say);
            verify_lanes(reg, t, &mut say);
            verify_steps(t, &mut say);
        }
    }
}

fn verify_statuses(t: &ItemType, say: &mut impl FnMut(&str)) {
    if t.statuses.is_empty() || !t.stages.is_empty() {
        say("a type kept in tests has statuses and no stages");
    }
}

fn verify_ids(t: &ItemType, say: &mut impl FnMut(&str)) {
    if !t.statuses.is_empty() {
        say("statuses are only for a type kept in tests");
    }
    let ids: HashSet<_> = t.stages.iter().map(|s| s.id.as_str()).collect();
    if ids.len() != t.stages.len() {
        say("a stage id is listed twice");
    }
}

fn verify_lanes(reg: &Registry, t: &ItemType, say: &mut impl FnMut(&str)) {
    for stage in &t.stages {
        if !reg.lanes.contains(&stage.lane) {
            say(&format!(
                "stage {} is in unknown lane {}",
                stage.id, stage.lane
            ));
        }
    }
    for lane in &reg.lanes {
        if !t.stages.iter().any(|s| &s.lane == lane) {
            say(&format!("no stage is in lane {lane}"));
        }
    }
}

fn verify_steps(t: &ItemType, say: &mut impl FnMut(&str)) {
    for stage in &t.stages {
        for next in stage.next.iter().flatten() {
            if !t.stages.iter().any(|s| &s.id == next) {
                say(&format!("stage {} moves to unknown stage {next}", stage.id));
            }
        }
        if let Some(field) = stage.requires.iter().find(|f| !t.fields.contains(f)) {
            say(&format!(
                "stage {} requires {field}, not a field of the type",
                stage.id
            ));
        }
        if let Some(gate) = stage
            .gates
            .iter()
            .find(|g| !GATE_NEEDS.contains(&g.need.as_str()))
        {
            say(&format!(
                "stage {} has unknown gate {}",
                stage.id, gate.need
            ));
        }
    }
}
