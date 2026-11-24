//! Rule 4: a lead with no turn, no agent and no process for the limit, while unblocked work is open.

use super::lead::{has_running_agent, idle_start, live_lead, open_items};
use super::{Found, Kind, RuleInput, projects, text};
use crate::board::{BoardView, ThreadView};

/// The idle leads with open work at `now`. A project whose lead is gone, paused or busy loses its idle clock.
pub(super) fn find(now: i64, input: &RuleInput<'_>) -> Vec<Found> {
    projects(input)
        .into_iter()
        .filter_map(|(project, board)| one(now, input, &project, &board))
        .collect()
}

fn one(now: i64, input: &RuleInput<'_>, project: &str, board: &BoardView) -> Option<Found> {
    let idle_lead =
        live_lead(board).filter(|(id, lead)| is_idle(now, input, project, board, id, lead));
    let Some((lead_id, lead)) = idle_lead else {
        input.idle.forget(project);
        return None;
    };
    let since = input.idle.seen_idle(project, idle_start(now, lead));
    let age = now - since;
    if age < input.limits.idle_ms {
        return None;
    }
    let open = open_items(board, &lead_id);
    (!open.is_empty()).then(|| Found {
        kind: Kind::Idle,
        project: project.to_owned(),
        subject: lead_id.clone(),
        lead: lead_id,
        since,
        text: text::idle(project, age, &open),
    })
}

/// No turn running, no agent of the lead running and no process of the project at work.
fn is_idle(
    now: i64,
    input: &RuleInput<'_>,
    project: &str,
    board: &BoardView,
    lead_id: &str,
    lead: &ThreadView,
) -> bool {
    !lead.in_turn(now) && !has_running_agent(board, lead_id) && !(input.busy)(project)
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use crate::memory::MemoryBoards;
    use crate::stall_rules::fixtures::{NOW, boards, item, project, run_of};
    use crate::stall_rules::{Found, IdleWatch, Kind};
    use crate::testkit::MIN;
    use crate::time::iso;

    fn rotp(lead: &Value, items: &[Value], extra: &Value) -> MemoryBoards {
        boards(&[("rotp", project(lead, items, extra))])
    }

    fn idle_found(boards: &MemoryBoards, watch: &IdleWatch, now: i64, busy: bool) -> Vec<Found> {
        run_of(Kind::Idle, boards, watch, (now, busy))
    }

    fn todo(id: &str) -> Value {
        item(id, "todo", &json!({}))
    }

    #[test]
    fn an_idle_lead_with_an_unblocked_todo_item_stalls_with_the_spec_words() {
        let world = rotp(
            &json!({}),
            &[todo("I-4"), item("I-7", "doing", &json!({}))],
            &json!({}),
        );
        let found = idle_found(&world, &IdleWatch::new(), NOW, false);
        assert_eq!(
            found,
            [Found {
                kind: Kind::Idle,
                project: "rotp".to_owned(),
                lead: "lead".to_owned(),
                subject: "lead".to_owned(),
                since: NOW - 20 * MIN,
                text: "stall: rotp idle 20 min, 2 open items without a blocker (K-4, K-7)"
                    .to_owned(),
            }]
        );
    }

    #[test]
    fn the_text_lists_the_items_with_the_letters_of_their_type() {
        let bug = item("I-4", "todo", &json!({"type": "bug"}));
        let world = rotp(&json!({}), &[bug, todo("I-7")], &json!({}));
        let found = idle_found(&world, &IdleWatch::new(), NOW, false);
        assert_eq!(
            found[0].text,
            "stall: rotp idle 20 min, 2 open items without a blocker (B-4, K-7)"
        );
    }

    #[test]
    fn a_blocker_a_dep_wait_or_an_open_ask_keeps_it_quiet() {
        let ask =
            json!({"entries": {"E-1": {"item": "I-4", "kind": "question", "status": "open"}}});
        let cases = [
            (json!({"blocker": "needs the schema"}), json!({})),
            (json!({"waitsOn": {"state": "open"}}), json!({})),
            (json!({}), ask),
        ];
        for (extra, board_extra) in cases {
            let world = rotp(&json!({}), &[item("I-4", "todo", &extra)], &board_extra);
            assert!(
                idle_found(&world, &IdleWatch::new(), NOW, false).is_empty(),
                "{extra} {board_extra}"
            );
        }
    }

    #[test]
    fn a_turn_an_agent_or_a_busy_process_is_not_idle() {
        let agent = json!({"agents": {"A-1": {"thread": "lead", "status": "working"}}});
        let cases = [
            (
                json!({"activity": "working", "lastSeen": iso(NOW - MIN)}),
                json!({}),
                false,
            ),
            (json!({}), agent, false),
            (json!({}), json!({}), true),
        ];
        for (lead, extra, busy) in cases {
            let world = rotp(&lead, &[todo("I-4")], &extra);
            assert!(
                idle_found(&world, &IdleWatch::new(), NOW, busy).is_empty(),
                "{lead} {extra} {busy}"
            );
        }
    }

    #[test]
    fn a_lead_without_a_live_session_or_paused_is_skipped() {
        for lead in [
            json!({"status": "done"}),
            json!({"status": "paused"}),
            json!({"placeholder": true}),
        ] {
            let world = rotp(&lead, &[todo("I-4")], &json!({}));
            assert!(
                idle_found(&world, &IdleWatch::new(), NOW, false).is_empty(),
                "{lead}"
            );
        }
    }

    #[test]
    fn the_work_of_another_live_thread_does_not_count() {
        let worker = json!({"threads": {"lead": {"id": "lead", "role": "lead", "status": "active", "activity": "idle", "lastSeen": iso(NOW - 20 * MIN)}, "ui": {"id": "ui", "role": "thread", "status": "active"}}});
        let owned = item("I-4", "todo", &json!({"thread": "ui"}));
        let world = rotp(&json!({}), &[owned], &worker);
        assert_eq!(idle_found(&world, &IdleWatch::new(), NOW, false).len(), 0);
    }

    #[test]
    fn the_idle_clock_starts_at_the_first_sight_and_a_busy_sight_resets_it() {
        let watch = IdleWatch::new();
        let fresh = json!({"lastSeen": iso(NOW - 5 * MIN)});
        let world = rotp(&fresh, &[todo("I-4")], &json!({}));
        assert!(
            idle_found(&world, &watch, NOW, false).is_empty(),
            "idle 5 min is below the limit"
        );
        let later = idle_found(&world, &watch, NOW + 6 * MIN, false);
        assert_eq!(later.len(), 1);
        assert_eq!(later[0].since, NOW - 5 * MIN);
        let busy = json!({"activity": "working", "lastSeen": iso(NOW + 7 * MIN)});
        let working = rotp(&busy, &[todo("I-4")], &json!({}));
        assert_eq!(idle_found(&working, &watch, NOW + 7 * MIN, false).len(), 0);
        let quiet = json!({"lastSeen": iso(NOW + 7 * MIN)});
        let again = rotp(&quiet, &[todo("I-4")], &json!({}));
        let restarted = idle_found(&again, &watch, NOW + 8 * MIN, false);
        assert!(
            restarted.is_empty(),
            "the clock started again at the last turn"
        );
    }

    #[test]
    fn a_lead_with_no_open_item_left_is_quiet() {
        let world = rotp(&json!({}), &[item("I-4", "done", &json!({}))], &json!({}));
        assert_eq!(idle_found(&world, &IdleWatch::new(), NOW, false).len(), 0);
    }
}
