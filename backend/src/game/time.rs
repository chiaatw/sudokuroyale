use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct TimeControl {
    started_at: Option<Instant>,
}

impl TimeControl {
    pub fn new() -> Self {
        Self { started_at: None }
    }

    pub fn start(&mut self, now: Instant) {
        if self.started_at.is_none() {
            self.started_at = Some(now);
        }
    }

    pub fn elapsed(&self, now: Instant) -> Duration {
        match self.started_at {
            None => Duration::ZERO,
            Some(start) => now.saturating_duration_since(start),
        }
    }

    pub fn is_expired(&self, _now: Instant) -> bool {
        false
    }

    pub fn has_started(&self) -> bool {
        self.started_at.is_some()
    }
}