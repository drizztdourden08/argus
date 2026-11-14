//! The key file: the sealed blob at `<home>/secrets/anthropic-key.bin`.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[cfg(windows)]
use super::dpapi::{seal, unseal};
#[cfg(not(windows))]
use super::dpapi_stub::{seal, unseal};
use super::{SecretError, SecretText, hint_of};

/// The file name of the sealed key inside the `secrets` folder.
pub const KEY_FILE: &str = "anthropic-key.bin";

/// The store of the one key. Built over a home folder (the dashboard home, or a temp folder in tests).
#[derive(Debug, Clone)]
pub struct KeyStore {
    path: PathBuf,
}

fn io(error: &std::io::Error) -> SecretError {
    SecretError::Io(error.kind().to_string())
}

impl KeyStore {
    /// A store whose file is `<home>/secrets/anthropic-key.bin`.
    #[must_use]
    pub fn new(home: &Path) -> Self {
        Self {
            path: home.join("secrets").join(KEY_FILE),
        }
    }

    /// The file the sealed key lives in.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Seals and writes the key (trimmed), replacing any earlier one.
    ///
    /// # Errors
    /// `Empty` for a blank key, `Platform` when sealing fails, `Io` when the file cannot be written.
    pub fn save(&self, key: &str) -> Result<(), SecretError> {
        let key = key.trim();
        if key.is_empty() {
            return Err(SecretError::Empty);
        }
        let sealed = seal(key.as_bytes())?;
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir).map_err(|e| io(&e))?;
        }
        fs::write(&self.path, sealed).map_err(|e| io(&e))
    }

    /// The key, or `None` when none is saved. The one place the text comes out.
    ///
    /// # Errors
    /// `Io` when the file cannot be read, `Platform` when it cannot be unsealed.
    pub fn load(&self) -> Result<Option<SecretText>, SecretError> {
        let sealed = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(io(&error)),
        };
        let plain = unseal(&sealed)?;
        let text = String::from_utf8(plain)
            .map_err(|_| SecretError::Platform("the saved key is not text".to_owned()))?;
        Ok(Some(SecretText::new(text)))
    }

    /// Removes the saved key; no key is not an error.
    ///
    /// # Errors
    /// `Io` when the file exists and cannot be removed.
    pub fn clear(&self) -> Result<(), SecretError> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(error) => Err(io(&error)),
        }
    }

    /// Whether a key file exists, without reading it.
    #[must_use]
    pub fn is_set(&self) -> bool {
        self.path.is_file()
    }

    /// The last 4 characters of the saved key, or `None` when none is saved or it is too short.
    ///
    /// # Errors
    /// The same as `load`.
    pub fn hint(&self) -> Result<Option<String>, SecretError> {
        let hint = self.load()?.map(|key| hint_of(key.expose()));
        Ok(hint.filter(|text| !text.is_empty()))
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    const KEY: &str = "sk-ant-api03-SECRETSECRETSECRET-wxyz";

    fn store() -> (tempfile::TempDir, KeyStore) {
        let dir = tempfile::tempdir().expect("temp dir");
        let store = KeyStore::new(dir.path());
        (dir, store)
    }

    // test-tag: critical
    #[test]
    fn a_saved_key_round_trips_and_the_blob_does_not_hold_the_text() {
        let (_dir, store) = store();
        store.save(KEY).expect("save");
        let blob = fs::read(store.path()).expect("blob");
        let leaked = blob.windows(KEY.len()).any(|w| w == KEY.as_bytes());
        assert!(!leaked);
        assert_eq!(store.load().expect("load").expect("some").expose(), KEY);
    }

    #[test]
    fn the_hint_is_the_last_four_characters() {
        let (_dir, store) = store();
        assert_eq!(store.hint().expect("hint"), None);
        store.save(KEY).expect("save");
        assert_eq!(store.hint().expect("hint").as_deref(), Some("wxyz"));
    }

    #[test]
    fn clear_removes_the_file_and_a_second_clear_is_fine() {
        let (_dir, store) = store();
        store.save(KEY).expect("save");
        assert!(store.is_set());
        store.clear().expect("clear");
        assert!(!store.is_set());
        assert_eq!(store.load().expect("load"), None);
        store.clear().expect("clear again");
    }

    #[test]
    fn a_blank_key_is_refused_and_nothing_is_written() {
        let (_dir, store) = store();
        assert_eq!(store.save("   "), Err(SecretError::Empty));
        assert!(!store.is_set());
    }

    #[test]
    fn a_loaded_key_never_shows_in_debug() {
        let (_dir, store) = store();
        store.save(KEY).expect("save");
        let loaded = store.load().expect("load");
        assert!(!format!("{loaded:?}").contains("SECRET"));
    }
}

#[cfg(all(test, not(windows)))]
mod tests {
    use super::*;

    #[test]
    fn without_dpapi_save_answers_an_error() {
        let dir = tempfile::tempdir().expect("temp dir");
        let store = KeyStore::new(dir.path());
        let outcome = store.save("sk-ant-abcdefgh");
        assert!(matches!(outcome, Err(SecretError::Platform(_))));
        assert!(!store.is_set());
    }
}
