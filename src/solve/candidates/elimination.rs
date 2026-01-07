//! Eliminations module
//!
//! Provides functions to remove candidates from Sudoku cells based on strategies.

use crate::validator::validate_grid;
use crate::Strategy;

/// Represents a single cell in a Sudoku grid.
#[derive(Debug, Clone)]
pub struct Cell {
    pub value: Option<u8>,        // Some(1..9) if solved, None if unsolved
    pub candidates: [bool; 9],    // candidates[0] = 1, candidates[1] = 2, ..., candidates[8] = 9
}

impl Cell {
    /// Creates a new empty cell with all candidates possible
    pub fn new() -> Self {
        Self {
            value: None,
            candidates: [true; 9],
        }
    }

    /// Solves the cell with a given value and clears candidates
    pub fn solve(&mut self, val: u8) {
        self.value = Some(val);
        self.candidates = [false; 9];
    }

    /// Eliminates a candidate from this cell
    /// Returns true if a candidate was actually removed
    pub fn eliminate(&mut self, val: u8) -> bool {
        if val >= 1 && val <= 9 && self.candidates[(val - 1) as usize] {
            self.candidates[(val - 1) as usize] = false;
            return true;
        }
        false
    }

    /// Counts how many candidates remain
    pub fn remaining_candidates(&self) -> usize {
        self.candidates.iter().filter(|&&b| b).count()
    }

    /// Returns Some(value) if only one candidate remains
    pub fn naked_single(&self) -> Option<u8> {
        if self.value.is_some() {
            return self.value;
        }
        let mut candidate = None;
        for (i, &present) in self.candidates.iter().enumerate() {
            if present {
                if candidate.is_some() {
                    return None; // More than one candidate
                }
                candidate = Some((i + 1) as u8);
            }
        }
        candidate
    }
}

/// Eliminates a candidate from peers (row, column, block) of a solved cell
///
/// `grid` is a 9x9 array of `Cell`s
/// `row`, `col` is the position of the solved cell
/// `val` is the solved value
pub fn eliminate_peers(grid: &mut [[Cell; 9]; 9], row: usize, col: usize, val: u8) {
    // Eliminate in row
    for c in 0..9 {
        if c != col {
            grid[row][c].eliminate(val);
        }
    }

    // Eliminate in column
    for r in 0..9 {
        if r != row {
            grid[r][col].eliminate(val);
        }
    }

    // Eliminate in 3x3 block
    let start_row = (row / 3) * 3;
    let start_col = (col / 3) * 3;
    for r in start_row..start_row + 3 {
        for c in start_col..start_col + 3 {
            if r != row || c != col {
                grid[r][c].eliminate(val);
            }
        }
    }
}

/// Applies a single strategy elimination to the grid
pub fn apply_strategy(
    grid: &mut [[Cell; 9]; 9],
    row: usize,
    col: usize,
    strategy: Strategy,
) -> Option<u8> {
    match strategy {
        Strategy::NakedSingle => grid[row][col].naked_single(),
        Strategy::Peer => {
            if let Some(val) = grid[row][col].value {
                eliminate_peers(grid, row, col, val);
            }
            None
        }
        _ => None, // Other strategies can be added later
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Strategy;

    #[test]
    fn test_cell_elimination() {
        let mut cell = Cell::new();
        assert_eq!(cell.remaining_candidates(), 9);

        assert!(cell.eliminate(3));
        assert!(!cell.eliminate(3)); // Already eliminated
        assert_eq!(cell.remaining_candidates(), 8);

        cell.solve(5);
        assert_eq!(cell.value, Some(5));
        assert_eq!(cell.remaining_candidates(), 0);
    }

    #[test]
    fn test_naked_single() {
        let mut cell = Cell::new();
        for val in 1..9 {
            cell.eliminate(val);
        }
        assert_eq!(cell.naked_single(), Some(9));

        // Multiple candidates remaining
        let mut cell2 = Cell::new();
        cell2.eliminate(1);
        cell2.eliminate(2);
        assert_eq!(cell2.naked_single(), None);
    }

    #[test]
    fn test_eliminate_peers() {
        let mut grid = [[Cell::new(); 9]; 9];
        grid[0][0].solve(1);
        eliminate_peers(&mut grid, 0, 0, 1);

        // Row
        for c in 1..9 {
            assert!(!grid[0][c].candidates[0]);
        }
        // Column
        for r in 1..9 {
            assert!(!grid[r][0].candidates[0]);
        }
        // Block
        for r in 0..3 {
            for c in 0..3 {
                if r != 0 || c != 0 {
                    assert!(!grid[r][c].candidates[0]);
                }
            }
        }
    }

    #[test]
    fn test_apply_strategy_naked_single() {
        let mut cell = Cell::new();
        for val in 1..9 {
            cell.eliminate(val);
        }
        let result = apply_strategy(&mut [[cell; 9]; 9], 0, 0, Strategy::NakedSingle);
        assert_eq!(result, Some(9));
    }
}
