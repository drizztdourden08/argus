//! Launcher names: a slug short enough that `launcher-<name>` is still a run name (40
//! characters), and the name a bundle tag gets from `coord build --cmd`.

use std::sync::LazyLock;

use regex::Regex;

use super::record::RuleError;

pub const KINDS: [&str; 3] = ["app", "dev", "other"];
pub const DEF_PREFIX: &str = "launcher-";
const SLUG_MAX: usize = 31;

static NAME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^[a-z0-9][a-z0-9-]{0,30}$").unwrap());
static NON_SLUG: LazyLock<Regex> = LazyLock::new(|| Regex::new("[^a-z0-9]+").unwrap());

/// The run definition name of a launcher's Play.
pub fn launcher_def_name(name: &str) -> String {
    format!("{DEF_PREFIX}{name}")
}

pub fn is_launcher_name(name: &str) -> bool {
    NAME_RE.is_match(name)
}

pub fn require_launcher_name(value: &str) -> Result<String, RuleError> {
    let name = value.trim();
    if name.is_empty() {
        return Err(RuleError::new(
            "name the launcher: a slug such as master, build-2 or editor-1.",
        ));
    }
    if !is_launcher_name(name) {
        return Err(RuleError::new(format!(
            "\"{name}\" is not a launcher name: lowercase letters, digits and dashes, starting with a letter or digit, 31 characters at most (e.g. master, build-2)."
        )));
    }
    Ok(name.to_owned())
}

/// The launcher name a bundle tag gets: lowercased, every run of other characters one dash,
/// cut to fit. `None` when nothing of the tag is left.
pub fn launcher_slug(tag: &str) -> Option<String> {
    let lowered = tag.to_lowercase();
    let dashed = NON_SLUG.replace_all(&lowered, "-");
    let slug = dashed.trim_matches('-');
    if slug.is_empty() {
        return None;
    }
    let cut: String = slug.chars().take(SLUG_MAX).collect();
    Some(cut.trim_end_matches('-').to_owned())
}
