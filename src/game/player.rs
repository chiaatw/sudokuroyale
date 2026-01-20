use crate::game::time::TimeControl;
use std::time::Duration;

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
        if self.mistakes < MAX_MISTAKES {
            self.mistakes += 1;
        }
        self.mistakes >= MAX_MISTAKES
    }

    pub fn mistakes_left(&self) -> u8 {
        MAX_MISTAKES - self.mistakes
    }
}
