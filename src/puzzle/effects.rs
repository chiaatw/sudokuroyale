use std::fmt;
use std::collections::HashMap;

use crate::layout::{Cell, CellSet, Known, KnownSet};
use super::{Action, Board, Change, Error, Strategy};

// Collects actions and errors encountered while modifying a board
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
            actions: Vec::with_capacity(self.actions.len())
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
        self.actions.iter().fold(Change::None, |chg, a| {
            chg & a.apply(board, effects)
        })
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