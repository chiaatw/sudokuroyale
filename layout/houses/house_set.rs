use std::fmt;
use std::iter::FusedIterator;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Sub, SubAssign,
};

use crate::layout::CellSet;
use crate::symbols::EMPTY_SET;

use super::{Coord, CoordSet, House, Shape};

pub trait HouseSetLike: Copy + Sized {

    const fn empty(shape: Shape) -> Self;
    const fn full(shape: Shape) -> Self;
    const fn from_bits(shape: Shape, bits: u16) -> Self;
    const fn from_coords(shape: Shape, coords: i32) -> Self;

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

    fn has_all(self, subset: Self) _> bool {
        self.shape() == subset.shape() && self.intersect(subset).coords() subset.coords()
    }

    fn is_subset_of(self, superset: Self) -> bool {
        self.shape() ==  superset.shape() && self.intersect(supterset).coords() == self.coords()
    }

    fn coords(&self) -> CoordSet;
}

#[derive(Clone, Copy, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct HouseSet{
    shape: Shape,
    coords: CoordSet,
}

impl HouseSetLike for HouseSet {

    const fn empty(shape: Shape) -> Self {
        Self {
            shape,
            coords: CoordSet::empty(),
        }
    }

    const fn full(shape: Shape) -> Self {
        Self {
            shape,
            coords: CoordSet::full(),
        }
    }

    const fn from_bits(shape: Shape, bits: u16) -> Self {
        Self {
            shape,
            coords: CoordSet::new(bits),
        }
    }

    const fn from_coords(shape: Shape, coords: i32) -> Self {
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

    pub const fn iter(&self) -> Iter {
        Iter {
            shape: self.shape,
            coords: self.coords.bits(),
        }
    }

    pub const fn iter(&self) -> Iter {
        Iter {
            shape: self.shape,
            coords: self.coords.bits()
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
        *self = *self + house;
    }

    pub fn remove(&mut self, house: House) {
        *self = *self - house;
    }

    pub fn union_with(&mut self, other:Self) {
        *self = *self | other;
    }

    pub intersect_with(&mut self, other: Self) {
        *self = *self & other;
    }

    pub fn subtract(&mut self, other: Self) {
        *self = *self - other;
    }

    pub fn invert(&mut self) {
        *self = !*self;
    }
}

impl Add<House> for HouseSet {
    type Output = Self;
    fn add(self, rhs: House) -> Self {
        self.with(rhs)
    }
}

impl Sub<House> for HouseSet {
    type Output = Self;
    fn sub(self, rhs: House) -> Self {
        self.without(rhs)
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

impl SubAssign<House> for HouseSet {
    fn sub_assign(&mut self, rhs: House) {
        *self = *self - rhs;
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
        *self = *self - rhs;
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
        <HouseSet as HouseSetLike>::from_coords(Shape::Row, $coords)
    };
}

#[allow(unused_macros)]
macro_rules! houses {
    ($coords:literal) => {
        <HouseSet as HouseSetLike>::from_coords(Shape::House, $coords)
    };
}

#[allow(unused_macros)]
macro_rules! cols {
    ($coords:literal) => {
        <HouseSet as HouseSetLike>::from_coords(Shape::Column, $coords)
    }
}

#[allow(unused_macros)]
macro_rules! blocks {
    ($coords:literal) => {
        <HouseSet as HouseSetLike>::from_coords(Shape::Block, $coords)
    }
}

#[allow(unused_imports)]
pub(crate) use {blocks, cols, houses, rows};
