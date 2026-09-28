//! Output when pasting isn't safe: the user switched to another window
//! between pressing and releasing the key (acceptance criterion 7). We copy
//! the text and show a short notice instead of pasting into the wrong place.

use std::time::Duration;

use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

use super::session;

const NOTICE_DURATION: Duration = Duration::from_millis(1800);

pub fn copy_instead(app: &AppHandle, text: &str) -> Result<(), String> {
    app.clipboard()
        .write_text(text)
        .map_err(|e| format!("failed to copy to clipboard: {e}"))?;
    crate::overlay::show_copied_overlay(app);

    let generation = session::generation();
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(NOTICE_DURATION);
        // Don't hide the pill of a dictation that started in the meantime.
        if session::generation() == generation {
            crate::utils::hide_recording_overlay(&app);
        }
    });
    Ok(())
}
