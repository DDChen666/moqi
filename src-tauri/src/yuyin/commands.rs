//! Tauri commands for the Yuyin settings panel (`src/yuyin/` in the frontend).

use tauri::AppHandle;

use super::config::{self, YuyinConfig};
use super::{polish, secrets};

#[tauri::command]
#[specta::specta]
pub fn yuyin_get_config(app: AppHandle) -> YuyinConfig {
    config::get(&app)
}

#[tauri::command]
#[specta::specta]
pub fn yuyin_set_config(app: AppHandle, config: YuyinConfig) -> Result<(), String> {
    config::set(&app, config)
}

/// Whether an API key is stored. The key itself is never sent to the frontend.
#[tauri::command]
#[specta::specta]
pub fn yuyin_has_api_key() -> bool {
    secrets::has_api_key()
}

/// Store the key in the Keychain; an empty string removes it.
#[tauri::command]
#[specta::specta]
pub fn yuyin_set_api_key(key: String) -> Result<(), String> {
    secrets::set_api_key(&key)
}

/// The home page's numbers (all computed locally from the timings log).
#[tauri::command]
#[specta::specta]
pub fn yuyin_stats(app: AppHandle) -> super::stats::Stats {
    super::stats::stats(&app)
}

/// Per-entry details for the history page, keyed by the recording's file name.
#[tauri::command]
#[specta::specta]
pub fn yuyin_history_meta(
    app: AppHandle,
) -> std::collections::HashMap<String, super::stats::EntryMeta> {
    super::stats::history_meta(&app)
}

/// A frontend crash, written to the app log (a blank window says nothing).
#[tauri::command]
#[specta::specta]
pub fn yuyin_report_error(message: String) {
    log::error!("frontend: {message}");
}

/// Run the clean-up on sample text with the current settings and report
/// errors (bad key, timeout) instead of falling back.
#[tauri::command]
#[specta::specta]
pub async fn yuyin_test_polish(app: AppHandle, text: String) -> Result<String, String> {
    polish::test(&app, &text).await
}
