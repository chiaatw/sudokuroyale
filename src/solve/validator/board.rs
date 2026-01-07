use crate::layout::{
    Cell, CellSet, Known, KnownSet, Value,
    House, HouseSet, Shape, Coord, CoordSet,
};

/// Represents a 9x9 Sudoku board.
#[derive(Clone, Debug)]
pub struct Board {
    cells: [Cell; 81],
}

impl Board {
    /// Creates a new empty board (all cells unknown).
    pub fn new() -> Self {
        Self {
            cells: [Cell::default(); 81],
        }
    }

    /// Returns a copy of the cell at the given index (0..80).
    pub fn cell(&self, index: usize) -> Cell {
        self.cells[index]
    }

    /// Sets a value in the cell at the given index.
    pub fn set_cell(&mut self, index: usize, value: Value) {
        self.cells[index].set(value);
    }

    /// Checks if the entire board is valid according to Sudoku rules.
    /// That is, no duplicate known values exist in any row, column, or block.
    pub fn is_valid(&self) -> bool {
        for house in HouseSet::all() {
            if !self.is_house_valid(&house) {
                return false;
            }
        }
        true
    }

    /// Checks if a single house (row, column, block) is valid.
    fn is_house_valid(&self, house: &House) -> bool {
        let mut seen = KnownSet::empty();
        for cell in house.cells() {
            if let Some(value) = cell.known_value() {
                if seen.contains(value) {
                    return false; // Duplicate value found
                }
                seen.insert(value);
            }
        }
        true
    }

    /// Returns a `CellSet` of all cells in a given row (0..8).
    pub fn row(&self, row: usize) -> CellSet {
        let mut set = CellSet::empty();
        for col in 0..9 {
            set.insert(self.cells[row * 9 + col]);
        }
        set
    }

    /// Returns a `CellSet` of all cells in a given column (0..8).
    pub fn column(&self, col: usize) -> CellSet {
        let mut set = CellSet::empty();
        for row in 0..9 {
            set.insert(self.cells[row * 9 + col]);
        }
        set
    }

    /// Returns a `CellSet` of all cells in a given block (0..8).
    pub fn block(&self, block_index: usize) -> CellSet {
        let mut set = CellSet::empty();
        let start_row = (block_index / 3) * 3;
        let start_col = (block_index % 3) * 3;
        for r in start_row..start_row + 3 {
            for c in start_col..start_col + 3 {
                set.insert(self.cells[r * 9 + c]);
            }
        }
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::CellIndex;

    #[test]
    fn test_new_board_empty() {
        let board = Board::new();
        for i in 0..81 {
            assert!(board.cell(i).is_unknown());
        }
    }

    #[test]
    fn test_set_cell_and_get() {
        let mut board = Board::new();
        board.set_cell(0, Value::One);
        assert_eq!(board.cell(0).value(), Some(Value::One));
    }

    #[test]
    fn test_valid_board() {
        let mut board = Board::new();
        // Fill first row with unique values
        for i in 0..9 {
            board.set_cell(i, Value::from(i as u8 + 1));
        }
        assert!(board.is_valid());
    }

    #[test]
    fn test_invalid_board() {
        let mut board = Board::new();
        // Duplicate in first row
        board.set_cell(0, Value::One);
        board.set_cell(1, Value::One);
        assert!(!board.is_valid());
    }

    #[test]
    fn test_row_column_block_sets() {
        let board = Board::new();
        assert_eq!(board.row(0).len(), 9);
        assert_eq!(board.column(0).len(), 9);
        assert_eq!(board.block(0).len(), 9);
    }
}
