//! In-process keyboard collector.
//!
//! Runs two threads:
//!   1. rdev listener — captures key events into CollectorState (blocking, OS thread).
//!   2. Metrics loop  — wakes every 1 s, computes RawMetrics, writes to shared arc.
//!
//! Call `start(raw)` from a dedicated `std::thread::spawn`.

use rdev::{listen, Event, EventType, Key};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct RawMetrics {
    pub is_active: bool,
    pub kpm_60: f64,
    pub kpm_10: f64,
    pub var_dt: f64,
    pub correction_rate: f64,
}

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct KeyEvent {
    timestamp: Instant,
    is_correction: bool,
}

struct CollectorState {
    key_events: VecDeque<KeyEvent>,
    last_input: Instant,
    ctrl_pressed: bool,
    cmd_pressed: bool,
    alt_pressed: bool,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const WINDOW_SECONDS: f64 = 60.0;
const SHORT_WINDOW_SECONDS: f64 = 10.0;
const ACTIVE_THRESHOLD_SECS: f64 = 5.0;

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

fn is_typing_key(key: Key) -> bool {
    matches!(
        key,
        Key::Space
            | Key::Return
            | Key::Tab
            | Key::Backspace
            | Key::Delete
            | Key::KeyA
            | Key::KeyB
            | Key::KeyC
            | Key::KeyD
            | Key::KeyE
            | Key::KeyF
            | Key::KeyG
            | Key::KeyH
            | Key::KeyI
            | Key::KeyJ
            | Key::KeyK
            | Key::KeyL
            | Key::KeyM
            | Key::KeyN
            | Key::KeyO
            | Key::KeyP
            | Key::KeyQ
            | Key::KeyR
            | Key::KeyS
            | Key::KeyT
            | Key::KeyU
            | Key::KeyV
            | Key::KeyW
            | Key::KeyX
            | Key::KeyY
            | Key::KeyZ
    )
}

fn is_correction(key: Key, ctrl: bool, _cmd: bool, _alt: bool) -> bool {
    match key {
        Key::Backspace | Key::Delete => true,
        Key::KeyW | Key::KeyH | Key::KeyU | Key::KeyK if ctrl => true,
        _ => false,
    }
}

fn prune_old_events(deque: &mut VecDeque<KeyEvent>, now: Instant) {
    while let Some(front) = deque.front() {
        if now.duration_since(front.timestamp).as_secs_f64() > WINDOW_SECONDS {
            deque.pop_front();
        } else {
            break;
        }
    }
}

fn compute_metrics(state: &CollectorState, now: Instant) -> RawMetrics {
    let seconds_since_input = now.duration_since(state.last_input).as_secs_f64();
    let is_active = seconds_since_input < ACTIVE_THRESHOLD_SECS;

    let key_count_60 = state.key_events.len() as f64;
    let kpm_60 = (key_count_60 / WINDOW_SECONDS) * 60.0;

    let key_count_10 = state
        .key_events
        .iter()
        .filter(|e| now.duration_since(e.timestamp).as_secs_f64() <= SHORT_WINDOW_SECONDS)
        .count() as f64;
    let kpm_10 = (key_count_10 / SHORT_WINDOW_SECONDS) * 60.0;

    let backspaces = state.key_events.iter().filter(|e| e.is_correction).count() as f64;
    let correction_rate = if key_count_60 > 0.0 {
        backspaces / key_count_60
    } else {
        0.0
    };

    let mut intervals = Vec::new();
    let mut prev: Option<Instant> = None;
    for event in state.key_events.iter() {
        if let Some(prev_ts) = prev {
            intervals.push(event.timestamp.duration_since(prev_ts).as_secs_f64());
        }
        prev = Some(event.timestamp);
    }
    let var_dt = if intervals.len() > 1 {
        let mean = intervals.iter().sum::<f64>() / intervals.len() as f64;
        intervals.iter().map(|dt| (dt - mean).powi(2)).sum::<f64>() / intervals.len() as f64
    } else {
        0.0
    };

    RawMetrics {
        is_active,
        kpm_60,
        kpm_10,
        var_dt,
        correction_rate,
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Launch the collector. Blocks forever on the calling thread (rdev::listen).
/// Call from a dedicated `std::thread::spawn`.
pub fn start(raw: Arc<Mutex<Option<RawMetrics>>>) {
    let state = Arc::new(Mutex::new(CollectorState {
        key_events: VecDeque::new(),
        last_input: Instant::now(),
        ctrl_pressed: false,
        cmd_pressed: false,
        alt_pressed: false,
    }));

    // Metrics computation thread — wakes every 1 s.
    let state_for_metrics = state.clone();
    let raw_for_metrics = raw.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));
        let now = Instant::now();
        let mut s = state_for_metrics.lock().unwrap();
        prune_old_events(&mut s.key_events, now);
        let metrics = compute_metrics(&s, now);
        drop(s);
        *raw_for_metrics.lock().unwrap() = Some(metrics);
    });

    // rdev listener — blocks on this thread.
    let callback = move |event: Event| {
        let now = Instant::now();
        let mut s = state.lock().unwrap();
        match event.event_type {
            EventType::KeyPress(key) => {
                match key {
                    Key::ControlLeft | Key::ControlRight => s.ctrl_pressed = true,
                    Key::MetaLeft | Key::MetaRight => s.cmd_pressed = true,
                    Key::Alt | Key::AltGr => s.alt_pressed = true,
                    _ => {}
                }
                if is_typing_key(key) {
                    let correction =
                        is_correction(key, s.ctrl_pressed, s.cmd_pressed, s.alt_pressed);
                    s.key_events.push_back(KeyEvent {
                        timestamp: now,
                        is_correction: correction,
                    });
                    s.last_input = now;
                }
            }
            EventType::KeyRelease(key) => match key {
                Key::ControlLeft | Key::ControlRight => s.ctrl_pressed = false,
                Key::MetaLeft | Key::MetaRight => s.cmd_pressed = false,
                Key::Alt | Key::AltGr => s.alt_pressed = false,
                _ => {}
            },
            _ => {}
        }
    };

    if let Err(e) = listen(callback) {
        eprintln!("[pace-coach] rdev listen error: {e:?}");
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backspace_is_correction() {
        assert!(is_correction(Key::Backspace, false, false, false));
    }

    #[test]
    fn delete_is_correction() {
        assert!(is_correction(Key::Delete, false, false, false));
    }

    #[test]
    fn ctrl_w_is_correction() {
        assert!(is_correction(Key::KeyW, true, false, false));
    }

    #[test]
    fn ctrl_h_is_correction() {
        assert!(is_correction(Key::KeyH, true, false, false));
    }

    #[test]
    fn ctrl_u_is_correction() {
        assert!(is_correction(Key::KeyU, true, false, false));
    }

    #[test]
    fn ctrl_k_is_correction() {
        assert!(is_correction(Key::KeyK, true, false, false));
    }

    #[test]
    fn alt_backspace_is_correction() {
        assert!(is_correction(Key::Backspace, false, false, true));
    }

    #[test]
    fn cmd_backspace_is_correction() {
        assert!(is_correction(Key::Backspace, false, true, false));
    }

    #[test]
    fn regular_keys_not_corrections() {
        assert!(!is_correction(Key::KeyA, false, false, false));
        assert!(!is_correction(Key::Space, false, false, false));
        assert!(!is_correction(Key::KeyW, false, false, false));
    }

    #[test]
    fn compute_metrics_empty_state_returns_inactive() {
        let state = CollectorState {
            key_events: VecDeque::new(),
            last_input: Instant::now() - Duration::from_secs(10),
            ctrl_pressed: false,
            cmd_pressed: false,
            alt_pressed: false,
        };
        let m = compute_metrics(&state, Instant::now());
        assert!(!m.is_active);
        assert_eq!(m.kpm_60, 0.0);
        assert_eq!(m.correction_rate, 0.0);
        assert_eq!(m.var_dt, 0.0);
    }

    #[test]
    fn compute_metrics_counts_corrections() {
        let now = Instant::now();
        let mut events = VecDeque::new();
        events.push_back(KeyEvent {
            timestamp: now - Duration::from_millis(200),
            is_correction: false,
        });
        events.push_back(KeyEvent {
            timestamp: now - Duration::from_millis(100),
            is_correction: true,
        });
        let state = CollectorState {
            key_events: events,
            last_input: now - Duration::from_millis(100),
            ctrl_pressed: false,
            cmd_pressed: false,
            alt_pressed: false,
        };
        let m = compute_metrics(&state, now);
        assert!(m.is_active);
        assert!((m.correction_rate - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn prune_removes_old_events() {
        let now = Instant::now();
        let mut deque = VecDeque::new();
        deque.push_back(KeyEvent {
            timestamp: now - Duration::from_secs(61),
            is_correction: false,
        });
        deque.push_back(KeyEvent {
            timestamp: now - Duration::from_secs(1),
            is_correction: false,
        });
        prune_old_events(&mut deque, now);
        assert_eq!(deque.len(), 1);
    }
}
