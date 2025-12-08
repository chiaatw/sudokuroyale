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
  
