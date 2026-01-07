//! Utility functions for candidate operations
//!
//! Provides helper functions for candidate counting, filtering and block/row/column operations.

use crate::candidates::eliminations::Cell;

/// Counts the number of candidates in a cell
pub fn count_candidates(cell: &Cell) -> usize {
    cell.candidates.iter().filter(|&&b| b).count()
}

/// Returns a Vec<u8> of remaining candidate values in a cell
pub fn remaining_candidates(cell: &Cell) -> Vec<u8> {
    cell
        .candidates
        .iter()
        .enumerate()
        .filter_map(|(i, &b)| if b { Some((i + 1) as u8) } else { None })
        .collect()
}

/// Checks if a candidate exists in a row
pub fn candidate_in_row(grid: &[[Cell; 9]; 9], row: usize, candidate: u8) -> bool {
    grid[row].iter().any(|cell| cell.candidates[(candidate - 1) as usize])
}

/// Checks if a candidate exists in a column
pub fn candidate_in_col(grid: &[[Cell; 9]; 9], col: usize, candidate: u8) -> bool {
    (0..9).any(|row| grid[row][col].candidates[(candidate - 1) as usize])
}

/// Checks if a candidate exists in a 3x3 block
pub fn candidate_in_block(grid: &[[Cell; 9]; 9], block_row: usize, block_col: usize, candidate: u8) -> bool {
    for r in 0..3 {
        for c in 0..3 {
            if grid[block_row + r][block_col + c].candidates[(candidate - 1) as usize] {
                return true;
            }
        }
    }
    false
}

/// Returns all positions (row,col) in a block where a candidate exists
pub fn candidate_positions_in_block(
    grid: &[[Cell; 9]; 9],
    block_row: usize,
    block_col: usize,
    candidate: u8,
) -> Vec<(usize, usize)> {
    let mut positions = Vec::new();
    for r in 0..3 {
        for c in 0..3 {
            if grid[block_row + r][block_col + c].candidates[(candidate - 1) as usize] {
                positions.push((block_row + r, block_col + c));
            }
        }
    }
    positions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidates::eliminations::Cell;

    #[test]
    fn test_count_candidates() {
        let mut cell = Cell::new();
        assert_eq!(count_candidates(&cell), 9);
        cell.eliminate(3);
        assert_eq!(count_candidates(&cell), 8);
    }

    #[test]
    fn test_remaining_candidates() {
        let mut cell = Cell::new();
        cell.eliminate(1);
        cell.eliminate(5);
        let rem = remaining_candidates(&cell);
        assert_eq!(rem.len(), 7);
        assert!(!rem.contains(&1));
        assert!(!rem.contains(&5));
    }

    #[test]
    fn test_candidate_in_row_col_block() {
        let mut grid = [[Cell::new(); 9]; 9];
        grid[0][0].eliminate(2); // candidate 2 removed
        assert!(!candidate_in_row(&grid, 0, 2));
        assert!(candidate_in_row(&grid, 1, 2));
        assert!(!candidate_in_col(&grid, 0, 2));
        assert!(candidate_in_col(&grid, 1, 2));
        assert!(!candidate_in_block(&grid, 0, 0, 2));
        assert!(candidate_in_block(&grid, 3, 3, 2));
    }

    #[test]
    fn test_candidate_positions_in_block() {
        let mut grid = [[Cell::new(); 9]; 9];
        grid[0][0].eliminate(1);
        grid[0][1].eliminate(1);
        let positions = candidate_positions_in_block(&grid, 0, 0, 1);
        assert_eq!(positions.len(), 7); // 9 cells - 2 eliminated
    }
}
