//! State of the current dictation, from key press to paste.
//!
//! - On press we remember the frontmost app, its focused window and the
//!   writing context, so the context is judged where the user started typing.
//! - Before pasting we check that the same window still has focus
//!   (acceptance criterion 7).
//! - Each stage is timed and appended to `yuyin_timings.jsonl` in the app data
//!   directory: numbers and the context label only, never any text. This is
//!   the evidence for acceptance criteria 1 and 2.

use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use log::{debug, warn};
use once_cell::sync::Lazy;
use serde::Serialize;
use tauri::{AppHandle, Manager};

use super::config::Level;
use super::context::{classify, Context, FrontApp};

const TIMINGS_FILE: &str = "yuyin_timings.jsonl";

/// Presses shorter than this are accidental taps and produce no text
/// (acceptance criterion 3).
const MIN_HOLD: Duration = Duration::from_millis(300);

/// What happened to the LLM clean-up step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolishOutcome {
    /// Raw level, or nothing to clean up.
    Skipped,
    Ok,
    /// Timed out, network or API error, missing key, or the output failed the
    /// sanity check. The raw transcript was used instead.
    Failed,
}

struct Session {
    pressed: Instant,
    front: Option<FrontApp>,
    context: Context,
    window: Option<platform::Window>,
    mic_ready: Option<Instant>,
    released: Option<Instant>,
    transcribed: Option<Instant>,
    polished: Option<Instant>,
    polish: PolishOutcome,
    level: Option<Level>,
    chars_in: usize,
    chars_out: usize,
    focus_changed: bool,
}

static CURRENT: Lazy<Mutex<Option<Session>>> = Lazy::new(|| Mutex::new(None));

/// Bumped on every key press, so delayed UI work (like hiding the "copied"
/// notice) can tell whether a new dictation has started since.
static GENERATION: AtomicU64 = AtomicU64::new(0);

pub fn generation() -> u64 {
    GENERATION.load(Ordering::SeqCst)
}

fn with_session(f: impl FnOnce(&mut Session)) {
    if let Ok(mut guard) = CURRENT.lock() {
        if let Some(session) = guard.as_mut() {
            f(session);
        }
    }
}

/// Key press (at `pressed`): remember where the user is typing.
pub fn begin(pressed: Instant) {
    GENERATION.fetch_add(1, Ordering::SeqCst);
    let (front, window) = match platform::frontmost() {
        Some((app, window)) => (Some(app), window),
        None => (None, None),
    };
    let context = front.as_ref().map(classify).unwrap_or(Context::Other);
    debug!(
        "yuyin session: context={:?} app={:?} window_known={}",
        context,
        front.as_ref().map(|a| a.bundle_id.as_str()),
        window.is_some()
    );
    if let Ok(mut guard) = CURRENT.lock() {
        *guard = Some(Session {
            pressed,
            front,
            context,
            window,
            mic_ready: None,
            released: None,
            transcribed: None,
            polished: None,
            polish: PolishOutcome::Skipped,
            level: None,
            chars_in: 0,
            chars_out: 0,
            focus_changed: false,
        });
    }
}

pub fn context() -> Context {
    CURRENT
        .lock()
        .ok()
        .and_then(|g| g.as_ref().map(|s| s.context))
        .unwrap_or(Context::Other)
}

pub fn mark_mic_ready() {
    with_session(|s| s.mic_ready = s.mic_ready.or(Some(Instant::now())));
}

pub fn mark_released() {
    with_session(|s| s.released = Some(Instant::now()));
}

/// Whether the key was released within [`MIN_HOLD`] of being pressed.
pub fn was_tap() -> bool {
    CURRENT
        .lock()
        .ok()
        .and_then(|g| g.as_ref().and_then(|s| s.released.map(|r| r - s.pressed)))
        .is_some_and(|held| held < MIN_HOLD)
}

pub fn mark_transcribed(chars: usize) {
    with_session(|s| {
        s.transcribed = Some(Instant::now());
        s.chars_in = chars;
    });
}

pub fn mark_polished(level: Level, outcome: PolishOutcome, chars_out: usize) {
    with_session(|s| {
        s.polished = Some(Instant::now());
        s.level = Some(level);
        s.polish = outcome;
        s.chars_out = chars_out;
    });
}

/// Before paste: has the user moved to another app or window since pressing
/// the key? When we can't tell (no Accessibility, no session), assume not, so
/// a missing permission never blocks pasting.
pub fn focus_changed() -> bool {
    let Ok(mut guard) = CURRENT.lock() else {
        return false;
    };
    let Some(session) = guard.as_mut() else {
        return false;
    };
    let Some(before) = session.front.as_ref() else {
        return false;
    };
    let changed = match platform::frontmost() {
        None => false,
        Some((now, _)) if now.pid != before.pid => true,
        Some((_, window_now)) => match (&session.window, window_now) {
            (Some(a), Some(b)) => !a.same_as(&b),
            _ => false,
        },
    };
    session.focus_changed = changed;
    changed
}

#[derive(Serialize)]
struct TimingRecord {
    /// Unix time in milliseconds.
    at: u128,
    context: Context,
    level: Option<Level>,
    polish: PolishOutcome,
    /// Criterion 1: key press → first microphone samples.
    press_to_mic_ms: Option<u128>,
    /// How long the user held the key (≈ audio length).
    press_to_release_ms: Option<u128>,
    release_to_transcribed_ms: Option<u128>,
    transcribed_to_polished_ms: Option<u128>,
    /// Criterion 2: key release → text pasted (or copied).
    release_to_output_ms: Option<u128>,
    focus_changed: bool,
    chars_in: usize,
    chars_out: usize,
}

fn between(a: Option<Instant>, b: Option<Instant>) -> Option<u128> {
    match (a, b) {
        (Some(a), Some(b)) if b >= a => Some((b - a).as_millis()),
        _ => None,
    }
}

/// Text was pasted (or copied): write the timing record and clear the session.
pub fn finish(app: &AppHandle) {
    let output = Instant::now();
    let Some(s) = CURRENT.lock().ok().and_then(|mut g| g.take()) else {
        return;
    };
    let record = TimingRecord {
        at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
        context: s.context,
        level: s.level,
        polish: s.polish,
        press_to_mic_ms: between(Some(s.pressed), s.mic_ready),
        press_to_release_ms: between(Some(s.pressed), s.released),
        release_to_transcribed_ms: between(s.released, s.transcribed),
        transcribed_to_polished_ms: between(s.transcribed, s.polished),
        release_to_output_ms: between(s.released, Some(output)),
        focus_changed: s.focus_changed,
        chars_in: s.chars_in,
        chars_out: s.chars_out,
    };
    let Ok(dir) = app.path().app_data_dir() else {
        return;
    };
    let line = match serde_json::to_string(&record) {
        Ok(line) => line,
        Err(_) => return,
    };
    let result = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(TIMINGS_FILE))
        .and_then(|mut f| writeln!(f, "{line}"));
    if let Err(e) = result {
        warn!("Failed to write yuyin timing record: {e}");
    }
}

#[cfg(target_os = "macos")]
mod platform {
    //! Frontmost app via NSWorkspace; focused window and its title via the
    //! Accessibility API (the same permission paste already needs).

    use std::ffi::c_void;

    use objc2_app_kit::NSWorkspace;

    use super::FrontApp;

    type CFTypeRef = *const c_void;
    const UTF8: u32 = 0x0800_0100;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXUIElementCreateApplication(pid: i32) -> CFTypeRef;
        fn AXUIElementCopyAttributeValue(
            element: CFTypeRef,
            attribute: CFTypeRef,
            value: *mut CFTypeRef,
        ) -> i32;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(cf: CFTypeRef);
        fn CFEqual(a: CFTypeRef, b: CFTypeRef) -> u8;
        fn CFGetTypeID(cf: CFTypeRef) -> usize;
        fn CFStringGetTypeID() -> usize;
        fn CFStringCreateWithBytes(
            alloc: CFTypeRef,
            bytes: *const u8,
            len: isize,
            encoding: u32,
            external: u8,
        ) -> CFTypeRef;
        fn CFStringGetLength(s: CFTypeRef) -> isize;
        fn CFStringGetMaximumSizeForEncoding(len: isize, encoding: u32) -> isize;
        fn CFStringGetCString(s: CFTypeRef, buf: *mut u8, size: isize, encoding: u32) -> u8;
    }

    /// An owned Core Foundation reference, released on drop.
    struct Owned(CFTypeRef);

    impl Drop for Owned {
        fn drop(&mut self) {
            // SAFETY: we only wrap references we own (Create/Copy rule).
            unsafe { CFRelease(self.0) }
        }
    }

    // SAFETY: AXUIElement and CFString are immutable CF objects; retaining,
    // comparing and releasing them from any thread is allowed.
    unsafe impl Send for Owned {}

    fn cf_string(s: &str) -> Option<Owned> {
        // SAFETY: bytes/len describe a valid UTF-8 buffer for the call's duration.
        let r = unsafe {
            CFStringCreateWithBytes(std::ptr::null(), s.as_ptr(), s.len() as isize, UTF8, 0)
        };
        (!r.is_null()).then_some(Owned(r))
    }

    fn to_string(cf: &Owned) -> Option<String> {
        // SAFETY: cf is a live CF object; we check it is a CFString first.
        unsafe {
            if CFGetTypeID(cf.0) != CFStringGetTypeID() {
                return None;
            }
            let len = CFStringGetLength(cf.0);
            let size = CFStringGetMaximumSizeForEncoding(len, UTF8) + 1;
            let mut buf = vec![0u8; size.max(1) as usize];
            if CFStringGetCString(cf.0, buf.as_mut_ptr(), size, UTF8) == 0 {
                return None;
            }
            let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            String::from_utf8(buf[..end].to_vec()).ok()
        }
    }

    fn copy_attribute(element: &Owned, name: &str) -> Option<Owned> {
        let attribute = cf_string(name)?;
        let mut value: CFTypeRef = std::ptr::null();
        // SAFETY: element and attribute are live; value receives a +1 reference.
        let err = unsafe { AXUIElementCopyAttributeValue(element.0, attribute.0, &mut value) };
        (err == 0 && !value.is_null()).then_some(Owned(value))
    }

    /// The focused window of an app, kept alive so it can be compared later.
    pub struct Window(Owned);

    impl Window {
        pub fn same_as(&self, other: &Window) -> bool {
            // SAFETY: both are live AXUIElements.
            unsafe { CFEqual(self.0 .0, other.0 .0) != 0 }
        }
    }

    fn focused_window(pid: i32) -> Option<Window> {
        // SAFETY: AXUIElementCreateApplication returns a +1 reference.
        let app = unsafe { AXUIElementCreateApplication(pid) };
        if app.is_null() {
            return None;
        }
        let app = Owned(app);
        copy_attribute(&app, "AXFocusedWindow").map(Window)
    }

    fn window_title(window: &Window) -> String {
        copy_attribute(&window.0, "AXTitle")
            .and_then(|t| to_string(&t))
            .unwrap_or_default()
    }

    /// The frontmost app and its focused window (None without Accessibility).
    pub fn frontmost() -> Option<(FrontApp, Option<Window>)> {
        let workspace = NSWorkspace::sharedWorkspace();
        let app = workspace.frontmostApplication()?;
        let pid = app.processIdentifier();
        let bundle_id = app
            .bundleIdentifier()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let window = focused_window(pid);
        let window_title = window.as_ref().map(window_title).unwrap_or_default();
        Some((
            FrontApp {
                bundle_id,
                pid,
                window_title,
            },
            window,
        ))
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    // M2 (Windows) will read the foreground window title instead.
    use super::FrontApp;

    pub struct Window;

    impl Window {
        pub fn same_as(&self, _other: &Window) -> bool {
            true
        }
    }

    pub fn frontmost() -> Option<(FrontApp, Option<Window>)> {
        None
    }
}
