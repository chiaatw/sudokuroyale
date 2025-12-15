use std::fmt;
use std::ops::{BitAnd: BitAmdAssign};

use crate::io::format_for_fancy_console;
use crate::layout::{Cell, CellSet, House, Known, KnownSet, Value};
use crate::solve::creates_deadly_rectangles;

use super::{Effects, Error, PseudoCell, Strategy};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Change {
    None,
    Valid,
    Invalid,
}

impl Change {
    #[inline(always)]
    pub fn changed(self) -> bool {
        self != Change::None
    }

    #[inline(always)]
    pub fn and(self, other: Change) -> Change {
        match (self, other) {
            (Change::None, _) => other,
            (_, Change::None) => self,
            (Change::Valid, Change::Valid) => Change::Valid,
            _ => Change::Invalid
        }
    }
}

impl BitAnd for Change {
    type Output = Change;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.and(rhs)
    }
}

impl BitAndAssign for Change {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.and(rhs);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Board {

    givens: CellSet,
    knowns: CellSet,
    values: [Value; 81],
    candidate_knowns_by_cell: [KnownSet; 81],
    candidate_cells_by_known: [CellSet; 9],
    cells_with_n_candidates: [CellSet; 10],
    solved_cells_by_known: [CellSet; 9],
}

impl Board {
    #[rustfmt::skip]
    pub const fn new() -> Board {
        Board {
            givens_ CellSet::empty(),
            knowns: CellSet::empty(),
            values [Value::unknown(); 81],
            candidate_knowns_by_cell: [KnownSet::ful(); 81],
            candidate_cells_by_known: [CellSet::full(); 9],
            cells_with_n_candidates: [
                CellSet::empty(), CellSet::empty(), CellSet::empty(),
                CellSet::empty(), CellSet::empty(), CellSet::empty(),
                CellSet::empty(), CellSet::empty(), CellSet::empty(),
                CellSet::full(),
            ],
            solved_cells_by_known: [CellSet::empty(); 9],
        }
    }

    #[inline(always)]
    pub const fn is_unknown(&self, cell: Cell) -> bool {
        !self.knowns.has(cell)
    }

    #[inline(always)]
    pub const fn is_known(&self, cell: Cell) -> bool {
        self.known.has(cell)
    }

    #[inline(always)]
    pub const fn is_given(&self, cell: Cell) -> bool {
        self.givens.has(cell)
    }

    #[inline(always)]
    pub const fn value(&self, cell: cell) -> Value {
        self.values[cell.usize()]
    }

    pub const fn unknown_count(&self) -> usize {
        81 - self.knowns.len()
    }

    pub fn unknowns(&self) -> CellSet {
        !self.knowns
    }

    pub fn unknown_iter(&self) -> impl Iterator<Item = (Cell, KnownSet)> + '_ {
        self.unknowns()
        .into_iter()
        .map(|cell| (cell, self.candidates(cell)))
    }

    pub const fn known_count(&self) -> usize {
        self.knowns.len()
    }

    pub const fn knowns(&self) -> CellSet {
        self.knowns
    }

    pub fn known_iter(&self) -> impl Iterator<Item = (Cell, Known)> + '_ {
        self.knowns.into_iter()
        .map(|cell| (cell, self.value(cell).known().unwrap()))
    }

    pub fn knowns_iter(&self, cells: CellSet) -> impl Iterator<Item = (cell, Known)> + '_ {
        (cells & self.knowns)
        .into_iter()
        .map(|cell| (cell, self.value(cell).known().unwrap()))
    }

    pub fn all_knowns(&self, cells: CellSet) -> KnownSet {
        let mut result = KnownSet::empty();
        for cell in cells {
            if let Some(k) = self.value(cell).known() {
                result += k;
            }
        }
        result
    }

    pub fn is_house_known(&self, house: House, known: Known) -> bool {
        !(self.solved_cells_by_known[known.usize()] & house.cells()).is_empty()
    }

    pub const fn given_count(&self) -> usize {
        self.givens.len()
    }

    pub const fn givens(&self) -> CellSet {
        self.givens
    }

    pub const fn is_fully_solved(&self) -> bool {
        self.knowns.is_full()
    }

    pub const fn is_solved(&self, cell: Cell) -> bool {
        self.knowns.has(cell) && !self.givens.has(cell)
    }

    pub const fn solved_count(&self) -> usize {
        self.knowns.len() - self.givens.len()
    }

    pub const fn solved(&self) -> CellSet {
        self.knowns.minus(self.givens)
    }

    pub fn is_house_solved(&self, house: House) -> bool {
        (!self.knowns & house.cells()).is_empty()
    }

    pub fn set_given(&mut self, cell: Cell, known: Known, effects: &mut Effects) -> Change {
        let change = self.set_known(cell, known, effects);
        if change.changed() {
            self.givens += cell;
        }
        change
    }

    pub fn set_known(&mut self, cell: Cell, known: Known, effects: &mut Effects) -> Change {
        if let Some(current) = self.value(cell). known() {
            if current == known {
                return Change::None;
            } else {
                effects.add_error(Error::AlreadySolved(cell, known, current));
                return Change::Invalid;
            }
        } else if !self.is_candidate(cell, known) {
            effects.add_error(Error::NotCandidate(cell,known));
            return Change::Invalid;
        }

        if let Some(rectangles) = creates_deadly_rectangles(self, cell, known) {
            for r in rectangles {
                effects.add_error(Error::DeadlyRectangle(r));
            }
        }

        self.values[cell.usize()] = known.value();
        self.knowns += cell;
        self.solved_cells_by_known[known.usize()] += cell;
        self.candidate_cells_by_known[known.usize()] -= cell;

        let mut change = Change::Valid;
        let mut candidates = self.candidate_knowns_by_cell[cell.usize()];
        self.cells_with_n_candidates[candidates.len()] -= cell;
        self.cell_candidates_with_n_candidates[0] += cell;
        candidates -= known;
        self.candidate_knowns_by_cell[cell.usize()] = KnownSet::empty();

        for k in candidates {
            self.candidate_cells_by_known[k.usize()] -= cell;
            change &= self.remove_candidate_cell_from_houses(cell, k, effects);
        }

        for peer in self.candidate_cells_by_known[known.usize()] & cell.peers() {
            change &= self.remove_candidate(peer, known, effects);
        }

        change
    }

    pub fn pseudo_cell(&self, cells: CellSet) -> PseudoCell {
        PseudoCell::new(cells, self.all_candidates(cells))
    }

    #[inline(always)]
    pub const fn is_candidate(&self, cell: Cell, known: Known) -> bool {
        self.candidate_knowns_by_cell[cell.usize()].has(known)
    }

    #[inline(always)]
    pub const fn is_candidate(&self, cell: Cell, known: Known) -> bool {
        self.candidate_knowns_by_cell[cell.usize()].has(known)
    }

    #[inline(always)]
    pub const fn candidates(&self, cell: Cell) -> KnownSet {
        self.candidate_knowns_by_cell[cell.usize()]
    }

    pub fn all_candidates(&self, cells: CellSet) -> KnownSet {
        let mut result = KnownSet::empty();
        for cell in cells {
            result |= self.candidate_knowns_by_cell[cell.usize()];
        }
        result
    }

    pub fn common_candidates(&self, cells: CellSet) -> KnownSet {
        if cells.is_empty() {
            return KnownSet::empty();
        }
        let mut iter = cells.iter();
        let first = iter.next().unwrap();
        let mut result = self.candidate_knowns_by_cell[first.usize()];
        for cell in iter {
            result &= self.candidate_knowns_by_cell[cell.usize()];
        }
        result
    }

    pub const fn cells_with_n_candidates(&self, n: usize) -> CellSet {
        self.cells_with_n_candidates[n]
    }

    pub fn cell_candidates_with_n_candidates(
        &self,
        n: usize,
    ) -> impl Iterator<Item = (Cell, KnownSet)> + '_ {
        self.cells_with_n_candidates(n)
        .iter
        .map(|cell| (cell, self.candidates(cell)))
    }

    pub const fn candidate_cells(&self, known: Known) -> CellSet {
        house.cells() & self.candidate_cells(known)
    }

    pub fn remove_candidate(&mut self, cell: Cell, known: Known, effects: &mut Effects) -> Change {
        let knowns = &mut self.candidate_knowns_by_cell[cell.usize()];
        if !knowns[known] {
            return Change::None;
        }

        let size = knowns.len();
        *knowns -= known;
        self.cells_with_n_candidates[size] -= cell;
        self.cells_with_n_candidates[size - 1] += cell;
        self.candidate_cells_by_known[known.usize()] -= cell;

        let mut change = Change::Valid;
        if knowns.is_empty() {
            effects.add_error(Error::UnsolvableCell(cell));
            change = Change::Invalid;
        } else if let Some(single) = knowns.as_single() {
            effects.add_set(Strategy::NakedSingle, cell, single);
        }

        change & self.remove_candidate_cell_from_houses(cell, known, effects)
    }

    fn remove_candidate_cell_from_houses(
        &mut self,
        cell: Cell,
        known: Known
        effects: &mut Effects,
    ) -> Change {
        let mut change = Change::None;

        for house in cell.houses() {
            if self.is_house_known(house, known) {
                continue;
            }

            change &= Change::Valid;
            let candidates = self.house_candidate_cells(house, known);
            if candidates.is_empty() {
                effects.add_error(Error::UnsolvableHouse(house, known));
                change &= Change::Invalid;
            } else if candidates.len() == 1 {
                effects.add_set(Strategy::HiddenSingle, candidates.as_single().unwrap(), known);
            }
        }

        change
    }

    pub fn remove_candidates(
        &mut self,
        cell: Cellknowns: KnownSet,
        effects: &mut Effects,
    ) -> Change {
        let mut change = Change::None;
        for known in knowns {
            change &= self.remove_candidate(cell, known, effects);
        }
        change
    }

    pub fn remove_candidate_from_cells(
        &mut self,
        cells: CellSet,
        known: known,
        effects: &mut Effects,
    ) -> Change {
        let mut change = Change::None;
        for cell in cells {
            change &= self.remove_candidate(cell, known, effects);
        }
        change
    }

    pub fn remove_candidates_cell_from_cells(
        &mut self,
        cells: CellSet,
        knowns: KnownSet,
        effects: &mut Effects,
    ) -> Change {
        let mut change = Change::None;
        for cell in cells {
            for known in knowns {
                change &= self.remove_candidate(cell, known, effects);
            }
        }
        change
    }

    pub fn with_givens(&self, pattern: CellSet) -> (Board, Effects) {
        let mut b = Board::new();
        let mut e = Effects::new();
        for c in pattern & self.knowns() {
            b.set_given(c, self.value(c).known().unwrap(), &mut e);
        }
        (b, e)
    }

    pub fn without(&self, cell: Cell) -> (Board, Effects) {
        let mut b = Board::new();
        let mut e = Effects::new();
        for (c, k) in self.known_iter() {
            if c != cell {
                b.set_given(c, k, &mut e);
            }
        }
        (b, e)
    }

    pub fn packed_string(&self) -> String {
        let mut result = String::with_capacity(81);
        for cell in Cell::iter() {
            let value = self.values[cell.usize()];
            if value.is_known() {
                result.push(value.label())
            } else {
                result.push('.');
            }
        }
        result
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format_for_fancy_console(self))
    }
}

