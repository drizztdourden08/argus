//! Saves `overlord.budgetUsd` into `config.json`, keeping every other field of `overlord`.

use argus_core::StorageError;
use serde_json::json;

use super::setting::save_field;
use crate::Services;

/// Writes the monthly budget; `None` clears it back to no cap. The in-memory config is rolled back when the file cannot be written.
///
/// # Errors
/// A [`StorageError`] when `config.json` cannot be saved.
pub(crate) fn save_budget(svc: &Services, usd: Option<f64>) -> Result<(), StorageError> {
    save_field(svc, "budgetUsd", json!(usd))
}
