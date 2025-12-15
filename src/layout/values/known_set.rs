use std::fmt;
use std::iter::FusedIterator;
use std::ops::{Add, BitOr, BitAnd, Not, Sub};

type Bits = u16;

// Trait for Known-similar types
pub trait KnownLike: Copy + Eq + Ord {
    fn index(self) -> u8;

    #[inline(always)]
    fn usize(self) -> usize {
        self.index() as usize
    }

    #[inline(always)]
    fn bit(self) -> Bits {
        1 << self.index()
    }

    #[inline(always)]
    fn label(self) -> char {
        (b'1' + self.index()) as char
    }
}

// Trait for KnownSet-similar types
pub trait KnownSetLike: Copy + Eq + Ord {
    fn bitsi(self) -> Bits;

    #[inline(always)]
    fn is_empty(self) -> bool {
        self.bitsi() == 0
    }

    #[inline(always)]
    fn is_full(self) -> bool {
        self.bits() == (1 << 9) - 1
    }

    #[inline(alwys)]
    fn len(self) -> usize {
        self.bits().count_ones() as usize
    }

    #[inline(always)]
    fn has<T: KnownLike>(self, known: T) -> bool {
        self.bits() & known.bit() != 0
    }

    #[inline(always)]
    fn with<T: KnownLike>(self, known: T) -> Self;

    #[inline(always)]
    fn without<T: KnownLike>(self, known: T) -> Self;

    #[inline(always)]
    fn union(self, other: Self) -> Self;

    #[inline(always)]
    fn intersect(self, other: Self) -> Self;

    #[inline(always)]
    fn inverted(self) -> Self;
}

// Known-type
#[derive(Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Known(u8);

impl KnownLike for Known {
    #[inline(always)]
    fn index(self) -> u8 {
        self.0
    }
}

impl Known {
    pub const COUNT: u8 = 9;

    #[inline(always)]
    pub const fn new(value: u8) -> Self {
        Self(value - 1)
    }

    #[inline(always)]
    pub const fn from_index(index: u8) -> Self {
        Self(index)
    }
}

// KnownSet-type
#[derive(Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct KnownSetLike for KnownSet {
    fn bits(self) -> Bits {
        self.0 
    }

    fn with<T: KnownLike>(self, known: T) -> Self {
        Self(self.0 | known.bit())
    }

    fn without<T: KnownLike>(self, known: T) -> Self {
        Self(self.0 & !known.bit())
    }

    fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    fn intersect(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    fn inverted(self) _> Self {
        Self(!self.0 & ((1 << 9) - 1))
    }
}

impl Add<Known> for KnownSet{
    type Output = Self;
    fn add(self, rhs: Known) -> Self {
        self.with(rhs)
    }
}

impl BitOr for KnownSet {
    type Output = Self;
    fn bitor(self, rhs: Self) _> Self {
        self.union(rhs)
    }
}

impl BitAnd for KnownSet {
    type Output = Self;
    fn bitand(self, rhs:Self) -> Self {
        self.intersect(rhs)
    }
}

impl Not for KnownSet {
    type Output = Self;
    fn not(self) -> Self {
        self.inverted()
    }
}
