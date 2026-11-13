//! Reading and writing JSON files the way the board store does: two-space indent, a newline.

use std::fs;
use std::path::Path;

use argus_core::fs::write_atomic;
use serde_json::Value;

use crate::error::Result;

/// Reads a JSON file.
pub fn read(path: &Path) -> Result<Value> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

/// Writes a JSON file atomically (temp file, flush, rename).
pub fn write(path: &Path, value: &Value) -> Result<()> {
    let text = format!("{}\n", serde_json::to_string_pretty(value)?);
    write_atomic(path, text)?;
    Ok(())
}
