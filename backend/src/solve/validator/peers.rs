use crate::layout::{Cell, Value, ValueLike};
use crate::solve::validator::{Board, ValidationError};

/// Validate a PLACE move

pub fn validate_place_move(
    board: &Board,
    cell: Cell,
    attempted: Value,
) -> Result<(), ValidationError> {
    // Value must be in 1..=9
    let raw = attempted.raw();
    if raw == 0 || raw > 9 {
        return Err(ValidationError::InvalidValue {
            cell,
            value: attempted,
        });
    }

    let existing = board.get(cell);

    // Given cells are immutable
    if board.is_given(cell) {
        if existing != attempted {
            return Err(ValidationError::GivenCellWasModified {
                cell,
                existing,
                attempted,
            });
        }
        return Ok(());
    }

    if existing.is_known() && existing != attempted {
        return Err(ValidationError::CellAlreadyHasValue {
            cell,
            existing,
            attempted,
        });
    }

    // Check row / column / box peers
    for peer in cell.peers().iter() {
        if board.get(peer) == attempted {
            return Err(ValidationError::ConflictWithPeer {
                cell,
                value: attempted,
                peer,
            });
        }
    }

    Ok(())
}

/// Validate a CLEAR move.
///
/// Clearing a given is illegal.
pub fn validate_clear_move(board: &Board, cell: Cell) -> Result<(), ValidationError> {
    if board.is_given(cell) {
        return Err(ValidationError::GivenCellWasCleared { cell });
    }
    Ok(())
}
