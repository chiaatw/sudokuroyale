use crate::layout::{Cell, Known};
use crate::puzzle::{Change, Strategy};
use crate::solve::find_intersection_removals;
use crate::solve::strategy_ord::action::AppliesToBoard;

use super::{Action, Board, Effects, Options};

// Indicates the result of a single action
pub enum ChangeResult {
    None,
    Valid(Board, Effects),
    Invalid(Board, Board, Action, Effects),
}

// Applies actions to a board based on the selected options
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Changer {
    pub options: Options,
}

impl Changer {
    pub const fn new(options: Options) -> Self {
        Self {
            options
        }
    }
// Sets the given (clue) for a single cell
    pub fn set_given(
        &self,
        board: &Board,
        strategy: Strategy,
        cell: Cell,
        known: Known,
    ) -> ChangeResult {
        self.apply(board, &Action::new_set(strategy, cell, known))
    }

// Solves a single cell to one of its candidates
    pub fn set_known(
        &self,
        board: &Board,
        strategy: Strategy,
        cell: Cell,
        known: Known,
    ) -> ChangeResult {
        self.apply(board, &Action::new_set(strategy, cell, known))
    }

// Remove a candidate from a single cell
    pub fn remove_candidate(
        &self,
        board: &Board,
        strategy: Strategy,
        cell: Cell,
        known: Known,
    ) -> ChangeResult {
        self.apply(board, &Action::new_erase(strategy, cell, known))
    }

// Applies the given action it creates
    pub fn apply(&self, board: &Board, action: &Action) -> ChangeResult {
        let mut after = *board;
        let mut effects = Effects::new();

        let change = action.apply(&mut after, &mut effects);

        if self.options.stop_on_error() && effects.has_errors() {
            ChangeResult::Invalid(*board, after, action.clone(), effects)
        } else {
            self.apply_all_changed(board, &mut after, &mut effects, change)
        }
    }

// Applies all automatic actions to the given board
    pub fn apply_all(&self, board: &mut Board, actions: &Effects) -> ChangeResult {
        let mut effects = actions.clone();
        let mut after = board.clone();
        self.apply_all_changed(board, &mut after, &mut effects, Change::None)
    }

    fn apply_all_changed(
        &self,
        before: &Board,
        board: &mut Board,
        actions: &mut Effects,
        mut change: Change,
    ) -> ChangeResult {
        let mut good = *board;
        let mut applying = actions.clone();
        let mut unapplied = Effects::new();

        while applying.has_actions() {
            let mut next = Effects::new();

            for action in applying.actions() {
                if self.options.should_apply(action.strategy()) {
                    let mut maybe = good;
                    change &= action.apply(&mut maybe, &mut next);

                    if self.options.stop_on_error() && next.has_errors() {
                        return ChangeResult::Invalid(*before, maybe, action.clone(), next);
                    }

                    if next.has_errors() {
                        eprintln!("warning: action caused errors: {}", action);
                        next.print_errors();
                    }

                    good = maybe;
                } else {
                    unapplied.add_action(action.clone());
                }
            }

            if self.options.solve_intersection_removals() && next.is_empty() {
                if let Some(effects) = find_intersection_removals(&good, false) {
                    next = effects;
                }
            }

            applying = next;
        }

        if change.changed() {
            ChangeResult::Valid(good, unapplied)
        } else {
            ChangeResult::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Cell, Known};
    use crate::puzzle::{Board, Effects, Strategy};
    use crate::solve::strategy_ord::Options;

    fn cell(i: usize) -> Cell {
        Cell::new(i as u8)
    }

    fn known(n: u8) -> Known {
        Known::from(n)
    }

    fn changer_default() -> Changer {
        Changer::new(Options::default())
    }

    fn changer_stop_on_error() -> Changer {
    Changer::new(Options::errors())
    }

    /* ---------------- set_known ---------------- */

    #[test]
    fn set_known_valid() {
        let board = Board::new();
        let changer = changer_default();

        let result = changer.set_known(&board, Strategy::NakedSingle, cell(0), known(1));

        match result {
            ChangeResult::Valid(after, effects) => {
                assert!(after.is_known(cell(0)));
                assert_eq!(after.value(cell(0)).known(), Some(known(1)));
                assert!(effects.is_empty());
            }
            _ => panic!("expected ChangeResult::Valid"),
        }
    }

    #[test]
    fn set_known_no_change_returns_none() {
        let changer = changer_default();
        let mut board = Board::new();
        let mut effects = Effects::new();

        board.set_known(cell(0), known(1), &mut effects);

        let result = changer.set_known(&board, Strategy::NakedSingle, cell(0), known(1));

        matches!(result, ChangeResult::None);
    }

    #[test]
    fn set_known_conflict_invalid() {
        let changer = changer_stop_on_error();
        let mut board = Board::new();
        let mut effects = Effects::new();

        board.set_known(cell(0), known(1), &mut effects);

        let result = changer.set_known(&board, Strategy::NakedSingle, cell(0), known(2));

        match result {
            ChangeResult::Invalid(before, after, action, effects) => {
                assert!(before.is_known(cell(0)));
                assert!(after.is_known(cell(0)));
                assert!(!effects.is_empty());
                assert_eq!(action.strategy(), Strategy::NakedSingle);
            }
            _ => panic!("expected ChangeResult::Invalid"),
        }
    }

    /* ---------------- set_given ---------------- */

    #[test]
    fn set_given_marks_given() {
        let board = Board::new();
        let changer = changer_default();

        let result = changer.set_given(&board, Strategy::Given, cell(1), known(3));

        match result {
            ChangeResult::Valid(after, _) => {
                assert!(after.is_known(cell(1)));
                assert!(after.is_given(cell(1)));
            }
            _ => panic!("expected ChangeResult::Valid"),
        }
    }

    /* ---------------- remove_candidate ---------------- */

    #[test]
    fn remove_candidate_valid() {
        let board = Board::new();
        let changer = changer_default();

        let result =
            changer.remove_candidate(&board, Strategy::LockedCandidates, cell(2), known(4));

        match result {
            ChangeResult::Valid(after, _) => {
                assert!(!after.is_candidate(cell(2), known(4)));
            }
            _ => panic!("expected ChangeResult::Valid"),
        }
    }

    #[test]
    fn remove_candidate_noop_returns_none() {
        let changer = changer_default();
        let mut board = Board::new();
        let mut effects = Effects::new();

        board.remove_candidate(cell(2), known(5), &mut effects);

        let result =
            changer.remove_candidate(&board, Strategy::LockedCandidates, cell(2), known(5));

        matches!(result, ChangeResult::None);
    }

    /* ---------------- apply_all ---------------- */

    #[test]
    fn apply_all_applies_effects() {
        let changer = changer_default();
        let mut board = Board::new();
        let mut effects = Effects::new();

        effects.add_set(Strategy::NakedSingle, cell(3), known(6));

        let result = changer.apply_all(&mut board, &effects);

        match result {
            ChangeResult::Valid(after, unapplied) => {
                assert!(after.is_known(cell(3)));
                assert!(unapplied.is_empty());
            }
            _ => panic!("expected ChangeResult::Valid"),
        }
    }

    #[test]
    fn apply_all_respects_options() {
        let changer = Changer::new(Options::default().set_solve_naked_singles(false));

        let mut board = Board::new();
        let mut effects = Effects::new();
        effects.add_set(Strategy::NakedSingle, cell(4), known(7));

        let result = changer.apply_all(&mut board, &effects);

        match result {
            ChangeResult::Valid(after, unapplied) => {
                assert!(!after.is_known(cell(4)));
                assert!(!unapplied.is_empty());
            }
            _ => panic!("expected ChangeResult::Valid"),
        }
    }

    /* ---------------- safety ---------------- */

    #[test]
    fn changer_does_not_panic_on_empty_actions() {
        let changer = changer_default();
        let mut board = Board::new();
        let effects = Effects::new();

        let result = changer.apply_all(&mut board, &effects);

        matches!(result, ChangeResult::None);
    }
}
