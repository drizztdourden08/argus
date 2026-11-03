//! Who may change a launcher: the coordinator (or the owner), the thread that made it, or,
//! for one made for a bundle, a thread that manages that bundle. Anyone registered may
//! make a new one.

use super::record::{Launcher, LauncherBoard, RuleError, Who};

pub fn authorize_launcher(
    board: &LauncherBoard,
    who: &Who,
    rec: Option<&Launcher>,
) -> Result<(), RuleError> {
    let Some(rec) = rec else { return Ok(()) };
    if who.is_lead() {
        return Ok(());
    }
    let Some(me) = who.thread.as_ref().map(|t| t.id.as_str()) else {
        return Err(RuleError::new(
            "no thread is calling: this needs a registered session.",
        ));
    };
    let maker = rec.created_by.as_deref();
    let manages_bundle = rec
        .bundle
        .as_ref()
        .and_then(|tag| board.builds.get(tag))
        .is_some_and(|b| b.thread.as_deref() == Some(me));
    if maker == Some(me) || manages_bundle {
        return Ok(());
    }
    Err(refusal(board, rec, maker))
}

fn refusal(board: &LauncherBoard, rec: &Launcher, maker: Option<&str>) -> RuleError {
    let maker_is_lead = maker
        .and_then(|m| board.threads.get(m))
        .is_some_and(|t| t.role == "coordinator");
    let allowed = match maker {
        Some(m) if !maker_is_lead => format!("{m} or the coordinator"),
        _ => "the coordinator".to_owned(),
    };
    let bundle = rec
        .bundle
        .as_ref()
        .map(|b| format!(" (or a thread that manages bundle {b})"))
        .unwrap_or_default();
    RuleError::new(format!(
        "launcher {} was set by {}; only {allowed}{bundle} can change it. Ask with coord inbox send {} \"...\" or coord message.",
        rec.name,
        maker.unwrap_or("another thread"),
        maker.unwrap_or("coordinator"),
    ))
}

/// The launcher `name`, when it exists and the caller may change it.
pub(super) fn launcher_to_change<'a>(
    board: &'a LauncherBoard,
    who: &Who,
    name: &str,
) -> Result<&'a Launcher, RuleError> {
    let rec = board
        .launchers
        .get(name)
        .ok_or_else(|| RuleError::new(format!("no launcher \"{name}\".")))?;
    authorize_launcher(board, who, Some(rec))?;
    Ok(rec)
}
