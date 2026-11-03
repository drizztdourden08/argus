//! `coord item ac <I> add "<t>" | set <n> "<t>" | done <n> [--undo] | rm <n>[-<m>] | move <n> <pos>`.
//! Acceptance criteria are numbered by position and tests point at them by number; the shared op
//! renumbers the rest on rm and move, and each test's covered numbers follow.

use argus_board::json::{display, doc};
use argus_board::limits::require_text;
use argus_core::js;
use serde_json::{Map, Value, json};

use crate::cli::Outcome;
use crate::verbs::work::crud::{apply, checked, op};
use crate::verbs::work::run::{Res, Run, bad};

/// What one `ac` verb asks for, its numbers checked.
enum Edit {
    Add(String),
    Set(f64, String),
    Done(f64, bool),
    Rm(f64, f64),
    Move(f64, f64),
}

/// The digits of an AC reference after an optional `AC`, in any case.
fn digits(spec: &str) -> Option<f64> {
    let body = match spec.get(..2) {
        Some(head) if head.eq_ignore_ascii_case("ac") => &spec[2..],
        _ => spec,
    };
    if body.is_empty() || !body.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    body.parse().ok()
}

/// An AC reference: a number, optionally spelled `AC3`.
fn ac_number(spec: Option<&str>) -> Res<f64> {
    let text = spec.unwrap_or_default();
    digits(text).ok_or_else(|| bad(format!("pass the AC as a number, e.g. 3 (got \"{text}\").")))
}

fn ac_range(spec: Option<&str>) -> Res<(f64, f64)> {
    let text = spec.unwrap_or_default();
    let (first, last) = match text.split_once('-') {
        Some((a, b)) => (a, Some(b)),
        None => (text, None),
    };
    match (digits(first), last.map(digits)) {
        (Some(n), None) => Ok((n, n)),
        (Some(n), Some(Some(m))) => Ok((n, m)),
        _ => Err(bad(
            "pass the AC to remove as a number or a range, e.g. 4 or 10-17.",
        )),
    }
}

fn line(spec: Option<&String>) -> Res<String> {
    require_text("ac", &spec.map_or(Value::Null, |s| json!(s)))
}

impl Edit {
    fn parse(sub: &str, args: &[String], undo: bool) -> Res<Edit> {
        let at = |i: usize| args.get(i).map(String::as_str);
        Ok(match sub {
            "set" => Edit::Set(ac_number(at(0))?, line(args.get(1))?),
            "done" => Edit::Done(ac_number(at(0))?, undo),
            "rm" => {
                let (n, m) = ac_range(at(0))?;
                Edit::Rm(n, m)
            }
            "move" => Edit::Move(ac_number(at(0))?, ac_number(at(1))?),
            _ => Edit::Add(line(args.first())?),
        })
    }

    fn fields(&self) -> Map<String, Value> {
        let num = js::number;
        let fields = match self {
            Edit::Add(text) => json!({"do": "add", "text": text}),
            Edit::Set(n, text) => json!({"do": "set", "n": num(*n), "text": text}),
            Edit::Done(n, undo) => {
                json!({"do": if *undo { "undo" } else { "done" }, "n": num(*n)})
            }
            Edit::Rm(n, m) => json!({"do": "rm", "n": num(*n), "m": num(*m)}),
            Edit::Move(n, to) => json!({"do": "move", "n": num(*n), "to": num(*to)}),
        };
        fields.as_object().cloned().unwrap_or_default()
    }

    fn said(&self, id: &str, left: usize) -> String {
        match self {
            Edit::Add(text) => format!("{id} AC{left}: {text}"),
            Edit::Set(n, text) => format!("{id} AC{n}: {text}"),
            Edit::Done(n, true) => format!("{id} AC{n}: reopened"),
            Edit::Done(n, false) => format!("{id} AC{n}: done"),
            Edit::Rm(n, m) => {
                let span = if (n - m).abs() < f64::EPSILON {
                    format!("AC{n}")
                } else {
                    format!("AC{n}-{m}")
                };
                let gone = m - n + 1.0;
                format!("{id}: removed {gone} AC ({span}), {left} left, renumbered from 1.")
            }
            Edit::Move(n, to) => format!("{id}: AC{n} moved to position {to}."),
        }
    }
}

fn count(board: &Value, id: &str) -> usize {
    doc(board, "items", id)
        .and_then(|i| i["ac"].as_array())
        .map_or(0, Vec::len)
}

fn within(n: f64, m: f64, len: usize) -> bool {
    let top = f64::from(u32::try_from(len).unwrap_or(u32::MAX));
    n >= 1.0 && m >= n && m <= top
}

/// One criterion change on an item, in one locked write.
fn run(r: &mut Run<'_>, id: Option<&str>, sub: &str, args: &[String]) -> Outcome {
    let edit = Edit::parse(sub, args, r.options.flag("undo"))?;
    r.write(|board, r| {
        let who = r.begin(board)?;
        let found = checked(board, &who, ("items", "ac"), id)?;
        let key = display(&found["id"]);
        let len = count(board, &key);
        if let Edit::Rm(n, m) = edit {
            if !within(n, m, len) {
                let said = args.first().map_or("", String::as_str);
                return Err(bad(format!(
                    "{key} has {len} AC; {said} is not within 1-{len}."
                )));
            }
        }
        apply(
            board,
            &who,
            &op(("ac", "items", &key), edit.fields(), r.options)?,
        )?;
        r.say(&[&edit.said(&key, count(board, &key))]);
        Ok(())
    })
}

/// `coord item ac <I> ...`; the older spellings stay: `ac <I> "<text>"` and `ac rm <I> <n>`.
pub(super) fn ac(r: &mut Run<'_>) -> Outcome {
    let words = r.args.clone();
    let word = |i: usize| words.get(i).cloned();
    if word(0).as_deref() == Some("rm") {
        return run(
            r,
            word(1).as_deref(),
            "rm",
            words.get(2..).unwrap_or_default(),
        );
    }
    let known = word(1).filter(|s| ["add", "set", "done", "rm", "move"].contains(&s.as_str()));
    match known {
        Some(sub) => run(
            r,
            word(0).as_deref(),
            &sub,
            words.get(2..).unwrap_or_default(),
        ),
        None => run(
            r,
            word(0).as_deref(),
            "add",
            words.get(1..2).unwrap_or_default(),
        ),
    }
}

/// `coord item ac-done <I> <n>`.
pub(super) fn ac_done(r: &mut Run<'_>) -> Outcome {
    let words = r.args.clone();
    run(
        r,
        words.first().cloned(),
        "done",
        words.get(1..2).unwrap_or_default(),
    )
}

/// `coord item ac-rm <I> <n>`.
pub(super) fn ac_rm(r: &mut Run<'_>) -> Outcome {
    let words = r.args.clone();
    run(
        r,
        words.first().cloned(),
        "rm",
        words.get(1..2).unwrap_or_default(),
    )
}
