//! Checks on the item types: parents, fields, cards and done rules. What every kind shares (names,
//! codes, looks, tabs, links) is checked in [`super::kinds`].

use super::super::model::{Collection, ItemType, Registry};
use super::vocab::{DONE_NEEDS, MAX_DEPTH};

/// Every problem in the types, including the stages of each.
pub(super) fn verify(reg: &Registry, out: &mut Vec<String>) {
    for t in &reg.types {
        verify_type(reg, t, out);
    }
}

fn verify_type(reg: &Registry, t: &ItemType, out: &mut Vec<String>) {
    let mut say = |text: &str| out.push(format!("types.{}: {text}", t.id));
    if let Some(name) = t.fields.iter().find(|f| !reg.fields.contains_key(*f)) {
        say(&format!("unknown field {name}"));
    }
    verify_places(t, &mut say);
    verify_serves(reg, t, &mut say);
    verify_groups(t, &mut say);
    verify_done(t, &mut say);
    verify_parents(reg, t, &mut say);
    super::stages::verify(reg, t, out);
}

fn verify_serves(reg: &Registry, t: &ItemType, say: &mut impl FnMut(&str)) {
    for served in &t.serves {
        match reg.types.iter().find(|s| &s.id == served) {
            None => say(&format!("serves type {served} does not exist")),
            Some(s) if s.collection != Collection::Items => {
                say(&format!("serves type {served} is not a work item type"));
            }
            Some(_) => {}
        }
    }
}

fn verify_places(t: &ItemType, say: &mut impl FnMut(&str)) {
    if let Some(chip) = t.card.chips.iter().find(|c| !t.fields.contains(c)) {
        say(&format!("card chip {chip} is not a field of the type"));
    }
    if t.collection != Collection::Items && !t.parents.is_empty() {
        say("only a work item has a parent type; plans hold a test case through their cases");
    }
    if t.parent_required && t.parents.is_empty() {
        say("a type that requires a parent must name the parent types it may sit under");
    }
    verify_keys(t, say);
}

/// The id key and the served types belong to the types kept outside `items` and `tests`.
fn verify_keys(t: &ItemType, say: &mut impl FnMut(&str)) {
    let own = matches!(t.collection, Collection::Launchers | Collection::Worktrees);
    if own != t.id_key.is_some() {
        say("idKey is set exactly on a build or worktree type");
    }
    if (t.collection == Collection::Launchers) == t.serves.is_empty() {
        say("serves is set exactly on a build type");
    }
    if t.read_only && t.collection == Collection::Items {
        say("a work item type is written from the board, not read only");
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

fn verify_done(t: &ItemType, say: &mut impl FnMut(&str)) {
    let Some(done) = &t.done else { return };
    if t.collection != Collection::Items {
        say("only a work item type has a done rule");
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
