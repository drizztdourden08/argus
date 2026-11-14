//! The mode of the hosted Overlord and the steps between modes (I-173). Live always comes after a
//! shadow, and leaving live always goes to off, so the owner never jumps straight from the desktop
//! Overlord to the hosted one or from the hosted one back to a shadow. All pure; the server runs the
//! effect of each step.

use std::fmt;

/// Where `config.overlord.mode` stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// No child: the desktop Overlord alone.
    Off,
    /// A read-only child beside the desktop Overlord.
    Shadow,
    /// The hosted child is the Overlord.
    Live,
}

impl Mode {
    /// The mode a typed word names; `None` for any other word.
    #[must_use]
    pub fn parse(text: &str) -> Option<Mode> {
        match text {
            "off" => Some(Mode::Off),
            "shadow" => Some(Mode::Shadow),
            "live" => Some(Mode::Live),
            _ => None,
        }
    }

    /// The mode of the config value: a missing or unknown value is off, as the server reads it.
    #[must_use]
    pub fn of_config(text: Option<&str>) -> Mode {
        text.and_then(Mode::parse).unwrap_or(Mode::Off)
    }

    /// The word saved in the config.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Off => "off",
            Mode::Shadow => "shadow",
            Mode::Live => "live",
        }
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What a legal change of mode does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// The mode is already the one asked: nothing to do.
    Already,
    /// Off to shadow: start the shadow child.
    ShadowOn,
    /// Shadow to off: stop the shadow child.
    ShadowOff,
    /// Shadow to live: the switch-over.
    GoLive,
    /// Live to off: the rollback to the desktop Overlord.
    Rollback,
}

/// Why a change of mode is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeFault {
    /// Off to live or live to shadow.
    Jump { from: Mode, to: Mode },
}

impl fmt::Display for ModeFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ModeFault::Jump { from, to } = self;
        write!(
            f,
            "the Overlord mode cannot go from {from} to {to}: live comes only after shadow, and leaving live goes to off"
        )
    }
}

impl std::error::Error for ModeFault {}

/// The step from one mode to another.
///
/// # Errors
/// [`ModeFault::Jump`] for off to live and for live to shadow.
pub fn step(from: Mode, to: Mode) -> Result<Step, ModeFault> {
    if from == to {
        return Ok(Step::Already);
    }
    match (from, to) {
        (Mode::Off, Mode::Shadow) => Ok(Step::ShadowOn),
        (Mode::Shadow, Mode::Off) => Ok(Step::ShadowOff),
        (Mode::Shadow, Mode::Live) => Ok(Step::GoLive),
        (Mode::Live, Mode::Off) => Ok(Step::Rollback),
        _ => Err(ModeFault::Jump { from, to }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Mode; 3] = [Mode::Off, Mode::Shadow, Mode::Live];

    // test-tag: critical
    #[test]
    fn the_same_mode_is_already_there() {
        for mode in ALL {
            assert_eq!(step(mode, mode), Ok(Step::Already));
        }
    }

    #[test]
    fn the_four_legal_steps_name_their_effect() {
        assert_eq!(step(Mode::Off, Mode::Shadow), Ok(Step::ShadowOn));
        assert_eq!(step(Mode::Shadow, Mode::Off), Ok(Step::ShadowOff));
        assert_eq!(step(Mode::Shadow, Mode::Live), Ok(Step::GoLive));
        assert_eq!(step(Mode::Live, Mode::Off), Ok(Step::Rollback));
    }

    // test-tag: critical
    #[test]
    fn off_to_live_and_live_to_shadow_are_refused() {
        let jump = step(Mode::Off, Mode::Live);
        assert_eq!(
            jump,
            Err(ModeFault::Jump {
                from: Mode::Off,
                to: Mode::Live
            })
        );
        let back = step(Mode::Live, Mode::Shadow);
        assert_eq!(
            back,
            Err(ModeFault::Jump {
                from: Mode::Live,
                to: Mode::Shadow
            })
        );
    }

    #[test]
    fn every_pair_has_one_answer() {
        let refused = ALL
            .iter()
            .flat_map(|from| ALL.iter().map(move |to| step(*from, *to)))
            .filter(Result::is_err)
            .count();
        assert_eq!(refused, 2);
    }

    #[test]
    fn a_fault_names_both_modes() {
        let text = ModeFault::Jump {
            from: Mode::Off,
            to: Mode::Live,
        }
        .to_string();
        assert!(text.contains("from off to live"), "{text}");
    }

    #[test]
    fn the_config_word_parses_and_anything_else_is_off() {
        for mode in ALL {
            assert_eq!(Mode::parse(mode.as_str()), Some(mode));
        }
        assert_eq!(Mode::parse("Live"), None);
        assert_eq!(Mode::of_config(Some("bogus")), Mode::Off);
        assert_eq!(Mode::of_config(None), Mode::Off);
        assert_eq!(Mode::of_config(Some("shadow")), Mode::Shadow);
    }
}
