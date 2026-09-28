//! The LLM API key lives in the macOS Keychain, never in a settings file or
//! the repository (product definition, principle A). The frontend can set or
//! clear it and ask whether one exists, but can never read it back.

const SERVICE: &str = "tw.yuyin.dictation";
const ACCOUNT: &str = "llm-api-key";

#[cfg(target_os = "macos")]
mod imp {
    use super::{ACCOUNT, SERVICE};
    use security_framework::passwords::{
        delete_generic_password, get_generic_password, set_generic_password,
    };

    pub fn get() -> Option<String> {
        let bytes = get_generic_password(SERVICE, ACCOUNT).ok()?;
        String::from_utf8(bytes)
            .ok()
            .filter(|k| !k.trim().is_empty())
    }

    pub fn set(key: &str) -> Result<(), String> {
        set_generic_password(SERVICE, ACCOUNT, key.trim().as_bytes()).map_err(|e| e.to_string())
    }

    pub fn clear() -> Result<(), String> {
        match delete_generic_password(SERVICE, ACCOUNT) {
            Ok(()) => Ok(()),
            // errSecItemNotFound: nothing to delete is fine.
            Err(e) if e.code() == -25300 => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    // M2 (Windows) will use the Credential Manager.
    pub fn get() -> Option<String> {
        None
    }
    pub fn set(_key: &str) -> Result<(), String> {
        Err("API key storage is only implemented on macOS".into())
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
