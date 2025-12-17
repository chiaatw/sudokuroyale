use std::ops::{Deref, DerefMut};

use super::label::index_from_label;
use super::Cell;

/// Specifies a single cell by its position in a bit field.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Bit {
    Value(u128),
}

impl Bit {
    pub const MAX: u128 = 1 << (Cell::COUNT - 1);
    pub const ALL: u128 = (1 << Cell::COUNT) - 1;

    pub const fn new(bit: u128) -> Self {
        debug_assert!(bit <= Bit::MAX && bit.count_ones() == 1);
        Bit::Value(bit)
    }

    pub const fn bit(&self) -> u128 {
        match *self {
            Bit::Value(bit) => bit,
        }
    }

    pub const fn index(&self) -> u8 {
        match *self {
            Bit::Value(bit) => bit.trailing_zeros() as u8,
        }
    }

    pub const fn cell(&self) -> Cell {
        Cell::new(self.index())
    }
}

impl Deref for Bit {
    type Target = u128;

    fn deref(&self) -> &u128 {
        match self {
            Bit::Value(bit) => bit,
        }
    }
}

impl DerefMut for Bit {
    fn deref_mut(&mut self) -> &mut u128 {
        match self {
            Bit::Value(bit) => bit,
        }
    }
}

impl From<&str> for Bit {
    fn from(label: &str) -> Self {
        Bit::Value(1 << index_from_label(label))
    }
}

impl From<Cell> for Bit {
    fn from(cell: Cell) -> Self {
        cell.bit()
    }
}
