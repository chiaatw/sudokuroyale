use crate::solve::validator::{Board, ValidationError, HouseKind};
use crate::layout::{Cell, Value};

pub fn validate_cells(board: &Board) -> Result<(), ValidationError> {
    // rows
    for r in 0..9 {
        check_unit(board, unit_row(r), HouseKind::Row, r)?;
    }
    for c in 0..9 {
        check_unit(board, unit_col(c), HouseKind::Col, c)?;
    }
    for b in 0..9 {
        check_unit(board, unit_block(b), HouseKind::Block, b)?;
    }
    Ok(())
}
