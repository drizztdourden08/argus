//! Replaces the key in any text before it is shown or stored.

/// Knows the key and removes it from text.
#[derive(Clone)]
pub struct Redactor {
    key: String,
}

impl Redactor {
    /// A redactor for `key`.
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_owned(),
        }
    }

    /// `text` with every occurrence of the key replaced by `<redacted>`.
    pub fn apply(&self, text: &str) -> String {
        if self.key.is_empty() {
            return text.to_owned();
        }
        text.replace(&self.key, "<redacted>")
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::Redactor;

    #[test]
    fn every_occurrence_is_replaced() {
        let redactor = Redactor::new("sk-secret");
        assert_eq!(
            redactor.apply("a sk-secret b sk-secret"),
            "a <redacted> b <redacted>"
        );
        assert_eq!(redactor.apply("nothing"), "nothing");
    }
}
