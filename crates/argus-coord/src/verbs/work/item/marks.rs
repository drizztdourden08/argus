//! The two letters that stand for an assignee in `item tree`: the owner and the leads from the
//! registry, a thread from its name.

use argus_board::registry::registry;

/// The first letters of the words of a name, two at most; one word gives its first two letters.
fn initials(name: &str) -> String {
    let words: Vec<&str> = name
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    let letters: Vec<char> = match words.as_slice() {
        [] => Vec::new(),
        [one] => one.chars().take(2).collect(),
        [first, second, ..] => first
            .chars()
            .take(1)
            .chain(second.chars().take(1))
            .collect(),
    };
    letters.into_iter().flat_map(char::to_uppercase).collect()
}

/// The mark of an assignee: `ME` for the owner, the registry initials of a lead, else the
/// initials of the thread name. `--` when nobody holds the item.
pub(super) fn mark_of(assignee: &str) -> String {
    let people = &registry().people;
    let found = if assignee == "owner" {
        Some(people.owner.initials.clone())
    } else {
        assignee
            .strip_prefix("lead:")
            .and_then(|key| people.leads.get(key))
            .map(|lead| lead.initials.clone())
    };
    match found {
        Some(marks) => marks,
        None if assignee.is_empty() => "--".to_owned(),
        None => initials(assignee.strip_prefix("lead:").unwrap_or(assignee)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // test-tag: critical
    #[test]
    fn the_owner_and_the_leads_use_the_registry_and_a_thread_its_name() {
        assert_eq!(mark_of("owner"), "ME");
        assert_eq!(mark_of("lead:brock"), "BR");
        assert_eq!(mark_of("lead:nobody"), "NO");
        assert_eq!(mark_of("ui-work"), "UW");
        assert_eq!(mark_of("worker"), "WO");
        assert_eq!(mark_of(""), "--");
    }
}
