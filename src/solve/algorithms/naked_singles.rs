use super::*;

use crate::puzzle::{Action, KnownSet, Board, Effects, Strategy, Verdict};

pub struct NakedSingleSolver;

impl Solver for NakedSingleSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::NakedSingle
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_naked_singles(board, single)
    }
}

pub fn find_naked_singles(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    for (cell, knowns) in board.cell_candidates_with_n_candidates(1) {
        let known = knowns.as_single().unwrap();

        let mut action = Action::new_set(Strategy::NakedSingle, cell, known);
        action.clue_cell_for_knowns(
            Verdict::Related,
            cell,
            KnownSet::full() - known,
        );

        if effects.add_action(action) && single {
            return Some(effects);
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

#[cfg(test)]
mod naked_single_tests {
    use super::*;

    #[test]
    fn empty_board_has_no_naked_singles() {
        let board = Board::new();
        assert!(find_naked_singles(&board, false).is_none());
    }

    #[test]
    fn solver_delegates_to_find() {
        let board = Board::new();
        let solver = NakedSingleSolver;

        let via_solver = solver.apply(&board, true);
        let via_fn = find_naked_singles(&board, true);

        assert_eq!(via_solver.is_some(), via_fn.is_some());
    }

    #[test]
    fn single_mode_never_returns_more_than_one_action() {
        let board = Board::new();
        if let Some(effects) = find_naked_singles(&board, true) {
            assert!(effects.actions().len() <= 1);
        }
    }
}