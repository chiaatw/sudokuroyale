use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Write;

use itertools::Itertools;

use crate::layout::{Cell, CellSet, Known, KnownSet};
use crate::symbols::{EMPTY_SET, REMOVE_CANDIDATE, SET_KNOWN};

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
        for known in knowns {
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
}

impl SolverAction for Action {
    fn strategy(&self) -> Strategy {
        self.strategy
    }

    fn is_empty(&self) -> bool {
        self.set.is_empty() && self.erase.is_empty()
    }
}

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
            for known in knowns {
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
        if self.is_empty() {
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
        if self.is_empty() {
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

                for known in knowns {
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