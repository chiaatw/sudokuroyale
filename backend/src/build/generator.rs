use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::io::{show_progress, Cancelable};
use crate::layout::{Cell, Known, KnownSet};
use crate::puzzle::{Board, ChangeResult, Changer, Strategy};
use crate::layout::values::known_set::KnownSetLike;
/* use crate::solve::strategy_ord::algorithms::find_intersection_removals; */

/// Generates a complete puzzle solution.
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

        while let Some(Entry { board, cell, mut candidates }) = stack.pop() {
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

            // wir probieren "known" in dieser Zelle
            let next_board = match changer.set_known(&board, Strategy::BruteForce, cell, known) {
                ChangeResult::None => {
                    stack.push(Entry { board, cell, candidates });
                    continue;
                }        // das ist eher "hard fail"
                ChangeResult::Invalid(..) => {
                    // andere Kandidaten derselben Zelle probieren
                    stack.push(Entry { board, cell, candidates });
                    continue;
                }
                ChangeResult::Valid(after, _) => after,
            };

            // ✅ Intersection removals erstmal AUS (später wieder rein, wenn Generator stabil)
            // if let Some(effects) = find_intersection_removals(&next_board, false) {
            //     if effects.apply_all(&mut next_board).is_some() {
            //         stack.push(Entry { board, cell, candidates });
            //         continue;
            //     }
            // }

            // solved?
            if next_board.known_count() == 81 {
                return Some(next_board);
            }

            // Wichtig: aktuellen Zustand mit restlichen Kandidaten zurückpushen
            stack.push(Entry { board, cell, candidates });

            // nächste Zelle (MRV)
            let next_cell = match self.pick_next_cell(&next_board) {
                Some(c) => c,
                None => continue, // dead end
            };

            let next_cands = next_board.candidates(next_cell);
            if next_cands.len() == 0 {
                continue; // dead end
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
                return None; // dead end
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

/*     fn all_cells(&mut self) -> Vec<Cell> {
        let mut cells: Vec<Cell> = Vec::with_capacity(81);
        for i in 0..81 {
            cells.push(Cell::new(i));
        }
        if self.shuffle {
            cells.shuffle(&mut self.rng);
        }
        cells
    } */

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

    #[test]
    fn test_generate_returns_some_board() {
        let mut generator = Generator::new(false, false);

        // Echter Changer aus deinem Projekt (kein Dummy Trait)
        let changer = Changer::new(Options::errors());

        // NEU: Cancelable wird von außen übergeben
        let cancelable = Cancelable::new();

        let board = generator.generate(&changer, &cancelable);
        assert!(
            board.is_some(),
            "Generator should return a board (or partial board) in normal operation"
        );
    }

/*     #[test]
    fn test_all_cells_length_is_81() {
        let mut generator = Generator::new(false, false);
        let cells = generator.all_cells();
        assert_eq!(cells.len(), 81);
    } */

    #[test]
    fn test_shuffle_candidates_is_permutation() {
        let mut generator = Generator::new(true, false);

        let candidates = KnownSet::full();
        let shuffled = generator.shuffle_candidates(candidates);

        // gleiche Anzahl
        assert_eq!(shuffled.len(), KnownSet::full().len());

        // gleiche Elemente (Permutation), ohne flaky "order changed" assertion
        let mut a = shuffled.clone();
        a.sort();

        let mut b: Vec<Known> = KnownSet::full().iter().collect();
        b.sort();

        assert_eq!(
            a, b,
            "Shuffled candidates should contain exactly the same values"
        );
    }

    // Cancel-Tests sind ohne steuerbaren Cancelable schwer und tendieren zu Flakes.
    // ABER: durch Dependency Injection (Cancelable von außen) ist es prinzipiell testbar,
    // sofern Cancelable in deinem Projekt eine Möglichkeit bietet, "cancel" auszulösen.
    //
    // Wenn es z.B. `cancelable.cancel()` gibt, kannst du den Test ent-ignorieren
    // und entsprechend anpassen.
    #[test]
    #[ignore = "Enable once Cancelable can be triggered from tests (e.g., cancelable.cancel())"]
    fn test_generate_partial_board_on_cancel() {
        let mut generator = Generator::new(false, false);
        let changer = Changer::new(Options::errors());

        let cancelable = Cancelable::new();
        // cancelable.cancel(); // <- falls es so eine Methode gibt

        let result = generator.generate(&changer, &cancelable);
        assert!(result.is_some());
    }
}
