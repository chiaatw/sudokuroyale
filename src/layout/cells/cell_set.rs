// While the tuple struct is a thin wrapper (should be same memory storage),
// the fact that it's a struct means it cannot be passed by value without moving it.
//
// Or maybe not. References are about ownership--not pointers.

use std::fmt;
use std::iter::FusedIterator;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, Not, Sub, SubAssign,
};

use crate::layout::{House, HouseSet, Shape};
use crate::symbols::EMPTY_SET;

use super::{Bit, Cell};

type Bits = u128;
type Size = u8;

/// A set of cells implemented using a bit field.
#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct CellSet(CellSetRepr);

#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
enum CellSetRepr {
    Empty,
    Full,
    Bits(Bits),
}

const ALL_CELLS: std::ops::Range<Size> = 0..Cell::COUNT;
const ALL_SET: Bits = (1 << Cell::COUNT) - 1;

impl Default for CellSet {
    fn default() -> Self {
        Self::empty()
    }
}

impl CellSet {
    /// Internal constructor from raw bits.
    #[inline]
    const fn from_bits(bits: Bits) -> Self {
        debug_assert!(bits <= ALL_SET);
        match bits {
            0 => Self(CellSetRepr::Empty),
            ALL_SET => Self(CellSetRepr::Full),
            _ => Self(CellSetRepr::Bits(bits)),
        }
    }

    /// Returns the raw bit representation.
    #[inline]
    const fn bits(&self) -> Bits {
        match self.0 {
            CellSetRepr::Empty => 0,
            CellSetRepr::Full => ALL_SET,
            CellSetRepr::Bits(bits) => bits,
        }
    }

    /// Returns a new empty set.
    pub const fn empty() -> Self {
        Self(CellSetRepr::Empty)
    }

    /// Returns a new full set.
    pub const fn full() -> Self {
        Self(CellSetRepr::Full)
    }

    /// Returns a new set containing the cells with a digit in the packed string `puzzle`.
    pub fn new_from_pattern(puzzle: &str) -> Self {
        let mut bits: Bits = 0;
        let mut c = 0;

        for char in puzzle.chars() {
            match char {
                ' ' | '\r' | '\n' | '|' | '_' => continue,
                '1'..='9' => bits |= Cell::new(c).bit().bit(),
                _ => (),
            }
            c += 1;
        }
        Self::from_bits(bits)
    }

    /// Returns a new set containing each cell in `cells`.
    pub const fn of<const N: usize>(cells: &[Cell; N]) -> Self {
        let mut bits: Bits = 0;
        let mut i = 0;

        while i < N {
            bits |= cells[i].bit().bit();
            i += 1;
        }
        Self::from_bits(bits)
    }

    /// Returns true if this set is empty.
    pub const fn is_empty(&self) -> bool {
        matches!(self.0, CellSetRepr::Empty)
    }

    /// Returns true if this set is full.
    pub const fn is_full(&self) -> bool {
        matches!(self.0, CellSetRepr::Full)
    }

    /// Returns the number of cells in this set.
    pub const fn len(&self) -> usize {
        self.bits().count_ones() as usize
    }

    /// Returns true if `cell` is a member of this set.
    pub const fn has(&self, cell: Cell) -> bool {
        self.bits() & cell.bit().bit() != 0
    }

    /// Returns a copy of this set with `cell` as a member.
    pub const fn with(&self, cell: Cell) -> Self {
        Self::from_bits(self.bits() | cell.bit().bit())
    }

    /// Adds `cell` to this set.
    pub fn add(&mut self, cell: Cell) {
        *self = self.with(cell);
    }

    /// Returns a copy of this set without `cell` as a member.
    pub const fn without(&self, cell: Cell) -> Self {
        Self::from_bits(self.bits() & !(cell.bit().bit()))
    }

    /// Removes `cell` from this set.
    pub fn remove(&mut self, cell: Cell) {
        *self = self.without(cell);
    }
}


    /// Returns true if at least one of the members of `set` is a member of this set.
    pub const fn has_any(&self, set: CellSet) -> bool {
        !self.intersect(set).is_empty()
    }

    /// Returns true if all of the members of `subset` are members of this set.
    pub const fn has_all(&self, subset: CellSet) -> bool {
        self.intersect(subset).bits() == subset.bits()
    }

    /// Returns true if all of the members of this set are members of `superset`.
    pub const fn is_subset_of(&self, superset: CellSet) -> bool {
        self.intersect(superset).bits() == self.bits()
    }

    /// Returns the single cell in this set.
    pub const fn as_single(&self) -> Option<Cell> {
        if self.len() != 1 {
            None
        } else {
            Some(Cell::new(self.bits().trailing_zeros() as u8))
        }
    }

    /// Returns the two cells in this set as a tuple.
    pub const fn as_pair(&self) -> Option<(Cell, Cell)> {
        if self.len() != 2 {
            None
        } else {
            let mut bits = self.bits();
            let first = Cell::new(bits.trailing_zeros() as u8);
            bits &= !first.bit().bit();
            let second = Cell::new(bits.trailing_zeros() as u8);
            Some((first, second))
        }
    }

    /// Returns the three cells in this set as a tuple.
    pub const fn as_triple(&self) -> Option<(Cell, Cell, Cell)> {
        if self.len() != 3 {
            None
        } else {
            let mut bits = self.bits();
            let first = Cell::new(bits.trailing_zeros() as u8);
            bits &= !first.bit().bit();
            let second = Cell::new(bits.trailing_zeros() as u8);
            bits &= !second.bit().bit();
            let third = Cell::new(bits.trailing_zeros() as u8);
            Some((first, second, third))
        }
    }

    /// Returns the first cell in this set in row-then-column order.
    pub const fn first(&self) -> Option<Cell> {
        if self.is_empty() {
            None
        } else {
            Some(Cell::new(self.bits().trailing_zeros() as u8))
        }
    }

    /// Returns the first cell in this set and removes it.
    pub fn pop(&mut self) -> Option<Cell> {
        let cell = self.first()?;
        self.remove(cell);
        Some(cell)
    }

    /// Returns a new set containing the combined members of this set and `set`.
    pub const fn union(&self, set: Self) -> Self {
        let a = self.bits();
        let b = set.bits();
        if a == b {
            *self
        } else {
            Self::from_bits(a | b)
        }
    }

    /// Adds the members of `set` to this set.
    pub fn union_with(&mut self, set: Self) {
        *self = self.union(set);
    }

    /// Returns a new set containing the common members of this set and `set`.
    pub const fn intersect(&self, set: Self) -> Self {
        let a = self.bits();
        let b = set.bits();
        if a == b {
            *self
        } else {
            Self::from_bits(a & b)
        }
    }

    /// Removes all members of this set that are not members of `set`.
    pub fn intersect_with(&mut self, set: Self) {
        *self = self.intersect(set);
    }

    /// Returns a new set containing the members of this set that are not in `set`.
    pub const fn minus(&self, set: Self) -> Self {
        let a = self.bits();
        let b = set.bits();
        if a == b {
            Self::empty()
        } else {
            Self::from_bits(a & !b)
        }
    }

    /// Removes all members of this set that are members of `set`.
    pub fn subtract(&mut self, set: Self) {
        *self = self.minus(set);
    }

    /// Returns a new set containing all cells that are not in this set.
    pub const fn inverted(&self) -> Self {
        Self::from_bits(!self.bits() & ALL_SET)
    }

    /// Removes all cells from this set and adds all other cells to it.
    pub fn invert(&mut self) {
        *self = self.inverted();
    }

    /// Returns true if all cells in this set are all in any single house.
    pub fn share_any_house(&self) -> bool {
        self.share_row() || self.share_column() || self.share_block()
    }

    /// Returns true if all cells in this set are in the same row.
    pub fn share_row(&self) -> bool {
        self.share_house(Shape::Row)
    }

    /// Returns true if all cells in this set are in the same column.
    pub fn share_column(&self) -> bool {
        self.share_house(Shape::Column)
    }

    /// Returns true if all cells in this set are in the same block.
    pub fn share_block(&self) -> bool {
        self.share_house(Shape::Block)
    }

    /// Returns true if all cells in this set are in the same `shape` house.
    pub fn share_house(&self, shape: Shape) -> bool {
        if self.is_empty() {
            false
        } else {
            let house = self.first().unwrap().house(shape);
            for cell in self.iter() {
                if cell.house(shape) != house {
                    return false;
                }
            }
            true
        }
    }

    /// Returns the minimal set of rows containing the members of this set.
    pub fn rows(&self) -> HouseSet {
        self.houses(Shape::Row)
    }

    /// Returns the minimal set of columns containing the members of this set.
    pub fn columns(&self) -> HouseSet {
        self.houses(Shape::Column)
    }

    /// Returns the minimal set of blocks containing the members of this set.
    pub fn blocks(&self) -> HouseSet {
        self.houses(Shape::Block)
    }

    /// Returns the minimal set of `shape` houses containing the members of this set.
    pub fn houses(&self, shape: Shape) -> HouseSet {
        self.iter()
            .fold(HouseSet::empty(shape), |set, cell| set + cell.house(shape))
    }

    /// Returns the common peers of all members of this set.
    pub fn peers(&self) -> CellSet {
        self.iter()
            .fold(CellSet::full(), |set, cell| set & cell.peers())
    }

    /// Returns an iterator over the members of this set in row-then-column order.
    pub const fn iter(&self) -> CellIter {
        CellIter {
            iter: self.bit_iter(),
        }
    }

    /// Returns an iterator over the members of this set as bits in row-then-column order.
    pub const fn bit_iter(&self) -> BitIter {
        BitIter { bits: self.bits() }
    }

    /// Returns a packed pattern string with a `1` for each member of this set.
    pub fn pattern_string(&self) -> String {
        (0..Cell::COUNT)
            .map(|i| if self.has(Cell::new(i)) { '1' } else { '.' })
            .collect()
    }

    /// Returns the size and bits of this set as a debug string.
    pub fn debug(&self) -> String {
        format!(
            "{:02}:{:081b}",
            self.len(),
            self.bits().reverse_bits() >> (128 - 81)
        )
    }


impl From<House> for CellSet {
    /// Returns a set containing the cells in `house`.
    fn from(house: House) -> Self {
        house.cells()
    }
}

impl From<&str> for CellSet {
    /// Returns a set containing the cells in `labels` after splitting on space.
    fn from(labels: &str) -> Self {
        Self::from_str(labels)
    }
}

impl IntoIterator for CellSet {
    type Item = Cell;
    type IntoIter = CellIter;

    /// Returns an iterator over the members of this set in row-then-column order.
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub trait CellIteratorUnion {
    fn union(self) -> CellSet;
    fn union_cells(self) -> CellSet;
}

impl<I> CellIteratorUnion for I
where
    I: Iterator<Item = Cell>,
{
    /// Collects the cells in `iter` into a set.
    fn union(self) -> CellSet {
        self.union_cells()
    }

    /// Collects the cells in `iter` into a set.
    fn union_cells(self) -> CellSet {
        self.fold(CellSet::empty(), |acc, c| acc + c)
    }
}

pub trait CellSetIteratorUnion {
    fn union(self) -> CellSet;
    fn union_cells(self) -> CellSet;
}

impl<I> CellSetIteratorUnion for I
where
    I: Iterator<Item = CellSet>,
{
    /// Collects all members of the sets in `iter` into a set.
    fn union(self) -> CellSet {
        self.union_cells()
    }

    /// Collects all members of the sets in `iter` into a set.
    fn union_cells(self) -> CellSet {
        self.fold(CellSet::empty(), |acc, c| acc | c)
    }
}

pub trait CellSetIteratorIntersection {
    fn intersection(self) -> CellSet;
}

impl<I> CellSetIteratorIntersection for I
where
    I: Iterator<Item = CellSet>,
{
    /// Collects the common members of the sets in `iter` into a set.
    fn intersection(self) -> CellSet {
        self.fold(CellSet::full(), |acc, c| acc & c)
    }
}

impl FromIterator<Cell> for CellSet {
    /// Collects the cells in `iter` into a set.
    fn from_iter<I: IntoIterator<Item = Cell>>(iter: I) -> Self {
        let mut set = Self::empty();
        for cell in iter {
            set += cell;
        }
        set
    }
}

impl FromIterator<CellSet> for CellSet {
    /// Collects all members of the sets in `iter` into a set.
    fn from_iter<I: IntoIterator<Item = CellSet>>(iter: I) -> Self {
        let mut union = Self::empty();
        for set in iter {
            union |= set;
        }
        union
    }
}

impl Index<Bit> for CellSet {
    type Output = bool;

    /// Returns true if the cell represented by `bit` is a member of this set.
    fn index(&self, bit: Bit) -> &bool {
        if self.has(bit.cell()) {
            &true
        } else {
            &false
        }
    }
}

impl Index<Cell> for CellSet {
    type Output = bool;

    /// Returns true if `cell` is a member of this set.
    fn index(&self, cell: Cell) -> &bool {
        if self.has(cell) {
            &true
        } else {
            &false
        }
    }
}

impl Add<Cell> for CellSet {
    type Output = Self;

    /// Returns a copy of this set with `rhs` as a member.
    fn add(self, rhs: Cell) -> Self {
        self.with(rhs)
    }
}

impl AddAssign<Cell> for CellSet {
    /// Adds `rhs` to this set.
    fn add_assign(&mut self, rhs: Cell) {
        self.add(rhs)
    }
}

impl Sub<Cell> for CellSet {
    type Output = Self;

    /// Returns a copy of this set without `rhs` as a member.
    fn sub(self, rhs: Cell) -> Self {
        self.without(rhs)
    }
}

impl SubAssign<Cell> for CellSet {
    /// Removes `rhs` from this set.
    fn sub_assign(&mut self, rhs: Cell) {
        self.remove(rhs)
    }
}

impl Not for CellSet {
    type Output = Self;

    /// Returns a new set containing all cells that are not in this set.
    fn not(self) -> Self {
        self.inverted()
    }
}

impl BitOr for CellSet {
    type Output = Self;

    /// Returns a new set containing the combined members of this set and `rhs`.
    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl BitOrAssign for CellSet {
    /// Adds the members of `rhs` to this set.
    fn bitor_assign(&mut self, rhs: Self) {
        self.union_with(rhs)
    }
}

impl BitAnd for CellSet {
    type Output = Self;

    /// Returns a new set containing the common members of this set and `rhs`.
    fn bitand(self, rhs: Self) -> Self {
        self.intersect(rhs)
    }
}

impl BitAndAssign for CellSet {
    /// Removes all members of this set that are not members of `rhs`.
    fn bitand_assign(&mut self, rhs: Self) {
        self.intersect_with(rhs)
    }
}

impl Sub for CellSet {
    type Output = Self;

    /// Returns a new set containing the members of this set that are not in `rhs`.
    fn sub(self, rhs: Self) -> Self {
        self.minus(rhs)
    }
}

impl SubAssign for CellSet {
    /// Removes all members of this set that are members of `rhs`.
    fn sub_assign(&mut self, rhs: Self) {
        self.subtract(rhs)
    }
}

impl fmt::Display for CellSet {
    /// Returns a string containing the labels of the cells in this set separated by spaces.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "{}", EMPTY_SET)
        } else {
            let mut s = String::with_capacity(3 * self.len() + 2);
            let mut first = true;
            for cell in self.iter() {
                if first {
                    first = false;
                } else {
                    s.push(' ');
                }
                s.push_str(cell.label());
            }
            write!(f, "{}", s)
        }
    }
}

impl fmt::Debug for CellSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

/// Returns a new set from the given bits, cells, or labels.
#[allow(unused_macros)]
macro_rules! cells {
    ($s:expr) => {{
        CellSet::from($s)
    }};
}

#[allow(unused_imports)]
pub(crate) use cells;


pub struct CellIter {
    iter: BitIter,
}

impl Iterator for CellIter {
    type Item = Cell;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|bit| bit.cell())
    }
}

impl FusedIterator for CellIter {}

// TODO Inline this into CellIter?
pub struct BitIter {
    bits: Bits,
}

impl Iterator for BitIter {
    type Item = Bit;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            None
        } else {
            let bit = 1 << self.bits.trailing_zeros();
            self.bits &= !bit;
            Some(Bit::new(bit))
        }
    }
}

impl FusedIterator for BitIter {}

#[cfg(test)]
mod tests {
    use crate::layout::cells::cell::cell;
    use crate::layout::houses::house_set::houses;
    use crate::symbols::EMPTY_SET_STR;

    use super::*;

    // ⚠️ Tests bleiben UNVERÄNDERT
    // Sie validieren exakt, dass die Enum-Version
    // von außen identisches Verhalten zeigt.

    #[test]
    fn empty() {
        let set = CellSet::empty();

        assert!(set.is_empty());
        assert_eq!(0, set.len());
        for i in ALL_CELLS {
            assert!(!set[Cell::new(i)]);
        }
    }

    #[test]
    fn full() {
        let set = CellSet::full();

        assert!(!set.is_empty());
        assert_eq!(Cell::COUNT, set.len() as u8);
        for i in ALL_CELLS {
            assert!(set[Cell::new(i)]);
        }
    }

    #[test]
    fn new_from_pattern() {
        let set = CellSet::new_from_pattern(
            "
                7..1....9
                .2.3..7..
                4.9......
                .6.8..2..
                .........
                .7...1.5.
                .....49..
                .46..5..2
                .1...68..
            ",
        );

        assert_eq!(
            CellSet::from("A1 A4 A9 B2 B4 B7 C1 C3 D2 D4 D7 F2 F6 F8 G6 G7 H2 H3 H6 H9 J2 J6 J7"),
            set
        );
    }

    #[test]
    fn iter_order() {
        let set = cells!("D3 G5 H2");
        let got: Vec<_> = set.iter().collect();

        assert_eq!(got, vec![cell!("D3"), cell!("G5"), cell!("H2")]);
    }

    #[test]
    fn pattern_string() {
        assert_eq!(
            ".................................................................................",
            CellSet::empty().pattern_string()
        );
        assert_eq!(
            "................1....1...........................1..............1................",
            cells!("B8 C4 F5 H2").pattern_string()
        );
    }
}

