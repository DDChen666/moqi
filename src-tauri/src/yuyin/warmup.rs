//! Speech-model warm-up on Windows.
//!
//! ggml's Vulkan backend builds its GPU pipelines the first time each kernel
//! runs, and nothing keeps them between launches: on an RTX 4070 the first
//! transcription after every start took 16.9 s, the next ones 0.14 s for the
//! same 5 s of audio. Without a warm-up the user's first dictation of the day
//! would miss acceptance criterion 2 by fifteen seconds. So once the model is
//! loaded we transcribe two short synthetic clips in the background; a real
//! dictation during the warm-up simply waits for the engine as it would for
//! a load. Metal on macOS builds its pipelines fast and needs none of this.

use std::f32::consts::TAU;
use std::sync::Arc;
use std::time::Instant;

use log::{info, warn};
use tauri::{AppHandle, Manager};

use crate::managers::model::ModelManager;
use crate::managers::transcription::TranscriptionManager;

/// Load the speech model at launch, which also warms it (see [`start`]).
/// Handy loads it on the first key press instead; with Vulkan that first
/// dictation after every restart would wait for the load and the kernel
/// build. The model stays loaded anyway (Moqi never unloads it), so this
/// only moves the wait to before the user needs it. Skipped until the model
/// has been downloaded (first-run onboarding loads it itself).
pub fn preload_at_launch(app: &AppHandle) {
    let selected = crate::settings::get_settings(app).selected_model;
    let downloaded = app
        .try_state::<Arc<ModelManager>>()
        .map(|models| {
            models
                .get_available_models()
                .iter()
                .any(|m| m.id == selected && m.is_downloaded)
        })
        .unwrap_or(false);
    if !downloaded {
        return;
    }
    if let Some(manager) = app.try_state::<Arc<TranscriptionManager>>() {
        info!("preloading the speech model at launch");
        manager.initiate_model_load();
    }
}

const SAMPLE_RATE: usize = 16_000;

/// Clip lengths in seconds. Different lengths reach different kernel
/// variants (prefill tiles, attention sizes); these two cover a short
/// sentence and a pause-delimited piece from the chunker.
const CLIPS: [f32; 2] = [1.5, 6.0];

/// A quiet voice-like signal: a 140 Hz buzz with a few harmonics, its
/// loudness rising and falling like syllables, plus a little noise. Enough
/// for the encoder and decoder to run their full paths; the text is ignored.
fn clip(seconds: f32) -> Vec<f32> {
    let n = (seconds * SAMPLE_RATE as f32) as usize;
    let mut noise: u32 = 0x1234_5678;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let syllables = 0.5 + 0.5 * (TAU * 4.0 * t).sin();
            let voice: f32 = (1..=4)
                .map(|h| (TAU * 140.0 * h as f32 * t).sin() / h as f32)
                .sum();
            noise ^= noise << 13;
            noise ^= noise >> 17;
            noise ^= noise << 5;
            let hiss = (noise as f32 / u32::MAX as f32 - 0.5) * 0.02;
            0.08 * syllables * voice + hiss
        })
        .collect()
}

/// Run the warm-up in the background; returns at once.
pub fn start(manager: &TranscriptionManager) {
    let manager = manager.clone();
    let spawned = std::thread::Builder::new()
        .name("model-warmup".into())
        .spawn(move || {
            for seconds in CLIPS {
                let started = Instant::now();
                match manager.transcribe(clip(seconds)) {
                    Ok(_) => info!(
                        "model warm-up: {seconds}s clip took {} ms",
                        started.elapsed().as_millis()
                    ),
                    Err(e) => {
                        warn!("model warm-up failed: {e}");
                        return;
                    }
                }
            }
        });
    if let Err(e) = spawned {
        warn!("could not start the model warm-up: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clips_are_quiet_and_the_right_length() {
        let audio = clip(1.5);
        assert_eq!(audio.len(), 24_000);
        let peak = audio.iter().fold(0f32, |m, s| m.max(s.abs()));
        assert!(peak > 0.01 && peak < 0.5, "peak {peak}");
    }
}
