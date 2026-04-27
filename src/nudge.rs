use std::time::{Duration, Instant};

pub struct NudgeState {
    stressed_since: Option<Instant>,
    last_nudge: Option<Instant>,
}

impl NudgeState {
    pub fn new() -> Self {
        Self {
            stressed_since: None,
            last_nudge: None,
        }
    }

    pub fn update(
        &mut self,
        state_label: &str,
        stress_duration_secs: u64,
        nudge_cooldown_secs: u64,
    ) -> bool {
        if state_label == "STRESSED" {
            let now = Instant::now();
            let since = self.stressed_since.get_or_insert(now);

            let past_threshold =
                now.duration_since(*since) >= Duration::from_secs(stress_duration_secs);
            let past_cooldown = self
                .last_nudge
                .map(|t| now.duration_since(t) >= Duration::from_secs(nudge_cooldown_secs))
                .unwrap_or(true);

            if past_threshold && past_cooldown {
                self.last_nudge = Some(now);
                return true;
            }
        } else {
            self.stressed_since = None;
        }
        false
    }
}

pub fn notify(msg: &str) {
    let script = format!("display notification {:?} with title \"Pace Coach\"", msg);
    let _ = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .spawn();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nudge_not_fired_for_non_stressed_states() {
        let mut n = NudgeState::new();
        assert!(!n.update("NORMAL", 0, 0));
        assert!(!n.update("IDLE", 0, 0));
        assert!(!n.update("PASSIVE", 0, 0));
    }

    #[test]
    fn nudge_not_fired_below_stress_duration() {
        let mut n = NudgeState::new();
        assert!(!n.update("STRESSED", 60, 0));
    }

    #[test]
    fn nudge_fires_when_stress_duration_is_zero() {
        let mut n = NudgeState::new();
        assert!(n.update("STRESSED", 0, 0));
    }

    #[test]
    fn nudge_respects_cooldown() {
        let mut n = NudgeState::new();
        assert!(n.update("STRESSED", 0, 0));
        assert!(!n.update("STRESSED", 0, 60));
    }

    #[test]
    fn stressed_since_resets_on_non_stressed_state() {
        let mut n = NudgeState::new();
        n.update("STRESSED", 0, 0);
        n.update("NORMAL", 0, 0);
        assert!(n.stressed_since.is_none());
    }
}
