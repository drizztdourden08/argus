//! The rule of the boot revival (I-171): which project leads get a resume task after a machine
//! boot. Pure: the wiring in `boot_resume.rs` gathers the facts and writes the task.

/// The Overlord's mode, as `config.overlord.mode` and the shadow flag set it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Off,
    Shadow,
    Live,
}

/// A project lead as the wiring saw it at the check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Lead {
    pub board: String,
    pub thread: String,
    /// A wake listener is connected for the thread.
    pub listening: bool,
}

/// The key a lead is recorded under for one boot.
pub(crate) fn lead_key(lead: &Lead) -> String {
    format!("{}/{}", lead.board, lead.thread)
}

/// How far two readings of the boot time may differ and still be the same boot (the reading is the
/// clock minus the uptime counter, so it moves by a few milliseconds).
const SAME_BOOT_MS: i64 = 60_000;

/// Whether a recorded boot time is the boot of now.
pub(crate) fn same_boot(recorded_ms: i64, now_ms: i64) -> bool {
    (recorded_ms - now_ms).abs() <= SAME_BOOT_MS
}

/// The leads to resume: only in mode `live`, only a lead with no listener, and only one not yet
/// resumed in this boot (`done` holds their [`lead_key`]s).
pub(crate) fn needing_resume(mode: Mode, leads: &[Lead], done: &[String]) -> Vec<Lead> {
    if mode != Mode::Live {
        return Vec::new();
    }
    leads
        .iter()
        .filter(|lead| !lead.listening && !done.contains(&lead_key(lead)))
        .cloned()
        .collect()
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::{Lead, Mode, lead_key, needing_resume, same_boot};

    fn lead(board: &str, listening: bool) -> Lead {
        Lead {
            board: board.into(),
            thread: "coordinator".into(),
            listening,
        }
    }

    #[test]
    fn a_lead_with_a_listener_gets_none() {
        let got = needing_resume(Mode::Live, &[lead("rotp", true)], &[]);
        assert_eq!(got.len(), 0);
    }

    #[test]
    fn a_lead_without_a_listener_gets_one() {
        let leads = [lead("rotp", false), lead("brock", true)];
        let got = needing_resume(Mode::Live, &leads, &[]);
        assert_eq!(got, vec![lead("rotp", false)]);
    }

    #[test]
    fn a_second_run_in_the_same_boot_gets_none() {
        let leads = [lead("rotp", false)];
        let done = vec![lead_key(&leads[0])];
        assert_eq!(needing_resume(Mode::Live, &leads, &done).len(), 0);
    }

    #[test]
    fn a_lead_not_yet_recorded_still_gets_one_beside_a_recorded_one() {
        let leads = [lead("rotp", false), lead("brock", false)];
        let done = vec![lead_key(&leads[0])];
        assert_eq!(
            needing_resume(Mode::Live, &leads, &done),
            vec![lead("brock", false)]
        );
    }

    #[test]
    fn mode_off_or_shadow_gets_none() {
        let leads = [lead("rotp", false)];
        assert_eq!(needing_resume(Mode::Off, &leads, &[]).len(), 0);
        assert_eq!(needing_resume(Mode::Shadow, &leads, &[]).len(), 0);
    }

    #[test]
    fn a_boot_time_within_a_minute_is_the_same_boot() {
        assert!(same_boot(1_000_000, 1_000_050));
        assert!(!same_boot(1_000_000, 1_000_000 + 3_600_000));
    }
}
