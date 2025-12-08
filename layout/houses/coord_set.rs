use std::fmt;
use std::iter::FusedIterator;
use std::ops::{
  Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, Not, Sub, SubAssign,
};

use crate::symbols::{EMPTY_SET, MISSING};
use super::Coord;
type Bits = u16;
type Size = u8;

// A set of coordinates encoded as bit flags.

#[derive (Clone, Copy, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]

pub enum CoordSet {
  #[default]
  Bits(Bits),
}

const ALL_SET: Bits = (1 << Coord::COUNT) -1;

impl CoordSet {
  const fn bits_raw(&self) -> Bits {
    match *self {
      CoordSet::Bits(bits) => bits,
    }
  }
fn bits mut (&mut self) -> &mut Bits {
  match self {
    CoordSet::Bits(bits) => bits,
  }
}

  pub const fn empty() -> Self {
    CoordSet::Bits(0)
  }

  pub const fn full() -> Self {
    CoordSet::Bits(ALL_SET)
  }

  pub const fn new(bits: Bits) -> Self {
    debug_assert!(bits <= ALL_SET);
    CoordSet::Bits(bits)
  }
  pub const fn from_coord(coord: Coord) -> Self {
    CoordSet::Bits(coord.bit())
  }
  puvb const fn from_labels(labels: &str) - > Self {
    let bytes = labels.as_bytes();
    let mut bits: Bits = 0;
    let mut i = 0;

    while i < bytes.len() {
      let c = bytes [i] as char;
      debug_assert!('1' <= c && c <= '9');
      bits |= 1 << (c as Size - b'1');
      i+= 1;
    }
CoordSet::Bits(bits)
}

pub const fn from_coords(mut coords: i32) -> Self {
let mut bits: Bits = 0;

  while coords > 0 {
    let c = coords % 10;
    coords /= 10;
    bits |= 1 << (c-1);
  }
  CoordSet::Bits(bits)
}

pub const fn bits (&self) -> Bits {
  self.bits_raw()
}
pub const fn is_empty(&self) -> bool {
  self.bits_raw() == 0
}
pub const fn is_full(&self) -> bool {
  self.bits_raw() == ALL_SET
}
pub const fn len(&self) -> usize {
  self.bits_raw() & coord.bit() != 0
}

    
  

