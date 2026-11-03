//! The fields an agent verb sets from its options (`task`, `kind`, `branch`), the model and
//! its justification, the check-in time and the Claude id, each checked as the page's edit
//! checks it.

use argus_agents::model_rules::{ModelWhy, check_checkin, check_model_why};
use argus_board::limits::{optional_text, require_str};
use argus_core::js;
use serde_json::{Map, Value};

use crate::cli::Options;
use crate::cli::exit::CoordError;
use crate::verbs::agent::base::opt::value;

/// The fields a verb changes, in the order they are written.
pub type Fields = Map<String, Value>;

const KINDS: [&str; 4] = ["build", "test", "research", "chore"];
const CLAUDE_ID_MAX: usize = 120;

fn kind_of(v: &str) -> Result<Value, CoordError> {
    if KINDS.contains(&v) {
        return Ok(Value::from(v));
    }
    Err(CoordError::refuse(format!(
        "kind must be one of {}.",
        KINDS.join(", ")
    )))
}

fn check(key: &str, v: &str) -> Result<Value, CoordError> {
    Ok(match key {
        "task" => Value::from(require_str("task", v)?),
        "kind" => kind_of(v)?,
        other => optional_text(other, &Value::from(v))?.map_or(Value::Null, Value::from),
    })
}

/// The fields the options name among `keys` (`task`, `kind`, `branch`). The items, the place and
/// the blockers of an agent are read by [`super::links`].
///
/// # Errors
/// A bare flag, or a value the field refuses.
pub fn fields_of(options: &Options, keys: &[&str]) -> Result<Fields, CoordError> {
    let mut out = Fields::new();
    for key in keys {
        if let Some(v) = value(options, key)? {
            out.insert((*key).to_owned(), check(key, v)?);
        }
    }
    Ok(out)
}

/// The model and its justification as the fields they are stored in.
pub fn add_model(fields: &mut Fields, model: ModelWhy) {
    fields.insert("model".to_owned(), Value::from(model.model));
    fields.insert("modelWhy".to_owned(), Value::from(model.why));
}

/// `checkModelWhy`, with the rules' refusal as the verb's own.
///
/// # Errors
/// The model or its justification does not fit the standard.
pub fn model_why(
    model: Option<&str>,
    why: Option<&str>,
    kind: &str,
) -> Result<ModelWhy, CoordError> {
    check_model_why(model, why, kind).map_err(|e| CoordError::refuse(e.0))
}

/// `checkCheckin`: the minutes to store for the model, or none.
///
/// # Errors
/// A value outside 15 to 30, or none for an opus-level model.
pub fn checkin(model: Option<&str>, raw: Option<&str>) -> Result<Option<u32>, CoordError> {
    check_checkin(model, raw).map_err(|e| CoordError::refuse(e.0))
}

/// The `--claude-id` option: `None` when absent.
///
/// # Errors
/// A bare flag, or an id that is not letters, digits, dots, dashes and underscores.
pub fn claude_id_option(options: &Options) -> Result<Option<String>, CoordError> {
    let Some(v) = value(options, "claude-id")? else {
        return Ok(None);
    };
    let id = js::trim(v);
    let fits = !id.is_empty()
        && id.chars().count() <= CLAUDE_ID_MAX
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'));
    if fits {
        return Ok(Some(id.to_owned()));
    }
    Err(CoordError::refuse(format!(
        "--claude-id must be the id Claude printed as agentId (letters, digits, dots, dashes), got \"{v}\"."
    )))
}
