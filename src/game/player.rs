use std::time::Duration;
use crate::game::time::TimeControl;

pub const MAX_MISTAKES: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerId {
    PlayerA,
    PlayerB,
}

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub mistakes: u8,
    pub time: TimeControl,
}

impl PlayerState {
    pub fn new(time_limit: Duration) -> Self {
        Self {
            mistakes: 0,
            time: TimeControl::new(time_limit),
        }
    }

    pub fn register_mistake(&mut self) -> bool {
        self.mistakes += 1;
        self.mistakes >= MAX_MISTAKES
    }
}
