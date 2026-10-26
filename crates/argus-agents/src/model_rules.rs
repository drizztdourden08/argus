//! The rules for the model of a sub-agent record: every agent is recorded with the model it runs
//! on and why, and an opus-level model is checked on by a wake every 15 to 30 minutes.
//!
//! The standard is short: haiku for chores and tests, sonnet for most coding, opus only for design,
//! architecture, unknown-cause, risky-review, plan or an owner request.

/// The standard, as the refusals quote it.
pub const MODEL_STANDARD: &str = "haiku for chores and tests, sonnet for most coding including upgrades and lint/type fixes, opus only for design, architecture, unknown-cause, risky-review, plan or an owner request";

/// The help line for `--checkin`.
pub const CHECKIN_HELP: &str = "--checkin 15..30: minutes between check-in wakes for an opus-level agent, pick by assumed complexity (15 for risky or sprawling work, 30 for a well-bounded task)";

const OPUS_EXAMPLE: &str = "--why \"design: new document model touches every module\"";
const WHY_MIN: usize = 12;
const WHY_MAX: usize = 200;
const MODEL_MAX: usize = 40;
const CHECKIN_MIN: u32 = 15;
const CHECKIN_MAX: u32 = 30;
const OPUS_WORDS: [&str; 6] = [
    "design",
    "architecture",
    "unknown-cause",
    "risky-review",
    "plan",
    "owner",
];

/// A value the rules refuse, with the sentence the CLI prints.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct RuleError(pub String);

/// The model and its justification, checked together.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelWhy {
    /// The model, one word.
    pub model: String,
    /// The justification, trimmed.
    pub why: String,
}

fn refuse<T>(message: String) -> Result<T, RuleError> {
    Err(RuleError(message))
}

fn require_text(kind: &str, value: &str, max: usize) -> Result<String, RuleError> {
    let text = value.trim();
    if text.is_empty() {
        return refuse(format!("{kind} text is required."));
    }
    let len = text.chars().count();
    if len > max {
        return refuse(format!(
            "{kind} is {len} chars, the limit is {max}. Shorten it."
        ));
    }
    Ok(text.to_owned())
}

fn one_word(model: &str) -> Result<String, RuleError> {
    let name = require_text("model", model, MODEL_MAX)?;
    if name.contains(char::is_whitespace) {
        return refuse(format!("model must be one word, got \"{name}\"."));
    }
    Ok(name)
}

/// Whether a model is above haiku and sonnet, so it gets check-in wakes.
#[must_use]
pub fn high_tier(model: &str) -> bool {
    let lower = model.to_lowercase();
    !model.is_empty() && !lower.contains("haiku") && !lower.contains("sonnet")
}

fn names_category(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    OPUS_WORDS.iter().any(|word| {
        lower.match_indices(word).any(|(at, _)| {
            let before = lower[..at].chars().next_back();
            !before.is_some_and(|c| c.is_ascii_lowercase())
        })
    })
}

fn check_opus(kind: &str, why: &str) -> Result<(), RuleError> {
    if kind == "chore" || kind == "test" {
        return refuse(format!(
            "opus is not for {kind} agents: use haiku. Only an owner request overrides that: --why \"owner asked: ...\". Standard: {MODEL_STANDARD}."
        ));
    }
    if !names_category(why) {
        return refuse(format!(
            "opus needs a justification in one of these categories: design, architecture, unknown-cause, risky-review, plan (or \"owner asked: ...\" when the owner asked for opus). Standard: {MODEL_STANDARD}. Example: {OPUS_EXAMPLE}"
        ));
    }
    Ok(())
}

/// Checks the model and its justification. `kind` is the agent's kind; an opus chore or test
/// needs an owner request, and an owner request ("owner asked: ...") is never refused.
///
/// # Errors
/// The model or the justification is missing, too short, too long, or does not fit the standard.
pub fn check_model_why(
    model: Option<&str>,
    why: Option<&str>,
    kind: &str,
) -> Result<ModelWhy, RuleError> {
    let Some(model) = model.filter(|m| !m.is_empty()) else {
        return refuse(format!(
            "--model is required: haiku, sonnet or opus. Standard: {MODEL_STANDARD}."
        ));
    };
    let name = one_word(model)?;
    let Some(why) = why else {
        return refuse(format!(
            "--why is required with --model: one line on why {name} fits this task. Standard: {MODEL_STANDARD}. Example: {OPUS_EXAMPLE}"
        ));
    };
    let text = require_text("modelwhy", why, WHY_MAX)?;
    let owner = text.to_ascii_lowercase().starts_with("owner");
    let len = text.chars().count();
    if !owner && len < WHY_MIN {
        return refuse(format!(
            "--why is {len} chars, the minimum is {WHY_MIN}: say what about the task needs {name}."
        ));
    }
    if name.to_lowercase().contains("opus") && !owner {
        check_opus(kind, &text)?;
    }
    Ok(ModelWhy {
        model: name,
        why: text,
    })
}

fn parse_checkin(raw: &str) -> Result<u32, RuleError> {
    let text = raw.trim();
    let whole = !text.is_empty() && text.chars().all(|c| c.is_ascii_digit());
    match text.parse::<u32>() {
        Ok(n) if whole && (CHECKIN_MIN..=CHECKIN_MAX).contains(&n) => Ok(n),
        _ => refuse(format!(
            "--checkin must be a whole number of minutes from {CHECKIN_MIN} to {CHECKIN_MAX}, got \"{raw}\". {CHECKIN_HELP}"
        )),
    }
}

/// The `checkinMin` to store for a record whose model is `model` (`None`: not known yet), from
/// the raw `--checkin`. A value is checked for every model; it is kept only for an opus-level
/// one, which also requires it. Returns the minutes to store, `None` when nothing is stored.
///
/// # Errors
/// The value is not a whole number from 15 to 30, or an opus-level model has none.
pub fn check_checkin(model: Option<&str>, raw: Option<&str>) -> Result<Option<u32>, RuleError> {
    let given = raw.map(parse_checkin).transpose()?;
    let tier = model.filter(|m| !m.is_empty());
    match (tier, given) {
        (Some(m), _) if !high_tier(m) => Ok(None),
        (Some(m), None) => refuse(format!("--checkin is required for {m}. {CHECKIN_HELP}")),
        (_, given) => Ok(given),
    }
}
