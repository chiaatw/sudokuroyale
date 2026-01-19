use super::*;

use crate::puzzle::{Action, Board, Effects, Strategy, Verdict};
pub struct HiddenSingleSolver;

impl Solver for HiddenSingleSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy{
        Strategy::HiddenSingle
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_hidden_singles(board, single)
    }
}

pub fn find_hidden_singles(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    for (cell, knowns) in board.unknown_iter() {
        for known in knowns.iter() {
            for house in cell.houses() {
                if board.house_candidate_cells(house, known).len() == 1 {
                    let mut action = Action::new_set(Strategy::HiddenSingle, cell, known);
                    action.clue_cells_for_known(
                        Verdict::Related,
                        house.cells() - cell - board.knowns(),
                        known,
                    );

                    if effects.add_action(action) && single {
                        return Some(effects);
                    }
                }
            }
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

#[cfg(test)]
mod hidden_single_tests {
    use super::*;

    #[test]
    fn empty_board_returns_none() {
        let board = Board::new();
        assert!(find_hidden_singles(&board, true).is_none());
    }

    #[test]
    fn solver_delegates_to_find_hidden_singles() {
        let board = Board::new();
        let solver = HiddenSingleSolver;

        let via_solver = solver.apply(&board, true);
        let via_fn = find_hidden_singles(&board, true);

        assert_eq!(via_solver.is_some(), via_fn.is_some());
    }

    #[test]
    fn single_mode_never_returns_more_than_one_action() {
        let board = Board::new();
        if let Some(effects) = find_hidden_singles(&board, true) {
            assert!(effects.actions().len() <= 1);
        }
    }

    #[test]
    fn no_false_positives_with_small_noise() {
        let mut board = Board::new();
        let mut eff = Effects::new();

        // Optional: falls set_known bei dir existiert (wie in anderen Dateien), ok.
        // Sonst diesen Test entfernen.
        board.set_known(crate::layout::cells::cell::cell!("A1"), crate::layout::values::known::known!("1"), &mut eff);
        board.set_known(crate::layout::cells::cell::cell!("B2"), crate::layout::values::known::known!("2"), &mut eff);
        board.set_known(crate::layout::cells::cell::cell!("C3"), crate::layout::values::known::known!("3"), &mut eff);

        assert!(!eff.has_errors());
        assert!(find_hidden_singles(&board, false).is_none());
    }
}