use std::fmt;
use std::iter::FusedIterator;
use std::ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Sub, SubAssign};
use std::iter::FromIterator;

type Bits = u16;

// Trait for Known-like types
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

// Trait for KnownSet-like types
pub trait KnownSetLike: Copy + Eq + Ord {
    fn bits(self) -> Bits;

    #[inline(always)]
    fn is_empty(self) -> bool {
        self.bits() == 0
    }

    #[inline(always)]
    fn is_full(self) -> bool {
        self.bits() == (1 << Known::COUNT) - 1
    }

    #[inline(always)]
    fn len(self) -> usize {
        self.bits().count_ones() as usize
    }

    #[inline(always)]
    fn has<T: KnownLike>(self, known: T) -> bool {
        self.bits() & known.bit() != 0
    }

    fn with<T: KnownLike>(self, known: T) -> Self;
    fn without<T: KnownLike>(self, known: T) -> Self;
    fn union(self, other: Self) -> Self;
    fn intersect(self, other: Self) -> Self;
    fn inverted(self) -> Self;

    fn has_any(self, other: Self) -> bool;
    fn has_all(self, other: Self) -> bool;
    fn is_subset_of(self, other: Self) -> bool;

    fn add(&mut self, known: Known);
    fn remove(&mut self, known: Known);
    fn union_with(&mut self, other: Self);
    fn intersect_with(&mut self, other: Self);
    fn subtract(&mut self, other: Self);
    fn invert(&mut self);
}

// Known type
#[derive(Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Known(u8);

impl Known {
    pub const COUNT: u8 = 9;

    #[inline(always)]
    pub const fn new(value: u8) -> Self {
        debug_assert!(value >= 1 && value <= 9);
        Self(value - 1)
    }

    #[inline(always)]
    pub const fn from_index(index: u8) -> Self {
        debug_assert!(index < 9);
        Self(index)
    }

    pub fn iter() -> KnownIter {
        KnownIter::new()
    }
}

impl KnownLike for Known {
    #[inline(always)]
    fn index(self) -> u8 {
        self.0
    }
}

// KnownSet type
#[derive(Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct KnownSet(u16);

impl KnownSet {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn full() -> Self {
        Self((1 << Known::COUNT) - 1)
    }

    pub fn first(&self) -> Option<Known> {
        if self.is_empty() {
            None
        } else {
            Some(Known::from_index(self.bits().trailing_zeros() as u8))
        }
    }

    pub fn pop(&mut self) -> Option<Known> {
        if let Some(k) = self.first() {
            self.remove(k);
            Some(k)
        } else {
            None
        }
    }

    pub fn as_single(&self) -> Option<Known> {
        if self.len() != 1 {
            None
        } else {
            Some(Known::from_index(self.bits().trailing_zeros() as u8))
        }
    }

    pub fn as_pair(&self) -> Option<(Known, Known)> {
        if self.len() != 2 {
            None
        } else {
            let mut bits = self.bits();
            let first = Known::from_index(bits.trailing_zeros() as u8);
            bits &= !first.bit();
            let second = Known::from_index(bits.trailing_zeros() as u8);
            Some((first, second))
        }
    }

    pub fn as_triple(&self) -> Option<(Known, Known, Known)> {
        if self.len() != 3 {
            None
        } else {
            let mut bits = self.bits();
            let first = Known::from_index(bits.trailing_zeros() as u8);
            bits &= !first.bit();
            let second = Known::from_index(bits.trailing_zeros() as u8);
            bits &= !second.bit();
            let third = Known::from_index(bits.trailing_zeros() as u8);
            Some((first, second, third))
        }
    }

    pub fn iter(&self) -> KnownSetIter {
        KnownSetIter {
            bits: self.bits()
        }
    }

    pub fn debug(&self) -> String {
        format!(
            "{:01}:{:09b}",
            self.len(),
            self.bits().reverse_bits() >> (16 - 9)
        )
    }
}

impl KnownSetLike for KnownSet {
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

    fn inverted(self) -> Self {
        Self(!self.0 & ((1 << Known::COUNT) - 1))
    }

    fn has_any(self, other: Self) -> bool {
        self.bits() & other.bits() != 0
    }

    fn has_all(self, other: Self) -> bool {
        (self.bits() & other.bits()) == other.bits()
    }

    fn is_subset_of(self, other: Self) -> bool {
        self.bits() & other.bits() == self.bits()
    }

    fn add(&mut self, known: Known) {
        self.0 |= known.bit();
    }

    fn remove(&mut self, known:Known) {
        self.0 &= !known.bit();
    }

    fn union_with(&mut self, other: Self) {
        self.0 |= other.0;
    }

    fn intersect_with(&mut self, other: Self) {
        self.0 &= other.0;
    }

    fn subtract(&mut self, other: Self) {
        self.0 &= !other.0;
    }

    fn invert(&mut self) {
        self.0 = !self.0 & ((1 << Known::COUNT) - 1)
    }
}

// Operator implementations
impl Add<Known> for KnownSet{
    type Output = Self;
    fn add(self, rhs: Known) -> Self {
        self.with(rhs)
    }
}

impl AddAssign<Known> for KnownSet {
    fn add_assign(&mut self, rhs: Known) {
        KnownSetLike::add(self,rhs)
    }
}

impl Sub<Known> for KnownSet {
    type Output = Self;
    fn sub(self, rhs: Known) -> Self {
        self.without(rhs)
    }
}

impl SubAssign<Known> for KnownSet {
    fn sub_assign(&mut self, rhs: Known) {
        self.remove(rhs)
    }
}

impl BitOr for KnownSet {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl BitOrAssign for KnownSet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.union_with(rhs)
    }
}

impl BitAnd for KnownSet {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        self.intersect(rhs)
    }
}

impl BitAndAssign for KnownSet {
    fn bitand_assign(&mut self, rhs: Self) {
        self.intersect_with(rhs)
    }
}

impl Sub for KnownSet {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 & !rhs.0)
    }
}

impl SubAssign for KnownSet {
    fn sub_assign(&mut self, rhs: Self) {
        self.subtract(rhs)
    }
}

impl Not for KnownSet {
    type Output = Self;
    fn not(self) -> Self {
        self.inverted()
    }
}

// Iterator for KnownSet
pub struct KnownSetIter{
    bits: Bits,
}

impl Iterator for KnownSetIter {
    type Item = Known;
    fn next(&mut self) -> Option<Known> {
        if self.bits == 0 {
            None
        } else {
            let idx = self.bits.trailing_zeros() as u8;
            self.bits &= !(1 << idx);
            Some(Known::from_index(idx))
        }
    }
}

impl FusedIterator for KnownSetIter {}

// Iterator for Known
pub struct KnownIter(u8);

impl KnownIter {
    pub const fn new() -> Self {
        Self(0)
    }
}

impl Iterator for KnownIter{
    type Item = Known;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 < Known::COUNT {
            let k = Known::from_index(self.0);
            self.0 += 1;
            Some(k)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for KnownIter {
    fn len(&self) -> usize {
        (Known::COUNT - self.0) as usize
    }
}

// Display / Debug
impl fmt::Display for KnownSet {
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s: String = Known::iter()
        .map(|k| if self.has(k) {
        k.label()
    } else { '·'})
        .collect();
    write!(f, "({})", s)
    }
}

impl fmt::Debug for KnownSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for Known {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl fmt::Debug for Known {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl FromIterator<Known> for KnownSet {
    fn from_iter<I: IntoIterator<Item = Known>>(iter: I) -> Self {
        let mut set = KnownSet::empty();
        for k in iter {
            set.add(k);
        }
        set
    }
}

impl FromIterator<KnownSet> for KnownSet {
    fn from_iter<I: IntoIterator<Item = KnownSet>>(iter: I) -> Self {
        let mut set = KnownSet::empty();
        for s in iter {
            set |= s;
        }
        set
    }
}

pub trait KnownIteratorUnion {
    fn union(self) -> KnownSet;
    fn union_knowns(self) -> KnownSet;
}

impl<I> KnownIteratorUnion for I
    where
    I: Iterator<Item = Known>,
    {
        fn union(self) -> KnownSet{
            self.union_knowns()
        }

        fn union_knowns(self) -> KnownSet {
            self.fold(KnownSet::empty(), |acc, h| acc + h)
        }
    }

pub trait KnownSetIteratorUnion {
    fn union(self) -> KnownSet;
    fn union_knowns(self) -> KnownSet;
}

impl<I> KnownSetIteratorUnion for I
    where
    I: Iterator<Item = KnownSet>,
    {
        fn union(self) -> KnownSet {
            self.union_knowns()
        }

        fn union_knowns(self) -> KnownSet {
            self.fold(KnownSet::empty(), |acc, h| acc | h)
        }
    }

pub trait KnownSetIteratorIntersection {
    fn intersection(self) -> KnownSet;
}

impl<I> KnownSetIteratorIntersection for I
    where
    I: Iterator<Item = KnownSet>,
    {
        fn intersection(self) -> KnownSet {
            self.fold(KnownSet::full(), |acc, h| acc & h)
        }
    }