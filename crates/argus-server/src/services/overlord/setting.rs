//! Saves fields of `config.overlord` into `config.json`, keeping every other field.

use argus_core::StorageError;
use argus_core::config::save_extra_key;
use serde_json::{Map, Value};

use crate::Services;

const KEY: &str = "overlord";

/// Writes `overlord.<field>`. The in-memory config is rolled back when the file cannot be written.
///
/// # Errors
/// A [`StorageError`] when `config.json` cannot be saved.
pub(crate) fn save_field(svc: &Services, field: &str, value: Value) -> Result<(), StorageError> {
    save_fields(svc, vec![(field.to_owned(), value)])
}

/// Writes every `overlord.<field>` of `fields` in one save. The in-memory config is rolled back when
/// the file cannot be written.
///
/// # Errors
/// A [`StorageError`] when `config.json` cannot be saved.
pub(crate) fn save_fields(
    svc: &Services,
    fields: Vec<(String, Value)>,
) -> Result<(), StorageError> {
    let mut live = svc
        .config
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut setting = match live.extra.get(KEY) {
        Some(Value::Object(saved)) => saved.clone(),
        _ => Map::new(),
    };
    setting.extend(fields);
    save_extra_key(
        &svc.paths,
        &mut live,
        svc.env_port,
        KEY,
        Value::Object(setting),
    )
}
