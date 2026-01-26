use std::time::Duration;

use crate::game::state::GameState;
use crate::layout::Grid;

#[derive(Debug, Clone)]
pub struct GameView {
    pub revision: u64,
    pub state: GameState,

    pub givens: Grid,
    pub current: Grid,

    pub mistakes_left: u8,
    pub remaining_time: Duration,

    pub opponent_progress: Option<OpponentProgress>,
}

#[derive(Debug, Clone)]
pub struct OpponentProgress {
    pub filled: u8,
    pub mistakes_left: u8,
    pub remaining_time: Duration,
}
