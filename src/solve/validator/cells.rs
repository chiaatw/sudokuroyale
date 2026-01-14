use crate::layout::{Cell, Value, ValueLike};
use crate::solve::validator::{Board, HouseKind, ValidationError};

/// Validates Sudoku "cell rules" for the whole board:
/// - Unknown values are allowed
/// - Known values must be 1..=9
/// - No duplicates within any row/col/block
pub fn validate_cells(board: &Board) -> Result<(), ValidationError> {
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

fn check_unit(
    board: &Board,
    cells: [Cell; 9],
    kind: HouseKind,
    index: usize,
) -> Result<(), ValidationError> {
    // Track first occurrence of each value 1..9
    let mut seen: [Option<Cell>; 10] = [None; 10];

    for &cell in &cells {
        let v = board.get(cell);
        let raw = v.raw() as usize;

        if raw == 0 {
            continue; // unknown allowed
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
/// block 0 = top-left, 1 = top-middle, 2 = top-right, ..., 8 = bottom-right.
fn unit_block(b: usize) -> [Cell; 9] {
    let mut out = [Cell::new(0); 9];

    let br = (b / 3) * 3; // block row (0,3,6)
    let bc = (b % 3) * 3; // block col (0,3,6)

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
