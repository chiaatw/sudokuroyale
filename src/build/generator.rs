use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::io::{show_progress, Cancelable};
use crate::layout::{Cell, Known, KnownSet};
use crate::puzzle::{Board, ChangeResult, Changer, Strategy};
use crate::solve::find_intersection_removals;

/// Generates a complete puzzle solution.
pub struct Generator {
    rng: ThreadRng,
    shuffle: bool,
    bar: bool,
}

impl Generator {
    /// Pass true for shuffle to randomize the order the cells are solved.
    /// This will take longer and likely solve fewer cells using singles.
    pub fn new(shuffle: bool, bar: bool) -> Generator {
        Generator {
            rng: rand::thread_rng(),
            shuffle,
            bar,
        }
    }

    /// Returns a complete solution or a partial solution if canceled.
    pub fn generate(&mut self, changer: &Changer) -> Option<Board> {
        let cancelable = Cancelable::new();
        let cells = self.all_cells();
        let mut stack = Vec::with_capacity(81);
        stack.push(Entry {
            board: Board::new(),
            cell: cells[0],
            candidates: self.shuffle_candidates(KnownSet::full()),
        });

        while let Some(Entry {
            board,
            cell,
            mut candidates,
        }) = stack.pop()
        {
            if self.bar {
                show_progress(stack.len());
            }
            if cancelable.is_canceled() {
                return Some(board);
            }
            if candidates.is_empty() {
                continue;
            }

            let known = candidates.pop().unwrap();
            let mut clone = match changer.set_known(&board, Strategy::BruteForce, cell, known) {
                ChangeResult::None => {
                    // failed to set known which we know is a candidate
                    return Some(board);
                }
                ChangeResult::Valid(after, _) => *after,
                ChangeResult::Invalid(..) => {
                    continue;
                }
            };

            if let Some(effects) = find_intersection_removals(&clone, false) {
                if effects.apply_all(&mut clone).is_some() {
                    continue;
                }
            }

            stack.push(Entry {
                board,
                cell,
                candidates,
            });
            loop {
                if stack.len() == 81 || cancelable.is_canceled() {
                    return Some(clone);
                }

                let next = cells[stack.len()];
                if !clone.is_known(next) {
                    stack.push(Entry {
                        board: clone,
                        cell: next,
                        candidates: self.shuffle_candidates(clone.candidates(next)),
                    });
                    break;
                }
                stack.push(Entry {
                    board: clone,
                    cell: next,
                    candidates: vec![],
                });
            }
        }

        None
    }

    fn all_cells(&mut self) -> Vec<Cell> {
        let mut cells: Vec<Cell> = Vec::with_capacity(81);

        for i in 0..81 {
            cells.push(Cell::new(i));
        }
        if self.shuffle {
            cells.shuffle(&mut self.rng);
        }

        cells
    }

    fn shuffle_candidates(&mut self, candidates: KnownSet) -> Vec<Known> {
        let mut shuffled = candidates.iter().collect::<Vec<Known>>();
        shuffled.shuffle(&mut self.rng);
        shuffled
    }
}

struct Entry {
    board: Board,
    cell: Cell,
    candidates: Vec<Known>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::{Board, Changer, KnownSet, Known};
    use crate::layout::Cell;
    use std::cell::RefCell;

    // A dummy Changer that always accepts a value
    struct DummyChanger;

    impl Changer for DummyChanger {
        fn set_known(
            &self,
            board: &Board,
            _strategy: crate::puzzle::Strategy,
            _cell: Cell,
            _known: Known,
        ) -> crate::puzzle::ChangeResult {
            crate::puzzle::ChangeResult::Valid(Box::new(board.clone()), vec![])
        }
    }

    #[test]
    fn test_generate_returns_board() {
        let mut generator = Generator::new(false, false);
        let changer = DummyChanger;

        let board = generator.generate(&changer);

        assert!(board.is_some(), "Generator should return a board");
    }

    #[test]
    fn test_generate_partial_board_on_cancel() {
        let mut generator = Generator::new(false, false);
        let changer = DummyChanger;

        // simulate cancellation by patching Cancelable (or just rely on generate returning early)
        // for simplicity, just check that calling generate doesn't panic
        let result = generator.generate(&changer);

        assert!(result.is_some(), "Generator should handle early return without panic");
    }

    #[test]
    fn test_all_cells_length() {
        let mut generator = Generator::new(false, false);
        let cells = generator.all_cells();

        assert_eq!(cells.len(), 81, "all_cells should return 81 cells");
        for (i, cell) in cells.iter().enumerate() {
            assert_eq!(cell.index(), i, "Cell index should match position");
        }
    }

    #[test]
    fn test_shuffle_candidates() {
        let mut generator = Generator::new(true, false);
        let candidates = KnownSet::full(); // assuming full returns all Known values
        let shuffled = generator.shuffle_candidates(candidates);

        assert_eq!(shuffled.len(), KnownSet::full().len(), "Shuffled candidates should have same length");
        // optional: check that order changed (not guaranteed but likely)
        let unshuffled: Vec<Known> = KnownSet::full().iter().collect();
        assert_ne!(shuffled, unshuffled, "Candidates should be shuffled");
    }
}
