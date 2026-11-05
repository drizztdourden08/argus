//! The printout both migrations share: the report lines, each tagged on a dry run, then the summary.

use super::run::Run;

/// Says the `lines` (prefixed with `tag` and a space on a dry run) and then the `summary`.
pub(super) fn say_report(r: &mut Run<'_>, lines: &[String], tag: &str, dry: bool, summary: String) {
    let mut out: Vec<String> = lines
        .iter()
        .map(|l| if dry { format!("{tag} {l}") } else { l.clone() })
        .collect();
    out.push(summary);
    r.say(&[&out.join("\n")]);
}
