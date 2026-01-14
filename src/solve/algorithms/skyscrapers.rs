use super::*;

// Solver wrapper for the Skyscraper strategy
pub struct SkyscraperSolver;

impl Solver for SkyscraperSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::Skyscraper
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        self.find_skyscrapers(board, single)
    }
}

impl SkyscraperSolver {
    fn find_skyscrapers(&self, board: &Board, single: bool) -> Option<Effects> {
        let mut effects = Effects::new();

// Check rows first, then columns
        if !self.check_houses(board, single, House::all_rows(), Shape::Column, &mut effects) {
            self.check_houses(board, single, House::all_columns(), Shape::Row, &mut effects);
        }

        if effects.has_actions() {
            Some(effects)
        } else {
            None
        }
    }

    fn check_houses(
        &self,
        board: &Board,
        single: bool,
        houses: HouseSet,
        cross: Shape,
        effects: &mut Effects,
    ) -> bool {
        for known in Known::iter() {
            let candidate_cells = board.candidate_cells(known);

// Closure for checking a candidate skyscraper
            let mut check_candidate = |f1: Cell, c1: Cell, f2: Cell, c2: Cell| -> bool {
                if c1.house(cross) == c2.house(cross) {
// degenerate X-Wing
                    return false;
                }
                if (candidate_cells & f1.house(cross).cells()).len() == 2 {
// degenerate Singles Chain
                    return false;
                }

                let candidates = c1.peers() & c2.peers() & candidate_cells;
                if candidates.is_empty() {
                    return false;
                }

                let mut action = Action::new(Strategy::Skyscraper);
                action.erase_cells(candidates, known);
                action.clue_cell_for_known(Verdict::Secondary, f1, known);
                action.clue_cell_for_known(Verdict::Secondary, c2, known);
                action.clue_cell_for_known(Verdict::Tertiary, f2, known);
                action.clue_cell_for_known(Verdict::Tertiary, c1, known);

                effects.add_action(action) && single
            };

            for pair in houses
                .iter()
                .map(|house| board.house_candidate_cells(house, known))
                .filter(|cells| cells.len() == 2)
                .combinations(2)
            {
                let (c11, c12) = pair[0].as_pair().unwrap();
                let (c21, c22) = pair[1].as_pair().unwrap();

                if c11.house(cross) == c21.house(cross) {
                    if check_candidate(c11, c12, c21, c22) {
                        return true;
                    }
                } else if c11.house(cross) == c22.house(cross) {
                    if check_candidate(c11, c12, c22, c21) {
                        return true;
                    }
                } else if c12.house(cross) == c21.house(cross) {
                    if check_candidate(c12, c11, c21, c22) {
                        return true;
                    }
                } else if c12.house(cross) == c22.house(cross) {
                    if check_candidate(c12, c11, c22, c21) {
                        return true;
                    }
                }
            }
        }

        false
    }
}

pub fn find_skyscrapers(board: &Board, single: bool) -> Option<Effects> {
    SkyscraperSolver.apply(board, single)
}

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::known;

    use super::*;

    #[test]
    fn test_skyscraper() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse(
            "697000002001972063003006790912006073746095086579024148693275709024006006870009",
        );

        let solver = SkyscraperSolver;
        if let Some(effects) = solver.apply(&board, true) {
            // You can inspect effects here; check count > 0
            assert!(effects.has_actions());
        } else {
            panic!("Skyscraper not found");
        }
    }
    #[test]
    fn simple_skyscraper_elimination() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse(
            "697000002001972063003006790912006073746095086579024148693275709024006006870009",
        );

        let solver = SkyscraperSolver;
        let effects = solver.apply(&board, false).unwrap();

        // Prüft, dass mindestens eine Kandidatenelimination erfolgt
        assert!(effects.has_actions());

        // Optional: Konkrete Zellen prüfen, die vom Skyscraper betroffen sind
        let erased_cells = effects.erases_from_cells(known!("9"));
        assert!(!erased_cells.is_empty(), "Skyscraper should erase some 9s");
    }

    #[test]
    fn no_skyscraper_returns_none() {
        let board = Board::new(); // leeres Board → keine Skyscraper möglich
        let solver = SkyscraperSolver;
        assert!(solver.apply(&board, false).is_none());
    }

    #[test]
    fn degenerate_skyscraper_ignored() {
        let mut board = Board::new();

        // Konfiguration, die wie ein X-Wing aussieht, aber degenerate ist
        board.set_candidates(cell!("A1"), known!("5"), &mut Effects::new());
        board.set_candidates(cell!("A2"), known!("5"), &mut Effects::new());
        board.set_candidates(cell!("B1"), known!("5"), &mut Effects::new());
        board.set_candidates(cell!("B2"), known!("5"), &mut Effects::new());

        let solver = SkyscraperSolver;
        let effects = solver.apply(&board, false);
        assert!(effects.is_none(), "Degenerate skyscraper should be ignored");
    }
}