//! `coord log`: the work log, grouped by subject. One document per subject (never one per line):
//! a subject keeps its latest 200 lines so the document stays small. Each line carries a `seq`
//! (monotonic per subject) so `coord log rm <subject> <seq>` can name it: a thread removes its
//! own lines, the coordinator any.

mod list;
mod rm;
mod write;

use crate::cli::{Call, Outcome};
use crate::verbs::work::run::Run;

/// `coord log "<text>"`, `coord log list` and `coord log rm`.
///
/// # Errors
/// Refused as the sub-verb refuses.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    let mut run = Run::of(call)?;
    match run.arg(0) {
        Some("list") => {
            run.shift();
            list::list(&mut run)
        }
        Some("rm") => {
            run.shift();
            rm::rm(&mut run)
        }
        _ => write::write(&mut run),
    }
}
