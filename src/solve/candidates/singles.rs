//! Singles strategies module
//!
//! Provides functions to detect and solve Naked and Hidden Singles in Sudoku.

use crate::candidates::eliminations::Cell;

/// Solves Naked Singles in the grid.
/// 
/// Returns `true` if at least one cell was solved.
pub fn solve_naked_singles(grid: &mut [[Cell; 9]; 9]) -> bool {
    let mut solved_any = false;

    for row in 0..9 {
        for col in 0..9 {
            if grid[row][col].value.is_none() {
                if let Some(val) = grid[row][col].naked_single() {
                    grid[row][col].solve(val);
                    solved_any = true;
                }
            }
        }
    }

    solved_any
}

/// Finds and solves Hidden Singles in rows, columns, and blocks.
///
/// Returns `true` if at least one cell was solved.
pub fn solve_hidden_singles(grid: &mut [[Cell; 9]; 9]) -> bool {
    let mut solved_any = false;

    // Rows
    for row in 0..9 {
        for candidate in 1..=9 {
            let mut positions = vec![];
            for col in 0..9 {
                let cell = &grid[row][col];
                if cell.value.is_none() && cell.candidates[(candidate - 1) as usize] {
                    positions.push(col);
                }
            }
            if positions.len() == 1 {
                grid[row][positions[0]].solve(candidate as u8);
                solved_any = true;
            }
        }
    }

    // Columns
    for col in 0..9 {
        for candidate in 1..=9 {
            let mut positions = vec![];
            for row in 0..9 {
                let cell = &grid[row][col];
                if cell.value.is_none() && cell.candidates[(candidate - 1) as usize] {
                    positions.push(row);
                }
            }
            if positions.len() == 1 {
                grid[positions[0]][col].solve(candidate as u8);
                solved_any = true;
            }
        }
    }

    // 3x3 Blocks
    for block_row in (0..9).step_by(3) {
        for block_col in (0..9).step_by(3) {
            for candidate in 1..=9 {
                let mut positions = vec![];
                for r in 0..3 {
                    for c in 0..3 {
                        let cell = &grid[block_row + r][block_col + c];
                        if cell.value.is_none() && cell.candidates[(candidate - 1) as usize] {
                            positions.push((r, c));
                        }
                    }
                }
                if positions.len() == 1 {
                    let (r, c) = positions[0];
                    grid[block_row + r][block_col + c].solve(candidate as u8);
                    solved_any = true;
                }
            }
        }
    }

    solved_any
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidates::eliminations::Cell;

    #[test]
    fn test_naked_single() {
        let mut grid = [[Cell::new(); 9]; 9];
        // Cell with only one candidate
        grid[0][0].candidates = [true, false, false, false, false, false, false, false, false];
        let solved = solve_naked_singles(&mut grid);
        assert!(solved);
        assert_eq!(grid[0][0].value, Some(1));
    }

    #[test]
    fn test_hidden_single_row() {
        let mut grid = [[Cell::new(); 9]; 9];
        // Candidate 5 appears only in column 3 of row 0
        for col in 0..9 {
            grid[0][col].candidates[4] = col == 3;
        }
        let solved = solve_hidden_singles(&mut grid);
        assert!(solved);
        assert_eq!(grid[0][3].value, Some(5));
    }

    #[test]
    fn test_hidden_single_column() {
        let mut grid = [[Cell::new(); 9]; 9];
        // Candidate 7 appears only in row 2 of column 1
        for row in 0..9 {
            grid[row][1].candidates[6] = row == 2;
        }
        let solved = solve_hidden_singles(&mut grid);
        assert!(solved);
        assert_eq!(grid[2][1].value, Some(7));
    }

    #[test]
    fn test_hidden_single_block() {
        let mut grid = [[Cell::new(); 9]; 9];
        // Candidate 3 appears only in cell (1,1) of top-left block
        for r in 0..3 {
            for c in 0..3 {
                grid[r][c].candidates[2] = r == 1 && c == 1;
            }
        }
        let solved = solve_hidden_singles(&mut grid);
        assert!(solved);
        assert_eq!(grid[1][1].value, Some(3));
    }
}
