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

    // Feed it like the recorder does: fixed-size frames, speech to
    // on_speech, dropped frames to on_silence.
    chunker::begin(app, true);
    let mut kept = Vec::with_capacity(samples.len());
    for chunk in samples.chunks_exact(frame) {
        match vad.push_frame(chunk)? {
            VadFrame::Speech(buf) => {
                kept.extend_from_slice(buf);
                chunker::on_speech(buf);
            }
            VadFrame::Noise => chunker::on_silence(chunk.len()),
        }
    }

    let tm = app.state::<Arc<TranscriptionManager>>();
    if !kept.is_empty() {
        // Warm up, so a recording with no pieces isn't timed from a cold start.
        let _ = tm.transcribe(kept[..kept.len().min(16_000)].to_vec());
    }

    // Offline, every piece is queued at once; wait for them so the timing
    // below covers only what is left after release, as when speaking live.
    chunker::wait_for_background();
    let started = Instant::now();
    let (chunked, stats) = chunker::transcribe(&tm, kept.clone());
    let after_release_ms = started.elapsed().as_millis();

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
    })
}
