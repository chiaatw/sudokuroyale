use std::fmt;
use std::iter::FusedIterator;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, Not, Sub, SubAssign,
};

use crate::symbols::{EMPTY_SET, MISSING};

use super::Coord;

type Bits = u16;
type Size = u8;

/// A set of coordinates encoded as bit flags.
#[derive(Clone, Copy, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum CoordSet {
    #[default]
    Bits(Bits),
}

const ALL_SET: Bits = (1 << Coord::COUNT) - 1;

impl CoordSet {
    const fn bits_raw(&self) -> Bits {
        match *self {
            CoordSet::Bits(bits) => bits,
        }
    }

    fn bits_mut(&mut self) -> &mut Bits {
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

    pub const fn from_labels(labels: &str) -> Self {
        let bytes = labels.as_bytes();
        let mut bits: Bits = 0;
        let mut i = 0;

        while i < bytes.len() {
            let c = bytes[i] as char;
            debug_assert!('1' <= c && c <= '9');
            bits |= 1 << (c as Size - b'1');
            i += 1;
        }
        CoordSet::Bits(bits)
    }

    pub const fn from_coords(mut coords: i32) -> Self {
        let mut bits: Bits = 0;

        while coords > 0 {
            let c = coords % 10;
            coords /= 10;
            bits |= 1 << (c - 1);
        }
        CoordSet::Bits(bits)
    }

    pub const fn bits(&self) -> Bits {
        self.bits_raw()
    }

    pub const fn is_empty(&self) -> bool {
        self.bits_raw() == 0
    }

    pub const fn is_full(&self) -> bool {
        self.bits_raw() == ALL_SET
    }

    pub const fn len(&self) -> usize {
        self.bits_raw().count_ones() as usize
    }

    pub const fn has(&self, coord: Coord) -> bool {
        self.bits_raw() & coord.bit() != 0
    }

    pub const fn has_any(&self, set: CoordSet) -> bool {
        let bits = self.bits_raw() & set.bits_raw();
        bits != 0
    }

    pub const fn has_all(&self, subset: CoordSet) -> bool {
        let a = self.bits_raw() & subset.bits_raw();
        a == subset.bits_raw()
    }

    pub const fn is_subset_of(&self, superset: CoordSet) -> bool {
        let a = self.bits_raw() & superset.bits_raw();
        a == self.bits_raw()
    }

    pub const fn as_single(&self) -> Option<Coord> {
        if self.len() != 1 {
            None
        } else {
            Some(Coord::from_index(self.bits_raw().trailing_zeros()))
        }
    }

    pub const fn as_pair(&self) -> Option<(Coord, Coord)> {
        if self.len() != 2 {
            None
        } else {
            let mut bits = self.bits_raw();
            let first = Coord::from_index(bits.trailing_zeros());
            bits &= !first.bit();
            let second = Coord::from_index(bits.trailing_zeros());
            Some((first, second))
        }
    }

    pub const fn as_triple(&self) -> Option<(Coord, Coord, Coord)> {
        if self.len() != 3 {
            None
        } else {
            let mut bits = self.bits_raw();
            let first = Coord::from_index(bits.trailing_zeros());
            bits &= !first.bit();
            let second = Coord::from_index(bits.trailing_zeros());
            bits &= !second.bit();
            let third = Coord::from_index(bits.trailing_zeros());
            Some((first, second, third))
        }
    }

    pub const fn with(&self, coord: Coord) -> Self {
        CoordSet::Bits(self.bits_raw() | coord.bit())
    }

    pub fn add(&mut self, coord: Coord) {
        *self.bits_mut() |= coord.bit();
    }

    pub const fn without(&self, coord: Coord) -> Self {
        CoordSet::Bits(self.bits_raw() & !coord.bit())
    }

    pub fn remove(&mut self, coord: Coord) {
        *self.bits_mut() &= !coord.bit();
    }

    pub const fn first(&self) -> Option<Coord> {
        if self.is_empty() {
            None
        } else {
            Some(Coord::new(self.bits_raw().trailing_zeros() as Size))
        }
    }

    pub fn pop(&mut self) -> Option<Coord> {
        let bits = self.bits_raw();
        if bits == 0 {
            return None;
        }
        let tz = bits.trailing_zeros() as Size;
        let coord = Coord::new(tz);
        *self.bits_mut() &= !coord.bit();
        Some(coord)
    }

    pub const fn union(&self, set: Self) -> Self {
        CoordSet::Bits(self.bits_raw() | set.bits_raw())
    }

    pub fn union_with(&mut self, set: Self) {
        *self = self.union(set);
    }

    pub const fn intersect(&self, set: Self) -> Self {
        CoordSet::Bits(self.bits_raw() & set.bits_raw())
    }

    pub fn intersect_with(&mut self, set: Self) {
        *self = self.intersect(set);
    }

    pub const fn minus(&self, set: Self) -> Self {
        CoordSet::Bits(self.bits_raw() & !set.bits_raw())
    }

    pub fn subtract(&mut self, set: Self) {
        *self = self.minus(set);
    }

    pub const fn inverted(&self) -> Self {
        CoordSet::Bits(!self.bits_raw() & ALL_SET)
    }

    pub fn invert(&mut self) {
        *self = self.inverted();
    }

    pub const fn iter(&self) -> Iter {
        Iter { bits: self.bits_raw() }
    }

    pub fn debug(&self) -> String {
        format!(
            "{:01}:{:09b}",
            self.len(),
            self.bits_raw().reverse_bits() >> (16 - 9)
        )
    }
}

impl From<&str> for CoordSet {
    fn from(labels: &str) -> Self {
        labels.split(' ').map(Coord::from).union_coords()
    }
}

impl From<i32> for CoordSet {
    fn from(coords: i32) -> Self {
        CoordSet::from_coords(coords)
    }
}

impl IntoIterator for CoordSet {
    type Item = Coord;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub trait CoordIteratorUnion {
    fn union(self) -> CoordSet;
    fn union_coords(self) -> CoordSet;
}

impl<I> CoordIteratorUnion for I
where
    I: Iterator<Item = Coord>,
{
    fn union(self) -> CoordSet {
        self.union_coords()
    }

    fn union_coords(self) -> CoordSet {
        self.fold(CoordSet::empty(), |acc, h| acc + h)
    }
}

pub trait CoordSetIteratorUnion {
    fn union(self) -> CoordSet;
    fn union_coords(self) -> CoordSet;
}

impl<I> CoordSetIteratorUnion for I
where
    I: Iterator<Item = CoordSet>,
{
    fn union(self) -> CoordSet {
        self.union_coords()
    }

    fn union_coords(self) -> CoordSet {
        self.fold(CoordSet::empty(), |acc, h| acc | h)
    }
}

pub trait CoordSetIteratorIntersection {
    fn intersection(self) -> CoordSet;
}

impl<I> CoordSetIteratorIntersection for I
where
    I: Iterator<Item = CoordSet>,
{
    fn intersection(self) -> CoordSet {
        self.fold(CoordSet::full(), |acc, h| acc & h)
    }
}

impl FromIterator<Coord> for CoordSet {
    fn from_iter<I: IntoIterator<Item = Coord>>(iter: I) -> Self {
        let mut set = CoordSet::empty();
        for coord in iter {
            set += coord;
        }
        set
    }
}

impl FromIterator<Self> for CoordSet {
    fn from_iter<I: IntoIterator<Item = Self>>(iter: I) -> Self {
        let mut union = CoordSet::empty();
        for set in iter {
            union |= set;
        }
        union
    }
}

impl Index<Coord> for CoordSet {
    type Output = bool;

    fn index(&self, coord: Coord) -> &bool {
        if self.has(coord) {
            &true
        } else {
            &false
        }
    }
}

impl Add<Coord> for CoordSet {
    type Output = Self;

    fn add(self, rhs: Coord) -> Self::Output {
        self.with(rhs)
    }
}

impl AddAssign<Coord> for CoordSet {
    fn add_assign(&mut self, rhs: Coord) {
        self.add(rhs)
    }
}

impl Sub<Coord> for CoordSet {
    type Output = Self;
    fn sub(self, rhs: Coord) -> Self {
        self.without(rhs)
    }
}

impl SubAssign<Coord> for CoordSet {
    fn sub_assign(&mut self, rhs: Coord) {
        self.remove(rhs)
    }
}

impl Not for CoordSet {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.inverted()
    }
}

impl BitOr for CoordSet {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl BitOrAssign for CoordSet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.union_with(rhs)
    }
}

impl BitAnd for CoordSet {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersect(rhs)
    }
}

impl BitAndAssign for CoordSet {
    fn bitand_assign(&mut self, rhs: Self) {
        self.intersect_with(rhs)
    }
}

impl Sub for CoordSet {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.minus(rhs)
    }
}

impl SubAssign for CoordSet {
    fn sub_assign(&mut self, rhs: Self) {
        self.subtract(rhs)
    }
}

impl fmt::Display for CoordSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "{}", EMPTY_SET)
        } else {
            let mut s = String::with_capacity(2 + 9);
            s.push('(');
            (0..9).for_each(|c| {
                if self.has(c.into()) {
                    s.push((b'1' + c as u8) as char);
                } else {
                    s.push(MISSING);
                }
            });
            s.push(')');
            write!(f, "{}", s)
        }
    }
}

impl fmt::Debug for CoordSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

pub struct Iter {
    bits: Bits,
}

impl Iterator for Iter {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            None
        } else {
            let bit = 1 << self.bits.trailing_zeros();
            self.bits &= !bit;
            Some(Coord::from(bit.trailing_zeros() as u8))
        }
    }
}

impl FusedIterator for Iter {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Coord;

    #[test]
    fn test_empty_and_full() {
        let empty = CoordSet::empty();
        assert!(empty.is_empty());
        assert!(!empty.is_full());
        assert_eq!(empty.len(), 0);

        let full = CoordSet::full();
        assert!(!full.is_empty());
        assert!(full.is_full());
        assert_eq!(full.len(), 9);
    }

    #[test]
    fn test_from_coord_and_bits() {
        let c = Coord::C3;
        let set = CoordSet::from_coord(c);
        assert!(set.has(c));
        assert_eq!(set.len(), 1);
        assert_eq!(set.bits(), c.bit());
    }

    #[test]
    fn test_from_coords_and_labels() {
        let set = CoordSet::from_coords(123);
        assert!(set.has(Coord::C0));
        assert!(set.has(Coord::C1));
        assert!(set.has(Coord::C2));
        assert_eq!(set.len(), 3);

        let set2 = CoordSet::from_labels("1 2 3");
        assert_eq!(set, set2);
    }

    #[test]
    fn test_union_intersect_minus_invert() {
        let set1 = CoordSet::from_labels("1 2 3");
        let set2 = CoordSet::from_labels("3 4 5");

        let union = set1.union(set2);
        for c in &[Coord::C0, Coord::C1, Coord::C2, Coord::C3, Coord::C4] {
            assert!(union.has(*c));
        }
        assert_eq!(union.len(), 5);

        let intersect = set1.intersect(set2);
        assert!(intersect.has(Coord::C2));
        assert_eq!(intersect.len(), 1);

        let minus = set1.minus(set2);
        assert!(minus.has(Coord::C0));
        assert!(minus.has(Coord::C1));
        assert!(!minus.has(Coord::C2));
        assert_eq!(minus.len(), 2);

        let inverted = set1.inverted();
        for c in &[Coord::C0, Coord::C1, Coord::C2] {
            assert!(!inverted.has(*c));
        }
        assert_eq!(inverted.len(), 6);
    }

    #[test]
    fn test_as_single_pair_triple() {
        let single = CoordSet::from_coord(Coord::C5);
        assert_eq!(single.as_single(), Some(Coord::C5));
        assert_eq!(single.as_pair(), None);
        assert_eq!(single.as_triple(), None);

        let pair = CoordSet::from_labels("2 4");
        assert_eq!(pair.as_single(), None);
        assert_eq!(pair.as_pair(), Some((Coord::C1, Coord::C3)));
        assert_eq!(pair.as_triple(), None);

        let triple = CoordSet::from_labels("1 3 5");
        assert_eq!(triple.as_single(), None);
        assert_eq!(triple.as_pair(), None);
        assert_eq!(triple.as_triple(), Some((Coord::C0, Coord::C2, Coord::C4)));
    }

    #[test]
    fn test_iteration_and_pop() {
        let mut set = CoordSet::from_labels("1 2 3");
        let mut collected = vec![];
        while let Some(c) = set.pop() {
            collected.push(c);
        }
        collected.sort_by_key(|c| c.index());
        assert_eq!(collected, vec![Coord::C0, Coord::C1, Coord::C2]);
        assert!(set.is_empty());
    }

    #[test]
    fn test_operator_traits() {
        let mut set = CoordSet::empty();
        set += Coord::C0;
        set += Coord::C2;
        assert!(set.has(Coord::C0));
        assert!(set.has(Coord::C2));
        assert_eq!(set.len(), 2);

        let set2 = CoordSet::from_labels("2 3");
        let union = set | set2;
        for c in &[Coord::C0, Coord::C1, Coord::C2] {
            assert!(union.has(*c));
        }

        let intersect = set & set2;
        assert!(intersect.has(Coord::C2));
        assert_eq!(intersect.len(), 1);

        let minus = union - set;
        assert!(minus.has(Coord::C1));
        assert!(!minus.has(Coord::C0));
        assert!(!minus.has(Coord::C2));

        let inverted = !set;
        assert!(!inverted.has(Coord::C0));
        assert!(inverted.has(Coord::C1));
    }

    #[test]
    fn test_index_trait() {
        let set = CoordSet::from_labels("1 3 5");
        assert!(set[Coord::C0]);
        assert!(!set[Coord::C1]);
        assert!(set[Coord::C2]);
    }

    #[test]
    fn test_display_debug() {
        let empty = CoordSet::empty();
        assert_eq!(format!("{}", empty), EMPTY_SET.to_string());

        let set = CoordSet::from_labels("1 3 5");
        let display = format!("{}", set);
        assert!(display.contains('1'));
        assert!(display.contains('3'));
        assert!(display.contains('5'));
    }
}
