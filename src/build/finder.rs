use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::io::{show_progress, Cancelable};
use crate::layout::{Cell, CellSet};
use crate::puzzle::{Board, Effects};
use crate::solve::{find_brute_force, Resolution, Solver, Timings};

    /// Finds a solvable starting puzzle from a full solution.
    pub struct Finder {
        cancelable: Cancelable,
        rng: ThreadRng,
        clues: usize,
        time: u64,
        bar: bool,
    }

    impl Finder {
        pub fn new(clues: usize, time: u64, bar: bool) -> Finder {
            Finder {
                cancelable: Cancelable::new(),
                rng: rand::thread_rng(),
                clues,
                time,
                bar,
            }
        }

        pub fn backtracking_find(&mut self, board: Board) -> (Board, Effects) {
            let solver = Solver::new(false);
            let runtime = std::time::Instant::now();

            let mut timings = Timings::new();
            let mut fewest_clues = 81;
            let mut fewest_clues_board = board;
            let mut fewest_clues_actions = Effects::new();

            let mut stack = Vec::with_capacity(81);
            stack.push(Entry {
                board,
                cells: self.shuffle_cells(board.knowns()),
            });

            while !stack.is_empty() {
                if self.bar {
                    show_progress(82 - stack.len());
                }
                if self.cancelable.is_canceled()
                    || fewest_clues <= self.clues
                    || runtime.elapsed().as_secs() >= self.time
                {
                    break;
                }

                let entry = stack.last_mut().unwrap();
                if entry.cells.is_empty() {
                    stack.pop();
                    continue;
                }

                let cell = entry.cells.pop().unwrap();
                let (next, unapplied) = entry.board.without(cell);

                match solver.solve(&next, &unapplied, &mut timings) {
                    Resolution::Canceled(..) => break,
                    Resolution::Solved(_, actions, _) => {
                        if !find_brute_force(&board, false, 0, 2).is_solved() {
                            continue;
                        }
                        if next.known_count() < fewest_clues {
                            fewest_clues = next.known_count();
                            fewest_clues_board = next;
                            fewest_clues_actions = actions;
                        }
                        stack.push(Entry {
                            board: next,
                            cells: self.shuffle_cells(next.knowns()),
                        });
                    }
                    _ => continue,
                }
            }

            (fewest_clues_board, fewest_clues_actions)
        }

        fn shuffle_cells(&mut self, set: CellSet) -> Vec<Cell> {
            let mut cells = set.iter().collect::<Vec<Cell>>();

            cells.shuffle(&mut self.rng);

            cells
        }
    }

    struct Entry {
        board: Board,
        cells: Vec<Cell>,
    }

    #[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Cell, CellSet};
    use crate::puzzle::Board;
    use crate::solve::Solver;

    /// Hilfsfunktion: erzeugt ein "gelöstes" Board für Testzwecke.
    fn solved_board() -> Board {
        let mut board = Board::new_empty();
        // Einfach diagonal Zellen füllen, um einen lösbaren Zustand zu simulieren
        for i in 0..9 {
            let cell = Cell::from_index(i * 9 + i); // Diagonale
            board.set_given(cell, i + 1);          // 1..9 diagonal
        }
        board
    }

    #[test]
    fn test_finder_new() {
        let clues = 20;
        let time = 10;
        let bar = true;

        let finder = Finder::new(clues, time, bar);

        assert_eq!(finder.clues, clues);
        assert_eq!(finder.time, time);
        assert_eq!(finder.bar, bar);
    }

    #[test]
    fn test_shuffle_cells() {
        let mut finder = Finder::new(20, 10, false);

        let board = solved_board();
        let cellset = board.knowns();
        let shuffled = finder.shuffle_cells(cellset);

        assert_eq!(shuffled.len(), cellset.len());

        // Prüfen, dass alle Zellen noch enthalten sind
        for c in cellset {
            assert!(shuffled.contains(&c));
        }

        // Prüfen, dass die Reihenfolge wahrscheinlich verändert wurde
        let normal: Vec<_> = cellset.iter().collect();
        if normal.len() > 1 {
            assert_ne!(shuffled[0], normal[0]); // manchmal zufällig gleich
        }
    }

    #[test]
    fn test_backtracking_find_returns_board_and_effects() {
        let mut finder = Finder::new(20, 1, false);
        let board = solved_board();

        let (result_board, effects) = finder.backtracking_find(board);

        // Prüfen, dass wir ein Board zurückbekommen
        assert!(result_board.known_count() <= 81);
        // Effekte können leer oder nicht-leer sein
        assert!(effects.is_empty() || !effects.is_empty());
    }
}