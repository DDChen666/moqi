//! The LLM API key lives in the system keychain (macOS Keychain, Windows
//! Credential Manager), never in a settings file or the repository (product
//! definition, principle A). The frontend can set or clear it and ask whether
//! one exists, but can never read it back.
//!
//! On macOS the entry is a generic password (service `tw.yuyin.dictation`,
//! account `llm-api-key`), the same item the pre-1.0 builds wrote with
//! security-framework, so an existing key keeps working.

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

pub fn api_key() -> Option<String> {
    imp::get()
}

pub fn has_api_key() -> bool {
    imp::get().is_some()
}

pub fn set_api_key(key: &str) -> Result<(), String> {
    if key.trim().is_empty() {
        return imp::clear();
    }
    imp::set(key)
}
