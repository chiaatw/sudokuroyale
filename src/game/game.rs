use std::collections::HashMap;
use std::time::Duration;

use crate::game::state::{GameState, LoseReason};
use crate::game::player::{PlayerId, PlayerState};
use crate::layout::Sudoku;
use crate::layout::{Cell, Value};

pub enum MoveResult {
    Accepted,
    Rejected,
    Won,
    Lost,
}

pub struct Game {
    sudoku: Sudoku,
    state: GameState,
    players: HashMap<PlayerId, PlayerState>,
}

impl Game {
    pub fn new(sudoku: Sudoku, time_limit: Duration) -> Self {
        let mut players = HashMap::new();
        players.insert(PlayerId::PlayerA, PlayerState::new(time_limit));
        players.insert(PlayerId::PlayerB, PlayerState::new(time_limit));

        Self {
            sudoku,
            state: GameState::InProgress,
            players,
        }
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn apply_move(
        &mut self,
        player: PlayerId,
        cell: Cell,
        value: Value,
    ) -> MoveResult {
        if !matches!(self.state, GameState::InProgress) {
            return MoveResult::Rejected;
        }

        if !self.sudoku.is_correct_move(cell, value) {
            let p = self.players.get_mut(&player).unwrap();
            if p.register_mistake() {
                self.state = GameState::Lost {
                    player,
                    reason: LoseReason::TooManyMistakes,
                };
                return MoveResult::Lost;
            }
            return MoveResult::Accepted;
        }

        self.sudoku.set(cell, value);

        if self.sudoku.is_solved() {
            self.state = GameState::Won { player };
            return MoveResult::Won;
        }

        MoveResult::Accepted
    }

    pub fn tick(&mut self, player: PlayerId, delta: Duration) {
        if let Some(p) = self.players.get_mut(&player) {
            p.time.tick(delta);
            if p.time.is_expired() {
                self.state = GameState::Lost {
                    player,
                    reason: LoseReason::TimeExpired,
                };
            }
        }
    }
}
