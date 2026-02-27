use super::*;

use crate::layout::{House, HouseSet, Shape};
use crate::puzzle::{Action, Board, Cell, Effects, Known, Strategy, Verdict};
use itertools::Itertools;
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

        if !self.check_houses(
            board,
            single,
            House::all_rows(),
            Shape::Column,
            &mut effects,
        ) {
            self.check_houses(
                board,
                single,
                House::all_columns(),
                Shape::Row,
                &mut effects,
            );
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

            let mut check_candidate = |f1: Cell, c1: Cell, f2: Cell, c2: Cell| -> bool {
                if c1.house(cross) == c2.house(cross) {
                    return false;
                }
                if (candidate_cells & f1.house(cross).cells()).len() == 2 {
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
    use super::*;

    #[test]
    fn empty_board_returns_none() {
        let board = Board::new();
        let solver = SkyscraperSolver;

        assert!(solver.apply(&board, false).is_none());
        assert!(solver.apply(&board, true).is_none());
    }

    #[test]
    fn solver_delegates_and_matches_free_function() {
        let board = Board::new();
        let solver = SkyscraperSolver;

        let via_solver = solver.apply(&board, false);
        let via_fn = find_skyscrapers(&board, false);

        assert_eq!(via_solver.is_some(), via_fn.is_some());
    }

    #[test]
    fn single_mode_never_returns_more_than_one_action() {
        let board = Board::new();
        let solver = SkyscraperSolver;

        if let Some(effects) = solver.apply(&board, true) {
            assert!(effects.actions().len() <= 1);
        }
    }

    #[test]
    fn does_not_panic_on_trivial_board() {
        let board = Board::new();
        let solver = SkyscraperSolver;

        let eff = Effects::new();

        let _ = eff;

        let _ = solver.apply(&board, false);
    }
}
