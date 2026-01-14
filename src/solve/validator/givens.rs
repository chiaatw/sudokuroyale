use crate::solve::validator::{Board, ValidationError};
use crate::layout::ValueLike;

pub fn validate_givens(board: &Board) -> Result<(), ValidationError> {
    for cell in board.givens().iter() {
        if board.get(cell).is_unknown() {
            return Err(ValidationError::GivenCellWasCleared { cell });
        }
    }
    Ok(())
}
