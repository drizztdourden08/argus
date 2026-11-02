//! The words of the dependency wakes.

use super::view::{DepStep, DepView};
use crate::text::one_line;

fn id(dep: &DepView) -> &str {
    dep.id.as_str()
}

/// `dep D-4: alpha/I-7 needs your I-12 "Expose the save hook"`
pub(super) fn request(dep: &DepView) -> String {
    format!(
        "dep {}: {}/{} needs your {} \"{}\"",
        id(dep),
        dep.from.project.as_str(),
        dep.from.item.as_str(),
        dep.to.item.as_str(),
        one_line(dep.title.as_str())
    )
}

/// `beta/I-12 is done (D-4): resume I-7`
pub(super) fn resolved(dep: &DepView, step: &DepStep) -> String {
    format!(
        "{}/{} is {} ({}): resume {}",
        dep.to.project.as_str(),
        dep.to.item.as_str(),
        step.to.as_str(),
        id(dep),
        dep.from.item.as_str()
    )
}

/// `beta/I-12 is blocked on you (D-4): needs the schema first`
pub(super) fn blocked(dep: &DepView, step: &DepStep) -> String {
    let reason = step.reason.some().or_else(|| dep.reason.some());
    let head = format!(
        "{}/{} is blocked on you ({})",
        dep.to.project.as_str(),
        dep.to.item.as_str(),
        id(dep)
    );
    match reason {
        Some(reason) => format!("{head}: {}", one_line(reason)),
        None => head,
    }
}

/// `alpha/I-7 cancelled its request I-12 (D-4)`
pub(super) fn cancelled(dep: &DepView) -> String {
    format!(
        "{}/{} cancelled its request {} ({})",
        dep.from.project.as_str(),
        dep.from.item.as_str(),
        dep.to.item.as_str(),
        id(dep)
    )
}

/// `beta/I-12 is open again (D-4)`
pub(super) fn reopened(dep: &DepView) -> String {
    format!(
        "{}/{} is open again ({})",
        dep.to.project.as_str(),
        dep.to.item.as_str(),
        id(dep)
    )
}
