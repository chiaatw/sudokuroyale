use std::time::{Duration, Instant};

/// Server-authoritative Timer.
/// - Vor `start(now)` läuft die Uhr nicht.
/// - Nach `start(now)` läuft sie bis `deadline`.
#[derive(Debug, Clone)]
pub struct TimeControl {
    limit: Duration,
    deadline: Option<Instant>,
}

impl TimeControl {
    pub fn new(limit: Duration) -> Self {
        Self {
            limit,
            deadline: None,
        }
    }

    /// Startet die Uhr. Vorher läuft sie nicht.
    /// Wenn mehrfach aufgerufen: setzt die Deadline neu
    pub fn start(&mut self, now: Instant) {
        if self.deadline.is_none(){
            self.deadline = Some(now + self.limit);
        }
    }

    /// Gibt die verbleibende Zeit zurück
    /// Vor Start: volle Zeit
    pub fn remaining(&self, now: Instant) -> Duration {
        match self.deadline {
            None => self.limit,
            Some(deadline) => deadline.saturating_duration_since(now),
        }
    }

    /// True, wenn die Zeit abgelaufen ist.
    /// Vor Start: immer false.
    pub fn is_expired(&self, now: Instant) -> bool {
        match self.deadline {
            None => false,
            Some(deadline) => now >= deadline,
        }
    }

    /// Optional: Erlaubt dir, die Deadline z.B. beim Laden aus Persistenz zu setzen:
    /// `set_remaining(now, remaining)`.
    pub fn set_remaining(&mut self, now: Instant, remaining: Duration) {
        self.deadline = Some(now + remaining);
    }

    /// Optional: Für UI/Debug.
    pub fn has_started(&self) -> bool {
        self.deadline.is_some()
    }

}
