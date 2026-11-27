//! The check after a switch: once Claude started again and its Local Storage went quiet, it
//! checks that the sidebar copy held, and ends the switch as done or failed.

use std::path::Path;
use std::time::{Duration, SystemTime};

use argus_accounts::{Account, find_app_data};
use argus_notify::{By, Level, PingRequest};
use serde_json::json;

use super::AccountsService;
use super::accounts_switch::cached_version;
use super::accounts_switch_log::{Line, LineState, Stage, sidebar_check_line};
use crate::Services;

/// How often the Local Storage is looked at while it settles, at most.
const SETTLE_POLL: Duration = Duration::from_secs(1);
/// How many looks in a row must find it unchanged before it counts as settled.
const QUIET_POLLS: u32 = 3;

/// What the Local Storage folder looks like now: the number of files, their bytes and the latest
/// change. Two equal looks in a row mean Claude is not writing.
type Look = (usize, u64, Option<SystemTime>);

fn look_at(dir: &Path) -> Look {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return (0, 0, None);
    };
    entries
        .filter_map(Result::ok)
        .filter_map(|e| e.metadata().ok())
        .fold((0, 0, None), |(count, bytes, latest), meta| {
            let at = meta.modified().ok();
            (count + 1, bytes + meta.len(), latest.max(at))
        })
}

/// Waits until `look` finds the same thing [`QUIET_POLLS`] times in a row, and never longer than
/// `cap`. A cap of zero does not wait.
fn settle<T: PartialEq>(cap: Duration, mut look: impl FnMut() -> T, mut nap: impl FnMut(Duration)) {
    if cap.is_zero() {
        return;
    }
    let poll = SETTLE_POLL.min(cap / (QUIET_POLLS + 1));
    let polls = cap.as_nanos().div_ceil(poll.as_nanos().max(1));
    let mut last = look();
    let mut quiet = 0;
    for _ in 0..polls {
        nap(poll);
        let now = look();
        if now == last {
            quiet += 1;
            if quiet >= QUIET_POLLS {
                return;
            }
        } else {
            quiet = 0;
            last = now;
        }
    }
}

/// The wait before the check: until Claude's Local Storage is quiet, up to the settle time.
fn wait_for_quiet(accounts: &AccountsService) {
    let cap = accounts.sidebar_settle();
    match find_app_data(accounts.config().app_data.as_deref()) {
        Some(app) => {
            let dir = app.join("Local Storage").join("leveldb");
            settle(cap, || look_at(&dir), std::thread::sleep);
        }
        None => std::thread::sleep(cap),
    }
}

/// What the check found, and the one repair it made when sidebar changes were missing.
struct Checked {
    /// How many changes were missing at the first look.
    missing: usize,
    /// The last look: what is still missing.
    left: Result<usize, String>,
    /// How the one repair went; `None` when nothing was missing.
    repair: Option<Result<(), String>>,
}

/// Looks; when changes are missing, writes the copy once more and looks again.
fn check_and_repair(
    mut look: impl FnMut() -> Result<usize, String>,
    mut write: impl FnMut() -> Result<(), String>,
) -> Checked {
    let first = look();
    let missing = match first {
        Ok(n) if n > 0 => n,
        _ => {
            return Checked {
                missing: 0,
                left: first,
                repair: None,
            };
        }
    };
    let repair = write();
    Checked {
        missing,
        left: look(),
        repair: Some(repair),
    }
}

/// The step-log line of the one repair.
fn repair_line(missing: usize, repair: &Result<(), String>) -> Line {
    match repair {
        Ok(()) => Line::new(
            Stage::Verify,
            LineState::Done,
            format!(
                "Sidebar repair: {missing} sidebar changes were missing after Claude started, so the copy was written once more."
            ),
        ),
        Err(why) => Line::new(
            Stage::Verify,
            LineState::Failed,
            format!("Sidebar repair failed: the copy could not be written again ({why})."),
        ),
    }
}

/// After a confirmed switch: checks that the sync held once Claude started again, and ends the
/// switch `id` (verifying until then) as done or failed. A check that passes confirms the
/// installed Claude version; one that fails is a failed step, the error of the doc, an urgent
/// toast and a log line, never silent. Blocking.
pub(crate) fn check_sidebars(
    svc: &Services,
    accounts: &AccountsService,
    ends: &(Account, Account),
    id: &str,
) {
    let (from, to) = ends;
    wait_for_quiet(accounts);
    let checked = check_and_repair(
        || accounts.copy_left(from, to),
        || accounts.copy_onto(from, to, false).map(|_| ()),
    );
    let left = checked.left;
    let line = sidebar_check_line(&left, &to.label);
    let mut lines: Vec<Line> = (checked.repair.iter())
        .map(|r| repair_line(checked.missing, r))
        .collect();
    lines.push(line.clone());
    if left == Ok(0) {
        accounts.finish_switch(id, ("done", None), &lines);
        verify_version(svc, accounts);
        return;
    }
    let detail = match &left {
        Ok(n) => format!("{n} sidebar changes of the copy are missing again after Claude started"),
        Err(why) => why.clone(),
    };
    let error = format!("the switch worked but the sidebar check failed: {detail}");
    accounts.finish_switch(id, ("failed", Some(error.as_str())), &lines);
    svc.log
        .warn("accounts sidebar check failed", json!({ "error": detail }));
    if let Some(notify) = &svc.notify {
        notify.send(PingRequest {
            title: Some("The sidebars are not the same after the switch".to_owned()),
            text: line.text,
            level: Level::Urgent,
            project: None,
            thread: None,
            by: By::Auto,
            url: Some(format!("http://127.0.0.1:{}/?tab=accounts", svc.port())),
        });
    }
}

/// A switch whose account and sidebars checked out: the installed Claude version is now the
/// verified one.
fn verify_version(svc: &Services, accounts: &AccountsService) {
    let Some(version) = cached_version() else {
        return;
    };
    if accounts.config().verified_app_version.as_deref() == Some(version.as_str()) {
        return;
    }
    let saved = accounts
        .update(&json!({ "verifiedAppVersion": version }))
        .and_then(|config| super::accounts::save(svc, &config).map_err(|e| e.to_string()));
    if let Err(e) = saved {
        svc.log
            .error("accounts verified version not saved", json!({ "error": e }));
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::time::Duration;

    use super::{QUIET_POLLS, check_and_repair, repair_line, settle};
    use crate::services::accounts_switch_log::LineState;

    #[test]
    fn a_clean_first_look_makes_no_repair() {
        let written = Cell::new(0_u32);
        let checked = check_and_repair(
            || Ok(0),
            || {
                written.set(written.get() + 1);
                Ok(())
            },
        );
        assert_eq!(checked.left, Ok(0));
        assert_eq!(written.get(), 0);
        assert_eq!(checked.repair, None);
    }

    #[test]
    fn missing_changes_are_written_once_and_looked_at_again() {
        let looks = Cell::new(0_u32);
        let written = Cell::new(0_u32);
        let checked = check_and_repair(
            || {
                looks.set(looks.get() + 1);
                Ok(if looks.get() == 1 { 4 } else { 0 })
            },
            || {
                written.set(written.get() + 1);
                Ok(())
            },
        );
        assert_eq!(checked.missing, 4);
        assert_eq!(checked.left, Ok(0));
        assert_eq!(checked.repair, Some(Ok(())));
        assert_eq!(written.get(), 1);
    }

    #[test]
    fn changes_still_missing_after_the_repair_stay_missing() {
        let written = Cell::new(0_u32);
        let checked = check_and_repair(
            || Ok(2),
            || {
                written.set(written.get() + 1);
                Ok(())
            },
        );
        assert_eq!(checked.left, Ok(2));
        assert_eq!(written.get(), 1);
    }

    #[test]
    fn an_unreadable_sidebar_is_not_repaired() {
        let checked = check_and_repair(|| Err("locked".to_owned()), || Ok(()));
        assert_eq!(checked.left, Err("locked".to_owned()));
        assert_eq!(checked.repair, None);
    }

    #[test]
    fn a_failed_repair_is_a_failed_line() {
        let line = repair_line(3, &Err("busy".to_owned()));
        assert_eq!(line.state, LineState::Failed);
    }

    #[test]
    fn a_quiet_folder_settles_after_a_few_looks() {
        let naps = Cell::new(0_u32);
        settle(Duration::from_secs(20), || 7, |_| naps.set(naps.get() + 1));
        assert_eq!(naps.get(), QUIET_POLLS);
    }

    #[test]
    fn a_folder_that_keeps_changing_waits_for_the_cap() {
        let naps = Cell::new(0_u32);
        let look = Cell::new(0_u32);
        settle(
            Duration::from_secs(4),
            || {
                look.set(look.get() + 1);
                look.get()
            },
            |_| naps.set(naps.get() + 1),
        );
        assert_eq!(naps.get(), 4);
    }

    #[test]
    fn a_change_restarts_the_quiet_count() {
        let naps = Cell::new(0_u32);
        let looks = Cell::new(0_u32);
        settle(
            Duration::from_secs(20),
            || {
                looks.set(looks.get() + 1);
                u32::from(looks.get() > 2)
            },
            |_| naps.set(naps.get() + 1),
        );
        assert_eq!(naps.get(), QUIET_POLLS + 2);
    }

    #[test]
    fn a_zero_cap_does_not_wait() {
        let naps = Cell::new(0_u32);
        settle(Duration::ZERO, || 1, |_| naps.set(naps.get() + 1));
        assert_eq!(naps.get(), 0);
    }
}
