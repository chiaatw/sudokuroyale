use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::io::{show_progress, Cancelable};
use crate::layout::values::known_set::KnownSetLike;
use crate::layout::{Cell, Known, KnownSet};
use crate::puzzle::{Board, ChangeResult, Changer, Strategy};


// Erzeugt eine vollständige Puzzle-Lösung
pub struct Generator {
    rng: ThreadRng,
    bar: bool,
}

impl Generator {
    pub fn new(_shuffle: bool, bar: bool) -> Generator {
        Generator {
            rng: rand::thread_rng(),
            bar,
        }
    }

    pub fn generate(&mut self, changer: &Changer, cancelable: &Cancelable) -> Option<Board> {
        let mut stack: Vec<Entry> = Vec::with_capacity(81);

        let start = Board::new();
        let first = self.pick_next_cell(&start)?;
        stack.push(Entry {
            board: start,
            cell: first,
            candidates: self.shuffle_candidates(KnownSet::full()),
        });

        while let Some(Entry {
            board,
            cell,
            mut candidates,
        }) = stack.pop()
        {
            if self.bar {
                let filled = board.known_count().min(81);
                show_progress(filled, 81);
            }

            if cancelable.is_canceled() {
                return Some(board);
            }

            // keine Kandidaten mehr -> backtrack
            let known = match candidates.pop() {
                Some(k) => k,
                None => continue,
            };

            let next_board = match changer.set_known(&board, Strategy::BruteForce, cell, known) {
                ChangeResult::None => {
                    stack.push(Entry {
                        board,
                        cell,
                        candidates,
                    });
                    continue;
                }
                ChangeResult::Invalid(..) => {
                    stack.push(Entry {
                        board,
                        cell,
                        candidates,
                    });
                    continue;
                }
                ChangeResult::Valid(after, _) => after,
            };

            if next_board.known_count() == 81 {
                return Some(next_board);
            }

            stack.push(Entry {
                board,
                cell,
                candidates,
            });

            let next_cell = match self.pick_next_cell(&next_board) {
                Some(c) => c,
                None => continue,
            };

            let next_cands = next_board.candidates(next_cell);
            if next_cands.len() == 0 {
                continue;
            }

            stack.push(Entry {
                board: next_board,
                cell: next_cell,
                candidates: self.shuffle_candidates(next_cands),
            });
        }

        None
    }

    fn pick_next_cell(&mut self, board: &Board) -> Option<Cell> {
        let mut best: Option<(Cell, usize)> = None;

        for cell in board.unknowns().iter() {
            let n = board.candidates(cell).len();
            if n == 0 {
                return None;
            }
            match best {
                None => best = Some((cell, n)),
                Some((_, best_n)) if n < best_n => best = Some((cell, n)),
                _ => {}
            }
            if n == 1 {
                break;
            }
        }

        best.map(|(c, _)| c)
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

    use crate::layout::values::known_set::KnownSetLike;
    use crate::puzzle::Options;
    use crate::puzzle::Strategy;

    #[test]
    fn test_generate_returns_some_board() {
        let mut generator = Generator::new(false, false);

        let changer = Changer::new(Options::errors());

        let cancelable = Cancelable::new();

        let board = generator.generate(&changer, &cancelable);
        assert!(
            board.is_some(),
            "Generator should return a board (or partial board) in normal operation"
        );
    }

    

    #[test]
    fn test_shuffle_candidates_is_permutation() {
        let mut generator = Generator::new(true, false);

        let candidates = KnownSet::full();
        let shuffled = generator.shuffle_candidates(candidates);

        assert_eq!(shuffled.len(), KnownSet::full().len());

        let mut a = shuffled.clone();
        a.sort();

        let mut b: Vec<Known> = KnownSet::full().iter().collect();
        b.sort();

        assert_eq!(
            a, b,
            "Shuffled candidates should contain exactly the same values"
        );
    }

    #[test]
    #[ignore = "Enable once Cancelable can be triggered from tests (e.g., cancelable.cancel())"]
    fn test_generate_partial_board_on_cancel() {
        let mut generator = Generator::new(false, false);
        let changer = Changer::new(Options::errors());

        let cancelable = Cancelable::new();

        let result = generator.generate(&changer, &cancelable);
        assert!(result.is_some());
    }

    #[test]
    fn test_generate_returns_fully_solved_board_when_not_canceled() {
        let mut generator = Generator::new(false, false);
        let changer = Changer::new(Options::errors());
        let cancelable = Cancelable::new();

        let board = generator.generate(&changer, &cancelable).expect("expected Some(board)");

        assert_eq!(board.known_count(), 81, "generated board should have 81 known cells");
        assert!(board.is_fully_solved(), "generated board should be fully solved");
    }

    #[test]
    fn test_pick_next_cell_on_empty_board_returns_some_cell() {
        let mut generator = Generator::new(false, false);
        let board = Board::new();

        let cell = generator.pick_next_cell(&board).expect("empty board should have a next cell");
        assert!(board.is_unknown(cell));
        assert!(board.candidates(cell).len() > 0);
    }

    #[test]
    fn test_pick_next_cell_prefers_cell_with_single_candidate() {
        let changer = Changer::new(Options::errors());
        let mut board = Board::new();

        for col in 0u8..8u8 {
            let cell = Cell::new(col);  
            let known = Known::new(col + 1); 

            board = match changer.set_known(&board, Strategy::BruteForce, cell, known) {
                ChangeResult::Valid(after, _) => after,
                _other => panic!("expected Valid when setting known, got unexpected result"),
            };
        }

        let mut generator = Generator::new(false, false);
        let picked = generator
            .pick_next_cell(&board)
            .expect("should pick a next cell");

        let expected = Cell::new(8);
        assert_eq!(
            picked, expected,
            "generator should pick the cell with the fewest candidates (here: 1 candidate)"
        );

        let cands = board.candidates(picked);
        assert_eq!(cands.len(), 1, "expected exactly 1 candidate for the picked cell");
        let only = cands.iter().next().unwrap();
        assert_eq!(only, Known::new(9), "expected the only candidate to be 9");
    }

    #[test]
    fn test_shuffle_candidates_subset_is_permutation_of_subset() {
        let mut generator = Generator::new(true, false);

        let subset = KnownSet::empty() + Known::new(2) + Known::new(5) + Known::new(9);
        let shuffled = generator.shuffle_candidates(subset);

        assert_eq!(shuffled.len(), 3);

        let mut a = shuffled.clone();
        a.sort();

        let mut b = vec![Known::new(2), Known::new(5), Known::new(9)];
        b.sort();

        assert_eq!(a, b, "shuffled subset should contain exactly the same elements");
    }

    #[test]
    fn test_generate_board_has_no_unknown_cells() {
        let mut generator = Generator::new(false, false);
        let changer = Changer::new(Options::errors());
        let cancelable = Cancelable::new();

        let board = generator.generate(&changer, &cancelable).expect("expected Some(board)");

        for cell in Cell::iter() {
            assert!(board.is_known(cell), "cell {:?} should be known", cell);
        }
    }
}
