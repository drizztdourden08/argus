//! What the tray shows: the state of the server and how many things wait for the owner. Pure, so
//! it is tested without a window.
// test-tag: critical

use super::icon::IconKind;

/// Where the server stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    /// Not polled yet.
    #[default]
    Checking,
    /// The health check answers; `runs` launches are running.
    Running {
        /// How many launches are running.
        runs: u32,
    },
    /// The server is being started and has not answered yet.
    Starting,
    /// The health check fails.
    Stopped,
    /// The owner chose Quit and the server is being stopped.
    Closing,
}

/// The whole state of the tray.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TrayState {
    /// The server.
    pub phase: Phase,
    /// How many things wait for the owner (questions, tests, decisions).
    pub attention: u32,
    /// True when the home is a shadow: the tooltip and the status line say so.
    pub shadow: bool,
}

impl TrayState {
    /// A running server with `runs` live launches.
    #[must_use]
    pub const fn running(runs: u32) -> Self {
        Self {
            phase: Phase::Running { runs },
            attention: 0,
            shadow: false,
        }
    }

    /// A server that is being started and has not answered yet.
    #[must_use]
    pub const fn starting() -> Self {
        Self {
            phase: Phase::Starting,
            attention: 0,
            shadow: false,
        }
    }

    /// A stopped server.
    #[must_use]
    pub const fn stopped() -> Self {
        Self {
            phase: Phase::Stopped,
            attention: 0,
            shadow: false,
        }
    }

    /// The state with `attention` things waiting for the owner.
    #[must_use]
    pub const fn with_attention(mut self, attention: u32) -> Self {
        self.attention = attention;
        self
    }

    /// The state of a shadow home when `shadow` is true: the tooltip says so.
    #[must_use]
    pub const fn shadowed(mut self, shadow: bool) -> Self {
        self.shadow = shadow;
        self
    }

    /// The same state while the owner's Quit is being carried out.
    #[must_use]
    pub const fn closing(mut self) -> Self {
        self.phase = Phase::Closing;
        self
    }

    /// The tooltip of the icon: `Argus: running (2 runs), 3 need you`.
    #[must_use]
    pub fn tooltip(&self) -> String {
        self.status_label()
    }

    /// The disabled status line of the menu, the same words as the tooltip.
    #[must_use]
    pub fn status_label(&self) -> String {
        let server = match self.phase {
            Phase::Checking => "checking".to_owned(),
            Phase::Running { runs } => {
                let noun = if runs == 1 { "run" } else { "runs" };
                format!("running ({runs} {noun})")
            }
            Phase::Starting => "starting".to_owned(),
            Phase::Stopped => "stopped".to_owned(),
            Phase::Closing => "closing".to_owned(),
        };
        let name = if self.shadow {
            "Argus (shadow)"
        } else {
            "Argus"
        };
        match self.attention {
            0 => format!("{name}: {server}"),
            n => format!("{name}: {server}, {n} need you"),
        }
    }

    /// Which icon to draw: the plain one while the server runs, the greyed one with a red badge
    /// in every other phase.
    #[must_use]
    pub const fn icon(&self) -> IconKind {
        match self.phase {
            Phase::Running { .. } => IconKind::Running,
            _ => IconKind::Stopped,
        }
    }

    /// True when the server is down, so the menu offers Start.
    #[must_use]
    pub const fn is_stopped(&self) -> bool {
        matches!(self.phase, Phase::Stopped)
    }

    /// True while Quit is being carried out.
    #[must_use]
    pub const fn is_closing(&self) -> bool {
        matches!(self.phase, Phase::Closing)
    }
}

#[cfg(test)]
mod tests {
    use super::{Phase, TrayState};
    use crate::tray::icon::IconKind;

    #[test]
    fn the_tooltip_counts_runs_with_the_right_noun() {
        assert_eq!(TrayState::running(0).tooltip(), "Argus: running (0 runs)");
        assert_eq!(TrayState::running(1).tooltip(), "Argus: running (1 run)");
        assert_eq!(TrayState::running(3).tooltip(), "Argus: running (3 runs)");
    }

    #[test]
    fn every_phase_has_its_words() {
        assert_eq!(TrayState::default().tooltip(), "Argus: checking");
        assert_eq!(TrayState::starting().tooltip(), "Argus: starting");
        assert_eq!(TrayState::stopped().tooltip(), "Argus: stopped");
        assert_eq!(TrayState::running(2).closing().tooltip(), "Argus: closing");
    }

    #[test]
    fn a_shadow_home_says_shadow_in_the_tooltip_and_the_status_line() {
        let state = TrayState::running(1).shadowed(true);
        assert_eq!(state.tooltip(), "Argus (shadow): running (1 run)");
        assert_eq!(state.status_label(), state.tooltip());
        assert_eq!(
            TrayState::stopped().shadowed(true).tooltip(),
            "Argus (shadow): stopped"
        );
    }

    #[test]
    fn what_waits_for_the_owner_is_added_only_when_there_is_something() {
        let state = TrayState::running(2).with_attention(3);
        assert_eq!(state.tooltip(), "Argus: running (2 runs), 3 need you");
        assert_eq!(state.status_label(), state.tooltip());
        assert_eq!(
            TrayState::stopped().with_attention(1).tooltip(),
            "Argus: stopped, 1 need you"
        );
    }

    #[test]
    fn the_icon_follows_the_server_and_a_tooltip_always_fits_windows() {
        assert_eq!(TrayState::running(0).icon(), IconKind::Running);
        for state in [
            TrayState::default(),
            TrayState::starting(),
            TrayState::stopped(),
            TrayState::stopped().closing(),
        ] {
            assert_eq!(state.icon(), IconKind::Stopped);
        }
        let worst = TrayState {
            phase: Phase::Running { runs: u32::MAX },
            attention: u32::MAX,
            shadow: true,
        };
        assert!(worst.tooltip().chars().count() <= 127);
    }
}
