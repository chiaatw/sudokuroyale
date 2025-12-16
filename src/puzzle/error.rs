use std::fmt;

use crate::layout::{Cell, House, Known, Rectangle};

// Tracks an error encountered while solving a cell or removing a candidate
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Error {
    // Cannot solve a cell to a non-candidate
    NotCandidate(Cell, Known),
    // Cannot solve a cell that is already solved with a different konwn
    AlreadySolved(Cell, Known, Known),
    // The unsolved cell has no more candidates remaining
    UnsolvableCell(Cell),
    // An unsolved value has no more candidate cells in the hosue
    UnsolvableHouse(House, Known),
    // Four cells in two boxes form a deadly rectangle
    DeadlyRectangle(Rectangle),
}

impl Error {
    
    #[inline(always)]
    pub fn cell(&self) -> Option<Cell> {
        match *self {
            Error::NotCandidate(cell, _)
            | Error::AlreadySolved(cell, _, _)
            | Error::UnsolvableCell(cell) => Some(cell),
            _ => None,
        }
    }

    //Returns true if the error makes the board invalid
    #[inline(always)]
    pub fn is_invalid(&self) -> bool {
        matches!(
            *self,
            Error::AlreadySolved(_, _, _) | Error::NotCandidate(_, _) | Error::UnsolvableCell(_) | Error::UnsolvableHouse(_, _)
        )
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::NotCandidate(cell, known) => {
                write!(f, "{} cannot be solved with {}", cell, known)
            }
            Error::AlreadySolved(cell, known, current) => write!(
                f,
                "{} cannot be changed from {} to {}",
                cell, current, known
            ),
            Error::UnsolvableCell(cell) => write!(f, "{} has no candidates", cell),
            Error::UnsolvableHouse(house, known) => {
                write!(f, "{} has no candidate cells for {}", house, known)
            }
            Error::DeadlyRectangle(rectangle) => {
                write!(f, "{} form a deadly rectangle", rectangle)
            }
        }
    }
}

impl std::error::Error for Error{}