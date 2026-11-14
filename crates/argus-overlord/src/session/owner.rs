//! The owner's chat line to the hosted Overlord: the text is checked, prefixed so the Overlord can
//! tell the owner from a wake, and given to the child through the supervisor's [`Link`]. Whether
//! it is held while a turn runs is the supervisor's rule, not this module's.

use super::supervisor::Link;

/// What starts every owner message, so a wake is never taken for the owner.
pub const PREFIX: &str = "Owner (Argus Overlord tab): ";

/// The longest text the tab may send, in characters.
pub const MAX_CHARS: usize = 20_000;

/// Why a text was not sent.
#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The text is empty or only blanks.
    Empty,
    /// The text is longer than [`MAX_CHARS`].
    TooLong,
    /// No child is up to take it.
    NotRunning,
}

impl Refusal {
    /// A plain sentence for the owner.
    #[must_use]
    pub fn sentence(&self) -> &'static str {
        match self {
            Self::Empty => "write a message first",
            Self::TooLong => "the message is over 20,000 characters",
            Self::NotRunning => "the hosted Overlord is not running",
        }
    }
}

/// The user message for the child, or why there is none.
///
/// # Errors
/// [`Refusal::Empty`] or [`Refusal::TooLong`].
pub fn compose(text: &str) -> Result<String, Refusal> {
    if text.trim().is_empty() {
        return Err(Refusal::Empty);
    }
    if text.chars().count() > MAX_CHARS {
        return Err(Refusal::TooLong);
    }
    Ok(format!("{PREFIX}{text}"))
}

/// Checks the text and gives it to the child.
///
/// # Errors
/// A [`Refusal`]; `NotRunning` when there is no link or no child is up.
pub fn deliver(link: Option<&Link>, text: &str) -> Result<(), Refusal> {
    let message = compose(text)?;
    match link {
        Some(link) if link.send(message) => Ok(()),
        _ => Err(Refusal::NotRunning),
    }
}

#[cfg(test)]
mod specs {
    use super::*;
    use crate::session::inbox::Inbox;

    #[test]
    fn the_text_gets_the_owner_prefix() {
        assert_eq!(
            compose("hello").unwrap(),
            "Owner (Argus Overlord tab): hello"
        );
    }

    #[test]
    fn empty_and_long_texts_are_refused() {
        assert_eq!(compose("  \n"), Err(Refusal::Empty));
        assert_eq!(compose(&"a".repeat(MAX_CHARS + 1)), Err(Refusal::TooLong));
        assert!(compose(&"a".repeat(MAX_CHARS)).is_ok());
    }

    #[test]
    fn no_link_is_not_running() {
        assert_eq!(deliver(None, "hi"), Err(Refusal::NotRunning));
    }

    #[test]
    fn an_owner_message_given_during_a_turn_is_held_and_joined_after_the_wake() {
        let mut inbox = Inbox::default();
        inbox.push("WAKE inbox".to_owned());
        inbox.push(compose("stop the build").unwrap());
        assert_eq!(
            inbox.take_combined().unwrap(),
            "WAKE inbox\n\nOwner (Argus Overlord tab): stop the build"
        );
    }
}
