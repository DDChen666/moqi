//! Auto-gain for the recording capsule's waveform.
//!
//! The visualiser maps a fixed loudness window, calibrated on laptop
//! microphones, to bar heights. A quiet USB microphone records speech about
//! 20 dB lower (the user's: −42 dBFS for its loudest tenth, at 100 % Windows
//! gain), so most of each word fell below the window and the bars barely
//! moved. This lifts the level by up to 20 dB so that the loudest recent
//! sound reaches the top of the window, but never so far that the quietest
//! recent sound (the room) enters it: between words the bars stay still.
//!
//! Only the picture changes; the audio sent to the speech model is untouched.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const MAX_BOOST_DB: f32 = 20.0;
/// A loud word stops counting as "the loudest recent sound" this fast.
const PEAK_FALL_DB_PER_S: f32 = 6.0;
/// The room is the quietest moment in this long: long enough to include
/// the silence before the first word or a pause between sentences.
const FLOOR_WINDOW: Duration = Duration::from_secs(8);
const FLOOR_SLOT: Duration = Duration::from_millis(500);
// Digital silence says nothing about the room: streams often start with a
// few buffers of zeros even where the microphone hisses, and some USB
// microphones gate their pauses to zeros (the user's does) while the moments
// around words still carry the room. So only real sound measures the room.

/// Peak and room level of the last recording, so the next one starts with
/// the right boost instead of learning it again on the first word.
static REMEMBERED: Mutex<Option<(f32, f32)>> = Mutex::new(None);

pub struct LevelBoost {
    target_peak: f32,
    room_limit: f32,
    peak: f32,
    /// Quietest level per half-second slot, newest last.
    slots: VecDeque<(Instant, f32)>,
    last: Option<Instant>,
}

impl LevelBoost {
    /// `target_peak`: where the loudest sound should land (the window's top);
    /// `room_limit`: what the room must stay below (under the window's bottom).
    pub fn new(target_peak: f32, room_limit: f32) -> Self {
        let mut boost = Self {
            target_peak,
            room_limit,
            peak: target_peak,
            slots: VecDeque::new(),
            last: None,
        };
        boost.restore(Instant::now());
        boost
    }

    /// A new recording: start from the last one's levels.
    pub fn restart(&mut self, now: Instant) {
        self.slots.clear();
        self.last = None;
        self.restore(now);
    }

    fn restore(&mut self, now: Instant) {
        let remembered = REMEMBERED.lock().ok().and_then(|r| *r);
        if let Some((peak, room)) = remembered {
            self.peak = peak;
            self.slots.push_back((now, room));
        }
    }

    /// One frame's loudest band in dB (−∞ for digital silence); returns the
    /// boost in dB to add to every band of this frame.
    pub fn update(&mut self, loudest: f32, now: Instant) -> f32 {
        let dt = self
            .last
            .map_or(0.0, |t| now.saturating_duration_since(t).as_secs_f32())
            .min(1.0);
        self.last = Some(now);

        self.peak = loudest.max(self.peak - PEAK_FALL_DB_PER_S * dt);
        if loudest.is_finite() {
            match self.slots.back_mut() {
                Some((started, quietest))
                    if now.saturating_duration_since(*started) < FLOOR_SLOT =>
                {
                    *quietest = quietest.min(loudest);
                }
                _ => self.slots.push_back((now, loudest)),
            }
        }
        while self
            .slots
            .front()
            .is_some_and(|(t, _)| now.saturating_duration_since(*t) > FLOOR_WINDOW)
        {
            self.slots.pop_front();
        }
        let room = self.room();
        if let Ok(mut r) = REMEMBERED.lock() {
            *r = Some((self.peak, room));
        }
        boost_for(self.peak, room, self.target_peak, self.room_limit)
    }

    fn room(&self) -> f32 {
        self.slots
            .iter()
            .map(|(_, q)| *q)
            .fold(f32::INFINITY, f32::min)
    }
}

fn boost_for(peak: f32, room: f32, target_peak: f32, room_limit: f32) -> f32 {
    (target_peak - peak)
        .clamp(0.0, MAX_BOOST_DB)
        .min((room_limit - room).max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOP: f32 = -30.0;
    const ROOM_LIMIT: f32 = -72.0;

    fn fresh() -> LevelBoost {
        LevelBoost {
            target_peak: TOP,
            room_limit: ROOM_LIMIT,
            peak: TOP,
            slots: VecDeque::new(),
            last: None,
        }
    }

    /// Feeds `levels` at 30 frames a second; returns the last boost.
    fn run(b: &mut LevelBoost, start: Instant, levels: &[f32]) -> (f32, Instant) {
        let mut now = start;
        let mut boost = 0.0;
        for &l in levels {
            boost = b.update(l, now);
            now += Duration::from_millis(33);
        }
        (boost, now)
    }

    #[test]
    fn a_loud_microphone_is_left_alone() {
        let mut b = fresh();
        let mut frames = vec![-80.0; 30];
        frames.extend([-28.0, -35.0, -40.0, -30.0].repeat(20));
        let (boost, _) = run(&mut b, Instant::now(), &frames);
        assert_eq!(boost, 0.0);
    }

    #[test]
    fn a_quiet_microphone_in_a_quiet_room_is_lifted() {
        let mut b = fresh();
        let mut frames = vec![-110.0; 30];
        frames.extend([-50.0, -56.0, -62.0, -52.0].repeat(20));
        let (boost, _) = run(&mut b, Instant::now(), &frames);
        assert!((boost - 20.0).abs() < 0.01, "boost {boost}");
    }

    #[test]
    fn the_room_never_rises_into_view() {
        // Quiet speech, but the room is only 10 dB under the window: lifting
        // the speech by 20 dB would light the bars between words.
        let mut b = fresh();
        let mut frames = vec![-82.0; 30];
        frames.extend([-50.0, -56.0].repeat(20));
        let (boost, _) = run(&mut b, Instant::now(), &frames);
        assert!((boost - 10.0).abs() < 0.01, "boost {boost}");
    }

    #[test]
    fn zeros_at_the_start_do_not_hide_a_hiss() {
        // The stream's first buffers are zeros, then the room hisses at -80:
        // only 8 dB of room fits under the window.
        let mut b = fresh();
        let mut frames = vec![f32::NEG_INFINITY; 15];
        frames.extend([-80.0; 30]);
        frames.extend([-52.0, -80.0].repeat(60));
        let (boost, _) = run(&mut b, Instant::now(), &frames);
        assert!((boost - 8.0).abs() < 0.01, "boost {boost}");
    }

    #[test]
    fn a_gated_microphone_is_measured_by_its_quiet_edges() {
        // Zeros in the pauses (the microphone's noise gate), a faint tail
        // as the gate opens and closes, quiet words in between.
        let mut b = fresh();
        let mut word = vec![f32::NEG_INFINITY; 10];
        word.extend([-100.0, -100.0, -52.0, -56.0, -54.0, -100.0]);
        let (boost, _) = run(&mut b, Instant::now(), &word.repeat(10));
        assert!((boost - 20.0).abs() < 0.01, "boost {boost}");
    }

    #[test]
    fn a_noisy_moment_ages_out_of_the_room_level() {
        let mut b = fresh();
        let start = Instant::now();
        // A door slams (loud "room"), then 9 s of quiet room and quiet speech.
        let (_, now) = run(&mut b, start, &[-60.0; 10]);
        let mut frames = vec![-110.0; 30];
        frames.extend([-52.0, -110.0].repeat(135));
        let (boost, _) = run(&mut b, now, &frames);
        assert!((boost - 20.0).abs() < 0.01, "boost {boost}");
    }
}
