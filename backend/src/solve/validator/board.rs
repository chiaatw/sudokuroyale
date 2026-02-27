use crate::layout::values::known_set::KnownSetLike;
use crate::layout::{Cell, CellSet, Known, KnownSet, Value, ValueLike};

use crate::solve::candidates::{recompute_all_candidates, update_after_set_known, Candidates};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    cells: [Value; 81],
    givens: CellSet,
    candidates: Candidates,
}

impl Board {
    /// New empty board (all unknown, no givens, candidates full)
    pub fn new() -> Self {
        let mut b = Self {
            cells: [Value::unknown(); 81],
            givens: CellSet::empty(),
            candidates: Candidates::new_full(),
        };
        b.recompute_candidates();
        b
    }

    #[inline(always)]
    pub fn get(&self, cell: Cell) -> Value {
        self.cells[cell.usize()]
    }

    #[inline(always)]
    pub fn set_raw(&mut self, cell: Cell, value: Value) {
        self.cells[cell.usize()] = value;
    }

    pub fn clear(&mut self, cell: Cell) {
        self.cells[cell.usize()] = Value::unknown();
        self.recompute_candidates();
    }

    #[inline(always)]
    pub fn givens(&self) -> CellSet {
        self.givens
    }

    #[inline(always)]
    pub fn is_given(&self, cell: Cell) -> bool {
        self.givens.has(cell)
    }

    #[inline(always)]
    pub fn mark_given(&mut self, cell: Cell) {
        self.givens = self.givens + cell;
    }

    pub fn mark_all_known_as_givens(&mut self) {
        let mut g = CellSet::empty();
        for cell in Cell::iter() {
            if self.get(cell).is_known() {
                g = g + cell;
            }
        }
        self.givens = g;
    }

    #[inline(always)]
    pub fn candidates(&self, cell: Cell) -> KnownSet {
        self.candidates.get(cell)
    }

    pub fn recompute_candidates(&mut self) {
        let mut candidates = std::mem::take(&mut self.candidates);
        recompute_all_candidates(self, &mut candidates);
        self.candidates = candidates;
    }

    pub fn set_known(&mut self, cell: Cell, known: Known) {
        self.cells[cell.usize()] = Value::from(known);

        let mut candidates = std::mem::take(&mut self.candidates);
        update_after_set_known(self, &mut candidates, cell, known);
        self.candidates = candidates;
    }

    pub fn knowns(&self) -> CellSet {
        let mut set = CellSet::empty();
        for cell in Cell::iter() {
            if self.get(cell).is_known() {
                set = set + cell;
            }
        }
        set
    }

    pub fn unknowns(&self) -> CellSet {
        let mut set = CellSet::empty();
        for cell in Cell::iter() {
            if self.get(cell).is_unknown() {
                set = set + cell;
            }
        }
        set
    }

    pub fn is_solved(&self) -> bool {
        self.unknowns().is_empty()
    }

    pub fn cells_with_n_candidates(&self, n: usize) -> CellSet {
        let mut out = CellSet::empty();
        for cell in Cell::iter() {
            if self.get(cell).is_unknown() && self.candidates(cell).len() == n {
                out = out + cell;
            }
        }
        out
    }

    pub fn remove_candidates_from_cells(&mut self, cells: CellSet, remove: KnownSet) -> CellSet {
        if remove.is_empty() || cells.is_empty() {
            return CellSet::empty();
        }

        let mut changed = CellSet::empty();

        for cell in cells.iter() {
            if self.get(cell).is_known() {
                continue;
            }

            let before = self.candidates.get(cell);
            let mut after = before;
            after -= remove;

            if after != before {
                self.candidates.set(cell, after);
                changed = changed + cell;
            }
        }

        changed
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        let chars: Vec<char> = s.chars().filter(|c| !c.is_whitespace()).collect();
        if chars.len() != 81 {
            return Err(format!(
                "Expected 81 cells, got {} chars (after trimming whitespace)",
                chars.len()
            ));
        }

        let mut b = Self {
            cells: [Value::unknown(); 81],
            givens: CellSet::empty(),
            candidates: Candidates::new_full(),
        };

        for (i, ch) in chars.into_iter().enumerate() {
            let v = match ch {
                '.' | '0' => Value::unknown(),
                '1'..='9' => Value::new(ch.to_digit(10).unwrap() as u8),
                _ => return Err(format!("Invalid char '{}' at index {}", ch, i)),
            };
            b.cells[i] = v;
        }

        b.mark_all_known_as_givens();
        b.recompute_candidates();
        Ok(b)
    }

    pub fn as_slice(&self) -> &[Value] {
        &self.cells
    }
}
