//! The home page's numbers and each history entry's details, read from the
//! local timings log (`yuyin_timings.jsonl`, written by [`super::session`]).
//! Everything is computed on this machine.

use std::collections::{BTreeMap, HashMap};

use chrono::{Datelike, Duration, Local, NaiveDate, TimeZone};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;

/// "Time saved" compares speaking with typing at this many characters a
/// minute (a typical Zhuyin typist), and says so on the home page.
pub const TYPING_CHARS_PER_MINUTE: u64 = 40;
/// The activity grid shows this many weeks.
const WEEKS: i64 = 26;

/// One line of the timings log; only the fields the UI needs. Older lines
/// lack the newer fields, so all of them default.
#[derive(Deserialize, Default)]
#[serde(default)]
struct Record {
    at: i64,
    context: String,
    level: Option<String>,
    polish: String,
    press_to_release_ms: Option<u64>,
    release_to_output_ms: Option<u64>,
    chars_out: u64,
    app: String,
    sent_chars: u64,
    sent_to: Option<String>,
    file_name: Option<String>,
}

#[derive(Serialize, Type, Debug, PartialEq)]
pub struct DayCount {
    /// `YYYY-MM-DD`, local time.
    pub date: String,
    pub dictations: u32,
}

#[derive(Serialize, Type, Debug, PartialEq)]
pub struct Privacy {
    /// Always zero: recognition runs on this machine. Reported rather than
    /// hard-coded in the UI so the claim is visibly backed by the log.
    pub audio_uploaded_ms: u64,
    /// Always zero: only a style label ("chat") leaves the machine.
    pub app_names_sent: u64,
    /// Characters of transcript sent to the clean-up service.
    pub text_sent_chars: u64,
    /// Hosts text was sent to, e.g. `api.deepseek.com`.
    pub sent_to: Vec<String>,
}

#[derive(Serialize, Type, Debug, PartialEq)]
pub struct Stats {
    pub dictations: u32,
    /// Characters pasted.
    pub chars: u64,
    /// Time spent holding the key.
    pub speaking_ms: u64,
    /// Typing the same text at [`TYPING_CHARS_PER_MINUTE`], minus speaking.
    pub saved_ms: u64,
    pub chars_per_minute: u32,
    pub active_days: u32,
    pub current_streak: u32,
    pub longest_streak: u32,
    /// The last [`WEEKS`] weeks, oldest first, starting on a Sunday.
    pub days: Vec<DayCount>,
    pub privacy: Privacy,
}

/// What the history page shows under an entry.
#[derive(Serialize, Type, Debug, Clone, PartialEq)]
pub struct EntryMeta {
    pub app: String,
    pub context: String,
    pub level: Option<String>,
    pub polish: String,
    pub sent_chars: u64,
    pub sent_to: Option<String>,
    pub spoke_ms: Option<u64>,
    pub output_ms: Option<u64>,
}

fn read_records(app: &AppHandle) -> Vec<Record> {
    let Some(path) = super::session::timings_path(app) else {
        return Vec::new();
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    parse(&text)
}

fn parse(text: &str) -> Vec<Record> {
    text.lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

fn local_date(at_ms: i64) -> Option<NaiveDate> {
    Local
        .timestamp_millis_opt(at_ms)
        .single()
        .map(|t| t.date_naive())
}

pub fn stats(app: &AppHandle) -> Stats {
    compute(&read_records(app), Local::now().date_naive(), local_date)
}

pub fn history_meta(app: &AppHandle) -> HashMap<String, EntryMeta> {
    read_records(app)
        .into_iter()
        .filter_map(|r| {
            let file_name = r.file_name.clone()?;
            Some((
                file_name,
                EntryMeta {
                    app: r.app,
                    context: r.context,
                    level: r.level,
                    polish: r.polish,
                    sent_chars: r.sent_chars,
                    sent_to: r.sent_to,
                    spoke_ms: r.press_to_release_ms,
                    output_ms: r.release_to_output_ms,
                },
            ))
        })
        .collect()
}

fn compute(
    records: &[Record],
    today: NaiveDate,
    date_of: impl Fn(i64) -> Option<NaiveDate>,
) -> Stats {
    let chars: u64 = records.iter().map(|r| r.chars_out).sum();
    let speaking_ms: u64 = records.iter().filter_map(|r| r.press_to_release_ms).sum();
    let typing_ms = chars * 60_000 / TYPING_CHARS_PER_MINUTE;
    let chars_per_minute = if speaking_ms > 0 {
        (chars * 60_000 / speaking_ms) as u32
    } else {
        0
    };

    let mut per_day: BTreeMap<NaiveDate, u32> = BTreeMap::new();
    for r in records {
        if let Some(d) = date_of(r.at) {
            *per_day.entry(d).or_default() += 1;
        }
    }

    let mut longest = 0u32;
    let mut run = 0u32;
    let mut previous: Option<NaiveDate> = None;
    for d in per_day.keys() {
        run = match previous {
            Some(p) if *d - p == Duration::days(1) => run + 1,
            _ => 1,
        };
        longest = longest.max(run);
        previous = Some(*d);
    }
    // A streak still counts today before the first dictation of the day.
    let mut current = 0u32;
    let mut day = if per_day.contains_key(&today) {
        today
    } else {
        today - Duration::days(1)
    };
    while per_day.contains_key(&day) {
        current += 1;
        day -= Duration::days(1);
    }

    // Weeks run Sunday to Saturday; the grid ends with this week.
    let this_sunday = today - Duration::days(today.weekday().num_days_from_sunday() as i64);
    let first = this_sunday - Duration::weeks(WEEKS - 1);
    let days = (0..WEEKS * 7)
        .map(|i| first + Duration::days(i))
        .filter(|d| *d <= today)
        .map(|d| DayCount {
            date: d.format("%Y-%m-%d").to_string(),
            dictations: per_day.get(&d).copied().unwrap_or(0),
        })
        .collect();

    let mut sent_to: Vec<String> = records.iter().filter_map(|r| r.sent_to.clone()).collect();
    sent_to.sort();
    sent_to.dedup();

    Stats {
        dictations: records.len() as u32,
        chars,
        speaking_ms,
        saved_ms: typing_ms.saturating_sub(speaking_ms),
        chars_per_minute,
        active_days: per_day.len() as u32,
        current_streak: current,
        longest_streak: longest,
        days,
        privacy: Privacy {
            audio_uploaded_ms: 0,
            app_names_sent: 0,
            text_sent_chars: records.iter().map(|r| r.sent_chars).sum(),
            sent_to,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    /// Records whose `at` is a day number, so tests pick dates directly.
    fn rec(at_day: &str, chars: u64, spoke_ms: u64, sent: u64) -> Record {
        Record {
            at: day(at_day)
                .and_hms_opt(12, 0, 0)
                .unwrap()
                .and_utc()
                .timestamp_millis(),
            chars_out: chars,
            press_to_release_ms: Some(spoke_ms),
            sent_chars: sent,
            sent_to: (sent > 0).then(|| "api.deepseek.com".to_string()),
            ..Default::default()
        }
    }

    fn date_of_utc(at: i64) -> Option<NaiveDate> {
        chrono::DateTime::from_timestamp_millis(at).map(|t| t.date_naive())
    }

    #[test]
    fn totals_speed_and_saved_time() {
        let records = vec![
            rec("2026-09-27", 120, 40_000, 120),
            rec("2026-09-28", 48, 20_000, 0),
        ];
        let s = compute(&records, day("2026-09-28"), date_of_utc);
        assert_eq!(s.chars, 168);
        assert_eq!(s.speaking_ms, 60_000);
        assert_eq!(s.chars_per_minute, 168);
        // Typing 168 characters at 40/min takes 252 s; speaking took 60 s.
        assert_eq!(s.saved_ms, 192_000);
        assert_eq!(s.privacy.text_sent_chars, 120);
        assert_eq!(s.privacy.sent_to, vec!["api.deepseek.com".to_string()]);
        assert_eq!(s.privacy.audio_uploaded_ms, 0);
    }

    #[test]
    fn streaks_and_grid() {
        let records = vec![
            rec("2026-09-20", 1, 1, 0),
            rec("2026-09-21", 1, 1, 0),
            rec("2026-09-22", 1, 1, 0),
            rec("2026-09-26", 1, 1, 0),
            rec("2026-09-27", 1, 1, 0),
            rec("2026-09-27", 1, 1, 0),
        ];
        // Monday 2026-09-28, nothing yet today: the streak ending yesterday counts.
        let s = compute(&records, day("2026-09-28"), date_of_utc);
        assert_eq!(s.active_days, 5);
        assert_eq!(s.longest_streak, 3);
        assert_eq!(s.current_streak, 2);
        // 25 full weeks plus this week's Sunday and Monday.
        assert_eq!(s.days.len(), 25 * 7 + 2);
        assert_eq!(s.days.first().unwrap().date, "2026-04-05");
        let last = s.days.last().unwrap();
        assert_eq!((last.date.as_str(), last.dictations), ("2026-09-28", 0));
        let sunday = &s.days[s.days.len() - 2];
        assert_eq!((sunday.date.as_str(), sunday.dictations), ("2026-09-27", 2));
    }

    #[test]
    fn old_lines_without_new_fields_still_count() {
        let text = r#"{"at":1790598013964,"context":"to_ai","level":"tidy","polish":"ok","press_to_release_ms":11475,"chars_out":68}
not json
{"at":1790598060223,"context":"to_ai","polish":"ok","chars_out":135}"#;
        let records = parse(text);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].chars_out, 68);
        assert!(records[1].file_name.is_none());
    }
}
