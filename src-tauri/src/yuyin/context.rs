//! Which kind of writing the user is doing, decided locally from the
//! frontmost app and its window title. Only the resulting label ever leaves
//! the machine (as a style instruction to the LLM); the app name and title
//! never do.

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum Context {
    /// LINE, Messenger and other chat apps.
    Chat,
    /// Instructions to an AI: Claude Code, ChatGPT, terminals, editors.
    ToAi,
    /// Note taking: Google Keep, Apple Notes, Obsidian.
    Notes,
    Other,
}

/// The app that had focus when the user pressed the shortcut.
#[derive(Clone, Debug, Default)]
pub struct FrontApp {
    pub bundle_id: String,
    pub pid: i32,
    pub window_title: String,
}

const CHAT_APPS: &[&str] = &[
    "jp.naver.line.mac",
    "com.facebook.archon",
    "com.facebook.archon.developerID",
    "com.apple.MobileSMS",
    "ru.keepcoder.Telegram",
    "net.whatsapp.WhatsApp",
    "com.hnc.Discord",
    "com.tinyspeck.slackmacgap",
];

const AI_APPS: &[&str] = &[
    "com.anthropic.claudefordesktop",
    "com.openai.chat",
    // Claude Code runs in terminals and editors.
    "com.apple.Terminal",
    "com.googlecode.iterm2",
    "dev.warp.Warp-Stable",
    "com.mitchellh.ghostty",
    "com.microsoft.VSCode",
    "com.todesktop.230313mzl4w4u92", // Cursor
];

const NOTES_APPS: &[&str] = &["com.apple.Notes", "md.obsidian"];

const BROWSERS: &[&str] = &[
    "com.google.Chrome",
    "com.apple.Safari",
    "company.thebrowser.Browser", // Arc
    "org.mozilla.firefox",
    "com.microsoft.edgemac",
    "com.brave.Browser",
];

/// Web apps are identified by the tab title that browsers put in the window
/// title. Checked in order; the first match wins.
const WEB_TITLES: &[(&str, Context)] = &[
    ("messenger", Context::Chat),
    ("facebook", Context::Chat),
    ("line", Context::Chat),
    ("google keep", Context::Notes),
    ("keep", Context::Notes),
    ("chatgpt", Context::ToAi),
    ("claude", Context::ToAi),
    ("gemini", Context::ToAi),
    ("deepseek", Context::ToAi),
];

pub fn classify(app: &FrontApp) -> Context {
    let id = app.bundle_id.as_str();
    if CHAT_APPS.contains(&id) {
        return Context::Chat;
    }
    if AI_APPS.contains(&id) {
        return Context::ToAi;
    }
    if NOTES_APPS.contains(&id) {
        return Context::Notes;
    }
    if BROWSERS.contains(&id) {
        let title = app.window_title.to_lowercase();
        for (needle, context) in WEB_TITLES {
            if contains_word(&title, needle) {
                return *context;
            }
        }
    }
    Context::Other
}

/// `needle` appears in `haystack` as a whole word, so "line" matches
/// "LINE - Google Chrome" but not "Timeline" or "online".
fn contains_word(haystack: &str, needle: &str) -> bool {
    haystack.match_indices(needle).any(|(start, _)| {
        let before = haystack[..start].chars().next_back();
        let after = haystack[start + needle.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(bundle_id: &str, title: &str) -> FrontApp {
        FrontApp {
            bundle_id: bundle_id.into(),
            pid: 1,
            window_title: title.into(),
        }
    }

    #[test]
    fn native_apps() {
        assert_eq!(classify(&app("jp.naver.line.mac", "")), Context::Chat);
        assert_eq!(classify(&app("com.openai.chat", "")), Context::ToAi);
        assert_eq!(
            classify(&app("com.apple.Terminal", "claude")),
            Context::ToAi
        );
        assert_eq!(classify(&app("com.apple.Notes", "")), Context::Notes);
        assert_eq!(classify(&app("com.apple.finder", "")), Context::Other);
    }

    #[test]
    fn web_apps_by_title() {
        let chrome = "com.google.Chrome";
        assert_eq!(
            classify(&app(chrome, "(2) Messenger - Google Chrome")),
            Context::Chat
        );
        assert_eq!(
            classify(&app(chrome, "Google Keep - Google Chrome")),
            Context::Notes
        );
        assert_eq!(classify(&app(chrome, "ChatGPT")), Context::ToAi);
        assert_eq!(classify(&app("com.apple.Safari", "Claude")), Context::ToAi);
        assert_eq!(classify(&app(chrome, "YouTube")), Context::Other);
    }

    #[test]
    fn titles_only_count_in_browsers() {
        // A document named "Messenger notes" in a text editor is not a chat.
        assert_eq!(
            classify(&app("com.apple.TextEdit", "Messenger notes")),
            Context::Other
        );
    }

    #[test]
    fn whole_words_only() {
        let chrome = "com.google.Chrome";
        assert_eq!(
            classify(&app(chrome, "Timeline - Google Chrome")),
            Context::Other
        );
        assert_eq!(classify(&app(chrome, "Shop online")), Context::Other);
        assert_eq!(classify(&app(chrome, "Keeper Security")), Context::Other);
        assert_eq!(classify(&app(chrome, "LINE")), Context::Chat);
    }
}
