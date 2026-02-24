use std::time::{Duration, Instant};

/// Server-authoritative Stopwatch
/// - Vor start(now) läuft sie nicht
/// - Nach start(now) zählt sie hoch (elapsed)
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

    /// Vergangene Zeit seit Start
    pub fn elapsed(&self, now: Instant) -> Duration {
        match self.started_at {
            None => Duration::ZERO,
            Some(start) => now.saturating_duration_since(start),
        }
    }

    /// Für Stoppuhr: nie "expired"
    pub fn is_expired(&self, _now: Instant) -> bool {
        false
    }

    pub fn has_started(&self) -> bool {
        self.started_at.is_some()
    }
}