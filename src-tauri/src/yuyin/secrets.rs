//! The LLM API key lives in the system keychain (macOS Keychain, Windows
//! Credential Manager), never in a settings file or the repository (product
//! definition, principle A). The frontend can set or clear it and ask whether
//! one exists, but can never read it back.
//!
//! On macOS the entry is a generic password (service `tw.yuyin.dictation`,
//! account `llm-api-key`), the same item the pre-1.0 builds wrote with
//! security-framework, so an existing key keeps working.

use std::sync::Mutex;

const SERVICE: &str = "tw.yuyin.dictation";
const ACCOUNT: &str = "llm-api-key";

#[cfg(any(target_os = "macos", windows))]
mod imp {
    use super::{ACCOUNT, SERVICE};
    use keyring::{Entry, Error};

    fn entry() -> Result<Entry, String> {
        Entry::new(SERVICE, ACCOUNT).map_err(|e| e.to_string())
    }

    pub fn get() -> Option<String> {
        entry()
            .ok()?
            .get_password()
            .ok()
            .filter(|k| !k.trim().is_empty())
    }

    pub fn set(key: &str) -> Result<(), String> {
        entry()?.set_password(key.trim()).map_err(|e| e.to_string())
    }

    pub fn clear() -> Result<(), String> {
        match entry()?.delete_credential() {
            Ok(()) | Err(Error::NoEntry) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[cfg(not(any(target_os = "macos", windows)))]
mod imp {
    pub fn get() -> Option<String> {
        None
    }
    pub fn set(_key: &str) -> Result<(), String> {
        Err("API key storage is not implemented on this platform".into())
    }
    pub fn clear() -> Result<(), String> {
        Ok(())
    }
}

/// The key as last read from or written to the keychain; `None` until the
/// first read. Without a Team ID, macOS asks for the login password once per
/// new build before handing the key over. Reading it at launch (`warm`) puts
/// that prompt there: read on a key press, the password field's secure input
/// swallowed the talk key's release and the recording never stopped.
static CACHE: Mutex<Option<Option<String>>> = Mutex::new(None);

/// Read the key once in the background so any keychain prompt shows at launch.
pub fn warm() {
    std::thread::spawn(|| {
        let _ = api_key();
    });
}

pub fn api_key() -> Option<String> {
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    cache.get_or_insert_with(imp::get).clone()
}

pub fn has_api_key() -> bool {
    api_key().is_some()
}

pub fn set_api_key(key: &str) -> Result<(), String> {
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let key = key.trim();
    if key.is_empty() {
        imp::clear()?;
        *cache = Some(None);
    } else {
        imp::set(key)?;
        *cache = Some(Some(key.to_string()));
    }
    Ok(())
}
