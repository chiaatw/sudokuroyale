use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TimeControl {
    remaining: Duration,
}

impl TimeControl {
    pub fn new(total: Duration) -> Self {
        Self { remaining: total }
    }

    pub fn tick(&mut self, delta: Duration) {
        self.remaining = self.remaining.saturating_sub(delta);
    }

    pub fn is_expired(&self) -> bool {
        self.remaining.is_zero()
    }
}
