use std::fmt;
use std::iter::FusedIterator;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Sub, SubAssign, 
};

use crate::layout::CellSet;
use crate::symbols::EMPTY_SET;
use crate::layout::houses::shape::ShapeTrait;

use super::{Coord, CoordSet, House, Shape};

pub trait HouseSetLike: Copy + Sized {

    fn empty(shape: Shape) -> Self;
    fn full(shape: Shape) -> Self;
    fn from_bits(shape: Shape, bits: u16) -> Self;
    fn from_coords(shape: Shape, coords: i32) -> Self;

    fn shape(&self) -> Shape;
    fn is_empty(&self) -> bool;
    fn is_full(&self) -> bool;
    fn len(&self) -> usize;

    fn has_coord(&self, coord: Coord) -> bool;
    fn union(self, other: Self) -> Self;
    fn intersect(self, other: Self) -> Self;
    fn minus(self, other: Self) -> Self;
    fn inverted(self) -> Self;

    fn has_any(self, other: Self) -> bool {
        !self.intersect(other).is_empty()
    }

    fn has_all(self, subset: Self) -> bool {
        self.shape() == subset.shape() && self.intersect(subset).coords() == subset.coords()
    }

    fn is_subset_of(self, superset: Self) -> bool {
        self.shape() ==  superset.shape() && self.intersect(superset).coords() == self.coords()
    }

    fn coords(&self) -> CoordSet;
}

#[derive(Clone, Copy, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct HouseSet{
    shape: Shape,
    coords: CoordSet,
}

impl HouseSetLike for HouseSet {

    fn empty(shape: Shape) -> Self {
        Self {
            shape,
            coords: CoordSet::empty(),
        }
    }

    fn full(shape: Shape) -> Self {
        Self {
            shape,
            coords: CoordSet::full(),
        }
    }

    fn from_bits(shape: Shape, bits: u16) -> Self {
        Self {
            shape,
            coords: CoordSet::new(bits),
        }
    }

    fn from_coords(shape: Shape, coords: i32) -> Self {
        Self {
            shape,
            coords: CoordSet::from_coords(coords),
        }
    }

    fn shape(&self) -> Shape {
        self.shape
    }

    fn is_empty(&self) -> bool {
        self.coords.is_empty()
    }

    fn is_full(&self) -> bool {
        self.coords.is_full()
    }

    fn len(&self) -> usize {
        self.coords.len()
    }

    fn has_coord(&self, coord: Coord) -> bool {
        self.coords.has(coord)
    }

    fn union(self, other: Self) -> Self {
        if self.shape != other.shape {
            panic!("Cannot union {} and {}", self.shape, other.shape);
        }
        Self {
            shape: self.shape,
            coords: self.coords | other.coords,
        }
    }

    fn intersect(self, other: Self) -> Self {
        if self.shape != other.shape {
            panic!("Cannot intersect {} and {}", self.shape, other.shape);
        }
        Self {
            shape: self.shape,
            coords: self.coords & other.coords,
        }
    }

    fn minus(self, other: Self) -> Self {
        if self.shape != other.shape {
            panic!("Cannot subtract {} and {}", self.shape, other.shape);
        }
        Self {
            shape: self.shape,
            coords: self.coords & !other.coords,
        }
    }

    fn inverted(self) -> Self {
        Self {
            shape: self.shape,
            coords: !self.coords,
        }
    }

    fn coords(&self) -> CoordSet {
        self.coords
    }
}

impl HouseSet {

    pub const fn empty(shape: Shape) -> Self {
        Self {
            shape,
            coords: CoordSet::empty(),
        }
    }

    pub const fn full(shape: Shape) -> Self {
        Self {
            shape,
            coords: CoordSet::full(),
        }
    }

    pub const fn from_bits(shape: Shape, bits: u16) -> Self {
        Self {
            shape,
            coords: CoordSet::new(bits),
        }
    }

    pub const fn from_coords(shape: Shape, coords: i32) -> Self {
        Self {
            shape,
            coords: CoordSet::from_coords(coords),
        }
    }

    pub fn from_labels(shape: Shape, labels: &str) -> Self {
        labels
            .split_whitespace()
            .map(House::from)
            .fold(HouseSet::empty(shape), |set, h| set + h)
    }

    pub fn with_coord(&self, coord: Coord) -> Self {
        Self {
            shape: self.shape,
            coords: self.coords.with(coord),
        }
    }

    pub fn without_coord(&self, coord: Coord) -> Self {
        Self {
            shape: self.shape,
            coords: self.coords.without(coord),
        }
    }

    pub fn add_coord(&mut self, coord: Coord) {
        self.coords += coord;
    }

    pub fn remove_coord(&mut self, coord: Coord) {
        self.coords -= coord;
    }

    pub fn has(&self, house: House) -> bool {
        if self.shape != house.shape() {
            panic!("{} cannot be in {} set", house, self.shape);
        }
        self.coords.has(house.coord())
    }

    pub fn with(&self, house: House) -> Self {
        if self.shape != house.shape() {
            panic!("Cannot add {} to {} set", house, self.shape);
        }
        Self {
            shape: self.shape,
            coords: self.coords.with(house.coord())
        }
    }

    pub fn without(&self, house: House) -> Self {
        if self.shape != house.shape() {
            panic!("Cannot remove {} from {} set", house, self.shape);
        }
        Self {
            shape: self.shape,
            coords: self.coords.without(house.coord()),
        }
    }

    pub fn cells(&self) -> CellSet {
        self.iter().fold(CellSet::empty(), | acc, h| acc | h.cells())
    }

    pub fn iter(&self) -> Iter {
        Iter {
            shape: self.shape,
            coords: self.coords.bits(),
        }
    }

    pub fn as_single(&self) -> Option<House> {
        self.coords.as_single().map(|c| House::new(self.shape, c))
    }

    pub fn as_pair(&self) -> Option<(House, House)> {
        self.coords.as_pair().map(|(a, b)| (House::new(self.shape, a), House::new(self.shape, b)))
    }

    pub fn as_triple(&self) -> Option<(House, House, House)> {
        self.coords.as_triple().map(|(a, b, c)| (House::new(self.shape, a), House::new(self.shape, b), House::new(self.shape, c)))
    }

    pub fn add(&mut self, house: House) {
        self.coords += house.coord();
    }

    pub fn remove(&mut self, house: House) {
        self.coords -= house.coord();
    }

    pub fn union_with(&mut self, other:Self) {
        *self = *self | other;
    }

    pub fn intersect_with(&mut self, other: Self) {
        *self = *self & other;
    }

    pub fn subtract(&mut self, other: Self) {
        *self = *self - other;
    }

    pub fn invert(&mut self) {
        *self = !*self;
    }

    pub fn debug(&self) -> String {
        format!("HouseSet({} {})", self.shape, self.coords.debug())
    }
}

impl From<&str> for HouseSet {
    fn from(labels: &str) -> Self {
       labels
            .split_whitespace()
            .map(House::from)
            .fold(HouseSet::empty(self.shape), |mut acc, h| {
                acc.add(h);
                acc
            })
    }
}

impl Add<House> for HouseSet {
    type Output = Self;
    fn add(self, rhs: House) -> Self {
        self.with(rhs)
    }
}

impl Add<Coord> for HouseSet {
    type Output = Self;
    fn add(self, rhs: Coord) -> Self {
        self.with_coord(rhs)
    }
}

impl Sub<House> for HouseSet {
    type Output = Self;
    fn sub(self, rhs: House) -> Self {
        self.without(rhs)
    }
}

impl Sub<Coord> for HouseSet {
    type Output = Self;
    fn sub(self, rhs: Coord) -> Self {
        self.without_coord(rhs)
    }
}

impl BitOr for HouseSet {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl BitAnd for HouseSet {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        self.intersect(rhs)
    }
}

impl Not for HouseSet {
    type Output= Self;
    fn not(self) -> Self {
        self.inverted()
    }
}

impl AddAssign<House> for HouseSet {
    fn add_assign(&mut self, rhs: House) {
        *self = *self + rhs;
    }
}

impl AddAssign<Coord> for HouseSet {
    fn add_assign(&mut self, rhs: Coord) {
        self.add_coord(rhs)
    }
}

impl SubAssign<House> for HouseSet {
    fn sub_assign(&mut self, rhs: House) {
        *self = *self - rhs;
    }
}

impl SubAssign<Coord> for HouseSet {
    fn sub_assign(&mut self, rhs: Coord) {
        self.remove_coord(rhs)
    }
}

impl BitOrAssign for HouseSet {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl BitAndAssign for HouseSet {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl SubAssign for HouseSet {
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.minus(rhs);
    }
}

pub struct Iter {
    shape: Shape,
    coords: u16,
}

impl Iterator for Iter {
    type Item = House;

    fn next(&mut self) -> Option<Self::Item> {
        if self.coords == 0 {
            None
        } else {
            let coord = self.coords.trailing_zeros() as u8;
            self.coords &= !(1 << coord);
            Some(House::new(self.shape, coord.into()))
        }
    }
}

impl FusedIterator for Iter {}

impl IntoIterator for HouseSet {
    type Item = House;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter{
        self.iter()
    }
}

impl FromIterator<House> for HouseSet {
    fn from_iter<I: IntoIterator<Item = House>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        let first = match iter.next() {
            Some(h) => h,
            None => return HouseSet::empty(Shape::Row),
        };

        let mut set = HouseSet::empty(first.shape());
        set.add(first);

        for h in iter {
            set.add(h);
        }

        set
    }
}

impl FromIterator<HouseSet> for HouseSet {
    fn from_iter<I: IntoIterator<Item = HouseSet>>(iter: I) -> Self {
        let mut iter = iter.into_iter();

        let first = match iter.next() {
            Some(s) => s,
            None => return HouseSet::empty(Shape::Row),
        };

        let shape = first.shape();
        let mut acc = first;

        for set in iter {
            debug_assert!(set.shape() == shape);
            acc = acc | set;
        }

        acc
    }
}

impl fmt::Display for HouseSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "{} {}", self.shape.label(), EMPTY_SET)
        } else {
            write!(f, "{} {}", self.shape.label(), self.coords)
        }
    }
}

impl fmt::Debug for HouseSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HouseSet({} {})", self.shape, self.coords)
    }
}

#[allow(unused_macros)]
macro_rules! rows {
    ($coords:literal) => {
        HouseSet::from_coords(Shape::Row, $coords)
    };
}

#[allow(unused_macros)]
macro_rules! houses {
    ($coords:literal) => {
        HouseSet::from_coords(Shape::House, $coords)
    };
}

#[allow(unused_macros)]
macro_rules! cols {
    ($coords:literal) => {
        HouseSet::from_coords(Shape::Column, $coords)
    }
}

#[allow(unused_macros)]
macro_rules! blocks {
    ($coords:literal) => {
        HouseSet::from_coords(Shape::Block, $coords)
    }
}

#[allow(unused_imports)]
pub(crate) use {blocks, cols, houses, rows};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::houses::coord;

    #[test]
    fn empty_and_full_sets() {
        let empty = HouseSet::empty(Shape::Row);
        assert!(empty.is_empty());
        assert!(!empty.is_full());
        assert_eq!(empty.len(), 0);

        let full = HouseSet::full(Shape::Row);
        assert!(!full.is_empty());
        assert!(full.is_full());
        assert_eq!(full.len(), 9);
    }

    #[test]
    fn add_and_remove_houses() {
        let r1 = House::row(coord!(0));
        let r2 = House::row(coord!(1));

        let mut set = HouseSet::empty(Shape::Row);
        set.add(r1);
        assert!(set.has(r1));
        assert_eq!(set.len(), 1);

        set.add(r2);
        assert!(set.has(r2));
        assert_eq!(set.len(), 2);

        set.remove(r1);
        assert!(!set.has(r1));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn union_and_intersect() {
        let r1 = House::row(coord!(0));
        let r2 = House::row(coord!(1));

        let set1 = HouseSet::empty(Shape::Row).with(r1);
        let set2 = HouseSet::empty(Shape::Row).with(r2);

        let union = set1.union(set2);
        assert!(union.has(r1) && union.has(r2));
        assert_eq!(union.len(), 2);

        let intersect = union.intersect(set1);
        assert!(intersect.has(r1));
        assert!(!intersect.has(r2));
        assert_eq!(intersect.len(), 1);
    }

    #[test]
    fn minus_and_inverted() {
        let r1 = House::row(coord!(0));
        let r2 = House::row(coord!(1));

        let full = HouseSet::full(Shape::Row);
        let minus_set = full.minus(HouseSet::empty(Shape::Row).with(r1));
        assert!(!minus_set.has(r1));
        assert!(minus_set.has(r2));

        let inv = minus_set.inverted();
        assert!(inv.has(r1));
        assert!(!inv.has(r2));
    }

    #[test]
    #[should_panic]
    fn add_wrong_shape_panics() {
        let r1 = House::row(coord!(0));
        let c1 = House::column(coord!(0));
        let _ = HouseSet::empty(Shape::Row).with(c1);
    }

    #[test]
    fn iter_and_from_iter() {
        let r1 = House::row(coord!(0));
        let r2 = House::row(coord!(1));
        let set: HouseSet = vec![r1, r2].into_iter().collect();

        let mut iterated = vec![];
        for h in set {
            iterated.push(h);
        }
        assert_eq!(iterated.len(), 2);
        assert!(iterated.contains(&r1) && iterated.contains(&r2));
    }

    #[test]
    fn from_labels() {
        let set = HouseSet::from_labels("R1 R2");
        assert!(set.has(House::row(coord!(0))));
        assert!(set.has(House::row(coord!(1))));
        assert_eq!(set.len(), 2);
    }
}

