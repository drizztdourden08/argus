//! Who may hold the Overlord's seat, the thread `coordinator` of the Overlord's board (I-173, K4).
//! Two live coordinators never exist at once: while the mode is live only the hosted child may
//! register it, and while the mode is not live only a desktop session may.

use std::fmt;

use crate::mode::Mode;

/// The thread of the Overlord on its board.
pub const SEAT: &str = "coordinator";

/// Why a registration of the seat is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeatFault {
    /// A desktop session while Argus runs the hosted Overlord.
    DesktopWhileLive,
    /// The hosted child while the mode is off or shadow.
    HostedWhileNotLive,
}

impl fmt::Display for SeatFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SeatFault::DesktopWhileLive => {
                "Argus runs the hosted Overlord (mode live). Roll back first: POST /api/overlord/mode with {\"mode\":\"off\"}, then register here."
            }
            SeatFault::HostedWhileNotLive => {
                "the hosted Overlord takes the coordinator thread only in mode live; the desktop Overlord holds it now."
            }
        })
    }
}

impl std::error::Error for SeatFault {}

/// May a session register the seat? `hosted` is true for the hosted child (`ARGUS_HOSTED=1`).
///
/// # Errors
/// A [`SeatFault`] when the session is not the one the mode puts in the seat.
pub fn may_take_seat(mode: Mode, hosted: bool) -> Result<(), SeatFault> {
    match (mode == Mode::Live, hosted) {
        (true, false) => Err(SeatFault::DesktopWhileLive),
        (false, true) => Err(SeatFault::HostedWhileNotLive),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // test-tag: critical
    #[test]
    fn in_live_only_the_hosted_child_takes_the_seat() {
        let refused = Err(SeatFault::DesktopWhileLive);
        assert_eq!(may_take_seat(Mode::Live, true), Ok(()));
        assert_eq!(may_take_seat(Mode::Live, false), refused);
    }

    // test-tag: critical
    #[test]
    fn outside_live_only_a_desktop_session_takes_the_seat() {
        let refused = Err(SeatFault::HostedWhileNotLive);
        for mode in [Mode::Off, Mode::Shadow] {
            assert_eq!(may_take_seat(mode, false), Ok(()));
            assert_eq!(may_take_seat(mode, true), refused);
        }
    }

    #[test]
    fn the_desktop_refusal_says_how_to_roll_back() {
        let text = SeatFault::DesktopWhileLive.to_string();
        assert!(text.contains("/api/overlord/mode"), "{text}");
    }
}
