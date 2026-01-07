use std::ops::{Deref, DerefMut};

use super::{Cell, CellSet, Value, Known, KnownSet, Rectangle};

/// Represents a full 9x9 Sudoku grid.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Grid {
    cells: [Cell; 81],
    knowns: KnownSet,
}

impl Grid {
    /// Creates a new empty grid.
    pub const fn new() -> Self {
        // Leider kann man [Cell::new_empty(); 81] noch nicht const erzeugen,
        // aber hier als Beispiel:
        let cells = [Cell::new_empty(); 81];
        let knowns = KnownSet::new();
        Self { cells, knowns }
    }

    /// Returns a reference to a cell by index.
    pub fn cell(&self, index: usize) -> &Cell {
        &self.cells[index]
    }

    /// Returns a mutable reference to a cell.
    pub fn cell_mut(&mut self, index: usize) -> &mut Cell {
        &mut self.cells[index]
    }

    /// Sets a value in a cell and updates knowns.
    pub fn set(&mut self, index: usize, value: Value) {
        self.cells[index].set(value);
        self.knowns.insert(index, value);
    }

    /// Returns an iterator over all cells.
    pub fn iter(&self) -> impl Iterator<Item = &Cell> {
        self.cells.iter()
    }

    /// Returns an iterator over rows (slices of 9 cells).
    pub fn iter_rows(&self) -> impl Iterator<Item = &[Cell]> {
        self.cells.chunks(9)
    }

    /// Returns an iterator over columns (each column as a Vec of 9 cells)
    pub fn iter_columns(&self) -> impl Iterator<Item = Vec<&Cell>> {
        (0..9).map(move |col| (0..9).map(move |row| &self.cells[row * 9 + col]).collect())
    }

    /// Returns the set of known values.
    pub fn knowns(&self) -> &KnownSet {
        &self.knowns
    }
}

/// Allow deref to cells slice for convenience.
impl Deref for Grid {
    type Target = [Cell; 81];

    fn deref(&self) -> &Self::Target {
        &self.cells
    }
}

/// Allow mutable deref to cells slice.
impl DerefMut for Grid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cells
    }
}

/// Try to create a grid from a vector of values.
impl TryFrom<Vec<Value>> for Grid {
    type Error = ();

    fn try_from(values: Vec<Value>) -> Result<Self, Self::Error> {
        if values.len() != 81 {
            return Err(());
        }
        let mut grid = Grid::new();
        for (i, v) in values.into_iter().enumerate() {
            grid.set(i, v);
        }
        Ok(grid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cell, Value};

    #[test]
    fn test_grid_new_and_access() {
        let grid = Grid::new();
        assert_eq!(grid.iter().count(), 81);

        let cell = grid.cell(0);
        assert_eq!(cell.index(), 0);
    }

    #[test]
    fn test_grid_set_and_knowns() {
        let mut grid = Grid::new();
        let val = Value::new(5);
        grid.set(0, val);

        assert_eq!(grid.cell(0).value(), val);
        assert!(grid.knowns().contains(0, val));
    }

    #[test]
    fn test_grid_try_from_vec() {
        let values = vec![Value::new(0); 81];
        let grid = Grid::try_from(values.clone()).unwrap();
        assert_eq!(grid.iter().count(), 81);

        // too few values
        let result = Grid::try_from(vec![Value::new(0); 80]);
        assert!(result.is_err());
    }

    #[test]
    fn test_grid_iter_rows_and_columns() {
        let grid = Grid::new();
        let mut row_count = 0;
        for row in grid.iter_rows() {
            assert_eq!(row.len(), 9);
            row_count += 1;
        }
        assert_eq!(row_count, 9);

        let mut col_count = 0;
        for col in grid.iter_columns() {
            assert_eq!(col.len(), 9);
            col_count += 1;
        }
        assert_eq!(col_count, 9);
    }
}
