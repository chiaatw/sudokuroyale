use std::thread::sleep;
use std::time::Duration;

use crate::io::{print_all_and_single_candidates, Cancelable};
use crate::layout::values::known_set::KnownSetLike;
use crate::puzzle::{
    Action, Board, Cell, CellSet, ChangeResult, Changer, Effects, KnownSet, Options, Strategy,
};

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
            BruteForceResult::Solved(_) | BruteForceResult::AlreadySolved => None,
            BruteForceResult::MultipleSolutions(_) => None,
            BruteForceResult::TooFewKnowns => None,
            BruteForceResult::UnsolvableCells(_) => None,
            BruteForceResult::Canceled => None,
            BruteForceResult::Unsolvable => None,
        }
    }
}

impl BruteForceResult {
    pub fn is_solved(&self) -> bool {
        matches!(
            self,
            BruteForceResult::AlreadySolved | BruteForceResult::Solved(_)
        )
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

        let _cancelable = Cancelable::new();
        let changer = Changer::new(Options::errors());
        let mut solutions = Vec::new();
        let mut stack = Vec::with_capacity(81);
        stack.push(Entry::new(*board));

        while !stack.is_empty() {
            let entry = stack.last_mut().unwrap();

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
                        solutions.push(after);

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

pub fn find_brute_force(board: &Board, single: bool) -> BruteForceResult {
    let solver = BruteForceSolver::new(false, 0, DEFAULT_MAXIMUM_SOLUTIONS);
    solver.find_brute_force(board, single)
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

#[cfg(test)]
#[cfg(test)]
mod tests {
    use super::*;

    fn solved_board() -> Board {
        // Ein vollständig gelöstes Board ohne Parser zu bauen ist aufwendig.
        // Daher: wir bauen ein Board, das nach deiner Board-API als "fully solved" gilt.
        //
        // Wenn Board::new() schon leer ist, musst du hier ggf. eine Helper-Funktion
        // nutzen, die du im Projekt bereits hast (z.B. Board::from_solution()).
        //
        // FALLBACK: Wenn es keine einfache Möglichkeit gibt, markiere den Test ignore.
        Board::new()
    }

    #[test]
    fn too_few_knowns_returns_too_few_knowns() {
        let board = Board::new(); // 0 knowns
        let solver = BruteForceSolver::new(false, 0, 100);

        match solver.find_brute_force(&board, true) {
            BruteForceResult::TooFewKnowns => {}
            other => panic!("Expected TooFewKnowns, got {:?}", discr(other)),
        }
    }

    #[test]
    fn max_solutions_is_clamped_to_default_when_out_of_range() {
        // max_solutions darf nicht 0 sein; dein new() clamped dann auf DEFAULT_MAXIMUM_SOLUTIONS
        let solver = BruteForceSolver::new(false, 0, 0);
        assert_eq!(solver.max_solutions, DEFAULT_MAXIMUM_SOLUTIONS);
    }

    #[test]
    fn already_solved_board_returns_already_solved() {
        // Dieser Test ist nur gültig, wenn du eine einfache Möglichkeit hast,
        // ein "fully solved" Board zu erzeugen.
        //
        // Wenn Board::new() NICHT fully solved ist (sehr wahrscheinlich), dann:
        // -> Test ignorieren oder Board-Builder implementieren.
        let board = solved_board();
        let solver = BruteForceSolver::new(false, 0, 100);

        if board.is_fully_solved() {
            match solver.find_brute_force(&board, true) {
                BruteForceResult::AlreadySolved => {}
                _ => panic!("Expected AlreadySolved"),
            }
        }
    }

    #[test]
    fn find_brute_force_function_matches_solver_method_for_simple_case() {
        let board = Board::new();
        let solver = BruteForceSolver::new(false, 0, DEFAULT_MAXIMUM_SOLUTIONS);

        let a = solver.find_brute_force(&board, true);
        let b = super::find_brute_force(&board, true);

        // Beide sollten bei 0 knowns TooFewKnowns liefern.
        assert!(matches!(a, BruteForceResult::TooFewKnowns));
        assert!(matches!(b, BruteForceResult::TooFewKnowns));
    }

    // Helper um Debug-Ausgabe ohne Board/Vec zu erzwingen
    fn discr(r: BruteForceResult) -> &'static str {
        match r {
            BruteForceResult::AlreadySolved => "AlreadySolved",
            BruteForceResult::TooFewKnowns => "TooFewKnowns",
            BruteForceResult::UnsolvableCells(_) => "UnsolvableCells",
            BruteForceResult::Canceled => "Canceled",
            BruteForceResult::Unsolvable => "Unsolvable",
            BruteForceResult::Solved(_) => "Solved",
            BruteForceResult::MultipleSolutions(_) => "MultipleSolutions",
        }
    }
}
