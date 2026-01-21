use std::fmt;
use std::iter::FromIterator;
use std::iter::FusedIterator;
use std::ops::{Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Sub, SubAssign};

use crate::layout::values::known::{Known, KnownLike};

type Bits = u16;

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
        KnownSetIter { bits: self.bits() }
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

    fn remove(&mut self, known: Known) {
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
impl Add<Known> for KnownSet {
    type Output = Self;
    fn add(self, rhs: Known) -> Self {
        self.with(rhs)
    }
}

impl AddAssign<Known> for KnownSet {
    fn add_assign(&mut self, rhs: Known) {
        KnownSetLike::add(self, rhs)
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
pub struct KnownSetIter {
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

impl Iterator for KnownIter {
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
            .map(|k| if self.has(k) { k.label() } else { '·' })
            .collect();
        write!(f, "({})", s)
    }
}

impl fmt::Debug for KnownSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
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
            set = set.add(k);
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
    fn union(self) -> KnownSet {
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

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn known_new_and_index() {
        let k = Known::new(1);
        assert_eq!(k.index(), 0);
        let k = Known::new(9);
        assert_eq!(k.index(), 8);
    }

    #[test]
    fn known_from_index() {
        let k = Known::from_index(0);
        assert_eq!(k.value().value(), 1);
        let k = Known::from_index(8);
        assert_eq!(k.value().value(), 9);
    }

    #[test]
    fn known_bit() {
        let k = Known::new(3);
        assert_eq!(k.bit(), 1 << 2);
    }

    #[test]
    fn known_iter_yields_all() {
        let labels: Vec<_> = Known::iter().map(|k| k.label()).collect();
        assert_eq!(labels, ['1', '2', '3', '4', '5', '6', '7', '8', '9']);
    }

    #[test]
    fn knownset_empty_and_full() {
        let empty = KnownSet::empty();
        assert!(empty.is_empty());
        let full = KnownSet::full();
        assert!(full.is_full());
        assert_eq!(full.len(), 9);
    }

    #[test]
    fn knownset_add_remove() {
        let mut set = KnownSet::empty();
        let k = Known::new(4);
        KnownSetLike::add(&mut set, k);
        assert!(set.has(k));
        KnownSetLike::remove(&mut set, k);
        assert!(!set.has(k));
    }

    #[test]
    fn knownset_with_without_union_intersect() {
        let a = Known::new(2);
        let b = Known::new(5);
        let mut set = KnownSet::empty();
        set = set.with(a);
        set = set.with(b);
        assert!(set.has(a) && set.has(b));

        let set2 = set.without(a);
        assert!(!set2.has(a) && set2.has(b));

        let union = set.union(set2);
        assert!(union.has(a) && union.has(b));

        let intersect = set.intersect(set2);
        assert!(!intersect.has(a) && intersect.has(b));
    }

    #[test]
    fn knownset_inverted() {
        let mut set = KnownSet::empty();
        <KnownSet as KnownSetLike>::add(&mut set, Known::new(1));
        let inv = !set;
        assert!(!inv.has(Known::new(1)));
        assert!(inv.has(Known::new(2)));
    }

    #[test]
    fn knownset_as_single_pair_triple() {
        let set = KnownSet::empty().with(Known::new(1));
        assert_eq!(set.as_single().unwrap().index(), 0);

        let set2 = KnownSet::empty().with(Known::new(1)).with(Known::new(2));
        assert_eq!(set2.as_pair().unwrap(), (Known::new(1), Known::new(2)));

        let set3 = KnownSet::empty()
            .with(Known::new(1))
            .with(Known::new(2))
            .with(Known::new(3));
        assert_eq!(
            set3.as_triple().unwrap(),
            (Known::new(1), Known::new(2), Known::new(3))
        );
    }

    #[test]
    fn knownset_iter() {
        let set = KnownSet::empty().with(Known::new(1)).with(Known::new(3));
        let labels: Vec<_> = set.iter().map(|k| k.label()).collect();
        assert_eq!(labels, ['1', '3']);
    }

    #[test]
    fn operators_add_sub_bitwise() {
        let a = KnownSet::empty();
        let b = a + Known::new(2);
        assert!(b.has(Known::new(2)));
        let mut c = KnownSet::empty();
        c += Known::new(2);
        assert!(c.has(Known::new(2)));

        let d = KnownSet::full() - Known::new(2);
        assert!(!d.has(Known::new(2)));
        let mut e = KnownSet::full();
        e -= Known::new(2);
        assert!(!e.has(Known::new(2)));

        let f = KnownSet::empty() | KnownSet::empty();
        assert!(f.is_empty());
        let g = KnownSet::full() & KnownSet::full();
        assert!(g.is_full());
    }

    #[test]
    fn from_iterator_known() {
        let ks: KnownSet = vec![Known::new(1), Known::new(3)].into_iter().collect();
        assert!(ks.has(Known::new(1)));
        assert!(ks.has(Known::new(3)));
    }

    #[test]
    fn from_iterator_knownset() {
        let sets: KnownSet = vec![
            KnownSet::empty().with(Known::new(1)),
            KnownSet::empty().with(Known::new(2)),
        ]
        .into_iter()
        .collect();
        assert!(sets.has(Known::new(1)));
        assert!(sets.has(Known::new(2)));
    }
}
