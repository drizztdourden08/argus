//! `coord test migrate-runs [--dry-run]`: the bundles and the runs kept on test plans become test
//! runs, once per board and idempotent (item I-99). The lead's verb; a dry run writes nothing.

use argus_board::Board;
use argus_board::migrate_runs::{Report, migrate_runs};
use argus_board::ops::Role;

use crate::cli::Outcome;
use crate::verbs::work::report_lines::say_report;
use crate::verbs::work::run::{Res, Run, bad};
use crate::who::Caller;

fn summary(report: &Report, dry: bool) -> String {
    if report.already {
        return "test runs are already migrated on this board; nothing to do.".to_owned();
    }
    let counts = format!(
        "{} run(s) from plan runs, {} from bundles, {} new plan(s)",
        report.from_plan_runs, report.from_bundles, report.plans
    );
    if dry {
        format!("dry run: would create {counts}; nothing written.")
    } else {
        format!("migrated test runs: {counts}.")
    }
}

fn print(r: &mut Run<'_>, report: &Report, dry: bool) {
    say_report(r, &report.lines, "would:", dry, summary(report, dry));
}

fn lead_only(by: &Caller) -> Res<()> {
    if by.role == Role::Thread {
        return Err(bad(
            "coord test migrate-runs belongs to the lead: try --dry-run to see what it would do.",
        ));
    }
    Ok(())
}

fn migrate_board(board: &mut Board, by: &Caller) -> Res<Report> {
    migrate_runs(board, &by.actor)
}

/// `coord test migrate-runs [--dry-run]`.
pub(super) fn migrate(r: &mut Run<'_>) -> Outcome {
    if r.options.get("dry-run").is_some() {
        return r.read(|board, r| {
            let by = r.who(board)?;
            let report = migrate_board(&mut board.clone(), &by)?;
            print(r, &report, true);
            Ok(())
        });
    }
    r.write(|board, r| {
        let by = r.begin(board)?;
        lead_only(&by)?;
        let report = migrate_board(board, &by)?;
        print(r, &report, false);
        Ok(())
    })
}
