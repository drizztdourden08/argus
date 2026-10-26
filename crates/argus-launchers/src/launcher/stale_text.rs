//! The texts of the two launcher wakes. A wake text is clipped to 300 characters by the wake
//! engine, so the list shrinks until the whole text fits.

use super::every::duration_text;

pub const MAX_TEXT: usize = 300;
const MAX_LISTED: usize = 5;
const TAIL: &str = " Analyse: still needed? keep it (coord launcher keep <name> --days n), retire it, or change its --use-every.";

/// A launcher over its window, as the wake lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaleItem {
    pub name: String,
    pub use_every: String,
    pub legacy: bool,
    pub unused_ms: i64,
}

pub fn master_text() -> String {
    "No master launcher: every project must have one (the main checkout's app). Set it: coord launcher set master --cmd \"<command that starts it>\". Until then coord build hand is refused and the owner has no Play for this project.".to_owned()
}

fn cell(item: &StaleItem) -> String {
    format!(
        "{} (unused {}, expected every {}{})",
        item.name,
        duration_text(item.unused_ms),
        item.use_every,
        if item.legacy { " by default" } else { "" }
    )
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

fn no_field(names: &[String]) -> String {
    let listed = names
        .iter()
        .take(MAX_LISTED)
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    let more = if names.len() > MAX_LISTED { " ..." } else { "" };
    format!(
        " No --use-every on {listed}{more}: set it (coord launcher set <name> --use-every 7d); 7d is assumed."
    )
}

fn clip(text: &str) -> String {
    text.chars().take(MAX_TEXT).collect()
}

/// `2 launchers not used within their window: build-2 (unused 9d, expected every 7d), ...
/// Analyse: still needed? ...`. `legacy` are the names without a `--use-every` that are not
/// stale yet: they ride along when the text still fits, and alone they are a prompt to set
/// the field.
pub fn stale_text(stale: &[StaleItem], legacy: &[String]) -> String {
    if stale.is_empty() {
        if legacy.is_empty() {
            return String::new();
        }
        return clip(&format!(
            "{} launcher{} without a usage window.{} Reported when unused longer.",
            legacy.len(),
            plural(legacy.len()),
            no_field(legacy)
        ));
    }
    let total = stale.len();
    let mut text = String::new();
    for listed in (1..=total.min(MAX_LISTED)).rev() {
        let more = if total > listed {
            format!(" (+{} more)", total - listed)
        } else {
            String::new()
        };
        let cells: Vec<String> = stale.iter().take(listed).map(cell).collect();
        text = format!(
            "{total} launcher{} not used within their window: {}{more}.{TAIL}",
            plural(total),
            cells.join(", ")
        );
        if text.chars().count() <= MAX_TEXT {
            break;
        }
    }
    let extra = if legacy.is_empty() {
        String::new()
    } else {
        no_field(legacy)
    };
    if text.chars().count() + extra.chars().count() <= MAX_TEXT {
        text.push_str(&extra);
    }
    clip(&text)
}
