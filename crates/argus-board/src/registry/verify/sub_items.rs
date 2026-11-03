//! Checks on the sub-items: questions, decisions, to-dos and notes.

use std::collections::HashSet;

use super::super::model::Registry;
use super::vocab::{ICONS, TONES};

/// Every problem in the sub-items.
pub(super) fn verify(reg: &Registry, out: &mut Vec<String>) {
    let mut ids = HashSet::new();
    let mut prefixes = HashSet::new();
    for s in &reg.sub_items {
        let mut say = |text: &str| out.push(format!("subItems.{}: {text}", s.id));
        if !ids.insert(s.id.as_str()) {
            say("listed twice");
        }
        if !prefixes.insert(s.prefix.as_str()) {
            say("its prefix is used by another sub-item");
        }
        if s.prefix.len() != 1 || !s.prefix.chars().all(|c| c.is_ascii_uppercase()) {
            say("the prefix is one capital letter");
        }
        if !ICONS.contains(&s.icon.as_str()) {
            say(&format!("unknown icon {}", s.icon));
        }
        if !TONES.contains(&s.tone.as_str()) {
            say(&format!("unknown tone {}", s.tone));
        }
        if !reg.fields.contains_key(&s.field) {
            say(&format!("unknown field {}", s.field));
        }
    }
}
