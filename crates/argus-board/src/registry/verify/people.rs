//! Checks on the owner and the leads.

use super::super::model::Registry;
use super::vocab::{CATEGORY_COLORS, PERSON_COLORS};

/// Every problem in the people: two-letter initials and a known colour each.
pub(super) fn verify(reg: &Registry, out: &mut Vec<String>) {
    let owner = &reg.people.owner;
    let leads = reg
        .people
        .leads
        .iter()
        .map(|(k, l)| (k.as_str(), &l.initials, &l.color));
    let all = std::iter::once(("owner", &owner.initials, &owner.color)).chain(leads);
    for (who, initials, color) in all {
        if initials.chars().count() != 2 {
            out.push(format!("people.{who}: initials are two letters"));
        }
        let known =
            CATEGORY_COLORS.contains(&color.as_str()) || PERSON_COLORS.contains(&color.as_str());
        if !known {
            out.push(format!("people.{who}: unknown colour {color}"));
        }
    }
}
