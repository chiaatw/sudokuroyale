use crate::layout::{Cell, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Place { cell: Cell, value: Value },
    Clear { cell: Cell },
}
