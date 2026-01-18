use std::fmt;
use std::ops::{BitAnd, BitAndAssign};

use crate::io::format_for_fancy_console;
use crate::layout::{Cell, CellSet, House, Known, KnownSet, Value};
use crate::solve::creates_deadly_rectangles;

use crate::layout::values::known::KnownLike;
use crate::layout::values::known_set::KnownSetLike;
use crate::layout::values::value::ValueLike;

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
            givens: CellSet::empty(),
            knowns: CellSet::empty(),
            values: [Value::unknown(); 81],
            candidate_knowns_by_cell: [KnownSet::full(); 81],
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
        self.knowns.has(cell)
    }

    #[inline(always)]
    pub const fn is_given(&self, cell: Cell) -> bool {
        self.givens.has(cell)
    }

    #[inline(always)]
    pub const fn value(&self, cell: Cell) -> Value {
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

    pub fn known(&self, cell: Cell) -> Option<Known> {
    self.value(cell).known()
    }

    pub fn known_iter(&self) -> impl Iterator<Item = (Cell, Known)> + '_ {
        self.knowns.into_iter()
        .map(|cell| (cell, self.value(cell).known().unwrap()))
    }

    pub fn knowns_iter(&self, cells: CellSet) -> impl Iterator<Item = (Cell, Known)> + '_ {
        (cells & self.knowns)
        .into_iter()
        .map(|cell| (cell, self.value(cell).known().unwrap()))
    }

    pub fn all_knowns(&self, cells: CellSet) -> KnownSet {
        let mut result = KnownSet::empty();
        for cell in cells {
            if let Some(k) = self.value(cell).is_known() {
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
        if let Some(current) = self.value(cell).known() {
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
        self.cells_with_n_candidates[0] += cell;
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
        .iter()
        .map(|cell| (cell, self.candidates(cell)))
    }

    pub const fn candidate_cells(&self, known: Known) -> CellSet {
        self.candidate_cells_by_known[known.usize()]
    }

    pub fn house_candidate_cells(&self, house: House, known: Known) -> CellSet {
        self.candidate_cells(known) & house.cells()
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
        known: Known,
        effects: &mut Effects,
    ) -> Change {
        let mut change = Change::None;

        for house in cell.houses() {
            if self.is_house_known(house, known) {
                continue;
            }

            let candidates = self.house_candidate_cells(house, known);
            if candidates.is_empty() {
                effects.add_error(Error::UnsolvableHouse(house, known));
                change &= Change::Invalid;
            } else if candidates.len() == 1 {
                effects.add_set(Strategy::HiddenSingle, candidates.as_single().unwrap(), known);
                change &= Change::Valid;
            }
        }

        change
    }

    pub fn remove_candidates(
        &mut self,
        cell: Cell,
        knowns: KnownSet,
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
        known: Known,
        effects: &mut Effects,
    ) -> Change {
        let mut change = Change::None;
        for cell in cells {
            change &= self.remove_candidate(cell, known, effects);
        }
        change
    }

    pub fn remove_candidates_from_cells(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Cell, Known, KnownSet, CellSet, House};
    use crate::puzzle::{Effects, Strategy};

    fn cell(i: usize) -> Cell {
        Cell::new(i)
    }

    fn known(n: u8) -> Known {
        Known::from(n)
    }

    /* ---------------- Change ---------------- */

    #[test]
    fn change_and_logic() {
        assert_eq!(Change::None & Change::Valid, Change::Valid);
        assert_eq!(Change::Valid & Change::None, Change::Valid);
        assert_eq!(Change::Valid & Change::Valid, Change::Valid);
        assert_eq!(Change::Valid & Change::Invalid, Change::Invalid);
        assert_eq!(Change::Invalid & Change::Valid, Change::Invalid);
    }

    #[test]
    fn change_changed_flag() {
        assert!(!Change::None.changed());
        assert!(Change::Valid.changed());
        assert!(Change::Invalid.changed());
    }

    /* ---------------- Board init ---------------- */

    #[test]
    fn new_board_is_empty() {
        let board = Board::new();

        assert_eq!(board.known_count(), 0);
        assert_eq!(board.given_count(), 0);
        assert_eq!(board.unknown_count(), 81);
        assert!(board.unknowns().is_full());
        assert!(!board.is_fully_solved());
    }

    #[test]
    fn new_board_has_all_candidates() {
        let board = Board::new();

        for cell in Cell::iter() {
            assert_eq!(board.candidates(cell), KnownSet::full());
            assert!(board.cells_with_n_candidates(9).has(cell));
        }
    }

    /* ---------------- set_known / set_given ---------------- */

    #[test]
    fn set_known_sets_value() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let c = cell(0);
        let k = known(1);

        let change = board.set_known(c, k, &mut effects);

        assert_eq!(change, Change::Valid);
        assert!(board.is_known(c));
        assert_eq!(board.value(c).known(), Some(k));
        assert_eq!(board.candidates(c), KnownSet::empty());
        assert!(board.cells_with_n_candidates(0).has(c));
    }

    #[test]
    fn set_given_marks_given() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let c = cell(1);
        let k = known(2);

        board.set_given(c, k, &mut effects);

        assert!(board.is_given(c));
        assert!(board.is_known(c));
        assert_eq!(board.given_count(), 1);
    }

    #[test]
    fn set_known_same_value_is_noop() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let c = cell(2);
        let k = known(3);

        assert_eq!(board.set_known(c, k, &mut effects), Change::Valid);
        assert_eq!(board.set_known(c, k, &mut effects), Change::None);
    }

    #[test]
    fn set_known_conflict_is_invalid() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let c = cell(3);

        board.set_known(c, known(4), &mut effects);
        let change = board.set_known(c, known(5), &mut effects);

        assert_eq!(change, Change::Invalid);
        assert!(!effects.errors().is_empty());
    }

    /* ---------------- Candidate removal ---------------- */

    #[test]
    fn remove_candidate_removes_known() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let c = cell(4);
        let k = known(6);

        let change = board.remove_candidate(c, k, &mut effects);

        assert_eq!(change, Change::Valid);
        assert!(!board.is_candidate(c, k));
        assert_eq!(board.candidates(c).len(), 8);
    }

    #[test]
    fn remove_last_candidate_is_invalid() {
        let mut board = Board::new();
        let mut effects = Effects::new();
        let c = cell(5);

        for k in KnownSet::full() {
            board.remove_candidate(c, k, &mut effects);
        }

        assert!(!effects.errors().is_empty());
    }

    #[test]
    fn remove_candidates_triggers_naked_single() {
        let mut board = Board::new();
        let mut effects = Effects::new();
        let c = cell(6);

        let mut knowns = KnownSet::full();
        let last = known(9);
        knowns -= last;

        board.remove_candidates(c, knowns, &mut effects);

        assert!(effects
            .sets()
            .iter()
            .any(|s| s.strategy == Strategy::NakedSingle));
    }

    /* ---------------- Iterators & helpers ---------------- */

    #[test]
    fn unknown_and_known_iterators() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        board.set_known(cell(0), known(1), &mut effects);
        board.set_known(cell(1), known(2), &mut effects);

        assert_eq!(board.known_iter().count(), 2);
        assert_eq!(board.unknown_iter().count(), 79);
    }

    #[test]
    fn packed_string_representation() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        board.set_known(cell(0), known(1), &mut effects);
        board.set_known(cell(80), known(9), &mut effects);

        let s = board.packed_string();

        assert_eq!(s.len(), 81);
        assert_eq!(s.chars().next().unwrap(), '1');
        assert_eq!(s.chars().last().unwrap(), '9');
    }

    #[test]
    fn display_does_not_panic() {
        let board = Board::new();
        let _ = format!("{}", board);
    }
}
