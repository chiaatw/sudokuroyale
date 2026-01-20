use crate::layout::values::known::KnownLike;
use std::fmt;
use std::ops::Not;

use super::Known;
use crate::symbols::MISSING;

pub trait ValueLike: Copy + Eq + Ord {
    fn raw(self) -> u8;

    #[inline(always)]
    fn is_unknown(self) -> bool {
        self.raw() == 0
    }

    #[inline(always)]
    fn is_known(self) -> bool {
        self.raw() != 0
    }

    #[inline(always)]
    fn label(self) -> char {
        if self.is_unknown() {
            MISSING
        } else {
            (b'0' + self.raw()) as char
        }
    }
}

#[derive(Clone, Copy, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Value(u8);

impl Value {
    pub const UNKNOWN: u8 = 0;

    #[inline(always)]
    pub const fn unknown() -> Self {
        Self(Self::UNKNOWN)
    }

    #[inline(always)]
    pub const fn new(value: u8) -> Self {
        if value <= 9 {
            Self(value)
        } else {
            Self(Self::UNKNOWN)
        }
    }

    #[inline(always)]
    pub const fn value(self) -> u8 {
        self.0
    }
    #[inline(always)]
    pub fn known(self) -> Option<Known> {
        if self.is_known() {
            Some(Known::new(self.0)) // 1..=9, denn 0 ist unknown
        } else {
            None
        }
    }
}

impl ValueLike for Value {
    #[inline(always)]
    fn raw(self) -> u8 {
        self.0
    }
}

impl From<Known> for Value {
    #[inline(always)]
    fn from(known: Known) -> Self {
        Value::new(known.value().0)
    }
}

impl From<u8> for Value {
    #[inline(always)]
    fn from(value: u8) -> Self {
        Value::new(value)
    }
}

impl From<char> for Value {
    #[inline(always)]
    fn from(label: char) -> Self {
        if ('1'..='9').contains(&label) {
            Value::new(label as u8 - b'0')
        } else {
            Value::unknown()
        }
    }
}

impl From<&str> for Value {
    #[inline(always)]
    fn from(label: &str) -> Self {
        label
            .chars()
            .next()
            .map(Value::from)
            .unwrap_or(Value::unknown())
    }
}

impl Not for Value {
    type Output = bool;

    #[inline(always)]
    fn not(self) -> bool {
        self.is_unknown()
    }
}

impl fmt::Display for Value {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl fmt::Debug for Value {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[allow(unused_macros)]
macro_rules! value {
    ($k:expr) => {
        Value::new($k as u8)
    };
}

#[allow(unused_imports)]
pub(crate) use value;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_is_zero() {
        let v = Value::unknown();
        assert!(v.is_unknown());
        assert_eq!(v.raw(), 0);
    }

    #[test]
    fn new_out_of_range_is_unknown() {
        assert!(Value::new(10).is_unknown());
        assert!(Value::new(255).is_unknown());
    }

    #[test]
    fn from_u8() {
        for i in 1..=9 {
            let v = Value::from(i);
            assert!(v.is_known());
            assert_eq!(v.raw(), i);
        }
    }

    #[test]
    fn from_char() {
        assert_eq!(Value::from('5').raw(), 5);
        assert!(Value::from('.').is_unknown());
        assert!(Value::from('0').is_unknown());
    }

    #[test]
    fn from_str() {
        assert_eq!(Value::from("9").raw(), 9);
        assert!(Value::from("").is_unknown());
    }

    #[test]
    fn label_known_and_unknown() {
        assert_eq!(Value::from(3).label(), '3');
        assert_eq!(Value::unknown().label(), MISSING);
    }

    #[test]
    fn not_operator() {
        assert!(!Value::unknown());
        assert!(!(!Value::from(4)));
    }

    #[test]
    fn display_and_debug_match_label() {
        let v = Value::from(7);
        assert_eq!(format!("{}", v), "7");
        assert_eq!(format!("{:?}", v), "7");
    }
}
