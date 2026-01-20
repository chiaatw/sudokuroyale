pub mod board;
pub mod errors;

pub mod cells;
pub mod deadliness;
pub mod givens;
pub mod peers;

pub use board::Board;
pub use errors::{HouseKind, ValidationError};

// Public API
pub use cells::validate_cells;
pub use deadliness::validate_deadliness;
pub use givens::validate_givens;
pub use peers::{validate_clear_move, validate_place_move};

/// Runs all validator passes
pub fn validate(board: &Board) -> Result<(), ValidationError> {
    validate_cells(board)?;
    validate_givens(board)?;
    validate_deadliness(board)?;
    Ok(())
}
