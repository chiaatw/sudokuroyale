use crate::layout::values::known_set::KnownSetLike;
use crate::layout::{Cell, Known, KnownSet};

/// Candidate store: one KnownSet per cell.
/// - For known/filled cells: candidates should be empty.
/// - For unknown cells: candidates is the set of possible digits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidates {
    per_cell: [KnownSet; 81],
}

impl Candidates {
    /// Creates a candidates store with all cells = full (1..9).
    /// Typically you call `recompute_all_candidates(board, &mut candidates)` afterwards.
    pub fn new_full() -> Self {
        Self {
            per_cell: [KnownSet::full(); 81],
        }
    }

    /// Creates a candidates store with all cells = empty.
    pub fn new_empty() -> Self {
        Self {
            per_cell: [KnownSet::empty(); 81],
        }
    }

    #[inline(always)]
    pub fn get(&self, cell: Cell) -> KnownSet {
        self.per_cell[cell.usize()]
    }

    #[inline(always)]
    pub fn set(&mut self, cell: Cell, set: KnownSet) {
        self.per_cell[cell.usize()] = set;
    }

    #[inline(always)]
    pub fn remove(&mut self, cell: Cell, k: Known) {
        self.per_cell[cell.usize()] -= k;
    }

    #[inline(always)]
    pub fn remove_set(&mut self, cell: Cell, ks: KnownSet) {
        self.per_cell[cell.usize()] -= ks;
    }

    #[inline(always)]
    pub fn is_candidate(&self, cell: Cell, k: Known) -> bool {
        self.get(cell).has(k)
    }
}

impl Default for Candidates {
    fn default() -> Self {
        Self::new_full()
    }
}
