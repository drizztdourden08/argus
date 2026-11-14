//! Saves `overlord.budgetUsd` into `config.json`, keeping every other field of `overlord`.

use argus_core::StorageError;
use argus_core::config::save_extra_key;
use serde_json::{Map, Value, json};

use crate::Services;

const KEY: &str = "overlord";

/// Writes the monthly budget. The in-memory config is rolled back when the file cannot be written.
///
/// # Errors
/// A [`StorageError`] when `config.json` cannot be saved.
pub(crate) fn save_budget(svc: &Services, usd: f64) -> Result<(), StorageError> {
    let mut live = svc
        .config
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let before = live.extra.get(KEY).cloned();
    let mut setting = match &before {
        Some(Value::Object(fields)) => fields.clone(),
        _ => Map::new(),
    };
    setting.insert("budgetUsd".to_owned(), json!(usd));
    save_extra_key(
        &svc.paths,
        &mut live,
        svc.env_port,
        KEY,
        Value::Object(setting),
    )
}
