//! Transcribe while the user is still speaking (M1 step 2b).
//!
//! Qwen3-ASR has no streaming mode, so a 25-second instruction used to be
//! transcribed only after the key was released: about 2 s of waiting on top
//! of the clean-up (criterion 2). Instead, each time the VAD reports a real
//! pause after enough speech, the speech so far is transcribed on a
//! background thread. On release only the tail after the last pause is left.
//!
//! - Pieces end in silence, so no word is cut in half: at least 0.75 s (the
//!   VAD's 450 ms hangover plus [`PAUSE_SAMPLES`]), or any gap the VAD drops
//!   once a piece is past [`LONG_PIECE_SAMPLES`].
//! - The recorder still hands back the whole recording on release, for
//!   history and the WAV file. We only use it to find the tail.
//! - Anything unexpected (a piece failed, the lengths disagree, a piece takes
//!   too long) falls back to transcribing the whole recording in one go, as
//!   before. Nothing is ever dropped.

use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

use anyhow::Result;
use log::{debug, info, warn};
use once_cell::sync::Lazy;
use tauri::{AppHandle, Manager};

use crate::managers::transcription::TranscriptionManager;

const SAMPLE_RATE: usize = 16_000;
/// Non-speech after the VAD hangover before a gap counts as a pause.
const PAUSE_SAMPLES: usize = SAMPLE_RATE * 300 / 1000;
/// Shorter pieces lose too much context for the model; wait for more speech.
const MIN_PIECE_SAMPLES: usize = SAMPLE_RATE * 4;
/// Past this, any gap the VAD drops (≥ 450 ms) will do: people who speak
/// without clear pauses would otherwise leave everything to the release.
const LONG_PIECE_SAMPLES: usize = SAMPLE_RATE * 10;
/// A tail this short holds no word (the VAD adds 450 ms of pre-roll to any
/// speech), only the resampler flush at release.
const MIN_TAIL_SAMPLES: usize = SAMPLE_RATE / 5;
/// How long release waits for background pieces before transcribing the
/// whole recording instead.
const WAIT_LIMIT: Duration = Duration::from_secs(20);

#[derive(Default)]
struct State {
    /// Bumped on every recording; background results for an older one are
    /// thrown away.
    generation: u64,
    /// Taking frames: between `begin` and release.
    active: bool,
    /// What the recorder kept so far (speech plus VAD padding).
    samples: Vec<f32>,
    /// `samples[..committed]` has been handed to the background thread.
    committed: usize,
    /// Non-speech samples since the last speech frame.
    silence: usize,
    /// One slot per piece, filled in by the background thread.
    pieces: Vec<Option<std::result::Result<String, String>>>,
}

static STATE: Lazy<Mutex<State>> = Lazy::new(|| Mutex::new(State::default()));
static PIECE_DONE: Condvar = Condvar::new();

struct Job {
    generation: u64,
    index: usize,
    audio: Vec<f32>,
}

static WORKER: OnceLock<Mutex<mpsc::Sender<Job>>> = OnceLock::new();

fn worker(app: &AppHandle) -> mpsc::Sender<Job> {
    WORKER
        .get_or_init(|| {
            let (tx, rx) = mpsc::channel::<Job>();
            let app = app.clone();
            std::thread::Builder::new()
                .name("yuyin-chunker".into())
                .spawn(move || run_worker(app, rx))
                .expect("spawn chunker thread");
            Mutex::new(tx)
        })
        .lock()
        .expect("chunker sender")
        .clone()
}

fn run_worker(app: AppHandle, rx: mpsc::Receiver<Job>) {
    for job in rx {
        if STATE.lock().map(|s| s.generation).unwrap_or(0) != job.generation {
            continue; // cancelled or superseded before we got to it
        }
        let started = Instant::now();
        let seconds = job.audio.len() as f64 / SAMPLE_RATE as f64;
        let tm = app.state::<Arc<TranscriptionManager>>();
        let result = tm.transcribe(job.audio).map_err(|e| e.to_string());
        debug!(
            "chunker: piece {} ({:.1}s of audio) done in {:?}",
            job.index,
            seconds,
            started.elapsed()
        );
        if let Ok(mut s) = STATE.lock() {
            if s.generation == job.generation {
                if let Some(slot) = s.pieces.get_mut(job.index) {
                    *slot = Some(result);
                }
            }
        }
        PIECE_DONE.notify_all();
    }
}

/// Before capture starts. `enabled` is false for models with their own
/// streaming, without VAD (no pauses to find), or when the model is unloaded
/// after every transcription.
pub fn begin(app: &AppHandle, enabled: bool) {
    if enabled {
        let _ = worker(app); // start the thread outside the audio path
    }
    if let Ok(mut s) = STATE.lock() {
        let generation = s.generation + 1;
        *s = State {
            generation,
            active: enabled,
            ..State::default()
        };
    }
}

/// Speech frames, as kept by the recorder (audio thread: keep it cheap).
pub fn on_speech(frame: &[f32]) {
    let Ok(mut s) = STATE.lock() else { return };
    if !s.active {
        return;
    }
    s.silence = 0;
    s.samples.extend_from_slice(frame);
}

/// Frames the VAD dropped as silence (audio thread: keep it cheap).
pub fn on_silence(len: usize) {
    let job = {
        let Ok(mut s) = STATE.lock() else { return };
        if !s.active {
            return;
        }
        let before = s.silence;
        s.silence += len;
        let pending = s.samples.len() - s.committed;
        if pending < MIN_PIECE_SAMPLES {
            return;
        }
        let needed = if pending >= LONG_PIECE_SAMPLES {
            1
        } else {
            PAUSE_SAMPLES
        };
        if !(before < needed && s.silence >= needed) {
            return;
        }
        let (start, end) = (s.committed, s.samples.len());
        s.committed = end;
        s.pieces.push(None);
        Job {
            generation: s.generation,
            index: s.pieces.len() - 1,
            audio: s.samples[start..end].to_vec(),
        }
    };
    debug!(
        "chunker: pause after {:.1}s of speech, transcribing piece {}",
        job.audio.len() as f64 / SAMPLE_RATE as f64,
        job.index
    );
    if let Some(tx) = WORKER.get() {
        if let Ok(tx) = tx.lock() {
            let _ = tx.send(job);
        }
    }
}

/// Stop taking frames and forget this recording (tap, cancel, empty audio).
pub fn abandon() {
    if let Ok(mut s) = STATE.lock() {
        s.generation += 1;
        s.active = false;
        s.samples = Vec::new();
        s.pieces.clear();
    }
    PIECE_DONE.notify_all();
}

/// How the text for one recording was put together, for the timings log.
#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    /// Pieces transcribed while the user was speaking.
    pub pieces: usize,
    /// Audio left to transcribe after release.
    pub tail_samples: usize,
}

/// On release: the text of the whole recording. Waits for the background
/// pieces, transcribes the tail and joins them; otherwise transcribes
/// `samples` in one go.
pub fn transcribe(tm: &TranscriptionManager, samples: Vec<f32>) -> (Result<String>, Stats) {
    let whole = |samples: Vec<f32>| {
        let tail_samples = samples.len();
        (
            tm.transcribe(samples),
            Stats {
                pieces: 0,
                tail_samples,
            },
        )
    };

    let (generation, committed, count) = {
        let Ok(mut s) = STATE.lock() else {
            return whole(samples);
        };
        s.active = false;
        if s.pieces.is_empty() {
            return whole(samples);
        }
        if s.samples.len() != samples.len() {
            warn!(
                "chunker: kept {} samples but the recorder returned {}; transcribing all",
                s.samples.len(),
                samples.len()
            );
            return whole(samples);
        }
        (s.generation, s.committed, s.pieces.len())
    };

    let texts = match wait_for_pieces(generation) {
        Some(texts) => texts,
        None => return whole(samples),
    };

    let tail = &samples[committed..];
    let tail_text = if tail.len() >= MIN_TAIL_SAMPLES {
        match tm.transcribe(tail.to_vec()) {
            Ok(text) => text,
            Err(e) => {
                warn!("chunker: tail failed ({e}); transcribing all");
                return whole(samples);
            }
        }
    } else {
        String::new()
    };

    let stats = Stats {
        pieces: count,
        tail_samples: tail.len(),
    };
    info!(
        "chunker: {} pieces while speaking, {:.1}s tail after release",
        count,
        tail.len() as f64 / SAMPLE_RATE as f64
    );
    let mut parts = texts;
    parts.push(tail_text);
    (Ok(join(&parts)), stats)
}

/// Block until every piece handed to the background thread is done (or
/// [`WAIT_LIMIT`] passes). Only the replay check needs this.
pub fn wait_for_background() {
    let deadline = Instant::now() + WAIT_LIMIT;
    let Ok(mut s) = STATE.lock() else { return };
    while !s.pieces.iter().all(Option::is_some) {
        let now = Instant::now();
        if now >= deadline {
            return;
        }
        match PIECE_DONE.wait_timeout(s, deadline - now) {
            Ok((guard, _)) => s = guard,
            Err(_) => return,
        }
    }
}

/// The text of every piece, in order, once all are done. `None` if one
/// failed, the recording was abandoned, or they took longer than
/// [`WAIT_LIMIT`].
fn wait_for_pieces(generation: u64) -> Option<Vec<String>> {
    let deadline = Instant::now() + WAIT_LIMIT;
    let mut s = STATE.lock().ok()?;
    loop {
        if s.generation != generation {
            return None;
        }
        if s.pieces.iter().all(Option::is_some) {
            let texts: std::result::Result<Vec<String>, String> =
                s.pieces.drain(..).flatten().collect();
            s.samples = Vec::new();
            return match texts {
                Ok(texts) => Some(texts),
                Err(e) => {
                    warn!("chunker: a piece failed ({e}); transcribing all");
                    None
                }
            };
        }
        let now = Instant::now();
        if now >= deadline {
            warn!("chunker: pieces took over {WAIT_LIMIT:?}; transcribing all");
            return None;
        }
        s = PIECE_DONE.wait_timeout(s, deadline - now).ok()?.0;
    }
}

/// Join piece texts. Chinese needs no separator; two Latin words (or a
/// sentence end and a Latin word) need a space.
///
/// The model ends every piece with 。 because the audio ends there, but a
/// pause is often mid-sentence ("其實在我看來都。沒有辦法"). Between pieces
/// it becomes ，; a real sentence end reads fine either way (M0 replay,
/// `tools/chunk_eval.py`).
fn join(parts: &[String]) -> String {
    let mut out = String::new();
    for part in parts.iter().map(|p| p.trim()).filter(|p| !p.is_empty()) {
        if out.ends_with('。') {
            out.pop();
            out.push('，');
        }
        if let (Some(a), Some(b)) = (out.chars().last(), part.chars().next()) {
            let latin_end =
                a.is_ascii_alphanumeric() || matches!(a, '.' | ',' | '!' | '?' | ';' | ':');
            if latin_end && b.is_ascii_alphanumeric() {
                out.push(' ');
            }
        }
        out.push_str(part);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|p| p.to_string()).collect()
    }

    #[test]
    fn joins_chinese_with_commas_at_the_cuts() {
        assert_eq!(
            join(&s(&["我先講第一段。", "然後第二段。"])),
            "我先講第一段，然後第二段。"
        );
        // Questions keep their mark.
        assert_eq!(
            join(&s(&["你有空嗎？", "我們去吃拉麵。"])),
            "你有空嗎？我們去吃拉麵。"
        );
    }

    #[test]
    fn joins_latin_words_with_a_space() {
        assert_eq!(
            join(&s(&["幫我開 GitHub", "repo 的設定"])),
            "幫我開 GitHub repo 的設定"
        );
        assert_eq!(
            join(&s(&["Run the tests.", "Then commit."])),
            "Run the tests. Then commit."
        );
        assert_eq!(join(&s(&["用 React", "寫"])), "用 React寫");
    }

    #[test]
    fn skips_empty_parts() {
        assert_eq!(join(&s(&["第一段。", "  ", ""])), "第一段。");
        assert_eq!(join(&s(&["", "第二段。"])), "第二段。");
        assert_eq!(join(&s(&[])), "");
    }

    /// Feed the hooks the way the recorder does and check where pieces are
    /// cut. Uses the shared state, so everything runs in one test.
    #[test]
    fn cuts_only_at_pauses_after_enough_speech() {
        let speech = vec![0.1f32; SAMPLE_RATE]; // 1 s
        let frame = 480; // Silero: 30 ms
        let pause = |ms: usize| {
            for _ in 0..(SAMPLE_RATE * ms / 1000 / frame) {
                on_silence(frame);
            }
        };
        let reset = || {
            let mut st = STATE.lock().unwrap();
            let generation = st.generation + 1;
            *st = State {
                generation,
                active: true,
                ..State::default()
            };
        };

        // 2 s of speech, then a long pause: too short to cut.
        reset();
        on_speech(&speech);
        on_speech(&speech);
        pause(600);
        assert_eq!(STATE.lock().unwrap().pieces.len(), 0);

        // 3 more seconds (5 s total), then a short gap: still one piece of speech.
        on_speech(&speech);
        on_speech(&speech);
        on_speech(&speech);
        pause(150);
        assert_eq!(STATE.lock().unwrap().pieces.len(), 0);

        // A real pause cuts all 5 s, once.
        pause(600);
        {
            let st = STATE.lock().unwrap();
            assert_eq!(st.pieces.len(), 1);
            assert_eq!(st.committed, 5 * SAMPLE_RATE);
        }
        pause(2000);
        assert_eq!(STATE.lock().unwrap().pieces.len(), 1);

        // Past 10 s, a single dropped frame is enough.
        for _ in 0..11 {
            on_speech(&speech);
        }
        on_silence(frame);
        {
            let st = STATE.lock().unwrap();
            assert_eq!(st.pieces.len(), 2);
            assert_eq!(st.committed, 16 * SAMPLE_RATE);
        }

        // After abandon, frames are ignored.
        abandon();
        on_speech(&speech);
        assert!(STATE.lock().unwrap().samples.is_empty());
    }
}
