use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Write;

use itertools::Itertools;

use crate::layout::values::known::KnownLike;
use crate::layout::values::known_set::KnownSetLike;
use crate::layout::{Cell, CellSet, Known, KnownSet};
use crate::solve::strategy_ord::clues::ClueCollection;
use crate::symbols::{EMPTY_SET, REMOVE_CANDIDATE, SET_KNOWN};

use super::SolverAction;
use super::{Board, Change, Clues, Effects, Strategy, Verdict};

// Something that can be applied to a board and produce effects
pub trait AppliesToBoard {
    fn apply(&self, board: &mut Board, effects: &mut Effects) -> Change;
}

// Something that provides visual or logical clues
pub trait ProvidesClues {
    fn clues(&self) -> &Clues;
    fn has_clues(&self) -> bool;
}

#[derive(Clone, Eq, PartialEq)]
pub struct Action {
    strategy: Strategy,
    set: HashMap<Cell, Known>,
    erase: HashMap<Cell, KnownSet>,
    clues: Clues,
}

impl Action {
    pub fn new(strategy: Strategy) -> Self {
        Self {
            strategy,
            set: HashMap::new(),
            erase: HashMap::new(),
            clues: Clues::new(),
        }
    }

    pub fn new_set(strategy: Strategy, cell: Cell, known: Known) -> Self {
        let mut action = Self::new(strategy);
        action.set(cell, known);
        action
    }

    pub fn new_erase(strategy: Strategy, cell: Cell, known: Known) -> Self {
        let mut action = Self::new(strategy);
        action.erase(cell, known);
        action
    }

    pub fn new_erase_cells(strategy: Strategy, cells: CellSet, known: Known) -> Self {
        let mut action = Self::new(strategy);
        action.erase_cells(cells, known);
        action
    }

    pub fn new_erase_knowns(strategy: Strategy, cell: Cell, knowns: KnownSet) -> Self {
        let mut action = Self::new(strategy);
        action.erase_knowns(cell, knowns);
        action
    }

    pub fn set(&mut self, cell: Cell, known: Known) {
        self.set.insert(cell, known);
    }

    pub fn erase(&mut self, cell: Cell, known: Known) {
        *self.erase.entry(cell).or_insert_with(KnownSet::empty) += known;
    }

    pub fn erase_cells(&mut self, cells: CellSet, known: Known) {
        for cell in cells {
            self.erase(cell, known);
        }
    }

    pub fn erase_knowns(&mut self, cell: Cell, knowns: KnownSet) {
        for known in knowns.iter() {
            self.erase(cell, known);
        }
    }

    pub fn sets(&self, cell: Cell, known: Known) -> bool {
        self.set.get(&cell).copied() == Some(known)
    }

    pub fn erases(&self, cell: Cell, known: Known) -> bool {
        self.erase.get(&cell).map_or(false, |k| k.has(known))
    }

    pub fn affects_cell(&self, cell: Cell) -> bool {
        self.set.contains_key(&cell) || self.erase.contains_key(&cell)
    }

    pub fn affects_known(&self, known: Known) -> bool {
        self.set.values().any(|&k| k == known) || self.erase.values().any(|ks| ks.has(known))
    }

    pub fn collect_sets(&self) -> Vec<(Cell, Known)> {
        self.set
            .iter()
            .map(|(c, k)| (*c, *k))
            .sorted_by(|a, b| match a.0.cmp(&b.0) {
                Ordering::Equal => a.1.cmp(&b.1),
                r => r,
            })
            .collect()
    }

    pub fn collect_erases(&self) -> Vec<(Cell, KnownSet)> {
        self.erase
            .iter()
            .map(|(c, k)| (*c, *k))
            .sorted_by(|a, b| match a.0.cmp(&b.0) {
                Ordering::Equal => a.1.cmp(&b.1),
                r => r,
            })
            .collect()
    }

    pub fn clue_cell_for_known(&mut self, color: Verdict, cell: Cell, known: Known) {
        self.clues.clue_cell_for_known(color, cell, known);
    }

    pub fn clue_cells_for_known(&mut self, color: Verdict, cells: CellSet, known: Known) {
        self.clues.clue_cells_for_known(color, cells, known);
    }

    pub fn clue_cell_for_knowns(&mut self, color: Verdict, cell: Cell, knowns: KnownSet) {
        self.clues.clue_cell_for_knowns(color, cell, knowns);
    }

    pub fn clue_cells_for_knowns(&mut self, color: Verdict, cells: CellSet, knowns: KnownSet) {
        self.clues.clue_cells_for_knowns(color, cells, knowns);
    }

    pub fn strategy(&self) -> Strategy {
        self.strategy
    }
    pub fn is_empty(&self) -> bool {
        self.set.is_empty() && self.erase.is_empty()
    }
    pub fn has_strategy(&self, strategy: Strategy) -> bool {
        self.strategy == strategy
    }

    /// Gibt alle Zellen zurück, aus denen `known` entfernt wird.
    pub fn erases_from_cells(&self, known: Known) -> CellSet {
        self.erase
            .iter()
            .filter_map(|(cell, knowns)| if knowns.has(known) { Some(*cell) } else { None })
            .fold(CellSet::empty(), |acc, cell| acc + cell)
    }

    /// Gibt alle Kandidaten zurück, die aus `cell` entfernt werden.
    pub fn erases_knowns_from(&self, cell: Cell) -> KnownSet {
        self.erase
            .get(&cell)
            .copied()
            .unwrap_or_else(KnownSet::empty)
    }
}

impl SolverAction for Action {}

impl ProvidesClues for Action {
    fn clues(&self) -> &Clues {
        &self.clues
    }

    fn has_clues(&self) -> bool {
        !self.clues.is_empty()
    }
}

impl AppliesToBoard for Action {
    fn apply(&self, board: &mut Board, effects: &mut Effects) -> Change {
        let mut change = Change::None;

        for (cell, knowns) in &self.erase {
            for known in knowns.iter() {
                change &= board.remove_candidate(*cell, known, effects);
            }
        }

        if matches!(self.strategy, Strategy::Given) {
            for (cell, known) in &self.set {
                change &= board.set_given(*cell, *known, effects);
            }
        } else {
            for (cell, known) in &self.set {
                change &= board.set_known(*cell, *known, effects);
            }
        }

        change
    }
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.strategy)?;
        if self.clues.is_empty() && self.erase.is_empty() && self.set.is_empty() {
            write!(f, " {}", EMPTY_SET)
        } else {
            for (cell, knowns) in self.collect_erases() {
                write!(f, "\n- {} {} {}", cell, REMOVE_CANDIDATE, knowns)?;
            }
            for (cell, known) in self.collect_sets() {
                write!(f, "\n- {} {} {}", cell, SET_KNOWN, known)?;
            }
            for (cell, known, color) in self
                .clues
                .collect()
                .iter()
                .flat_map(|(c, m)| m.iter().map(|(k, v)| (*c, *k, *v)))
                .sorted()
            {
                write!(f, "\n- {} {} {:?}", cell, known, color)?;
            }
            Ok(())
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:20}", self.strategy)?;
        if self.clues.is_empty() && self.erase.is_empty() && self.set.is_empty() {
            f.write_char(EMPTY_SET)
        } else {
            let mut first = true;

            for (knowns, cells) in self
                .erase
                .iter()
                .fold(HashMap::<KnownSet, CellSet>::new(), |mut m, (c, k)| {
                    *m.entry(*k).or_default() += *c;
                    m
                })
                .iter()
                .sorted_by(|(_, a), (_, b)| b.len().cmp(&a.len()))
            {
                if !first {
                    f.write_str(", ")?;
                }
                first = false;

                for known in knowns.iter() {
                    f.write_char(known.label())?;
                }
                write!(f, " {} {}", REMOVE_CANDIDATE, cells)?;
            }

            for (known, cells) in self
                .set
                .iter()
                .fold(HashMap::<Known, CellSet>::new(), |mut m, (c, k)| {
                    *m.entry(*k).or_default() += *c;
                    m
                })
                .iter()
                .sorted_by(|(a, _), (b, _)| a.cmp(b))
            {
                if !first {
                    f.write_str(", ")?;
                }
                first = false;
                write!(f, "{} {} {}", known, SET_KNOWN, cells)?;
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Cell, CellSet, Known, KnownSet};

    fn cell(i: usize) -> Cell {
        Cell::new(i as u8)
    }

    #[test]
    fn new_action_is_empty() {
        let action = Action::new(Strategy::NakedSingle);
        assert!(action.is_empty());
        assert!(!action.has_clues());
    }

    #[test]
    fn new_set_creates_set_entry() {
        let c = cell(0);
        let k = Known::new(1);

        let action = Action::new_set(Strategy::NakedSingle, c, k);

        assert!(action.sets(c, k));
        assert!(!action.is_empty());
        assert!(action.affects_cell(c));
        assert!(action.affects_known(k));
    }

    #[test]
    fn new_erase_creates_erase_entry() {
        let c = cell(10);
        let k = Known::new(5);

        let action = Action::new_erase(Strategy::HiddenSingle, c, k);

        assert!(action.erases(c, k));
        assert!(!action.is_empty());
        assert!(action.affects_cell(c));
        assert!(action.affects_known(k));
    }

    #[test]
    fn erase_cells_erases_all_cells() {
        // "LockedCandidates" existiert nicht in deinem Strategy-Enum
        let mut action = Action::new(Strategy::IntersectionRemoval);

        // CellSet kann nicht aus Array via Into gebaut werden -> CellSet::of
        let cells: CellSet = CellSet::of(&[cell(0), cell(1), cell(2)]);
        let k = Known::new(3);

        action.erase_cells(cells, k);

        for c in cells {
            assert!(action.erases(c, k));
        }
    }

    #[test]
    fn erase_knowns_erases_all_knowns() {
        let mut action = Action::new(Strategy::IntersectionRemoval);
        let c = cell(4);
        let knowns = KnownSet::from_iter([Known::new(1), Known::new(2)]);

        action.erase_knowns(c, knowns);

        for k in knowns.iter() {
            assert!(action.erases(c, k));
        }
    }

    #[test]
    fn collect_sets_is_sorted() {
        let mut action = Action::new(Strategy::NakedSingle);
        action.set(cell(5), Known::new(2));
        action.set(cell(1), Known::new(1));
        action.set(cell(1), Known::new(3));

        let collected = action.collect_sets();

        assert_eq!(
            collected,
            vec![
                (cell(1), Known::new(1)),
                (cell(1), Known::new(3)),
                (cell(5), Known::new(2)),
            ]
        );
    }

    #[test]
    fn collect_erases_is_sorted() {
        let mut action = Action::new(Strategy::IntersectionRemoval);
        action.erase(cell(2), Known::new(3));
        action.erase(cell(0), Known::new(1));

        let collected = action.collect_erases();

        assert_eq!(collected[0].0, cell(0));
        assert_eq!(collected[1].0, cell(2));
    }

    #[test]
    fn affects_known_works_for_set_and_erase() {
        let mut action = Action::new(Strategy::NakedSingle);
        let k1 = Known::new(4);
        let k2 = Known::new(7);

        action.set(cell(0), k1);
        action.erase(cell(1), k2);

        assert!(action.affects_known(k1));
        assert!(action.affects_known(k2));
        assert!(!action.affects_known(Known::new(9)));
    }

    #[test]
    fn clues_are_recorded() {
        let mut action = Action::new(Strategy::XWing);
        let c = cell(8);
        let k = Known::new(6);

        // "Good" gibt es nicht -> nimm einen existierenden Verdict
        action.clue_cell_for_known(Verdict::Primary, c, k);

        assert!(action.has_clues());
        assert!(!action.clues().is_empty());
    }

    #[test]
    fn display_and_debug_do_not_panic() {
        let mut action = Action::new(Strategy::NakedSingle);
        action.set(cell(0), Known::new(1));
        action.erase(cell(1), Known::new(2));

        let _ = format!("{}", action);
        let _ = format!("{:?}", action);
    }

    #[test]
    fn apply_sets_and_erases_on_board() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let c = cell(0);
        let k = Known::new(1);

        let action = Action::new_set(Strategy::NakedSingle, c, k);
        let change = action.apply(&mut board, &mut effects);

        // Change ist ein Enum, kein Option -> changed() oder != None
        assert!(change.changed());
        assert!(board.is_known(c));
        assert_eq!(board.known(c), Some(k));
    }
}
