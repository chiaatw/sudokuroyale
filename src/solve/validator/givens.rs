use crate::solve::validator::{Board, ValidationError};
use crate::layout::Cell;

pub fn validate_givens(board: &Board) -> Result<(), ValidationError> {
    for cell in Cell::ALL {
        if cell.is_given() && board.get(cell).is_unknown() {
            return Err(ValidationError::GivenCellWasCleared { cell });
        }
    }
    Ok(())
}
