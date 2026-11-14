//! Windows DPAPI: `CryptProtectData` and `CryptUnprotectData` for the current user.

#![allow(unsafe_code)]

use windows::Win32::Foundation::{HLOCAL, LocalFree};
use windows::Win32::Security::Cryptography::{
    CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData,
};
use windows::core::PCWSTR;

use super::SecretError;

/// A fixed Argus entropy, so another program of the same user does not decrypt the blob by default.
const ENTROPY: &[u8] = b"argus.overlord.anthropic-key.v1";

fn blob(bytes: &[u8]) -> Result<CRYPT_INTEGER_BLOB, SecretError> {
    let len = u32::try_from(bytes.len())
        .map_err(|_| SecretError::Platform("the value is too large".to_owned()))?;
    Ok(CRYPT_INTEGER_BLOB {
        cbData: len,
        pbData: bytes.as_ptr().cast_mut(),
    })
}

/// Copies the output blob and frees the system buffer.
fn take(out: &CRYPT_INTEGER_BLOB) -> Vec<u8> {
    if out.pbData.is_null() {
        return Vec::new();
    }
    // SAFETY: the system filled `pbData` with `cbData` readable bytes and we free it once.
    unsafe {
        let copy = std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec();
        let _ = LocalFree(Some(HLOCAL(out.pbData.cast())));
        copy
    }
}

fn platform(error: &windows::core::Error) -> SecretError {
    SecretError::Platform(format!("windows error {:#x}", error.code().0))
}

/// Encrypts for the current user.
pub fn seal(plain: &[u8]) -> Result<Vec<u8>, SecretError> {
    let input = blob(plain)?;
    let entropy = blob(ENTROPY)?;
    let mut out = CRYPT_INTEGER_BLOB::default();
    // SAFETY: both blobs point at live slices for the whole call; `out` is a valid destination.
    unsafe {
        CryptProtectData(
            &raw const input,
            PCWSTR::null(),
            Some(&raw const entropy),
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &raw mut out,
        )
    }
    .map_err(|error| platform(&error))?;
    Ok(take(&out))
}

/// Decrypts a blob made by `seal` for the same user.
pub fn unseal(sealed: &[u8]) -> Result<Vec<u8>, SecretError> {
    let input = blob(sealed)?;
    let entropy = blob(ENTROPY)?;
    let mut out = CRYPT_INTEGER_BLOB::default();
    // SAFETY: both blobs point at live slices for the whole call; `out` is a valid destination.
    unsafe {
        CryptUnprotectData(
            &raw const input,
            None,
            Some(&raw const entropy),
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &raw mut out,
        )
    }
    .map_err(|error| platform(&error))?;
    Ok(take(&out))
}
