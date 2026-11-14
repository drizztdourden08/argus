//! Targets without DPAPI: every call answers an error.

use super::SecretError;

fn unsupported() -> SecretError {
    SecretError::Platform("this platform has no per-user key protection".to_owned())
}

pub fn seal(_plain: &[u8]) -> Result<Vec<u8>, SecretError> {
    Err(unsupported())
}

pub fn unseal(_blob: &[u8]) -> Result<Vec<u8>, SecretError> {
    Err(unsupported())
}
