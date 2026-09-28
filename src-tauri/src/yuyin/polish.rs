//! LLM clean-up of a transcript, set up the way the M0 evaluation validated:
//! DeepSeek flash with reasoning off, temperature 0, prompt v3 as the system
//! message and the transcript fenced in the user message.
//!
//! Never loses a draft: any failure returns the raw transcript, and the caller
//! records the outcome.

use std::time::Duration;

use log::{debug, warn};
use once_cell::sync::Lazy;
use serde_json::json;
use tauri::AppHandle;

use super::config::{self, Level, YuyinConfig};
use super::context::Context;
use super::prompt;
use super::secrets;
use super::session::{self, PolishOutcome};

/// One client for the whole app so the TLS connection is reused; creating a
/// client per request (as upstream does) costs a handshake every dictation.
static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(3))
        .pool_idle_timeout(Duration::from_secs(90))
        .user_agent(concat!("Yuyin/", env!("CARGO_PKG_VERSION")))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
});

fn endpoint(cfg: &YuyinConfig, path: &str) -> String {
    format!("{}{path}", cfg.base_url.trim_end_matches('/'))
}

/// Open the connection while the user is still speaking, so the request after
/// release doesn't pay for DNS + TLS. Sends no transcript content.
pub fn warm_up(app: &AppHandle) {
    let cfg = config::get(app);
    if cfg.level == Level::Raw {
        return;
    }
    let Some(key) = secrets::api_key() else {
        return;
    };
    let url = endpoint(&cfg, "/models");
    tauri::async_runtime::spawn(async move {
        let result = CLIENT
            .get(url)
            .bearer_auth(key)
            .timeout(Duration::from_secs(5))
            .send()
            .await;
        if let Err(e) = result {
            debug!("LLM connection warm-up failed: {e}");
        }
    });
}

/// Clean up `transcript` for the current session's context. Returns the text
/// to paste: the polished version, or the transcript itself when the level is
/// Raw or anything goes wrong.
pub async fn polish(app: &AppHandle, transcript: &str) -> String {
    let cfg = config::get(app);
    let context = session::context();
    let (text, outcome) = match run(&cfg, context, transcript).await {
        Ok(Some(text)) => (text, PolishOutcome::Ok),
        Ok(None) => (transcript.to_string(), PolishOutcome::Skipped),
        Err(e) => {
            warn!("Polish failed, pasting the raw transcript: {e}");
            (transcript.to_string(), PolishOutcome::Failed)
        }
    };
    session::mark_polished(cfg.level, outcome, text.chars().count());
    text
}

/// For the settings panel's test button: same request as a real dictation
/// (context Other), but errors are reported instead of silently falling back.
pub async fn test(app: &AppHandle, text: &str) -> Result<String, String> {
    let cfg = config::get(app);
    Ok(run(&cfg, Context::Other, text)
        .await?
        .unwrap_or_else(|| text.to_string()))
}

/// `Ok(None)`: nothing to do (Raw level or blank transcript).
async fn run(
    cfg: &YuyinConfig,
    context: Context,
    transcript: &str,
) -> Result<Option<String>, String> {
    if transcript.trim().is_empty() {
        return Ok(None);
    }
    let Some(system) = prompt::system_prompt(cfg.level, context, &cfg.vocab) else {
        return Ok(None);
    };
    let key = secrets::api_key().ok_or("no API key in the Keychain")?;

    let mut body = json!({
        "model": cfg.model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": prompt::user_message(transcript)},
        ],
        "temperature": 0,
        "stream": false,
    });
    if cfg.base_url.contains("deepseek.com") {
        // DeepSeek V4 reasons by default: up to 50 s on long input (M0).
        body["thinking"] = json!({"type": "disabled"});
    }

    let started = std::time::Instant::now();
    let response = CLIENT
        .post(endpoint(cfg, "/chat/completions"))
        .bearer_auth(key)
        .timeout(Duration::from_millis(cfg.timeout_ms))
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                format!("timed out after {} ms", cfg.timeout_ms)
            } else {
                format!("request failed: {e}")
            }
        })?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("HTTP {status}"));
    }
    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("bad response: {e}"))?;
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("response has no content")?;
    debug!("Polish took {:?}", started.elapsed());

    let text = clean_output(content, context);
    check_output(transcript, &text)?;
    Ok(Some(text))
}

/// Trim whitespace, and never end with a newline: in chat apps a trailing
/// newline sends the message (acceptance criterion 8).
fn clean_output(content: &str, context: Context) -> String {
    let text = content.trim();
    let text = if context == Context::Chat {
        text.trim_end_matches(['。', '\n'])
    } else {
        text
    };
    text.to_string()
}

/// Guard against the LLM answering the transcript instead of cleaning it, or
/// dropping most of it. Short inputs are exempt: removing fillers from a
/// three-word message legitimately halves it.
fn check_output(transcript: &str, output: &str) -> Result<(), String> {
    let n_in = transcript.chars().count();
    let n_out = output.chars().count();
    if n_out == 0 {
        return Err("empty output".into());
    }
    if n_in >= 12 && (n_out * 2 < n_in || n_out * 2 > n_in * 3) {
        return Err(format!("length changed too much ({n_in} → {n_out} chars)"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_output_has_no_trailing_period_or_newline() {
        assert_eq!(
            clean_output("好啊，沒問題。\n", Context::Chat),
            "好啊，沒問題"
        );
        assert_eq!(clean_output("收到。\n", Context::Other), "收到。");
    }

    #[test]
    fn guard_rejects_answers_and_truncation() {
        let q = "請你用 DeepSeek 跟 Claude 各跑一次，比較一下兩個結果";
        assert!(check_output(q, "請你用 DeepSeek 跟 Claude 各跑一次，比較一下兩個結果。").is_ok());
        let answer = q.repeat(3);
        assert!(check_output(q, &answer).is_err());
        assert!(check_output(q, "比較").is_err());
        assert!(check_output(q, "").is_err());
    }

    #[test]
    fn guard_allows_short_inputs_to_shrink() {
        assert!(check_output("呃，好啊好啊", "好啊").is_ok());
    }
}
