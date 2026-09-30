//! The main window's look on Windows. macOS gets its System Settings–style
//! window in `lib.rs`; this is the Windows 11 counterpart: Mica behind a
//! transparent web view. The page (App.css) keeps the content pane opaque and
//! lets the sidebar show Mica. Windows 10 has no Mica and keeps an ordinary
//! opaque window with a tinted sidebar.

use tauri::utils::config::WindowEffectsConfig;
use tauri::window::Effect;
use tauri::{Manager, Runtime, WebviewWindowBuilder};

/// Windows 11 is build 22000 and later; Mica needs it.
const FIRST_MICA_BUILD: u32 = 22000;

fn windows_build() -> Option<u32> {
    let key = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE)
        .open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion")
        .ok()?;
    let build: String = key.get_value("CurrentBuildNumber").ok()?;
    build.trim().parse().ok()
}

fn supports_mica(build: Option<u32>) -> bool {
    build.is_some_and(|b| b >= FIRST_MICA_BUILD)
}

/// Mica for the main window where Windows supports it. The page learns about
/// it from `window.__MOQI_MICA__` (read in main.tsx), set before any script.
pub fn apply<'a, R: Runtime, M: Manager<R>>(
    builder: WebviewWindowBuilder<'a, R, M>,
) -> WebviewWindowBuilder<'a, R, M> {
    if !supports_mica(windows_build()) {
        return builder;
    }
    builder
        .transparent(true)
        .effects(WindowEffectsConfig {
            // Plain Mica follows the window's light/dark theme.
            effects: vec![Effect::Mica],
            state: None,
            radius: None,
            color: None,
        })
        .initialization_script("window.__MOQI_MICA__ = true;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mica_only_from_windows_11() {
        assert!(!supports_mica(Some(19045))); // Windows 10 22H2
        assert!(supports_mica(Some(22000)));
        assert!(supports_mica(Some(26200)));
        assert!(!supports_mica(None));
    }

    #[test]
    fn reads_this_machines_build() {
        assert!(windows_build().is_some_and(|b| b >= 10240));
    }
}
