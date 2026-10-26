//! The numbered parts of a test: `coord test step <T> "<step>" | set <n> "<step>" | rm <n>`,
//! `setup <T> "<label>" "<text>" | rm <n>` and `expect <T> [--pass "..."]... [--fail "..."]...`
//! (or `rm [--pass <n>] [--fail <n>]`).

use argus_board::json::{array_mut, display, to_number};
use argus_board::limits::{require_text, text_list};
use argus_core::js;
use serde_json::{Value, json};

use super::edit::edit;
use crate::cli::Outcome;
use crate::verbs::work::listing::cell;
use crate::verbs::work::run::{Res, Run, bad};

const MAX_STEPS: usize = 15;
const MAX_EXPECT: usize = 8;

/// The zero-based position of a 1-based number in a list, or a clear refusal.
fn at(len: usize, spec: &Value, what: &str) -> Res<usize> {
    let n = to_number(Some(spec));
    let top = f64::from(u32::try_from(len).unwrap_or(u32::MAX));
    let whole = (n.fract() == 0.0 && (1.0..=top).contains(&n))
        .then(|| js::number(n).as_u64())
        .flatten();
    whole
        .and_then(|n| usize::try_from(n - 1).ok())
        .ok_or_else(|| bad(format!("no {what} {} (there are {len}).", cell(spec))))
}

fn listed<'a>(test: &'a mut Value, key: &str) -> &'a mut Vec<Value> {
    array_mut(test, key)
}

fn remove_step(test: &mut Value, spec: &Value) -> Res<String> {
    let steps = listed(test, "steps");
    let gone = steps.remove(at(steps.len(), spec, "step")?);
    Ok(format!(
        "{}: step removed ({}); {} left, renumbered.",
        display(&test["id"]),
        display(&gone),
        listed(test, "steps").len()
    ))
}

fn reword_step(test: &mut Value, spec: &Value, text: &Value) -> Res<()> {
    let steps = listed(test, "steps");
    let at = at(steps.len(), spec, "step")?;
    steps[at] = json!(require_text("step", text)?);
    Ok(())
}

/// `coord test step`.
pub(super) fn step(r: &mut Run<'_>) -> Outcome {
    edit(r, |r, test, _| {
        let id = display(&test["id"]);
        let (second, count) = (r.arg(1), r.args.len());
        let line = if second == Some("rm") && count == 3 {
            remove_step(test, &r.arg_json(2))?
        } else if second == Some("set") && count == 4 {
            reword_step(test, &r.arg_json(2), &r.arg_json(3))?;
            format!("{id} step {}: {}", r.arg(2).unwrap_or_default(), r.arg(3).unwrap_or_default())
        } else {
            if listed(test, "steps").len() >= MAX_STEPS {
                return Err(bad(format!("{id} already has {MAX_STEPS} steps.")));
            }
            let text = require_text("step", &r.arg_json(1))?;
            let steps = listed(test, "steps");
            steps.push(json!(text));
            format!("{id} step {}: {}", steps.len(), r.arg(1).unwrap_or_default())
        };
        r.say(&[&line]);
        Ok(())
    })
}

/// `coord test setup`.
pub(super) fn setup(r: &mut Run<'_>) -> Outcome {
    edit(r, |r, test, _| {
        let id = display(&test["id"]);
        let line = if r.arg(1) == Some("rm") && r.args.len() == 3 {
            let lines = listed(test, "setup");
            let gone = lines.remove(at(lines.len(), &r.arg_json(2), "setup line")?);
            format!("{id}: setup \"{}\" removed.", display(&gone["label"]))
        } else {
            let label = require_text("label", &r.arg_json(1))?;
            let text = require_text("setup", &r.arg_json(2))?;
            listed(test, "setup").push(json!({"label": label, "text": text}));
            format!("{id} setup: {}", r.arg(1).unwrap_or_default())
        };
        r.say(&[&line]);
        Ok(())
    })
}

fn remove_criteria(r: &Run<'_>, test: &mut Value) -> Res<()> {
    let (pass, fail) = (r.options.json("pass"), r.options.json("fail"));
    if pass.is_none() && fail.is_none() {
        return Err(bad(
            "say which to remove: coord test expect <T> rm --pass <n> | --fail <n>.",
        ));
    }
    for (key, given, what) in [("pass", pass, "pass criterion"), ("fail", fail, "fail criterion")] {
        if let Some(given) = given {
            let list = listed(test, key);
            list.remove(at(list.len(), &given, what)?);
        }
    }
    Ok(())
}

fn add_criteria(r: &Run<'_>, test: &mut Value) -> Res<()> {
    for key in ["pass", "fail"] {
        let given = Value::Array(r.options.many(key).iter().map(crate::cli::Val::to_json).collect());
        let texts = text_list("expect", &given, MAX_EXPECT)?;
        listed(test, key).extend(texts.into_iter().map(Value::from));
    }
    Ok(())
}

/// `coord test expect`.
pub(super) fn expect(r: &mut Run<'_>) -> Outcome {
    edit(r, |r, test, _| {
        if r.arg(1) == Some("rm") {
            remove_criteria(r, test)?;
        } else {
            add_criteria(r, test)?;
        }
        let (pass, fail) = (listed(test, "pass").len(), listed(test, "fail").len());
        r.say(&[&format!("{}: {pass} pass / {fail} fail criteria", display(&test["id"]))]);
        Ok(())
    })
}
