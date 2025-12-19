use std::thread::sleep;
use std::time::Duration;

use crate::io::{print_all_and_single_candidates, Cancelable};

use super::*;

const MINIMUM_KNOWNS_TO_BE_UNIQUELY_SOLVABLE: usize = 17;
const MAXIMUM_SOLUTIONS: usize = 1_000_000;
const DEFAULT_MAXIMUM_SOLUTIONS: usize = 1_000;

// Brute-force Sudoku solver implementing the Solver trait
pub struct BruteForceSolver {
    pub log: bool,
    pub pause: u32,
    pub max_solutions: usize,
}

impl Solver for BruteForceSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::BruteForce
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
       match self.find_brute_force(board, single) {
        BruteForceResult::Solved(_) | BruteForceResult::AlreadySolved => {
            None
        }
        BruteForceResult::MultipleSolutions(_) => None,
        BruteForceResult::TooFewKnowns => None,
        BruteForceResult::UnsolvableCells(_) => None,
        BruteForceResult::Canceled => None,
        BruteForceResult::Unsolvable => None,
       }
    }
}

impl BruteForceSolver {
    pub fn new(log: bool, pause: u32, max_solutions: usize) -> Self {
        let max_solutions = if (1..=MAXIMUM_SOLUTIONS).contains(&max_solutions) {
            max_solutions
        } else {
            DEFAULT_MAXIMUM_SOLUTIONS
        };

        Self {
            log,
            pause,
            max_solutions,
        }
    }

    fn find_brute_force(&self, board: &Board, single: bool) -> BruteForceResult {
        if board.is_fully_solved() {
// Already solved, nothing to do
            return BruteForceResult::AlreadySolved;
        }
        if board.known_count() < MINIMUM_KNOWNS_TO_BE_UNIQUELY_SOLVABLE {
// Too few clues
            return BruteForceResult::TooFewKnowns;
        }

        let empty = board.unknowns() & board.cells_with_n_candidates(0);
        if !empty.is_empty() {
//Unsolvable cells exist
            return BruteForceResult::UnsolvableCells(empty);
        }

        let cancelable = Cancelable::new();
        let changer = Changer::new(Options::errors());
        let mut solutions = Vec::new();
        let mut stack = Vec::with_capacity(81);
        stack.push(Entry::new(*board));

        while let Some(entry) = stack.last_mut() {
            if cancelable.is_canceled() {
                return BruteForceResult::Canceled;
            }

            if self.log {
                println!("stack size {}\n", stack.len());
            }

            if entry.candidates.is_empty() {
                if self.log {
                    println!("backtrack\n");
                }
                stack.pop();
                continue;
            }

            if self.log {
                print_all_and_single_candidates(&entry.board);
                println!("\ncell {} candidates {:?}\n", entry.cell, entry.candidates);
            }

            let known = entry.candidates.pop().unwrap();
            let action = Action::new_set(Strategy::BruteForce, entry.cell, known);

            if self.log {
                println!("try {}\n", action);
                if self.pause > 0 {
                    sleep(Duration::from_millis(self.pause as u64));
                }
            }

            match changer.apply(&entry.board, &action) {
                ChangeResult::None => {}
                ChangeResult::Valid(after, _) => {
                    if self.log {
                        print_all_and_single_candidates(&after);
                    }

                    if after.is_fully_solved() {
                        solutions.push(*after);

                        if self.log {
                            println!("found solution {}\n", solutions.len());
                        }

                        if single {
                            return BruteForceResult::Solved(Box::new(solutions.remove(0)));
                        }

                        if solutions.len() >= self.max_solutions {
                            return BruteForceResult::MultipleSolutions(solutions);
                        }

                            stack.pop();
                            continue;
                    } else {
                        stack.push(Entry::new(after));
                    }
                }
                ChangeResult::Invalid(_, _, _, errors) => {
                    if self.log {
                        println!("failed\n");
                        errors.print_errors();
                    }
                }
            }
        }

        match solutions.len() {
            0 => BruteForceResult::Unsolvable,
            1 => BruteForceResult::Solved(Box::new(solutions.remove(0))),
            _ => BruteForceResult::MultipleSolutions(solutions),
        }
    }
}

pub enum BruteForceResult {
    AlreadySolved,
    TooFewKnowns,
    UnsolvableCells(CellSet),
    Canceled,
    Unsolvable,
    Solved(Box<Board>),
    MultipleSolutions(Vec<Board>),
}

// Internal stack entry for DFS
struct Entry {
    board: Board,
    cell: Cell,
    candidates: KnownSet,
}

impl Entry {
    pub fn new(board: Board) -> Self {
        let cell = board.unknowns().first().expect("no unknown cells");
        let candidates = board.candidates(cell);

        Self {
            board,
            cell,
            candidates,
        }
    }
}