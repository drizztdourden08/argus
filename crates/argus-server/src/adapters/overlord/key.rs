//! The Anthropic key store of the hosted Overlord: `argus-native`'s DPAPI file behind the
//! `SecretStore` port. A fault carries a fixed reason, never the key.

use std::path::Path;
use std::time::SystemTime;

use argus_native::{KeyStore, SecretError};
use argus_overlord::ports::{ApiKey, SecretFault, SecretStore};

/// The DPAPI key file, as the `SecretStore` port.
pub struct DpapiKeyStore {
    store: KeyStore,
}

fn fault(e: &SecretError) -> SecretFault {
    SecretFault(
        match e {
            SecretError::Empty => "the key is empty",
            SecretError::Io(_) => "the key file could not be used",
            SecretError::Platform(_) => "the key could not be protected",
        }
        .to_owned(),
    )
}

impl DpapiKeyStore {
    /// The store under the dashboard home.
    #[must_use]
    pub fn new(home: &Path) -> Self {
        Self {
            store: KeyStore::new(home),
        }
    }

    /// When the key file was last written, or `None` when there is none.
    #[must_use]
    pub fn saved_at(&self) -> Option<SystemTime> {
        std::fs::metadata(self.store.path())
            .and_then(|m| m.modified())
            .ok()
    }
}

impl SecretStore for DpapiKeyStore {
    fn save(&self, key: &str) -> Result<(), SecretFault> {
        self.store.save(key).map_err(|e| fault(&e))
    }

    fn load(&self) -> Result<Option<ApiKey>, SecretFault> {
        let loaded = self.store.load().map_err(|e| fault(&e))?;
        Ok(loaded.map(|text| ApiKey::new(text.expose())))
    }

    fn clear(&self) -> Result<(), SecretFault> {
        self.store.clear().map_err(|e| fault(&e))
    }

    fn hint(&self) -> Result<Option<String>, SecretFault> {
        self.store.hint().map_err(|e| fault(&e))
    }
}
