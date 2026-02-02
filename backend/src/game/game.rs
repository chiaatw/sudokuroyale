use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::game::outcome::{AppliedMove, MoveOutcome, PenaltyReason, RejectReason};
use crate::game::player::{PlayerId, PlayerState};
use crate::game::puzzle::Puzzle;
use crate::game::r#move::Move;
use crate::game::state::{GameState, LoseReason};
use crate::game::view::{GameView, OpponentProgress};
use crate::layout::{Cell, Grid, Value, ValueLike};

#[derive(Debug, Clone)]
pub struct Game {
    puzzle: Puzzle,
    players: HashMap<PlayerId, PlayerState>,
    state: GameState,

    revision: u64,
    start_at: Option<Instant>,
}

impl Game {
    pub fn new(puzzle: Puzzle, time_limit: Duration) -> Self {
        let mut players = HashMap::new();
        players.insert(PlayerId::PlayerA, PlayerState::new(&puzzle, time_limit));
        players.insert(PlayerId::PlayerB, PlayerState::new(&puzzle, time_limit));

        Self {
            puzzle,
            players,
            state: GameState::Waiting,
            revision: 0,
            start_at: None,
        }
    }

    // --- Meta / Accessors ---

    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn start_at(&self) -> Option<Instant> {
        self.start_at
    }

    pub fn puzzle(&self) -> &Puzzle {
        &self.puzzle
    }

    pub fn player(&self, player: PlayerId) -> &PlayerState {
        self.players.get(&player).expect("missing player")
    }

    pub fn player_mut(&mut self, player: PlayerId) -> &mut PlayerState {
        self.players.get_mut(&player).expect("missing player")
    }

    // --- Lifecycle ---

    pub fn start(&mut self, now: Instant) {
        if self.state == GameState::Waiting {
            self.state = GameState::InProgress;
            self.start_at = Some(now);

            for p in self.players.values_mut() {
                p.time.start(now);
            }

            self.revision += 1;
        }
    }

    // --- Internal helpers ---

    fn set_win(&mut self, player: PlayerId) {
        self.state = GameState::Won { player };
        self.revision += 1;
    }

    fn set_loss(&mut self, loser: PlayerId, reason: LoseReason) {
        let winner = match loser {
            PlayerId::PlayerA => PlayerId::PlayerB,
            PlayerId::PlayerB => PlayerId::PlayerA,
        };

        self.state = GameState::Lost {
            loser,
            winner,
            reason,
        };
        self.revision += 1;
    }

    // --- Core API ---

    pub fn apply_move(
        &mut self,
        player: PlayerId,
        expected_revision: u64,
        mv: Move,
        now: Instant,
    ) -> MoveOutcome {
        if !matches!(self.state, GameState::InProgress) {
            return MoveOutcome::Rejected {
                reason: RejectReason::NotInProgress,
            };
        }

        if expected_revision != self.revision {
            return MoveOutcome::Rejected {
                reason: RejectReason::RevisionMismatch {
                    expected: expected_revision,
                    actual: self.revision,
                },
            };
        }

        if !self.players.contains_key(&player) {
            return MoveOutcome::Rejected {
                reason: RejectReason::UnknownPlayer,
            };
        }

        // Timeout check (borrow-safe)
        let expired = self.players.get(&player).unwrap().time.is_expired(now);
        if expired {
            self.set_loss(player, LoseReason::TimeExpired);
            return MoveOutcome::Lost {
                revision: self.revision,
                reason: LoseReason::TimeExpired,
            };
        }

        match mv {
            Move::Clear { cell } => {
                if self.puzzle.is_given(cell) {
                    return MoveOutcome::Rejected {
                        reason: RejectReason::GivenCell,
                    };
                }

                {
                    let p = self.players.get_mut(&player).unwrap();
                    p.current.set(cell, Value::unknown());
                }

                self.revision += 1;
                MoveOutcome::Applied {
                    revision: self.revision,
                    applied: AppliedMove::Cleared,
                }
            }

            Move::Place { cell, value } => {
                if self.puzzle.is_given(cell) {
                    return MoveOutcome::Rejected {
                        reason: RejectReason::GivenCell,
                    };
                }

                if !value.is_known() {
                    return MoveOutcome::Rejected {
                        reason: RejectReason::InvalidValue,
                    };
                }

                // Wrong value -> penalty or loss
                if !self.puzzle.is_correct_value(cell, value) {
                    let (lost, mistakes_left) = {
                        let p = self.players.get_mut(&player).unwrap();
                        let lost = p.register_mistake();
                        (lost, p.mistakes_left())
                    };

                    if lost {
                        self.set_loss(player, LoseReason::TooManyMistakes);
                        return MoveOutcome::Lost {
                            revision: self.revision,
                            reason: LoseReason::TooManyMistakes,
                        };
                    }

                    self.revision += 1;
                    return MoveOutcome::Penalty {
                        reason: PenaltyReason::WrongValue,
                        mistakes_left,
                        revision: self.revision,
                    };
                }

                // Correct value: set + check solved while holding borrow
                let solved = {
                    let p = self.players.get_mut(&player).unwrap();
                    p.current.set(cell, value);
                    p.current == *self.puzzle.solution()
                };

                if solved {
                    // IMPORTANT: don't bump revision before set_win, set_win bumps it
                    self.set_win(player);
                    return MoveOutcome::Won {
                        revision: self.revision,
                    };
                }

                self.revision += 1;
                MoveOutcome::Applied {
                    revision: self.revision,
                    applied: AppliedMove::Placed,
                }
            }
        }
    }

    pub fn view_for(&mut self, player: PlayerId, now: Instant) -> GameView {
        // Timeout “einschnappen” lassen, auch ohne Moves (polling-friendly)
        if matches!(self.state, GameState::InProgress) {
            if let Some(p) = self.players.get(&player) {
                if p.time.is_expired(now) && matches!(self.state, GameState::InProgress) {
                    self.set_loss(player, LoseReason::TimeExpired);
                }
            }

            let opponent = match player {
                PlayerId::PlayerA => PlayerId::PlayerB,
                PlayerId::PlayerB => PlayerId::PlayerA,
            };

            if let Some(op) = self.players.get(&opponent) {
                if op.time.is_expired(now) && matches!(self.state, GameState::InProgress) {
                    self.set_loss(opponent, LoseReason::TimeExpired);
                }
            }
        }

        let p = self.player(player);

        let opponent_id = match player {
            PlayerId::PlayerA => PlayerId::PlayerB,
            PlayerId::PlayerB => PlayerId::PlayerA,
        };

        let opponent_progress = self.players.get(&opponent_id).map(|op| OpponentProgress {
            filled: count_filled(&op.current),
            mistakes_left: op.mistakes_left(),
            remaining_time: op.time.remaining(now),
        });

        GameView {
            revision: self.revision,
            state: self.state.clone(),

            givens: self.puzzle.givens().clone(),
            current: p.current.clone(),

            mistakes_left: p.mistakes_left(),
            remaining_time: p.time.remaining(now),

            opponent_progress,
        }
    }
}

/// zählt wie viele Felder gesetzt sind
fn count_filled(grid: &Grid) -> u8 {
    let mut count = 0u8;
    for cell in Cell::iter() {
        if grid.get(cell).is_known() {
            count = count.saturating_add(1);
        }
    }
    count
}
