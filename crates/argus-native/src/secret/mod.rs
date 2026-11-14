//! The key store: one secret kept encrypted for the current user (Windows DPAPI).
//!
//! The key text leaves `KeyStore` only through `load`, wrapped in `SecretText`, whose `Debug`
//! prints a redaction. `hint` gives the last characters and nothing else.

#[cfg(windows)]
mod dpapi;
#[cfg(not(windows))]
mod dpapi_stub;
mod error;
mod store;
mod text;

pub use error::SecretError;
pub use store::{KEY_FILE, KeyStore};
pub use text::{SecretText, hint_of};
