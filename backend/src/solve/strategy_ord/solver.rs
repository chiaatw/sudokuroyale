use std::time::Instant;

use crate::io::Cancelable;
use crate::puzzle::{Action, Board, ChangeResult, Changer, Difficulty, Effects, Options};
use crate::solve::algorithms::brute_force::BruteForceResult;
use crate::solve::{find_brute_force, Timings, NON_PEER_TECHNIQUES};

pub enum Resolution {
    Canceled(Board, Effects, Difficulty),

    Failed(Board, Effects, Difficulty, Action, Effects),

    Unsolved(Board, Effects, Difficulty),

    Solved(Board, Effects, Difficulty),
}

impl Resolution {
    pub fn is_canceled(&self) -> bool {
        matches!(self, Self::Canceled(..))
    }

    pub fn is_solved(&self) -> bool {
        matches!(self, Self::Solved(..))
    }
}

pub struct Solver {
    changer: Changer,

    cancelable: Cancelable,

    check: bool,
}

impl Solver {
    pub fn new(check: bool) -> Self {
        Self {
            changer: Changer::new(Options::errors()),
            cancelable: Cancelable::new(),
            check,
        }
    }

    pub fn solve(&self, start: &Board, _: &Effects, timings: &mut Timings) -> Resolution {
        let mut board = *start;
        let mut applied = Effects::new();
        let mut difficulty = Difficulty::Basic;

        loop {
            if board.is_fully_solved() {
                return Resolution::Solved(board, applied, difficulty);
            }

            let action = self.find_next_action(&board, &mut difficulty, timings);
            let Some(action) = action else {
                return Resolution::Unsolved(board, applied, difficulty);
            };

            match self.changer.apply(&board, &action) {
                ChangeResult::None => {}

                ChangeResult::Valid(after, _) => {
                    applied.add_action(action);
                    board = after;
                }

                ChangeResult::Invalid(before, _, action, errors) => {
                    if self.check
                        && matches!(find_brute_force(start, false), BruteForceResult::Solved(_))
                    {
                        eprintln!(
                            "error: solver caused errors in solvable puzzle: {}",
                            start.packed_string()
                        );
                    }

                    return Resolution::Failed(before, applied, difficulty, action.clone(), errors);
                }
            }
        }
    }

    fn find_next_action(
        &self,
        board: &Board,
        difficulty: &mut Difficulty,
        timings: &mut Timings,
    ) -> Option<Action> {
        for solver in NON_PEER_TECHNIQUES {
            if self.cancelable.is_canceled() {
                return None;
            }

            let start = Instant::now();
            let result = solver.solve(board, true);
            let elapsed = start.elapsed();

            match result {
                Some(moves) => {
                    timings.add(solver.strategy(), moves.action_count(), elapsed);

                    if solver.difficulty() > *difficulty {
                        *difficulty = solver.difficulty();
                    }

                    return Some(moves.actions()[0].clone());
                }
                None => {
                    timings.add(solver.strategy(), 0, elapsed);
                }
            }
        }

        None
    }
}
