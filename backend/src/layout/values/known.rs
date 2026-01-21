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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

    fn next(&mut self) -> Option<Self::Item> {
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
    ($k:expr) => {
        crate::layout::values::known::Known::from_str($k)
    };
}

#[allow(unused_imports)]
pub(crate) use known;

const HIGHLIGHT_LABELS: [char; 9] = ['1', '2', '3', '4', '5', '6', '7', '8', '9'];

#[cfg(test)]
mod tests {

    use super::*;
    use crate::layout::values::known_set::KnownSetLike;

    #[test]
    fn new_maps_value_to_index() {
        let k = Known::new(1);
        assert_eq!(k.index(), 0);

        let k = Known::new(9);
        assert_eq!(k.index(), 8);
    }

    #[test]
    fn from_index_maps_back_to_value() {
        let k = Known::from_index(0);
        assert_eq!(k.value().value(), 1);

        let k = Known::from_index(8);
        assert_eq!(k.value().value(), 9);
    }

    #[test]
    fn known_like_consistency() {
        let k = Known::new(5);
        assert_eq!(k.index(), 4);
        assert_eq!(k.bit(), 1 << 4);
        assert_eq!(k.label(), '5');
    }

    #[test]
    fn try_from_char_valid() {
        let k = Known::try_from('7').unwrap();
        assert_eq!(k.index(), 6);
    }

    #[test]
    fn try_from_char_invalid() {
        assert!(Known::try_from('0').is_err());
        assert!(Known::try_from('x').is_err());
    }

    #[test]
    fn try_from_str() {
        assert!(Known::try_from("9").is_ok());
        assert!(Known::try_from("").is_err());
    }

    #[test]
    fn iter_yields_all_knowns() {
        let values: Vec<_> = Known::iter().map(|k| k.label()).collect();
        assert_eq!(values, ['1', '2', '3', '4', '5', '6', '7', '8', '9']);
    }

    #[test]
    fn iter_len_is_exact() {
        let mut iter = Known::iter();
        assert_eq!(iter.len(), 9);
        iter.next();
        assert_eq!(iter.len(), 8);
    }

    #[test]
    fn add_produces_knownset() {
        let a = Known::new(3);
        let b = Known::new(5);
        let set = a + b;
        assert!(set.has(a));
        assert!(set.has(b));
    }

    #[test]
    fn negation_excludes_value() {
        let k = Known::new(4);
        let set = -k;
        assert!(!set.has(k));
    }
}
