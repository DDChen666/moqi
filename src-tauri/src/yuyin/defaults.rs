//! Yuyin's defaults for Handy's own settings, applied once on first launch
//! (before shortcuts are registered). Changing upstream's defaults in
//! `settings.rs` would conflict with every upstream settings change, so we
//! write the user's store instead.

use log::info;
use tauri::AppHandle;

use super::config;
use crate::settings::{self, ModelUnloadTimeout, ShortcutActivation};

/// Hold right Option to talk (product definition: Fn or right Option on Mac;
/// Fn only works reliably on Apple keyboards).
const TALK_KEY: &str = "option_right";

pub fn apply_first_run(app: &AppHandle) {
    if !config::is_first_run(app) {
        return;
    }
    let mut s = settings::get_settings(app);
    // Traditional Chinese output; also sends `zh` to Qwen3-ASR, which would
    // otherwise translate English-heavy sentences into English (M0).
    s.selected_language = "zh-Hant".to_string();
    // Hold to talk, release to paste. Handy's default HoldOrToggle turns a
    // short tap into "keep recording", the opposite of criterion 3.
    s.shortcut_activation = ShortcutActivation::PushToTalk;
    if let Some(binding) = s.bindings.get_mut("transcribe") {
        binding.default_binding = TALK_KEY.to_string();
        binding.current_binding = TALK_KEY.to_string();
    }
    // Criterion 1: a start cue so the user waits for it before speaking.
    s.audio_feedback = true;
    // Criterion 2: reloading the 1.7B model on a press adds seconds.
    s.model_unload_timeout = ModelUnloadTimeout::Never;
    settings::write_settings(app, s);

    // Writing yuyin.json marks the first run as done.
    if let Err(e) = config::set(app, config::get(app)) {
        log::error!("Failed to write initial yuyin.json: {e}");
    }
    info!("Applied Yuyin first-run defaults");
}
