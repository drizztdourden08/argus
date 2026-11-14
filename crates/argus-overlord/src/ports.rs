//! The ports the hosted Overlord needs from the outside world.

use std::fmt;

/// The Anthropic key. `Debug` prints a redaction; only `expose` gives the text.
#[derive(Clone, PartialEq, Eq)]
pub struct ApiKey(String);

impl ApiKey {
    /// Wraps a key.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// The text, for the one place that puts it in the child env.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiKey(<redacted>)")
    }
}

/// Why the key store failed. Carries a reason, never the key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretFault(pub String);

impl fmt::Display for SecretFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for SecretFault {}

/// Where the one API key is kept. The real one is DPAPI in `argus-native`.
pub trait SecretStore {
    /// Saves the key, replacing any earlier one.
    ///
    /// # Errors
    /// A fault when the key cannot be kept.
    fn save(&self, key: &str) -> Result<(), SecretFault>;

    /// The key, or `None` when none is saved.
    ///
    /// # Errors
    /// A fault when the key cannot be read.
    fn load(&self) -> Result<Option<ApiKey>, SecretFault>;

    /// Removes the key.
    ///
    /// # Errors
    /// A fault when the key cannot be removed.
    fn clear(&self) -> Result<(), SecretFault>;

    /// The last 4 characters of the key, the only view of it besides `load`.
    ///
    /// # Errors
    /// A fault when the key cannot be read.
    fn hint(&self) -> Result<Option<String>, SecretFault>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_api_key_debug_is_redacted() {
        assert_eq!(
            format!("{:?}", ApiKey::new("sk-ant-secret")),
            "ApiKey(<redacted>)"
        );
    }
}
