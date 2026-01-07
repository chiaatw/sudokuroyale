//! Intersection eliminations module
//!
//! Provides functions for Sudoku strategies like Pointing Pairs/Triples
//! and Box-Line Reductions.

use crate::candidates::eliminations::Cell;

/// Eliminates candidates using pointing pairs/triples (box-line reduction)
///
/// # Arguments
/// * `grid` - mutable reference to 9x9 grid of Cells
/// * `block_row`, `block_col` - starting coordinates of the 3x3 block
/// * `candidate` - candidate value (1..9) to check for intersection
///
/// Returns `true` if any candidate was eliminated.
pub fn box_line_reduction(
    grid: &mut [[Cell; 9]; 9],
    block_row: usize,
    block_col: usize,
    candidate: u8,
) -> bool {
    let mut eliminated = false;

    // Track candidate positions within the block
    let mut rows_with_candidate = [false; 3];
    let mut cols_with_candidate = [false; 3];

    for r in 0..3 {
        for c in 0..3 {
            if grid[block_row + r][block_col + c].candidates[(candidate - 1) as usize] {
                rows_with_candidate[r] = true;
                cols_with_candidate[c] = true;
            }
        }
    }

    // Check for pointing pair/line in row
    let rows_count = rows_with_candidate.iter().filter(|&&b| b).count();
    if rows_count == 1 {
        let row_idx = rows_with_candidate.iter().position(|&b| b).unwrap();
        let global_row = block_row + row_idx;
        for c in 0..9 {
            if c < block_col || c >= block_col + 3 {
                if grid[global_row][c].eliminate(candidate) {
                    eliminated = true;
                }
            }
        }
    }

    // Check for pointing pair/line in column
    let cols_count = cols_with_candidate.iter().filter(|&&b| b).count();
    if cols_count == 1 {
        let col_idx = cols_with_candidate.iter().position(|&b| b).unwrap();
        let global_col = block_col + col_idx;
        for r in 0..9 {
            if r < block_row || r >= block_row + 3 {
                if grid[r][global_col].eliminate(candidate) {
                    eliminated = true;
                }
            }
        }
    }

    eliminated
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidates::eliminations::Cell;

    #[test]
    fn test_box_line_reduction_row() {
        let mut grid = [[Cell::new(); 9]; 9];
        // Candidate 5 appears only in row 0 of top-left block
        grid[0][0].candidates[4] = true; // 5
        grid[0][1].candidates[4] = false;
        grid[0][2].candidates[4] = false;
        // Other block cells in row 0 have candidate 5
        grid[0][3].candidates[4] = true;
        grid[0][4].candidates[4] = true;

        let eliminated = box_line_reduction(&mut grid, 0, 0, 5);
        assert!(eliminated);
        assert!(!grid[0][3].candidates[4]); // candidate 5 eliminated outside block
        assert!(!grid[0][4].candidates[4]);
    }

    #[test]
    fn test_box_line_reduction_column() {
        let mut grid = [[Cell::new(); 9]; 9];
        // Candidate 7 appears only in column 1 of top-left block
        grid[0][1].candidates[6] = true; // 7
        grid[1][1].candidates[6] = false;
        grid[2][1].candidates[6] = false;
        // Other block cells in column 1 outside block
        grid[3][1].candidates[6] = true;
        grid[4][1].candidates[6] = true;

        let eliminated = box_line_reduction(&mut grid, 0, 0, 7);
        assert!(eliminated);
        assert!(!grid[3][1].candidates[6]);
        assert!(!grid[4][1].candidates[6]);
    }

    #[test]
    fn test_no_elimination() {
        let mut grid = [[Cell::new(); 9]; 9];
        // Candidate 9 appears in multiple rows and columns of block
        for r in 0..3 {
            for c in 0..3 {
                grid[r][c].candidates[8] = true;
            }
        }
        let eliminated = box_line_reduction(&mut grid, 0, 0, 9);
        assert!(!eliminated); // no single row/column -> nothing eliminated
    }
}
