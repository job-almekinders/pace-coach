//! User-configurable settings, persisted to `~/.pace-coach/config.json`.
//! Missing or corrupt file silently falls back to defaults.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Correction rate above which typing is classified as Stressed.
    pub correction_rate_threshold: f64,
    /// Seconds of sustained Stressed state before a nudge fires.
    pub stress_duration_secs: u64,
    /// Minimum seconds between nudges.
    pub nudge_cooldown_secs: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            correction_rate_threshold: 0.06,
            stress_duration_secs: 10,
            nudge_cooldown_secs: 60,
        }
    }
}

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".pace-coach").join("config.json")
}

pub fn load_from(path: &Path) -> Settings {
    let Ok(contents) = std::fs::read_to_string(path) else {
        return Settings::default();
    };
    serde_json::from_str(&contents).unwrap_or_default()
}

// Only called from tests; clippy doesn't see test-only usage as "used".
#[allow(dead_code)]
pub fn save_to(path: &Path, s: &Settings) {
    let Ok(json) = serde_json::to_string_pretty(s) else {
        return;
    };
    if let Err(e) = std::fs::write(path, json + "\n") {
        eprintln!("[pace-coach] failed to write config: {e}");
    }
}

pub fn load() -> Settings {
    load_from(&config_path())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_correct() {
        let s = Settings::default();
        assert!((s.correction_rate_threshold - 0.06).abs() < f64::EPSILON);
        assert_eq!(s.stress_duration_secs, 10);
        assert_eq!(s.nudge_cooldown_secs, 60);
    }

    #[test]
    fn load_from_missing_file_returns_defaults() {
        let tmp = std::env::temp_dir().join("no_such_pace_coach_settings.json");
        let s = load_from(&tmp);
        assert!((s.correction_rate_threshold - 0.06).abs() < f64::EPSILON);
    }

    #[test]
    fn save_and_load_roundtrip() {
        let tmp = std::env::temp_dir().join("pace_coach_test_settings.json");
        let original = Settings {
            correction_rate_threshold: 0.042,
            stress_duration_secs: 5,
            nudge_cooldown_secs: 120,
        };
        save_to(&tmp, &original);
        let loaded = load_from(&tmp);
        assert!((loaded.correction_rate_threshold - 0.042).abs() < f64::EPSILON);
        assert_eq!(loaded.stress_duration_secs, 5);
        assert_eq!(loaded.nudge_cooldown_secs, 120);
        std::fs::remove_file(&tmp).unwrap();
    }

    #[test]
    fn corrupt_file_returns_defaults() {
        let tmp = std::env::temp_dir().join("pace_coach_test_settings_bad.json");
        std::fs::write(&tmp, "not valid json").unwrap();
        let s = load_from(&tmp);
        assert!((s.correction_rate_threshold - 0.06).abs() < f64::EPSILON);
        std::fs::remove_file(&tmp).unwrap();
    }
}
