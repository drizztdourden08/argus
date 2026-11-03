//! Rule 3: a dependency that is `open` for the limit while the item it asks for is still `todo`.

use super::{Found, Kind, RuleInput, text};
use crate::dep_wakes::DepView;
use crate::reasons::PORTFOLIO;
use crate::time::parse_ms;

/// The unstarted dependencies of the ledger at `now`.
pub(super) fn find(now: i64, input: &RuleInput<'_>) -> Vec<Found> {
    let Some(ledger) = input.boards.board(PORTFOLIO) else {
        return Vec::new();
    };
    ledger
        .deps
        .iter()
        .filter_map(|(key, dep)| one(now, input, key, dep))
        .collect()
}

fn one(now: i64, input: &RuleInput<'_>, key: &str, dep: &DepView) -> Option<Found> {
    if !dep.state.is("open") {
        return None;
    }
    let opened = parse_ms(dep.opened_at.some()?)?;
    let age = now - opened;
    if age < input.limits.unstarted_ms {
        return None;
    }
    let project = dep.to.project.some()?;
    let board = input.boards.board(project)?;
    let item = board.items.get(dep.to.item.some()?)?;
    if !(item.rec.is_live() && item.status.is("todo")) {
        return None;
    }
    let id = dep.id.some().unwrap_or(key);
    Some(Found {
        kind: Kind::Unstarted,
        project: project.to_owned(),
        lead: board.coordinator().unwrap_or_default(),
        subject: id.to_owned(),
        since: opened,
        text: text::unstarted(id, age, project, item.rec.id.as_str()),
    })
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use crate::memory::MemoryBoards;
    use crate::stall_rules::fixtures::{NOW, boards, fresh, item, project};
    use crate::stall_rules::{Found, Kind};
    use crate::testkit::MIN;
    use crate::time::iso;

    fn world(state: &str, opened_ago: i64, status: &str) -> MemoryBoards {
        let ledger = json!({"deps": {"D-11": {
            "id": "D-11", "state": state, "openedAt": iso(NOW - opened_ago),
            "from": {"project": "archipelia", "item": "I-124"},
            "to": {"project": "rotp", "item": "I-112", "thread": "lead"},
        }}});
        boards(&[("_portfolio", ledger), ("rotp", rotp(status))])
    }

    fn rotp(status: &str) -> Value {
        let lead = json!({"activity": "working", "lastSeen": iso(NOW)});
        project(&lead, &[item("I-112", status, &json!({}))], &json!({}))
    }

    fn found(boards: &MemoryBoards) -> Vec<Found> {
        fresh(Kind::Unstarted, boards)
    }

    #[test]
    fn an_open_dep_30_min_old_with_its_item_todo_is_a_stall() {
        let found = found(&world("open", 31 * MIN, "todo"));
        assert_eq!(
            found,
            [Found {
                kind: Kind::Unstarted,
                project: "rotp".to_owned(),
                lead: "lead".to_owned(),
                subject: "D-11".to_owned(),
                since: NOW - 31 * MIN,
                text: "stall: D-11 open 31 min, rotp I-112 still todo".to_owned(),
            }]
        );
    }

    #[test]
    fn a_young_dep_a_started_item_or_a_closed_dep_is_not() {
        let quiet = [
            ("open", 29 * MIN, "todo"),
            ("open", 40 * MIN, "doing"),
            ("done", 40 * MIN, "todo"),
            ("blocked", 40 * MIN, "todo"),
        ];
        for (state, age, status) in quiet {
            let found = found(&world(state, age, status));
            assert_eq!(found.len(), 0, "{state} {age} {status}");
        }
    }

    #[test]
    fn a_dep_without_an_opened_time_or_without_its_item_is_skipped() {
        let to = json!({"project": "rotp", "item": "I-112"});
        let no_time = json!({"deps": {"D-1": {"id": "D-1", "state": "open", "to": to}}});
        let other = json!({"project": "rotp", "item": "I-9"});
        let opened = iso(NOW - 99 * MIN);
        let no_item = json!({"deps": {"D-1": {"id": "D-1", "state": "open", "openedAt": opened, "to": other}}});
        for ledger in [no_time, no_item] {
            let world = boards(&[("_portfolio", ledger), ("rotp", rotp("todo"))]);
            assert_eq!(found(&world).len(), 0);
        }
    }
}
