use crate::solve::validator::{Board, ValidationError};

// TODO: detect unsolvable positions (cells with 0 candidates, contradictions, ...)
pub fn validate_deadliness(_board: &Board) -> Result<(), ValidationError> {
    // placeholder for later solving dead positions
    Ok(())
}
