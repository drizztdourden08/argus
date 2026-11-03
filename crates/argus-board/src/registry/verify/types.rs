//! Checks on the item types: names, parents, fields, tabs, cards and done rules.

use std::collections::HashSet;

use super::super::model::{Collection, ItemType, Registry};
use super::vocab::{CATEGORY_COLORS, DONE_NEEDS, ICONS, MAX_DEPTH, TABS};

/// Every problem in the types, including the stages of each.
pub(super) fn verify(reg: &Registry, out: &mut Vec<String>) {
    let mut seen = HashSet::new();
    for t in &reg.types {
        if !seen.insert(t.id.as_str()) {
            out.push(format!("types: {} is listed twice", t.id));
        }
        verify_type(reg, t, out);
    }
}

fn verify_type(reg: &Registry, t: &ItemType, out: &mut Vec<String>) {
    let mut say = |text: &str| out.push(format!("types.{}: {text}", t.id));
    if !ICONS.contains(&t.icon.as_str()) {
        say(&format!("unknown icon {}", t.icon));
    }
    if !CATEGORY_COLORS.contains(&t.color.as_str()) {
        say(&format!("unknown colour {}", t.color));
    }
    if let Some(name) = t.fields.iter().find(|f| !reg.fields.contains_key(*f)) {
        say(&format!("unknown field {name}"));
    }
    verify_places(t, &mut say);
    verify_groups(t, &mut say);
    verify_tabs(t, &mut say);
    verify_done(t, &mut say);
    verify_parents(reg, t, &mut say);
    super::stages::verify(reg, t, out);
}

fn verify_places(t: &ItemType, say: &mut impl FnMut(&str)) {
    if let Some(chip) = t.card.chips.iter().find(|c| !t.fields.contains(c)) {
        say(&format!("card chip {chip} is not a field of the type"));
    }
    if t.collection == Collection::Tests && !t.parents.is_empty() {
        say("a test case has no parent type; plans hold it through their cases");
    }
}

fn verify_groups(t: &ItemType, say: &mut impl FnMut(&str)) {
    for group in &t.groups {
        if let Some(name) = group.fields.iter().find(|f| !t.fields.contains(f)) {
            say(&format!(
                "group {} names {name}, not a field of the type",
                group.id
            ));
        }
    }
}

fn verify_tabs(t: &ItemType, say: &mut impl FnMut(&str)) {
    if t.tabs.first().map(String::as_str) != Some("overview") {
        say("the first tab must be overview");
    }
    if let Some(tab) = t.tabs.iter().find(|tab| !TABS.contains(&tab.as_str())) {
        say(&format!("unknown tab {tab}"));
    }
    let unique: HashSet<_> = t.tabs.iter().collect();
    if unique.len() != t.tabs.len() {
        say("a tab is listed twice");
    }
}

fn verify_done(t: &ItemType, say: &mut impl FnMut(&str)) {
    let Some(done) = &t.done else { return };
    if t.collection == Collection::Tests {
        say("a type with statuses has no done rule");
    }
    if let Some(need) = done
        .needs
        .iter()
        .find(|n| !DONE_NEEDS.contains(&n.as_str()))
    {
        say(&format!("unknown done condition {need}"));
    }
}

fn verify_parents(reg: &Registry, t: &ItemType, say: &mut impl FnMut(&str)) {
    for parent in &t.parents {
        match reg.types.iter().find(|p| &p.id == parent) {
            None => say(&format!("parent type {parent} does not exist")),
            Some(p) if p.collection != Collection::Items => {
                say(&format!("parent type {parent} is not a work item type"));
            }
            Some(_) => {}
        }
    }
    if levels(reg, &t.id, 1) > MAX_DEPTH {
        say(&format!(
            "parents nest deeper than {MAX_DEPTH} levels, or loop"
        ));
    }
}

/// The number of levels in the longest chain of parents from `id` up, itself included. It stops
/// descending once past the allowed depth, so a loop ends with a number above it.
fn levels(reg: &Registry, id: &str, guard: usize) -> usize {
    if guard > MAX_DEPTH {
        return guard;
    }
    let above = reg.types.iter().find(|t| t.id == id).map_or(0, |t| {
        t.parents
            .iter()
            .map(|p| levels(reg, p, guard + 1))
            .max()
            .unwrap_or(0)
    });
    above + 1
}
