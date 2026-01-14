pub mod board;
pub mod errors;

pub mod cells;
pub mod peers;
pub mod givens;
pub mod deadliness;

pub use board::Board;
pub use errors::{HouseKind, ValidationError};

// Public API
pub use cells::validate_cells;
pub use peers::validate_peers;
pub use givens::validate_givens;
pub use deadliness::validate_deadliness;

pub use strategy_ord::algorithms::find_intersection_removals;
pub use strategy_ord::deadly_rectangles::creates_deadly_rectangles;

/// Runs all validator passes
pub fn validate(board: &Board) -> Result<(), ValidationError> {
    validate_cells(board)?;
    validate_peers(board)?;
    validate_givens(board)?;
    validate_deadliness(board)?;
    Ok(())
}
