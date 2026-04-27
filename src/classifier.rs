//! Threshold-based state classifier.
//!
//! `stressed_threshold` is passed by the caller (from Settings) so the
//! function stays pure and testable without touching shared state.

use serde::Serialize;

const PASSIVE_KPM_THRESHOLD: f64 = 50.0;
const PASSIVE_VAR_DT_THRESHOLD: f64 = 7.0;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum State {
    /// Collector reports no active typing.
    Idle,
    /// Low typing activity or high rhythm variance — reading/browsing.
    Passive,
    /// Active typing, correction rate within normal range.
    Normal,
    /// Active typing, high correction rate — stress signal.
    Stressed,
}

impl State {
    pub fn emoji(&self) -> &'static str {
        match self {
            State::Idle => "⚪",
            State::Passive => "🔵",
            State::Normal => "🟡",
            State::Stressed => "🔴",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            State::Idle => "IDLE",
            State::Passive => "PASSIVE",
            State::Normal => "NORMAL",
            State::Stressed => "STRESSED",
        }
    }
}

pub fn classify(
    is_active: bool,
    kpm60: f64,
    var_dt: f64,
    correction_rate: f64,
    stressed_threshold: f64,
) -> State {
    if !is_active {
        return State::Idle;
    }
    if kpm60 < PASSIVE_KPM_THRESHOLD || var_dt > PASSIVE_VAR_DT_THRESHOLD {
        return State::Passive;
    }
    if correction_rate <= stressed_threshold {
        return State::Normal;
    }
    State::Stressed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_when_not_active() {
        assert_eq!(classify(false, 0.0, 0.0, 0.0, 0.056), State::Idle);
        assert_eq!(classify(false, 100.0, 1.0, 0.0, 0.056), State::Idle);
    }

    #[test]
    fn passive_when_kpm_below_50() {
        assert_eq!(classify(true, 1.0, 0.0, 0.0, 0.056), State::Passive);
        assert_eq!(classify(true, 49.9, 0.0, 0.0, 0.056), State::Passive);
    }

    #[test]
    fn passive_when_high_var_dt() {
        assert_eq!(classify(true, 50.0, 7.1, 0.0, 0.056), State::Passive);
        assert_eq!(classify(true, 100.0, 10.0, 0.5, 0.056), State::Passive);
    }

    #[test]
    fn normal_when_correction_rate_within_range() {
        assert_eq!(classify(true, 50.0, 1.0, 0.0, 0.056), State::Normal);
        assert_eq!(classify(true, 50.0, 1.0, 0.056, 0.056), State::Normal);
    }

    #[test]
    fn stressed_when_high_correction_rate() {
        assert_eq!(classify(true, 50.0, 1.0, 0.057, 0.056), State::Stressed);
        assert_eq!(classify(true, 200.0, 0.5, 0.9, 0.056), State::Stressed);
    }

    #[test]
    fn sensitive_profile_triggers_earlier() {
        assert_eq!(classify(true, 50.0, 1.0, 0.043, 0.042), State::Stressed);
        assert_eq!(classify(true, 50.0, 1.0, 0.043, 0.056), State::Normal);
    }

    #[test]
    fn relaxed_profile_requires_higher_rate() {
        assert_eq!(classify(true, 50.0, 1.0, 0.060, 0.067), State::Normal);
        assert_eq!(classify(true, 50.0, 1.0, 0.060, 0.056), State::Stressed);
    }
}
