use std::ops::{Deref, DerefMut};

use super::label::index_from_label;
use super::Cell;

// bezeichnet eine einzelne Zelle anhand ihrer Position in einem Bitfeld
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::Cell;

    #[test]
    fn test_bit_new_and_index() {
        // erstes Bit
        let bit = Bit::new(1);
        assert_eq!(bit.index(), 0);
        assert_eq!(bit.bit(), 1);

        // Bit an Position 5
        let bit2 = Bit::new(1 << 5);
        assert_eq!(bit2.index(), 5);
        assert_eq!(bit2.bit(), 32);
    }

    #[test]
    #[should_panic]
    fn test_bit_new_invalid() {
        // Mehr als ein Bit gesetzt -> soll panic
        Bit::new(3);
    }

    #[test]
    fn test_bit_cell_conversion() {
        let cell = Cell::new(7);
        let bit: Bit = cell.into();
        assert_eq!(bit.index(), 7);
        assert_eq!(bit.cell(), cell);
    }

    #[test]
    fn test_bit_from_label() {
        let bit = Bit::from("A1");
        assert_eq!(bit.index(), 0);

        let bit2 = Bit::from("C3");
        let expected_index = index_from_label("C3");
        assert_eq!(bit2.index(), expected_index);
    }

    #[test]
    fn test_bit_max_and_all() {
        assert_eq!(Bit::MAX, 1 << (Cell::COUNT - 1));
        assert_eq!(Bit::ALL, (1 << Cell::COUNT) - 1);
    }

    #[test]
    fn test_bit_deref() {
        let mut bit = Bit::new(1 << 4);
        assert_eq!(*bit, 1 << 4);

        // DerefMut testen
        *bit = 1 << 5;
        assert_eq!(bit.index(), 5);
    }
}
