//! Parses the registry file once and checks it.

use std::sync::LazyLock;

use super::model::Registry;
use super::verify;

/// The registry file, compiled into the binary.
pub(crate) const SOURCE: &str = include_str!("../../registry/item-types.json");

/// Why a registry text was not accepted.
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    /// The text is not JSON, or does not have the shape of the registry.
    #[error("the item type registry does not parse: {0}")]
    Parse(#[from] serde_json::Error),
    /// The shape is right but a name is used that is not defined.
    #[error("the item type registry is inconsistent: {}", .0.join("; "))]
    Inconsistent(Vec<String>),
}

/// Parses `text` as a registry and checks its cross references.
///
/// # Errors
/// [`RegistryError`] when the text does not parse or a name it uses is not defined.
pub fn parse(text: &str) -> Result<Registry, RegistryError> {
    let reg: Registry = serde_json::from_str(text)?;
    let problems = verify::problems(&reg);
    if problems.is_empty() {
        Ok(reg)
    } else {
        Err(RegistryError::Inconsistent(problems))
    }
}

static REGISTRY: LazyLock<Registry> =
    LazyLock::new(|| parse(SOURCE).expect("the embedded item type registry is valid"));

/// The registry of this build: parsed and checked on first use, then shared.
///
/// # Panics
/// Never in a build that passed its tests: the embedded file is checked by a critical test.
#[must_use]
pub fn registry() -> &'static Registry {
    &REGISTRY
}
