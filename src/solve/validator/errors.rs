use crate::layout::{Cell, Value};
use core::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HouseKind {
    Row,
    Col,
    Block,
}

impl fmt::Display for HouseKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HouseKind::Row => write!(f, "row"),
            HouseKind::Col => write!(f, "column"),
            HouseKind::Block => write!(f, "block"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationError {

    /// A given cell was cleared (set to unknown).
    GivenCellWasCleared { cell: Cell },

    /// A given cell was modified to a different value.
    GivenCellWasModified { cell: Cell, existing: Value, attempted: Value },

    /// Value is unknown (0) or out of range (>9).
    InvalidValue { cell: Cell, value: Value },

    /// Attempted to change an already-known cell to a different value.
    CellAlreadyHasValue {
        cell: Cell,
        existing: Value,
        attempted: Value,
    },

    /// Duplicate value found in a row/col/block.
    DuplicateInHouse {
        kind: HouseKind,
        index: usize,
        value: Value,
        first: Cell,
        second: Cell,
    },

    /// Move conflicts with a peer that already has that value.
    ConflictWithPeer { cell: Cell, value: Value, peer: Cell },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidValue { cell, value } => {
                write!(f, "Invalid value {:?} in cell {:?}", value, cell)
            }
            ValidationError::CellAlreadyHasValue {
                cell,
                existing,
                attempted,
            } => write!(
                f,
                "Cell {:?} already has {:?}, cannot set {:?}",
                cell, existing, attempted
            ),
            ValidationError::DuplicateInHouse {
                kind,
                index,
                value,
                first,
                second,
            } => write!(
                f,
                "Duplicate value {:?} in {} {} (cells {:?} and {:?})",
                value, kind, index, first, second
            ),
            ValidationError::ConflictWithPeer { cell, value, peer } => write!(
                f,
                "Move would conflict: placing {:?} in {:?} conflicts with peer {:?}",
                value, cell, peer
            ),
            ValidationError::GivenCellWasCleared { cell } => {
                write!(f, "Given cell {:?} was cleared", cell)
            }
            ValidationError::GivenCellWasModified { cell, existing, attempted } => {
                write!(f, "Given cell {:?} has {:?}, cannot set {:?}", cell, existing, attempted)
            }
        }
    }
}

impl std::error::Error for ValidationError {}
