//! The billing guard: a child that did not start on the API key must not run, because it
//! would bill the owner's subscription.

use std::fmt;

use crate::protocol::Init;

/// The only key source the hosted Overlord may run on.
const EXPECTED: &str = "ANTHROPIC_API_KEY";

/// The child started on something else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardFault {
    /// The `apiKeySource` the init event named, or `None` when it had none.
    pub seen: Option<String>,
}

impl fmt::Display for GuardFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.seen {
            Some(source) => write!(f, "billing guard: apiKeySource is {source}, not {EXPECTED}"),
            None => f.write_str("billing guard: the init event has no apiKeySource"),
        }
    }
}

impl std::error::Error for GuardFault {}

/// Ok only when the init names exactly `ANTHROPIC_API_KEY`.
///
/// # Errors
/// A [`GuardFault`] naming what the init said.
pub fn check_init(init: &Init) -> Result<(), GuardFault> {
    match init.api_key_source.as_deref() {
        Some(EXPECTED) => Ok(()),
        other => Err(GuardFault {
            seen: other.map(str::to_owned),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init(source: Option<&str>) -> Init {
        Init {
            api_key_source: source.map(str::to_owned),
            ..Init::default()
        }
    }

    #[test]
    fn the_api_key_passes() {
        assert_eq!(check_init(&init(Some("ANTHROPIC_API_KEY"))), Ok(()));
    }

    #[test]
    fn none_and_oauth_and_missing_are_refused() {
        for seen in [
            Some("none"),
            Some("oauth"),
            Some("claude.ai"),
            Some("anthropic_api_key"),
            None,
        ] {
            let fault = check_init(&init(seen)).unwrap_err();
            assert_eq!(fault.seen.as_deref(), seen);
        }
    }

    #[test]
    fn the_fault_names_what_it_saw() {
        let fault = check_init(&init(Some("none"))).unwrap_err();
        assert!(fault.to_string().contains("none"));
    }
}
