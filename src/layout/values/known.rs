use std::fmt;
use std::ops::{Add, Neg};

use super::{KnownSet, Value};

pub trait KnownLike: Copy + Eq + Ord {
    fn index(self) -> u8;

    #[inline(always)]
    fn usize(self) -> usize {
        self.index() as usize
    }

    #[inline(always)]
    fn bit(self) -> u16 {
        1u16 << self.index()
    }

    #[inline(always)]
    fn value(self) -> Value {
        Value::new(self.index() + 1)
    }

    #[inline(always)]
    fn label(self) -> char {
        (b'1' + self.index()) as char
    }

    #[inline(always)]
    fn highlight(self) -> char {
        HIGHLIGHT_LABELS[self.usize()]
    }
}

#[derive(Clone, Copy, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Known(u8);

impl Known {
    pub const COUNT: u8 = 9;

    #[inline(always)]
    pub const fn new(value: u8) -> Self {
        debug_assert!(1 <= value && value <= 9);
        Self(value - 1)
    }

    #[inline(always)]
    pub const fn from_index(index: u8) -> Self {
        debug_assert!(index < 9);
        Self(index)
    }

    #[inline(always)]
    pub fn iter() -> KnownIter {
        KnownIter::new()
    }

    pub fn from_char(label: char) -> Self {
        Self::try_from(label).unwrap_or_else(|e| panic!("{}", e))
    }

    pub fn from_str(label: &str) -> Self {
        Self::try_from(label).unwrap_or_else(|e| panic!("{}", e))
    }

    pub fn from_string(label: String) -> Self {
        Self::from_str(&label)
    }
}

impl KnownLike for Known {
    #[inline(always)]
    fn index(self) -> u8 {
        self.0
    }
}

impl TryFrom<char> for Known {
    type Error = String;

    fn try_from(label: char) -> Result<Self, Self::Error> {
        if ('1'..='9').contains(&label) {
            Ok(Known::new(label as u8 - b'0'))
        } else {
            Err(format!("Invalid digit \"{}\"", label))
        }
    }
}

impl TryFrom<&str> for Known {
    type Error = String;

    fn try_from(label: &str) -> Result<Self, Self::Error> {
        label
            .chars()
            .next()
            .ok_or_else(|| format!("Invalid digit \"{}\"", label))
            .and_then(Known::try_from)
    }
}

impl Add<Known> for Known {
    type Output = KnownSet;

    #[inline(always)]
    fn add(self, rhs: Known) -> KnownSet {
        KnownSet::empty() + self + rhs
    }
}

impl Neg for Known {
    type Output = KnownSet;

    #[inline(always)]
    fn neg(self) -> KnownSet {
        KnownSet::full() - self
    }
}

impl fmt::Display for Known {
    #[inline(always)]
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

pub struct KnownIter(u8);

impl KnownIter {
    #[inline(always)]
    pub const fn new() -> Self {
        Self(0)
    }
}

impl Iterator for KnownIter {
    type Item = Known;

    fn next(&mut self) -> Optioin<Self::Item> {
        if self.0 < 9 {
            let k = Known::from_index(self.0);
            self.0 += 1;
            Some(k)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for KnownIter {
    #[inline(always)]
    fn len(&self) -> usize {
        9 - self.0 as usize
    }
}

#[allow(unused_macros)]
macro_rules! known {
    ($:expr) => {
        Known::from_str($k)
    };
}

#[allow(unused_imports)]
pub(crate) use known;

const HIGHLIGHT_LABELS: [char; 9] = 
    ['1', '2', '3', '4', '5', '6', '7', '8', '9'];