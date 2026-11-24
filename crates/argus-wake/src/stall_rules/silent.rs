//! Rule 5: an item `doing` for the limit with no board write, no agent on it and no blocker.

use super::{Found, Kind, RuleInput, projects, text};
use crate::blockers::is_blocked;
use crate::board::{BoardView, ItemView};
use crate::item_wakes::{has_live_agent, last_change};

/// The silent `doing` items of every project at `now`.
pub(super) fn find(now: i64, input: &RuleInput<'_>) -> Vec<Found> {
    projects(input)
        .into_iter()
        .flat_map(|(project, board)| in_project(now, input, &project, &board))
        .collect()
}

fn in_project(now: i64, input: &RuleInput<'_>, project: &str, board: &BoardView) -> Vec<Found> {
    board
        .items
        .values()
        .filter_map(|item| one(now, input, project, board, item))
        .collect()
}

fn one(
    now: i64,
    input: &RuleInput<'_>,
    project: &str,
    board: &BoardView,
    item: &ItemView,
) -> Option<Found> {
    if !(item.rec.is_live() && item.status.is("doing")) {
        return None;
    }
    let id = item.rec.id.some()?;
    let last = last_change(board, item);
    let age = now - last;
    if last == 0
        || age < input.limits.silent_ms
        || has_live_agent(board, id)
        || is_blocked(board, item)
    {
        return None;
    }
    Some(Found {
        kind: Kind::Silent,
        project: project.to_owned(),
        lead: board.lead().unwrap_or_default(),
        subject: id.to_owned(),
        since: last,
        text: text::silent(project, &item.shown_id(), age),
    })
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use crate::stall_rules::fixtures::{NOW, boards, fresh, item, project};
    use crate::stall_rules::{Found, Kind};
    use crate::testkit::{HOUR, MIN};
    use crate::time::iso;

    fn silent_found(items: &[Value], extra: &Value) -> Vec<Found> {
        let lead = json!({"activity": "working", "lastSeen": iso(NOW)});
        fresh(
            Kind::Silent,
            &boards(&[("rotp", project(&lead, items, extra))]),
        )
    }

    fn doing(extra: &Value) -> Value {
        item("I-12", "doing", extra)
    }

    #[test]
    fn a_doing_item_60_min_without_a_write_or_agent_stalls() {
        let written = json!({"updatedAt": iso(NOW - HOUR)});
        let found = silent_found(&[doing(&written)], &json!({}));
        assert_eq!(
            found,
            [Found {
                kind: Kind::Silent,
                project: "rotp".to_owned(),
                lead: "lead".to_owned(),
                subject: "I-12".to_owned(),
                since: NOW - HOUR,
                text: "stall: rotp/K-12 doing 60 min, no agent".to_owned(),
            }]
        );
    }

    #[test]
    fn the_text_names_the_item_with_the_letters_of_its_type() {
        let written = json!({"updatedAt": iso(NOW - HOUR), "type": "bug"});
        let found = silent_found(&[doing(&written)], &json!({}));
        assert_eq!(found[0].text, "stall: rotp/B-12 doing 60 min, no agent");
        assert_eq!(found[0].subject, "I-12");
    }

    #[test]
    fn a_recent_write_to_the_item_or_to_a_record_about_it_keeps_it_quiet() {
        let recent = json!({"updatedAt": iso(NOW - 59 * MIN)});
        assert_eq!(silent_found(&[doing(&recent)], &json!({})).len(), 0);
        let result = json!({"entries": {"E-1": {"item": "I-12", "kind": "result", "createdAt": iso(NOW - 10 * MIN)}}});
        assert_eq!(silent_found(&[doing(&json!({}))], &result).len(), 0);
    }

    #[test]
    fn an_agent_on_it_or_a_blocker_keeps_it_quiet() {
        let agent = json!({"agents": {"A-1": {"item": "I-12", "thread": "lead", "status": "working", "createdAt": iso(NOW - 3 * HOUR)}}});
        assert_eq!(silent_found(&[doing(&json!({}))], &agent).len(), 0);
        let blocked = doing(&json!({"blocker": "waits for the owner's key"}));
        assert_eq!(silent_found(&[blocked], &json!({})).len(), 0);
    }

    #[test]
    fn only_a_live_doing_item_counts() {
        let todo = item("I-1", "todo", &json!({}));
        let review = item("I-2", "review", &json!({}));
        let deleted = item("I-3", "doing", &json!({"deletedAt": iso(NOW - HOUR)}));
        assert_eq!(silent_found(&[todo, review, deleted], &json!({})).len(), 0);
    }

    #[test]
    fn an_ended_agent_does_not_count() {
        let agent = json!({"agents": {"A-1": {"item": "I-12", "thread": "lead", "status": "done", "createdAt": iso(NOW - 3 * HOUR)}}});
        assert_eq!(silent_found(&[doing(&json!({}))], &agent).len(), 1);
    }
}
