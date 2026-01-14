//! Sudoku validation (board + moves).
//!
//! - `validate_board` checks if the current board violates Sudoku rules.
//! - `validate_move` checks if placing a value into a cell would violate rules.

pub mod board;
pub mod errors;

pub use board::Board;
pub use errors::{HouseKind, ValidationError};

use crate::layout::{Cell, Value};

/// Validates the whole board.
/// - Unknown cells are allowed.
/// - Known values must not duplicate within any row/col/block.
pub fn validate_board(board: &Board) -> Result<(), ValidationError> {
    // rows
    for r in 0..9 {
        check_unit(board, unit_row(r), HouseKind::Row, r)?;
    }
    // cols
    for c in 0..9 {
        check_unit(board, unit_col(c), HouseKind::Col, c)?;
    }
    // blocks (0..8)
    for b in 0..9 {
        check_unit(board, unit_block(b), HouseKind::Block, b)?;
    }
    Ok(())
}

/// Validates a single move (cell,value) against current board.
/// Returns an error if:
/// - value is unknown (0) or out of range (>9)
/// - the cell already contains a different known value
/// - the value would conflict with any peer (same row/col/block)
pub fn validate_move(board: &Board, cell: Cell, value: Value) -> Result<(), ValidationError> {
    let raw = value.raw();
    if raw == 0 || raw > 9 {
        return Err(ValidationError::InvalidValue { cell, value });
    }

    let current = board.get(cell);
    if current.raw() != 0 && current != value {
        return Err(ValidationError::CellAlreadyHasValue {
            cell,
            existing: current,
            attempted: value,
        });
    }

    for peer in peers(cell) {
        if board.get(peer) == value {
            return Err(ValidationError::ConflictWithPeer {
                cell,
                value,
                peer,
            });
        }
    }

    Ok(())
}

fn check_unit(
    board: &Board,
    cells: [Cell; 9],
    kind: HouseKind,
    index: usize,
) -> Result<(), ValidationError> {
    // track seen values 1..9
    let mut seen = [None::<Cell>; 10];

    for &cell in &cells {
        let v = board.get(cell);
        let raw = v.raw() as usize;

        if raw == 0 {
            continue; // unknown is allowed
        }
        if raw > 9 {
            return Err(ValidationError::InvalidValue { cell, value: v });
        }

        if let Some(first_cell) = seen[raw] {
            return Err(ValidationError::DuplicateInHouse {
                kind,
                index,
                value: v,
                first: first_cell,
                second: cell,
            });
        }
        seen[raw] = Some(cell);
    }
    Ok(())
}

/// Row unit (0..8), returns 9 cells.
fn unit_row(r: usize) -> [Cell; 9] {
    let mut out = [Cell::new(0); 9];
    let base = r * 9;
    for i in 0..9 {
        out[i] = Cell::new((base + i) as u8);
    }
    out
}

/// Column unit (0..8), returns 9 cells.
fn unit_col(c: usize) -> [Cell; 9] {
    let mut out = [Cell::new(0); 9];
    for i in 0..9 {
        out[i] = Cell::new((i * 9 + c) as u8);
    }
    out
}

/// Block unit (0..8), numbering left-to-right, top-to-bottom.
fn unit_block(b: usize) -> [Cell; 9] {
    let mut out = [Cell::new(0); 9];
    let br = (b / 3) * 3;
    let bc = (b % 3) * 3;

    let mut k = 0;
    for dr in 0..3 {
        for dc in 0..3 {
            let r = br + dr;
            let c = bc + dc;
            out[k] = Cell::new((r * 9 + c) as u8);
            k += 1;
        }
    }
    out
}

/// Returns all peers of a cell (row+col+block, without itself), de-duplicated.
/// Always length 20 for standard Sudoku.
fn peers(cell: Cell) -> Vec<Cell> {
    let idx = cell.usize();
    let r = idx / 9;
    let c = idx % 9;
    let b = (r / 3) * 3 + (c / 3);

    let mut v = Vec::with_capacity(20);

    // row
    for cc in 0..9 {
        if cc != c {
            v.push(Cell::new((r * 9 + cc) as u8));
        }
    }
    // col
    for rr in 0..9 {
        if rr != r {
            v.push(Cell::new((rr * 9 + c) as u8));
        }
    }
    // block
    let br = (b / 3) * 3;
    let bc = (b % 3) * 3;
    for dr in 0..3 {
        for dc in 0..3 {
            let rr = br + dr;
            let cc = bc + dc;
            if rr == r && cc == c {
                continue;
            }
            v.push(Cell::new((rr * 9 + cc) as u8));
        }
    }

    // de-duplicate
    v.sort_by_key(|x| x.usize());
    v.dedup_by_key(|x| x.usize());
    v
}
