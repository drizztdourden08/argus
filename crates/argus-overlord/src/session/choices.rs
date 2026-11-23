//! The owner's settings of the hosted child: the models, efforts and permission modes he may pick,
//! their defaults, and the check of a change against them.

use serde::Serialize;

/// The models the composer offers, id and label.
const MODELS: [(&str, &str); 4] = [
    ("claude-opus-5-5", "Opus 5.5"),
    ("claude-sonnet-5-5", "Sonnet 5.5"),
    ("claude-haiku-5-5", "Haiku 5.5"),
    ("claude-fable-5-2", "Fable 5.2"),
];

/// The effort levels the CLI takes with `--effort`, lowest first.
pub const EFFORTS: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];

/// The permission modes the CLI takes with `--permission-mode`, id and label.
const PERMISSION_MODES: [(&str, &str); 5] = [
    ("bypassPermissions", "Bypass"),
    ("acceptEdits", "Accept edits"),
    ("default", "Ask"),
    ("plan", "Plan"),
    ("auto", "Auto"),
];

/// The effort when none is saved.
pub const DEFAULT_EFFORT: &str = "high";

/// The permission mode when none is saved: every call runs, as the owner wants.
pub const DEFAULT_PERMISSION_MODE: &str = "bypassPermissions";

/// One entry of a picker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Choice {
    pub id: String,
    pub label: String,
}

fn choice((id, label): (&str, &str)) -> Choice {
    Choice {
        id: id.to_owned(),
        label: label.to_owned(),
    }
}

/// The models to offer: the fixed list, then `current` with its id as label when the list lacks it.
#[must_use]
pub fn models(current: &str) -> Vec<Choice> {
    let mut all: Vec<Choice> = MODELS.into_iter().map(choice).collect();
    if !all.iter().any(|c| c.id == current) {
        all.push(choice((current, current)));
    }
    all
}

/// The permission modes to offer.
#[must_use]
pub fn permission_modes() -> Vec<Choice> {
    PERMISSION_MODES.into_iter().map(choice).collect()
}

/// True for an effort level the CLI takes.
#[must_use]
pub fn is_effort(value: &str) -> bool {
    EFFORTS.contains(&value)
}

/// True for a permission mode the CLI takes.
#[must_use]
pub fn is_permission_mode(value: &str) -> bool {
    PERMISSION_MODES.iter().any(|(id, _)| *id == value)
}

/// A change the owner asks for; a `None` field stays as it is.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Change {
    pub model: Option<String>,
    pub effort: Option<String>,
    pub permission_mode: Option<String>,
}

/// The field of a change that is not one of the choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadChoice {
    Model,
    Effort,
    PermissionMode,
}

impl BadChoice {
    /// One plain sentence for the owner.
    #[must_use]
    pub fn sentence(self) -> &'static str {
        match self {
            BadChoice::Model => "model must be one of the offered models",
            BadChoice::Effort => "effort must be low, medium, high, xhigh or max",
            BadChoice::PermissionMode => {
                "permissionMode must be bypassPermissions, acceptEdits, default, plan or auto"
            }
        }
    }
}

fn holds(value: Option<&str>, ok: impl Fn(&str) -> bool, bad: BadChoice) -> Result<(), BadChoice> {
    match value {
        Some(v) if !ok(v) => Err(bad),
        _ => Ok(()),
    }
}

/// Checks each field of `change` against the choices; `current` is the saved model, which is
/// offered too.
///
/// # Errors
/// The first [`BadChoice`] found.
pub fn check(change: &Change, current: &str) -> Result<(), BadChoice> {
    let offered = models(current);
    holds(
        change.model.as_deref(),
        |v| offered.iter().any(|c| c.id == v),
        BadChoice::Model,
    )?;
    holds(change.effort.as_deref(), is_effort, BadChoice::Effort)?;
    holds(
        change.permission_mode.as_deref(),
        is_permission_mode,
        BadChoice::PermissionMode,
    )
}

#[cfg(test)]
mod specs {
    use super::*;

    fn change(model: Option<&str>, effort: Option<&str>, mode: Option<&str>) -> Change {
        Change {
            model: model.map(str::to_owned),
            effort: effort.map(str::to_owned),
            permission_mode: mode.map(str::to_owned),
        }
    }

    // test-tag: critical
    #[test]
    fn a_change_inside_the_choices_passes() {
        let good = change(Some("claude-opus-5-5"), Some("xhigh"), Some("plan"));
        assert_eq!(check(&good, "sonnet"), Ok(()));
        assert_eq!(check(&Change::default(), "sonnet"), Ok(()));
    }

    #[test]
    fn each_bad_field_is_named() {
        let effort = change(None, Some("huge"), None);
        assert_eq!(check(&effort, "sonnet"), Err(BadChoice::Effort));
        let mode = change(None, None, Some("yolo"));
        assert_eq!(check(&mode, "sonnet"), Err(BadChoice::PermissionMode));
        let model = change(Some("gpt"), None, None);
        assert_eq!(check(&model, "sonnet"), Err(BadChoice::Model));
    }

    #[test]
    fn the_current_model_is_offered_when_the_list_lacks_it() {
        let all = models("sonnet");
        assert_eq!(all.len(), 5);
        assert_eq!(all[4], choice(("sonnet", "sonnet")));
        assert_eq!(models("claude-haiku-5-5").len(), 4);
        assert_eq!(check(&change(Some("sonnet"), None, None), "sonnet"), Ok(()));
    }

    #[test]
    fn the_defaults_are_choices() {
        assert!(is_effort(DEFAULT_EFFORT));
        assert!(is_permission_mode(DEFAULT_PERMISSION_MODE));
        assert_eq!(permission_modes()[2], choice(("default", "Ask")));
    }
}
