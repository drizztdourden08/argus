//! `coord proc rules | rule set | rule rm`: the rules of the process monitor, in order (the
//! first that takes a process wins). Changing them is the Overlord's.

use argus_core::js;
use serde_json::{Value, json};
use std::fmt::Write;

use super::dashboard;
use super::list::{dur, mem};
use super::patch::patch_of;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::out;
use crate::cli::route::Call;
use crate::verbs::remote::support::link::list_of;
use crate::verbs::remote::support::overlord;
use crate::verbs::remote::support::text::{cell, say_all, truthy};

const ADVICE: &str = "Ask it with coord lead portfolio \"<what and why>\", or the owner uses the buttons on Argus > Processes.";

fn limits(rule: &Value) -> String {
    let part = |field: &str, text: &dyn Fn(&Value) -> String| {
        truthy(rule, field).then(|| text(&rule[field]))
    };
    [
        part("minMemMB", &|v| format!("listed >{}", mem(v))),
        part("onlyFlagged", &|_| "listed when flagged".to_owned()),
        part("maxMemMB", &|v| format!("mem >{}", mem(v))),
        part("maxGroupMemMB", &|v| format!("group >{}", mem(v))),
        part("maxAgeMin", &|v| format!("age >{}", dur(v))),
        part("idleMin", &|v| format!("idle {}", dur(v))),
        part("orphan", &|_| "orphan".to_owned()),
        part("hung", &|_| "hung".to_owned()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(", ")
}

fn matching(rule: &Value) -> String {
    let pairs: Vec<String> = rule["match"]
        .as_object()
        .map(|m| {
            m.iter()
                .map(|(k, v)| format!("{k}~{}", cell(Some(v))))
                .collect()
        })
        .unwrap_or_default();
    if pairs.is_empty() {
        "(everything)".to_owned()
    } else {
        pairs.join(" ")
    }
}

/// `coord proc rules [--json]`.
pub(super) fn rules(call: &mut Call<'_>) -> Outcome {
    let answer = dashboard(call)?.get("/api/procs/rules")?;
    if call.options.flag("json") {
        out::as_json(&mut call.inv.stdout, &answer);
        return Ok(());
    }
    let mut head = cell(answer.get("file"));
    if js::truthy(&answer["error"]) {
        write!(head, "\n{}", cell(answer.get("error"))).ok();
    }
    let mut rows = vec![
        ["rule", "match", "limits", "kill"]
            .map(str::to_owned)
            .to_vec(),
    ];
    for rule in list_of(&answer, "rules") {
        let kill = if truthy(rule, "killable") {
            "yes"
        } else {
            "no"
        };
        rows.push(vec![
            cell(rule.get("name")),
            matching(rule),
            limits(rule),
            kill.to_owned(),
        ]);
    }
    say_all(call.inv, &[head, out::table(&rows)]);
    Ok(())
}

/// `coord proc rule set <name> ... | rule rm <name>`.
pub(super) fn rule(call: &mut Call<'_>) -> Outcome {
    let sub = call.positional.first().cloned().unwrap_or_default();
    if sub != "set" && sub != "rm" {
        return Err(CoordError::refuse(
            "coord proc rule set <name> ... | coord proc rule rm <name>",
        ));
    }
    let Some(name) = call.positional.get(1).filter(|n| !n.is_empty()).cloned() else {
        return Err(CoordError::refuse(format!(
            "name the rule: coord proc rule {sub} <name>."
        )));
    };
    let by = overlord::actor(call, "change the process rules", ADVICE)?;
    if sub == "rm" {
        dashboard(call)?.post(
            "/api/procs/rules/remove",
            &json!({ "name": name, "by": by }),
        )?;
        call.inv.say(&[&format!("removed rule {name}.")]);
        return Ok(());
    }
    let patch = patch_of(&call.options)?;
    if patch.is_empty() {
        return Err(CoordError::refuse(
            "say what to set: --name-match, --cmd-match, --path-match, --ignore, --max-mem, --max-group-mem, --max-age, --idle, --orphan, --hung, --killable, --note (see coord proc).",
        ));
    }
    let mut fields = serde_json::Map::new();
    fields.insert("name".to_owned(), json!(name));
    fields.extend(patch);
    let body = json!({ "rule": fields, "by": by });
    let answer = dashboard(call)?.post("/api/procs/rules", &body)?;
    let verb = if truthy(&answer, "created") {
        "added"
    } else {
        "changed"
    };
    let line = format!("{verb} rule {name}: {}", answer["rule"]);
    call.inv.say(&[&line]);
    Ok(())
}
