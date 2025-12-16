use crate::layout::{Cell, CellSet, KnownSet};

// Combines multiple peer cells into a unit treated as a single cell

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PseudoCell {
    // Representative cell, used as pseudo cell
    pub pseudo: Cell,
    // the actual underlying cells
    pub cells: CellSet,
    // Candidates shared among these cells
    pub knowns: KnownSet,
}

impl PseudoCell {
    pub fn new(cells: CellSet, knowns: KnownSet) -> PseudoCell {
        let pseudo = cells.first().expect("CellSet must not be empty");
        PseudoCell {
            pseudo,
            cells,
            knowns,
        }
    }

    #[inline]
    pub fn as_cells(&self) -> CellSet {
        self.cells
    }

    #[inline]
    pub fn first(&self) -> Cell {
        self.pseudo
    }

    #[inline]
    pub fn shared_knowns(&self) -> KnownSet {
        self.knowns
    }
}

impl From<PseudoCell> for CellSet {
    fn from(p: PseudoCell) -> Self {
        p.cells
    }
}

impl From<PseudoCell> for Cell {
    fn from(p: PseudoCell) -> Self {
        p.pseudo
    }
}