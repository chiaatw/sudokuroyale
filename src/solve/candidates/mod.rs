//! Candidates module
//!
//! This module provides all candidate management functionality for Sudoku:
//! - Eliminations (peer elimination, apply strategies)
//! - Intersection strategies (pointing pairs/triples, box-line reductions)
//! - Singles strategies (naked singles, hidden singles)
//! - Utility functions (candidate counting, filtering, positions)

pub mod eliminations;
pub mod intersection;
pub mod singles;
pub mod utils;

// Re-exports for easy access outside the module
pub use eliminations::{Cell, eliminate_peers, apply_strategy};
pub use intersection::box_line_reduction;
pub use singles::{solve_naked_singles, solve_hidden_singles};
pub use utils::{
    count_candidates, 
    remaining_candidates, 
    candidate_in_row, 
    candidate_in_col, 
    candidate_in_block, 
    candidate_positions_in_block
};
