//! Yuyin's own settings, kept in `yuyin.json` next to Handy's settings store
//! so upstream's busy `settings.rs` stays untouched. The API key is NOT here:
//! it lives in the macOS Keychain (see [`super::secrets`]).

use std::path::PathBuf;
use std::sync::RwLock;

use log::{error, warn};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};

const CONFIG_FILE: &str = "yuyin.json";

/// How much the LLM may change the transcript (product definition, section B).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    /// 原話: punctuation, Traditional Chinese, dictionary. Never calls the LLM.
    Raw,
    /// 整理 (default): drop fillers, keep the last version of a correction,
    /// digits, spoken lists become lists. Never swaps the user's words.
    Tidy,
    /// 潤飾: rewrite into written style.
    Polish,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct YuyinConfig {
    pub level: Level,
    /// Personal dictionary: correct spellings of names and terms the user says.
    pub vocab: Vec<String>,
    /// Any OpenAI-compatible endpoint; DeepSeek by default.
    pub base_url: String,
    pub model: String,
    /// After this, paste the raw transcript instead of waiting.
    pub timeout_ms: u64,
}

impl Default for YuyinConfig {
    fn default() -> Self {
        Self {
            level: Level::Tidy,
            vocab: DEFAULT_VOCAB.iter().map(|s| s.to_string()).collect(),
            base_url: "https://api.deepseek.com".into(),
            // M0: flash with reasoning off is 0.8 s median; v4-pro is slower and worse.
            model: "deepseek-flash".into(),
            timeout_ms: 5_000,
        }
    }
}

/// The owner's everyday tools (same list as the M0 evaluation's vocab.txt).
const DEFAULT_VOCAB: &[&str] = &[
    "Claude",
    "Claude Code",
    "ChatGPT",
    "DeepSeek",
    "Gemini",
    "OpenRouter",
    "Qwen",
    "SenseVoice",
    "Whisper",
    "ElevenLabs",
    "Typeless",
    "Handy",
    "Tauri",
    "React",
    "Rust",
    "Python",
    "Next.js",
    "Supabase",
    "Vercel",
    "Cloudflare",
    "GitHub",
    "repo",
    "API key",
    ".env",
    "config.toml",
    "JSON",
    "CSV",
    "MCP",
    "skill",
    "vibe coder",
    "MVP",
    "landing page",
    "pipeline",
    "TTS",
    "Gumroad",
    "Patreon",
    "Substack",
    "Discord",
    "LINE",
    "Messenger",
    "Keep",
    "Notion",
    "YouTube",
    "Bilibili",
];

static CONFIG: Lazy<RwLock<Option<YuyinConfig>>> = Lazy::new(|| RwLock::new(None));

fn config_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_data_dir()
        .ok()
        .map(|dir| dir.join(CONFIG_FILE))
}

/// Whether this is the first launch of Yuyin (no config written yet).
pub fn is_first_run(app: &AppHandle) -> bool {
    config_path(app).is_some_and(|p| !p.exists())
}

pub fn get(app: &AppHandle) -> YuyinConfig {
    if let Some(cfg) = CONFIG.read().ok().and_then(|c| c.clone()) {
        return cfg;
    }
    let cfg = config_path(app)
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| match serde_json::from_str::<YuyinConfig>(&s) {
            Ok(cfg) => Some(cfg),
            Err(e) => {
                warn!("yuyin.json is invalid, using defaults: {e}");
                None
            }
        })
        .unwrap_or_default();
    if let Ok(mut slot) = CONFIG.write() {
        *slot = Some(cfg.clone());
    }
    cfg
}

pub fn set(app: &AppHandle, cfg: YuyinConfig) -> Result<(), String> {
    let path = config_path(app).ok_or("no app data directory")?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| {
        error!("Failed to write yuyin.json: {e}");
        e.to_string()
    })?;
    if let Ok(mut slot) = CONFIG.write() {
        *slot = Some(cfg);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_json_fills_defaults() {
        let cfg: YuyinConfig = serde_json::from_str(r#"{"level":"raw"}"#).unwrap();
        assert_eq!(cfg.level, Level::Raw);
        assert_eq!(cfg.model, "deepseek-flash");
        assert!(cfg.vocab.contains(&"Supabase".to_string()));
    }

    #[test]
    fn default_level_is_tidy() {
        assert_eq!(YuyinConfig::default().level, Level::Tidy);
    }
}
