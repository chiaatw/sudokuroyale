//! Validator module
//!
//! Provides functions to validate Sudoku grids (rows, columns, subgrids).

pub mod grid;

pub use grid::{validate_grid, ValidationError};

/// Enum representing which part of the Sudoku grid failed validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationError {
    Row(usize),
    Column(usize),
    Subgrid(usize, usize), // row, col starting position
}

/// Validates a 9x9 Sudoku grid.
///
/// Returns `Ok(())` if the grid is valid, otherwise returns
/// the first `ValidationError` encountered.
pub fn validate_grid(grid: &[[u8; 9]; 9]) -> Result<(), ValidationError> {
    // Validate rows
    for row in 0..9 {
        let mut seen = [false; 10]; // digits 1..9
        for col in 0..9 {
            let val = grid[row][col] as usize;
            if val == 0 || val > 9 {
                return Err(ValidationError::Row(row));
            }
            if seen[val] {
                return Err(ValidationError::Row(row));
            }
            seen[val] = true;
        }
    }

    // Validate columns
    for col in 0..9 {
        let mut seen = [false; 10];
        for row in 0..9 {
            let val = grid[row][col] as usize;
            if seen[val] {
                return Err(ValidationError::Column(col));
            }
            seen[val] = true;
        }
    }

    // Validate 3x3 subgrids
    for block_row in (0..9).step_by(3) {
        for block_col in (0..9).step_by(3) {
            let mut seen = [false; 10];
            for row in block_row..block_row + 3 {
                for col in block_col..block_col + 3 {
                    let val = grid[row][col] as usize;
                    if seen[val] {
                        return Err(ValidationError::Subgrid(block_row, block_col));
                    }
                    seen[val] = true;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_grid() {
        let grid: [[u8; 9]; 9] = [
            [5,3,4,6,7,8,9,1,2],
            [6,7,2,1,9,5,3,4,8],
            [1,9,8,3,4,2,5,6,7],
            [8,5,9,7,6,1,4,2,3],
            [4,2,6,8,5,3,7,9,1],
            [7,1,3,9,2,4,8,5,6],
            [9,6,1,5,3,7,2,8,4],
            [2,8,7,4,1,9,6,3,5],
            [3,4,5,2,8,6,1,7,9],
        ];

        assert_eq!(validate_grid(&grid), Ok(()));
    }

    #[test]
    fn test_invalid_row() {
        let mut grid = [[0u8; 9]; 9];
        grid[0][0] = 1;
        grid[0][1] = 1; // duplicate in row
        assert_eq!(validate_grid(&grid), Err(ValidationError::Row(0)));
    }

    #[test]
    fn test_invalid_column() {
        let mut grid = [[0u8; 9]; 9];
        grid[0][0] = 2;
        grid[1][0] = 2; // duplicate in column
        assert_eq!(validate_grid(&grid), Err(ValidationError::Column(0)));
    }

    #[test]
    fn test_invalid_subgrid() {
        let mut grid = [[0u8; 9]; 9];
        grid[0][0] = 3;
        grid[1][1] = 3; // duplicate in top-left subgrid
        assert_eq!(validate_grid(&grid), Err(ValidationError::Subgrid(0, 0)));
    }
}
