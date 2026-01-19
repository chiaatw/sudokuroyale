    use rand::rngs::ThreadRng;
    use rand::seq::SliceRandom;

    use crate::io::{show_progress, Cancelable};
    use crate::layout::{Cell, CellSet};
    use crate::puzzle::{Board, Effects};
    use crate::solve::strategy_ord::solver::{Resolution, Solver};
    use crate::solve::strategy_ord::timing::Timings;
    use crate::solve::strategy_ord::algorithms::find_brute_force;

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
                   show_progress(82 - stack.len(), 82);
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
                        if !find_brute_force(&board, false).is_solved() {
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
    fn test_shuffle_cells_returns_permutation() {
        let mut finder = Finder::new(20, 10, false);

        // Wir brauchen nur irgendein Set an Zellen; board.knowns() ist ok,
        // selbst wenn es leer ist.
        let board = Board::new();
        let set = board.knowns();

        let shuffled = finder.shuffle_cells(set);

        // Gleiche Anzahl Elemente
        assert_eq!(shuffled.len(), set.iter().count());

        // Alle Zellen aus set müssen enthalten sein
        for c in set.iter() {
            assert!(shuffled.contains(&c));
        }
    }

    #[test]
    fn test_backtracking_find_does_not_panic_on_empty_board() {
        // Wir testen hier bewusst nur: läuft durch und gibt ein Ergebnis zurück,
        // ohne Parser/solved-board Konstruktion.
        let mut finder = Finder::new(20, 0, false); // time=0 -> bricht schnell ab
        let board = Board::new();

        let (result_board, _effects) = finder.backtracking_find(board);

        // Resultat-Board muss gültig sein; minimal prüfen
        assert!(result_board.known_count() <= 81);
    }
}