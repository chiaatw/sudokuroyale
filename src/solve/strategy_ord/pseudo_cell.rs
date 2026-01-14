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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Cell, CellSet, Known, KnownSet};

    #[test]
    fn pseudo_cell_new_sets_pseudo_and_cells() {
        let cells: CellSet = (Cell::new(1) + Cell::new(2) + Cell::new(3));
        let knowns: KnownSet = KnownSet::from_iter([Known::from(1), Known::from(2)]);
        let p = PseudoCell::new(cells, knowns);

        // The first cell becomes the pseudo cell
        assert_eq!(p.first(), Cell::new(1));

        // All cells are included
        assert_eq!(p.as_cells(), cells);

        // Shared knowns are correct
        assert_eq!(p.shared_knowns(), knowns);
    }

    #[test]
    #[should_panic(expected = "CellSet must not be empty")]
    fn pseudo_cell_new_panics_on_empty_cells() {
        let empty_cells = CellSet::empty();
        let knowns: KnownSet = KnownSet::empty();
        let _p = PseudoCell::new(empty_cells, knowns);
    }

    #[test]
    fn pseudo_cell_from_trait_for_cellset() {
        let cells: CellSet = (Cell::new(4) + Cell::new(5));
        let knowns: KnownSet = KnownSet::from_iter([Known::from(3)]);
        let p = PseudoCell::new(cells, knowns);

        let cs: CellSet = p.into();
        assert_eq!(cs, cells);
    }

    #[test]
    fn pseudo_cell_from_trait_for_cell() {
        let cells: CellSet = (Cell::new(7) + Cell::new(8));
        let knowns: KnownSet = KnownSet::from_iter([Known::from(4)]);
        let p = PseudoCell::new(cells, knowns);

        let c: Cell = p.into();
        assert_eq!(c, Cell::new(7)); // first cell
    }
}
