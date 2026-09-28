//! Regression check for the chunker: `YUYIN_CHUNKED=1 handy --transcribe-file
//! clip.wav` replays a recording through the app's VAD (same backend and
//! timings as the recorder) and prints one JSON line with the text
//! transcribed in one go and the text put together from pieces, so the two
//! can be compared on the M0 recordings (`tools/chunk_eval.py`).

use std::sync::Arc;
use std::time::Instant;

use anyhow::{anyhow, Result};
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::audio_toolkit::vad::{
    frames_for_duration_ms, SmoothedVad, VadFrame, VAD_OFFLINE_HANGOVER_MS, VAD_ONSET_MS,
    VAD_PREFILL_MS,
};
use crate::audio_toolkit::{SileroVad, VoiceActivityDetector};
use crate::managers::transcription::TranscriptionManager;

use super::chunker;

/// Must match `SILERO_VAD_THRESHOLD` in managers/audio.rs.
const SILERO_THRESHOLD: f32 = 0.3;

#[derive(Serialize)]
pub struct Replay {
    /// Audio the VAD kept, i.e. what the app would have recorded.
    audio_ms: usize,
    whole: String,
    whole_ms: u128,
    chunked: String,
    pieces: usize,
    tail_ms: usize,
    /// Time from "release" to text once the pieces are done: the tail only.
    after_release_ms: u128,
    /// Time from release to text when speaking live: the recording plays
    /// out in real time, each piece starts when it is cut (or when the one
    /// before it is done), and release waits for any piece still running
    /// before the tail.
    live_ms: u128,
    /// Where each piece was cut, in the input.
    cuts_ms: Vec<usize>,
}

pub fn run(app: &AppHandle, samples: &[f32]) -> Result<Replay> {
    let vad_path = app
        .path()
        .resolve(
            "resources/models/silero_vad_v4.onnx",
            tauri::path::BaseDirectory::Resource,
        )
        .map_err(|e| anyhow!("VAD model: {e}"))?;
    let detector = SileroVad::new(vad_path, SILERO_THRESHOLD)?;
    let frame = detector.frame_samples();
    let mut vad = SmoothedVad::new(
        Box::new(detector),
        frames_for_duration_ms(VAD_PREFILL_MS, frame),
        frames_for_duration_ms(VAD_OFFLINE_HANGOVER_MS, frame),
        frames_for_duration_ms(VAD_ONSET_MS, frame),
    );

    // Warm up, so no piece is timed from a cold start.
    let tm = app.state::<Arc<TranscriptionManager>>();
    let _ = tm.transcribe(samples[..samples.len().min(16_000)].to_vec());

    // Feed it like the recorder does: fixed-size frames, kept audio to
    // on_speech, every frame's decision to on_vad.
    chunker::begin(app, true);
    let mut kept = Vec::with_capacity(samples.len());
    let mut cuts_ms = Vec::new();
    for (i, chunk) in samples.chunks_exact(frame).enumerate() {
        let speech = match vad.push_frame(chunk)? {
            VadFrame::Speech(buf) => {
                kept.extend_from_slice(buf);
                chunker::on_speech(buf);
                true
            }
            VadFrame::Noise => false,
        };
        chunker::on_vad(vad.last_frame_voiced().unwrap_or(speech), chunk.len());
        if chunker::piece_count() > cuts_ms.len() {
            cuts_ms.push((i + 1) * frame / 16);
        }
    }

    // Offline, every piece is queued at once; wait for them so the timing
    // below covers only what is left after release, as when speaking live.
    chunker::wait_for_background();
    let piece_times = chunker::piece_times_ms();
    let started = Instant::now();
    let (chunked, stats) = chunker::transcribe(&tm, kept.clone());
    let after_release_ms = started.elapsed().as_millis();

    let mut busy_until = 0u128;
    for (cut, took) in cuts_ms.iter().zip(piece_times) {
        busy_until = busy_until.max(*cut as u128) + took.unwrap_or(0);
    }
    let release = (samples.len() / 16) as u128;
    let live_ms = busy_until.saturating_sub(release) + after_release_ms;

    let started = Instant::now();
    let whole = tm.transcribe(kept.clone())?;
    let whole_ms = started.elapsed().as_millis();

    Ok(Replay {
        audio_ms: kept.len() / 16,
        whole,
        whole_ms,
        chunked: chunked?,
        pieces: stats.pieces,
        tail_ms: stats.tail_samples / 16,
        after_release_ms,
        live_ms,
        cuts_ms,
    })
}
