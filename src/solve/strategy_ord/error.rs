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
            Error::AlreadySolved(_, _, _)
                | Error::NotCandidate(_, _)
                | Error::UnsolvableCell(_)
                | Error::UnsolvableHouse(_, _)
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

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Cell, Known};

    fn cell(i: usize) -> Cell {
        Cell::new(i as u8)
    }

    fn known(n: u8) -> Known {
        Known::new(n)
    }

    fn some_house() -> House {
        // robust: nehme irgendein gültiges House aus einer echten Cell
        cell(0).houses().into_iter().next().unwrap()
    }

    fn some_rectangle() -> Rectangle {
        // robust: nehme irgendein gültiges Rectangle aus dem Iterator
        Rectangle::iter().next().unwrap()
    }

    #[test]
    fn cell_returns_some_for_cell_errors() {
        let e1 = Error::NotCandidate(cell(0), known(1));
        let e2 = Error::AlreadySolved(cell(1), known(2), known(3));
        let e3 = Error::UnsolvableCell(cell(2));

        assert_eq!(e1.cell(), Some(cell(0)));
        assert_eq!(e2.cell(), Some(cell(1)));
        assert_eq!(e3.cell(), Some(cell(2)));
    }

    #[test]
    fn cell_returns_none_for_non_cell_errors() {
        let e1 = Error::UnsolvableHouse(some_house(), known(1));
        let e2 = Error::DeadlyRectangle(some_rectangle());

        assert_eq!(e1.cell(), None);
        assert_eq!(e2.cell(), None);
    }

    #[test]
    fn is_invalid_returns_true_for_invalid_errors() {
        let e1 = Error::NotCandidate(cell(0), known(1));
        let e2 = Error::AlreadySolved(cell(1), known(2), known(3));
        let e3 = Error::UnsolvableCell(cell(2));
        let e4 = Error::UnsolvableHouse(some_house(), known(1));

        assert!(e1.is_invalid());
        assert!(e2.is_invalid());
        assert!(e3.is_invalid());
        assert!(e4.is_invalid());
    }

    #[test]
    fn is_invalid_returns_false_for_non_invalid_error() {
        let e = Error::DeadlyRectangle(some_rectangle());
        assert!(!e.is_invalid());
    }

    #[test]
    fn display_formats_correctly() {
        let h = some_house();
        let r = some_rectangle();

        let e1 = Error::NotCandidate(cell(0), known(1));
        let e2 = Error::AlreadySolved(cell(1), known(2), known(3));
        let e3 = Error::UnsolvableCell(cell(2));
        let e4 = Error::UnsolvableHouse(h, known(1));
        let e5 = Error::DeadlyRectangle(r);

        let s1 = format!("{}", e1);
        let s2 = format!("{}", e2);
        let s3 = format!("{}", e3);
        let s4 = format!("{}", e4);
        let s5 = format!("{}", e5);

        assert!(s1.contains(&cell(0).to_string()));
        assert!(s1.contains(&known(1).to_string()));

        assert!(s2.contains(&cell(1).to_string()));
        assert!(s2.contains(&known(2).to_string()));
        assert!(s2.contains(&known(3).to_string()));

        assert!(s3.contains(&cell(2).to_string()));

        assert!(s4.contains(&h.to_string()));
        assert!(s4.contains(&known(1).to_string()));

        assert!(s5.contains(&r.to_string()));
    }
}
