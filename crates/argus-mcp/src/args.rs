//! Reading the validated arguments of a tool call.

use argus_core::js;
use serde_json::Value;

/// A string argument, none when missing, null or empty (`if (a.x)` in JavaScript).
#[must_use]
pub fn arg_str<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
}

/// A boolean argument, false when missing.
#[must_use]
pub fn flag(args: &Value, key: &str) -> bool {
    args.get(key).is_some_and(|v| v == true)
}

/// A non-negative integer argument.
#[must_use]
pub fn arg_u64(args: &Value, key: &str) -> Option<u64> {
    let v = args.get(key)?;
    v.as_u64()
        .or_else(|| v.as_f64().and_then(|f| js::number(f).as_u64()))
}
