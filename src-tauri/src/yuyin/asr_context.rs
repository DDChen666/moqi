//! Dictionary words for the recognizer itself (1.1, stage 0b).
//!
//! Qwen3-ASR reads background text in its system turn and leans toward those
//! spellings (a patched engine; see `src-tauri/patches/`). On the user's 30 M0
//! clips, 44 words cut the error rate from 12.0% to 9.0% and fixed brand names
//! at the source (Patreon, Supabase, Gumroad), at about 0.16 s per piece.
//! 200 words made it worse and ~2 s slower, so at most [`MAX_WORDS`] go in.
//! (M0_引擎盲測/results/context_v1/比較.md)
//!
//! `YUYIN_ASR_CONTEXT=0` turns it off; `YUYIN_ASR_CONTEXT=<file>` uses that
//! file's words instead of the dictionary (one per line; for evaluations).

use tauri::AppHandle;

pub const MAX_WORDS: usize = 50;

pub fn context(app: &AppHandle) -> Option<String> {
    match std::env::var("YUYIN_ASR_CONTEXT") {
        Ok(v) if v == "0" => None,
        Ok(path) if !path.is_empty() => {
            let text = std::fs::read_to_string(&path).ok()?;
            join(text.lines().filter(|l| !l.starts_with('#')))
        }
        _ => join(super::config::get(app).vocab.iter().map(String::as_str)),
    }
}

/// The first [`MAX_WORDS`] non-empty words, joined the way the M0 test did.
fn join<'a>(words: impl Iterator<Item = &'a str>) -> Option<String> {
    let words: Vec<&str> = words
        .map(str::trim)
        .filter(|w| !w.is_empty())
        .take(MAX_WORDS)
        .collect();
    (!words.is_empty()).then(|| words.join("、"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_at_most_fifty_words() {
        let many: Vec<String> = (0..80).map(|i| format!("w{i}")).collect();
        let joined = join(many.iter().map(String::as_str)).unwrap();
        assert_eq!(joined.split('、').count(), MAX_WORDS);
        assert_eq!(join(["", "  "].into_iter()), None);
        assert_eq!(
            join([" Claude ", "repo"].into_iter()).as_deref(),
            Some("Claude、repo")
        );
    }
}
