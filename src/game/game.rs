use std::collections::HashMap;
use std::time::Duration;

use crate::game::player::{PlayerId, PlayerState};
use crate::game::state::{GameState, LoseReason};
use crate::layout::Sudoku;
use crate::layout::{Cell, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveResult {
    Accepted,
    Mistake { mistakes_left: u8 },
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

    pub fn apply_move(&mut self, player: PlayerId, cell: Cell, value: Value) -> MoveResult {
        if !matches!(self.state, GameState::InProgress) {
            return MoveResult::Rejected;
        }

        let Some(p) = self.players.get_mut(&player) else {
            return MoveResult::Rejected;
        };

        if !self.sudoku.is_correct_move(cell, value) {
            if p.register_mistake() {
                let winner = match player {
                    PlayerId::PlayerA => PlayerId::PlayerB,
                    PlayerId::PlayerB => PlayerId::PlayerA,
                };

                self.state = GameState::Lost {
                    loser: player,
                    winner,
                    reason: LoseReason::TooManyMistakes,
                };
                return MoveResult::Lost;
            }

            return MoveResult::Mistake {
                mistakes_left: p.mistakes_left(),
            };
        }

        self.sudoku.set(cell, value);

        if self.sudoku.is_solved() {
            self.state = GameState::Won { player };
            return MoveResult::Won;
        }

        MoveResult::Accepted
    }

    pub fn tick(&mut self, player: PlayerId, delta: Duration) {
        if !matches!(self.state, GameState::InProgress) {
            return;
        }

        if let Some(p) = self.players.get_mut(&player) {
            p.time.tick(delta);

            if p.time.is_expired() {
                let winner = match player {
                    PlayerId::PlayerA => PlayerId::PlayerB,
                    PlayerId::PlayerB => PlayerId::PlayerA,
                };

                self.state = GameState::Lost {
                    loser: player,
                    winner,
                    reason: LoseReason::TimeExpired,
                };
            }
        }
    }
}
