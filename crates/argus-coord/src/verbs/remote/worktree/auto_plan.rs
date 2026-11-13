//! `coord worktree auto-plan`: what Argus would remove under each automatic repo rule, and why. Read only.

use serde_json::Value;

use crate::cli::exit::Outcome;
use crate::cli::out;
use crate::cli::route::Call;
use crate::verbs::remote::support::link::{connect, list_of};
use crate::verbs::remote::support::text::{cell, say_all, truthy};

fn row(plan: &Value) -> Vec<String> {
    let remove = if truthy(plan, "wouldRemove") {
        "yes"
    } else {
        "no"
    };
    vec![
        cell(plan.get("board")),
        cell(plan.get("path")),
        cell(plan.get("class")),
        remove.to_owned(),
        cell(plan.get("reason")),
    ]
}

/// `coord worktree auto-plan [--json]`.
pub(super) fn auto_plan(call: &mut Call<'_>) -> Outcome {
    let client = connect(call, false)?;
    let answer = client.get("/api/worktrees/auto-plan")?;
    if call.options.flag("json") {
        out::as_json(&mut call.inv.stdout, &answer);
        return Ok(());
    }
    let mode = cell(answer.get("mode"));
    let plan = list_of(&answer, "plan");
    if plan.is_empty() {
        call.inv.say(&[&format!(
            "no worktrees under an automatic repo rule (autoRules is {mode})."
        )]);
        return Ok(());
    }
    let mut table = vec![
        ["board", "path", "class", "remove", "reason"]
            .map(str::to_owned)
            .to_vec(),
    ];
    table.extend(plan.iter().map(row));
    let removing = plan.iter().filter(|p| truthy(p, "wouldRemove")).count();
    let head = format!(
        "{} worktrees under automatic rules, {removing} would be removed (autoRules is {mode}; the hygiene acts on them only when it is on)",
        plan.len()
    );
    say_all(call.inv, &[head, out::table(&table)]);
    Ok(())
}
