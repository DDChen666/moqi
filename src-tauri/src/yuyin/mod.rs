//! Yuyin fork: the product layer we add on top of Handy.
//!
//! Everything in this module is new code. Upstream Handy files only get small
//! hooks marked `Yuyin fork:` so monthly cherry-picks from upstream stay easy.
//!
//! Flow of one dictation:
//! 1. key press   → [`session::begin`] records the frontmost app, its focused
//!    window and the writing context (chat / to-AI / notes / other).
//! 2. release     → Handy records and transcribes as usual.
//! 3. transcript  → [`polish::polish`] tidies it with the M0-validated prompt,
//!    unless the level is Raw. Any failure falls back to the raw transcript.
//! 4. before paste → [`session::focus_changed`] decides whether pasting is
//!    still safe; if the user switched windows we copy instead.

pub mod commands;
pub mod config;
pub mod context;
pub mod defaults;
pub mod output;
pub mod polish;
pub mod prompt;
pub mod secrets;
pub mod session;
