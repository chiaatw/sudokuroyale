use std::collections::HashMap;
use std::fmt;

use super::{Action, Board, Change, Error, Strategy};
use crate::layout::{Cell, CellSet, Known, KnownSet};
use crate::solve::strategy_ord::action::AppliesToBoard;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Effects {
    errors: Vec<Error>,
    actions: Vec<Action>,
}

pub type Result = std::result::Result<Effects, Effects>;

impl Effects {
    #[inline]
    pub const fn new() -> Self {
        Self {
            errors: Vec::new(),
            actions: Vec::new(),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty() && self.actions.is_empty()
    }

    #[inline]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    #[inline]
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    #[inline]
    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    #[inline]
    pub fn errors(&self) -> &[Error] {
        &self.errors
    }

    #[inline]
    pub fn errors_iter(&self) -> impl Iterator<Item = &Error> {
        self.errors.iter()
    }

    #[inline]
    pub fn add_error(&mut self, error: Error) {
        self.errors.push(error);
    }

    pub fn print_errors(&self) {
        for error in &self.errors {
            println!("- {}", error);
        }
    }

    #[inline]
    pub fn has_actions(&self) -> bool {
        !self.actions.is_empty()
    }

    #[inline]
    pub fn action_count(&self) -> usize {
        self.actions.len()
    }

    pub fn action_counts(&self) -> HashMap<Strategy, i32> {
        let mut counts = HashMap::new();
        for action in &self.actions {
            *counts.entry(action.strategy()).or_default() += 1;
        }
        counts
    }

    #[inline]
    pub fn clear_actions(&mut self) {
        self.actions.clear();
    }

    #[inline]
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    pub fn add_action(&mut self, action: Action) -> bool {
        if action.is_empty() {
            false
        } else {
            self.actions.push(action);
            true
        }
    }

    #[inline]
    pub fn add_set(&mut self, strategy: Strategy, cell: Cell, known: Known) {
        self.add_action(Action::new_set(strategy, cell, known));
    }

    #[inline]
    pub fn add_erase(&mut self, strategy: Strategy, cell: Cell, known: Known) {
        self.add_action(Action::new_erase(strategy, cell, known));
    }

    #[inline]
    pub fn add_erase_cells(&mut self, strategy: Strategy, cells: CellSet, known: Known) {
        self.add_action(Action::new_erase_cells(strategy, cells, known));
    }

    #[inline]
    pub fn add_erase_knowns(&mut self, strategy: Strategy, cell: Cell, knowns: KnownSet) {
        self.add_action(Action::new_erase_knowns(strategy, cell, knowns));
    }

    #[inline]
    pub fn erases(&self, cell: Cell, known: Known) -> bool {
        self.actions.iter().any(|a| a.erases(cell, known))
    }

    pub fn erases_from_cells(&self, known: Known) -> CellSet {
        self.actions
            .iter()
            .fold(CellSet::empty(), |acc, a| acc | a.erases_from_cells(known))
    }

    pub fn erases_knowns_from(&self, cell: Cell) -> KnownSet {
        self.actions
            .iter()
            .fold(KnownSet::empty(), |acc, a| acc | a.erases_knowns_from(cell))
    }

    pub fn affecting_cell(&self, cell: Cell) -> Self {
        let mut effects = Self {
            errors: Vec::new(),
            actions: Vec::with_capacity(self.actions.len()),
        };
        for action in &self.actions {
            if action.affects_cell(cell) {
                effects.actions.push(action.clone());
            }
        }
        effects
    }

    pub fn affecting_known(&self, known: Known) -> Self {
        let mut effects = Self {
            errors: Vec::new(),
            actions: Vec::with_capacity(self.actions.len()),
        };
        for action in &self.actions {
            if action.affects_known(known) {
                effects.actions.push(action.clone());
            }
        }
        effects
    }

    pub fn without_action(&self, index: usize) -> Self {
        let mut effects = self.clone();
        effects.actions.remove(index);
        effects
    }

    pub fn take_actions(&mut self, mut from: Effects) {
        self.actions.append(&mut from.actions);
    }

    pub fn apply(&self, board: &mut Board, effects: &mut Effects) -> Change {
        self.actions
            .iter()
            .fold(Change::None, |chg, a| chg & a.apply(board, effects))
    }

    pub fn apply_strategy(
        &self,
        board: &mut Board,
        strategy: Strategy,
        effects: &mut Effects,
    ) -> Change {
        self.actions.iter().fold(Change::None, |chg, a| {
            if a.has_strategy(strategy) {
                chg & a.apply(board, effects)
            } else {
                chg
            }
        })
    }

    pub fn apply_all(&self, board: &mut Board) -> Option<Effects> {
        if self.has_errors() {
            return Some(self.clone());
        }

        if self.has_actions() {
            let mut next = Effects::new();
            self.apply(board, &mut next);
            if next.has_errors() {
                return Some(next);
            }
        }

        None
    }

    pub fn apply_all_strategy(&self, board: &mut Board, strategy: Strategy) -> Option<Effects> {
        let mut effects = self.clone();
        loop {
            if effects.has_errors() {
                return Some(effects);
            }
            if effects.actions.is_empty() {
                return None;
            }
            let mut next = Effects::new();
            effects.apply_strategy(board, strategy, &mut next);
            effects = next;
        }
    }

    pub fn print_actions(&self) {
        for action in &self.actions {
            println!("- {}", action);
        }
    }
}

impl From<Action> for Effects {
    fn from(action: Action) -> Self {
        let mut effects = Effects::new();
        effects.add_action(action);
        effects
    }
}

impl fmt::Display for Effects {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.errors.is_empty() {
            writeln!(f, "Errors:")?;
            for error in &self.errors {
                writeln!(f, "- {}", error)?;
            }
        }

        if !self.actions.is_empty() {
            if !self.errors.is_empty() {
                writeln!(f)?;
            }
            writeln!(f, "Actions:")?;
            for action in &self.actions {
                writeln!(f, "- {}", action)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Cell, Known};

    fn cell(i: usize) -> Cell {
        Cell::new(i as u8)
    }

    fn known(n: u8) -> Known {
        Known::new(n)
    }

    fn action_set(c: Cell, k: Known) -> Action {
        Action::new_set(Strategy::NakedSingle, c, k)
    }

    fn action_erase(c: Cell, k: Known) -> Action {
        Action::new_erase(Strategy::IntersectionRemoval, c, k)
    }

    #[test]
    fn new_effects_empty() {
        let effects = Effects::new();
        assert!(effects.is_empty());
        assert!(!effects.has_errors());
        assert!(!effects.has_actions());
    }

    #[test]
    fn add_action_nonempty() {
        let mut effects = Effects::new();
        let action = action_set(cell(0), known(1));

        let added = effects.add_action(action.clone());
        assert!(added);
        assert!(effects.has_actions());
        assert_eq!(effects.actions().len(), 1);
        assert_eq!(effects.actions()[0], action);
    }

    #[test]
    fn add_action_empty_returns_false() {
        let mut effects = Effects::new();
        let empty_action = Action::new(Strategy::NakedSingle);

        let added = effects.add_action(empty_action);
        assert!(!added);
        assert!(!effects.has_actions());
    }

    #[test]
    fn add_error_and_clear() {
        let mut effects = Effects::new();
        let error = Error::UnsolvableCell(cell(0));
        effects.add_error(error.clone());

        assert!(effects.has_errors());
        assert_eq!(effects.error_count(), 1);
        assert_eq!(effects.errors()[0], error);

        effects.clear_errors();
        assert!(!effects.has_errors());
        assert_eq!(effects.error_count(), 0);
    }

    #[test]
    fn add_set_and_erase_shortcuts() {
        let mut effects = Effects::new();
        effects.add_set(Strategy::NakedSingle, cell(0), known(1));
        effects.add_erase(Strategy::IntersectionRemoval, cell(1), known(2));

        assert_eq!(effects.actions().len(), 2);
        assert!(effects.actions()[0].sets(cell(0), known(1)));
        assert!(effects.actions()[1].erases(cell(1), known(2)));
    }

    #[test]
    fn affecting_cell_and_known_filters_actions() {
        let mut effects = Effects::new();
        effects.add_set(Strategy::NakedSingle, cell(0), known(1));
        effects.add_erase(Strategy::IntersectionRemoval, cell(1), known(2));

        let cell_effects = effects.affecting_cell(cell(0));
        assert_eq!(cell_effects.actions().len(), 1);
        assert!(cell_effects.actions()[0].sets(cell(0), known(1)));

        let known_effects = effects.affecting_known(known(2));
        assert_eq!(known_effects.actions().len(), 1);
        assert!(known_effects.actions()[0].erases(cell(1), known(2)));
    }

    #[test]
    fn without_action_removes_action_by_index() {
        let mut effects = Effects::new();
        effects.add_set(Strategy::NakedSingle, cell(0), known(1));
        effects.add_erase(Strategy::IntersectionRemoval, cell(1), known(2));

        let new_effects = effects.without_action(0);
        assert_eq!(new_effects.actions().len(), 1);
        assert!(new_effects.actions()[0].erases(cell(1), known(2)));
    }

    #[test]
    fn take_actions_appends_from_other_effects() {
        let mut effects1 = Effects::new();
        effects1.add_set(Strategy::NakedSingle, cell(0), known(1));

        let mut effects2 = Effects::new();
        effects2.add_erase(Strategy::IntersectionRemoval, cell(1), known(2));

        effects1.take_actions(effects2);
        assert_eq!(effects1.actions().len(), 2);
    }

    #[test]
    fn apply_actions_returns_valid() {
        let mut board = Board::new();
        let mut effects = Effects::new();
        let mut applied_effects = Effects::new();

        effects.add_set(Strategy::NakedSingle, cell(0), known(1));

        let change = effects.apply(&mut board, &mut applied_effects);
        assert_eq!(change, Change::Valid);
        assert!(board.is_known(cell(0)));
        assert_eq!(board.value(cell(0)).known(), Some(known(1)));
    }

    #[test]
    fn apply_strategy_applies_only_matching_strategy() {
        let mut board = Board::new();
        let mut effects = Effects::new();
        effects.add_set(Strategy::NakedSingle, cell(0), known(1));
        effects.add_set(Strategy::HiddenSingle, cell(1), known(2));

        let mut applied = Effects::new();
        let change = effects.apply_strategy(&mut board, Strategy::NakedSingle, &mut applied);

        assert_eq!(change, Change::Valid);
        assert!(board.is_known(cell(0)));
        assert!(!board.is_known(cell(1)));
    }

    #[test]
    fn apply_all_returns_some_if_errors() {
        let mut effects = Effects::new();
        effects.add_error(Error::UnsolvableCell(cell(0)));
        let mut board = Board::new();

        let result = effects.apply_all(&mut board);
        assert!(result.is_some());
        assert!(result.unwrap().has_errors());
    }

    #[test]
    fn apply_all_strategy_loops_and_applies_correctly() {
        let mut board = Board::new();
        let mut effects = Effects::new();
        effects.add_set(Strategy::NakedSingle, cell(0), known(1));

        let result = effects.apply_all_strategy(&mut board, Strategy::NakedSingle);
        assert!(result.is_none());
        assert!(board.is_known(cell(0)));
    }

    #[test]
    fn display_outputs_actions_and_errors() {
        let mut effects = Effects::new();
        effects.add_error(Error::UnsolvableCell(cell(0)));
        effects.add_set(Strategy::NakedSingle, cell(1), known(2));

        let output = format!("{}", effects);
        assert!(output.contains("Errors:"));
        assert!(output.contains("Actions:"));
        assert!(output.contains(&cell(1).to_string()));
    }

    #[test]
    fn helpers_compile() {
        let _ = action_set(cell(0), known(1));
        let _ = action_erase(cell(1), known(2));
    }
}
