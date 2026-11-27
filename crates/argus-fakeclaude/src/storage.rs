//! The fake app's Local Storage: the same `LevelDB` folder and key encoding Argus reads and
//! writes. The database stays open while the app runs, which is what holds the folder's `LOCK`.

use std::path::Path;

use argus_accounts::sidebar::SidebarKeys;
use argus_accounts::sidebar::store::{
    KEY_NAMES, KEY_PREFIX, bump_timestamp, decode_value, encode_value,
};
use rusty_leveldb::{DB, LdbIterator, Options};

/// The encoding byte of a flag: Latin-1, like the values Argus writes.
const LATIN1: u8 = 1;

/// The store of one app data folder, open.
pub struct Storage {
    db: DB,
}

fn full_key(name: &str) -> Vec<u8> {
    let mut key = KEY_PREFIX.to_vec();
    key.extend_from_slice(name.as_bytes());
    key
}

/// The text of a stored flag: Latin-1 (`01` plus ASCII) or UTF-16LE (`00` plus units).
fn flag_text(raw: &[u8]) -> Option<String> {
    let (&enc, body) = raw.split_first()?;
    let text = if enc == 0 {
        let units: Vec<u16> = body
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16(&units).ok()?
    } else {
        body.iter().map(|&b| char::from(b)).collect()
    };
    Some(text)
}

impl Storage {
    /// Opens the store under `<app>\Local Storage\leveldb`.
    pub fn open(app: &Path) -> Result<Self, String> {
        let dir = app.join("Local Storage").join("leveldb");
        let options = Options {
            create_if_missing: false,
            ..Options::default()
        };
        let db = DB::open(&dir, options)
            .map_err(|e| format!("cannot open the store {}: {e}", dir.display()))?;
        Ok(Storage { db })
    }

    /// The three sidebar keys, decoded.
    pub fn read_keys(&mut self) -> Result<SidebarKeys, String> {
        let mut parts = Vec::with_capacity(KEY_NAMES.len());
        for name in KEY_NAMES {
            let raw = self
                .db
                .get(&full_key(name))
                .ok_or_else(|| format!("the store has no `{name}` key"))?;
            parts.push(decode_value(&raw).map_err(|e| format!("{name}: {e}"))?);
        }
        let (e2, dframe_store) = parts.remove(2);
        let (e1, group_scopes) = parts.remove(1);
        let (e0, code_sections) = parts.remove(0);
        Ok(SidebarKeys {
            code_sections,
            group_scopes,
            dframe_store,
            encodings: [e0, e1, e2],
        })
    }

    /// Writes the three sidebar keys, their wrapper timestamps set to `now_ms`.
    pub fn write_keys(&mut self, keys: &SidebarKeys, now_ms: i64) -> Result<(), String> {
        let mut stamped = keys.clone();
        bump_timestamp(&mut stamped.code_sections, now_ms);
        bump_timestamp(&mut stamped.group_scopes, now_ms);
        let values = [
            &stamped.code_sections,
            &stamped.group_scopes,
            &stamped.dframe_store,
        ];
        for ((name, value), enc) in KEY_NAMES.iter().zip(values).zip(stamped.encodings) {
            let raw = encode_value(enc, value)?;
            self.put(name, &raw)?;
        }
        self.db
            .flush()
            .map_err(|e| format!("cannot flush the store: {e}"))
    }

    /// The text of the flag `name`, if the store has it.
    pub fn flag(&mut self, name: &str) -> Option<String> {
        self.db.get(&full_key(name)).and_then(|raw| flag_text(&raw))
    }

    /// Sets the flag `name` to `text`.
    pub fn set_flag(&mut self, name: &str, text: &str) -> Result<(), String> {
        let mut raw = vec![LATIN1];
        raw.extend_from_slice(text.as_bytes());
        self.put(name, &raw)
    }

    /// Removes every key whose name starts with `prefix`.
    pub fn drop_flags(&mut self, prefix: &str) -> Result<(), String> {
        for key in self.keys_starting(&full_key(prefix))? {
            self.db
                .delete(&key)
                .map_err(|e| format!("cannot remove a flag: {e}"))?;
        }
        Ok(())
    }

    fn keys_starting(&mut self, wanted: &[u8]) -> Result<Vec<Vec<u8>>, String> {
        let mut found = Vec::new();
        let mut iter = self
            .db
            .new_iter()
            .map_err(|e| format!("cannot walk the store: {e}"))?;
        while let Some((key, _)) = iter.next() {
            if key.starts_with(wanted) {
                found.push(key);
            }
        }
        Ok(found)
    }

    /// Closes the store and so lets go of its lock.
    pub fn close(mut self) -> Result<(), String> {
        self.db
            .close()
            .map_err(|e| format!("cannot close the store: {e}"))
    }

    fn put(&mut self, name: &str, raw: &[u8]) -> Result<(), String> {
        self.db
            .put(&full_key(name), raw)
            .map_err(|e| format!("cannot write `{name}`: {e}"))
    }
}
