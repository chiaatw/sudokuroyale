use std::fmt;
use std::iter::FusedIterator;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Index, Not, Sub, SubAssign,
};

use crate::layout::{House, HouseSet, Shape};
use crate::symbols::EMPTY_SET;

use super::{Bit, Cell};

type Bits = u128;

#[allow(dead_code)]
type Size = u8;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct CellSet(CellSetRepr);

#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
enum CellSetRepr {
    Empty,
    Full,
    Bits(Bits),
}
#[allow(dead_code)]
const ALL_CELLS: std::ops::Range<Size> = 0..Cell::COUNT;
const ALL_SET: Bits = (1 << Cell::COUNT) - 1;

impl Default for CellSet {
    fn default() -> Self {
        Self::empty()
    }
}

impl CellSet {
    #[inline]
    const fn from_bits(bits: Bits) -> Self {
        debug_assert!(bits <= ALL_SET);
        match bits {
            0 => Self(CellSetRepr::Empty),
            ALL_SET => Self(CellSetRepr::Full),
            _ => Self(CellSetRepr::Bits(bits)),
        }
    }

    #[inline]
    const fn bits(&self) -> Bits {
        match self.0 {
            CellSetRepr::Empty => 0,
            CellSetRepr::Full => ALL_SET,
            CellSetRepr::Bits(bits) => bits,
        }
    }

    pub const fn empty() -> Self {
        Self(CellSetRepr::Empty)
    }

    pub const fn full() -> Self {
        Self(CellSetRepr::Full)
    }

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

    pub const fn of<const N: usize>(cells: &[Cell; N]) -> Self {
        let mut bits: Bits = 0;
        let mut i = 0;

        while i < N {
            bits |= cells[i].bit().bit();
            i += 1;
        }
        Self::from_bits(bits)
    }

    pub const fn is_empty(&self) -> bool {
        matches!(self.0, CellSetRepr::Empty)
    }

    pub const fn is_full(&self) -> bool {
        matches!(self.0, CellSetRepr::Full)
    }

    pub const fn len(&self) -> usize {
        self.bits().count_ones() as usize
    }

    pub const fn has(&self, cell: Cell) -> bool {
        self.bits() & cell.bit().bit() != 0
    }

    pub const fn with(&self, cell: Cell) -> Self {
        Self::from_bits(self.bits() | cell.bit().bit())
    }

    pub fn add(&mut self, cell: Cell) {
        *self = self.with(cell);
    }

    pub const fn without(&self, cell: Cell) -> Self {
        Self::from_bits(self.bits() & !(cell.bit().bit()))
    }

    pub fn remove(&mut self, cell: Cell) {
        *self = self.without(cell);
    }

    pub const fn has_any(&self, set: CellSet) -> bool {
        !self.intersect(set).is_empty()
    }

    pub const fn has_all(&self, subset: CellSet) -> bool {
        self.intersect(subset).bits() == subset.bits()
    }

    pub const fn is_subset_of(&self, superset: CellSet) -> bool {
        self.intersect(superset).bits() == self.bits()
    }

    pub const fn as_single(&self) -> Option<Cell> {
        if self.len() != 1 {
            None
        } else {
            Some(Cell::new(self.bits().trailing_zeros() as u8))
        }
    }

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

    pub const fn first(&self) -> Option<Cell> {
        if self.is_empty() {
            None
        } else {
            Some(Cell::new(self.bits().trailing_zeros() as u8))
        }
    }

    pub fn pop(&mut self) -> Option<Cell> {
        let cell = self.first()?;
        self.remove(cell);
        Some(cell)
    }

    pub const fn union(&self, set: Self) -> Self {
        let a = self.bits();
        let b = set.bits();
        if a == b {
            *self
        } else {
            Self::from_bits(a | b)
        }
    }

    pub fn union_with(&mut self, set: Self) {
        *self = self.union(set);
    }

    pub const fn intersect(&self, set: Self) -> Self {
        let a = self.bits();
        let b = set.bits();
        if a == b {
            *self
        } else {
            Self::from_bits(a & b)
        }
    }

    pub fn intersect_with(&mut self, set: Self) {
        *self = self.intersect(set);
    }

    pub const fn minus(&self, set: Self) -> Self {
        let a = self.bits();
        let b = set.bits();
        if a == b {
            Self::empty()
        } else {
            Self::from_bits(a & !b)
        }
    }

    pub fn subtract(&mut self, set: Self) {
        *self = self.minus(set);
    }

    pub const fn inverted(&self) -> Self {
        Self::from_bits(!self.bits() & ALL_SET)
    }

    pub fn invert(&mut self) {
        *self = self.inverted();
    }

    pub fn share_any_house(&self) -> bool {
        self.share_row() || self.share_column() || self.share_block()
    }

    pub fn share_row(&self) -> bool {
        self.share_house(Shape::Row)
    }

    pub fn share_column(&self) -> bool {
        self.share_house(Shape::Column)
    }

    pub fn share_block(&self) -> bool {
        self.share_house(Shape::Block)
    }

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

    pub fn rows(&self) -> HouseSet {
        self.houses(Shape::Row)
    }

    pub fn columns(&self) -> HouseSet {
        self.houses(Shape::Column)
    }

    pub fn blocks(&self) -> HouseSet {
        self.houses(Shape::Block)
    }

    pub fn houses(&self, shape: Shape) -> HouseSet {
        self.iter()
            .fold(HouseSet::empty(shape), |set, cell| set + cell.house(shape))
    }

    pub fn peers(&self) -> CellSet {
        self.iter()
            .fold(CellSet::full(), |set, cell| set & cell.peers())
    }

    pub const fn iter(&self) -> CellIter {
        CellIter {
            iter: self.bit_iter(),
        }
    }

    pub const fn bit_iter(&self) -> BitIter {
        BitIter { bits: self.bits() }
    }

    pub fn pattern_string(&self) -> String {
        (0..Cell::COUNT)
            .map(|i| if self.has(Cell::new(i)) { '1' } else { '.' })
            .collect()
    }

    pub fn debug(&self) -> String {
        format!(
            "{:02}:{:081b}",
            self.len(),
            self.bits().reverse_bits() >> (128 - 81)
        )
    }

    pub fn from_str(labels: &str) -> Self {
        let mut set = Self::empty();
        for token in labels.split_whitespace() {
            if token.is_empty() {
                continue;
            }
            set = set + Cell::from_str(token);
        }
        set
    }
}

impl From<House> for CellSet {
    // gibt eine Menge zurück, die die Zellen im Haus enthält
    fn from(house: House) -> Self {
        house.cells()
    }
}

impl From<&str> for CellSet {
    // Gibt eine Menge zurück, die die Zellen in den Labels enthält
    fn from(labels: &str) -> Self {
        Self::from_str(labels)
    }
}

impl From<String> for CellSet {
    fn from(labels: String) -> Self {
        Self::from(labels.as_str())
    }
}

impl From<&String> for CellSet {
    fn from(labels: &String) -> Self {
        Self::from(labels.as_str())
    }
}

impl IntoIterator for CellSet {
    type Item = Cell;
    type IntoIter = CellIter;

    // Gibt einen Iterator über die Elemente dieser Menge in Zeilen-dann-Spalten-Reihenfolge zurück
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
    fn union(self) -> CellSet {
        self.union_cells()
    }

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
    fn union(self) -> CellSet {
        self.union_cells()
    }

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
    fn intersection(self) -> CellSet {
        self.fold(CellSet::full(), |acc, c| acc & c)
    }
}

impl FromIterator<Cell> for CellSet {
    fn from_iter<I: IntoIterator<Item = Cell>>(iter: I) -> Self {
        let mut set = Self::empty();
        for cell in iter {
            set += cell;
        }
        set
    }
}

impl FromIterator<CellSet> for CellSet {
    fn from_iter<I: IntoIterator<Item = CellSet>>(iter: I) -> Self {
        let mut union = Self::empty();
        for set in iter {
            union |= set;
        }
        union
    }
}

static TRUE: bool = true;
static FALSE: bool = false;

impl Index<Bit> for CellSet {
    type Output = bool;
    fn index(&self, bit: Bit) -> &bool {
        if self.has(bit.cell()) {
            &TRUE
        } else {
            &FALSE
        }
    }
}

impl Index<Cell> for CellSet {
    type Output = bool;
    fn index(&self, cell: Cell) -> &bool {
        if self.has(cell) {
            &TRUE
        } else {
            &FALSE
        }
    }
}

impl Add<Cell> for CellSet {
    type Output = Self;

    fn add(self, rhs: Cell) -> Self {
        self.with(rhs)
    }
}

impl AddAssign<Cell> for CellSet {
    fn add_assign(&mut self, rhs: Cell) {
        self.add(rhs)
    }
}

impl Sub<Cell> for CellSet {
    type Output = Self;

    fn sub(self, rhs: Cell) -> Self {
        self.without(rhs)
    }
}

impl SubAssign<Cell> for CellSet {
    fn sub_assign(&mut self, rhs: Cell) {
        self.remove(rhs)
    }
}

impl Not for CellSet {
    type Output = Self;

    fn not(self) -> Self {
        self.inverted()
    }
}

impl BitOr for CellSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl BitOrAssign for CellSet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.union_with(rhs)
    }
}

impl BitAnd for CellSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        self.intersect(rhs)
    }
}

impl BitAndAssign for CellSet {
    fn bitand_assign(&mut self, rhs: Self) {
        self.intersect_with(rhs)
    }
}

impl Sub for CellSet {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self.minus(rhs)
    }
}

impl SubAssign for CellSet {
    fn sub_assign(&mut self, rhs: Self) {
        self.subtract(rhs)
    }
}

impl fmt::Display for CellSet {
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

#[macro_export]
macro_rules! cells {
    ($s:expr) => {{
        $crate::layout::CellSet::from($s)
    }};
}

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
    use super::*;
    use crate::layout::houses::house_set::HouseSetLike; // für rows.len(), columns.len()

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

        assert_eq!(
            got,
            vec![crate::cell!("D3"), crate::cell!("G5"), crate::cell!("H2")]
        );
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

    #[test]
    fn test_add_and_remove() {
        let mut set = CellSet::empty();
        let c1 = Cell::new(0);
        let c2 = Cell::new(10);

        CellSet::add(&mut set, c1);
        assert!(set.has(c1));
        CellSet::add(&mut set, c2);
        assert!(set.has(c2));
        assert_eq!(set.len(), 2);

        set.remove(c1);
        assert!(!set.has(c1));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_with_and_without() {
        let set = CellSet::empty();
        let c = Cell::new(5);
        let set2 = set.with(c);
        assert!(set2.has(c));
        let set3 = set2.without(c);
        assert!(!set3.has(c));
    }

    #[test]
    fn test_union_intersect_minus_invert() {
        let c1 = Cell::new(1);
        let c2 = Cell::new(2);
        let c3 = Cell::new(3);

        let set1 = CellSet::empty().with(c1).with(c2);
        let set2 = CellSet::empty().with(c2).with(c3);

        let union = set1.union(set2);
        assert!(union.has(c1));
        assert!(union.has(c2));
        assert!(union.has(c3));

        let intersect = set1.intersect(set2);
        assert!(intersect.has(c2));
        assert_eq!(intersect.len(), 1);

        let minus = set1.minus(set2);
        assert!(minus.has(c1));
        assert!(!minus.has(c2));

        let inverted = set1.inverted();
        assert!(!inverted.has(c1));
        assert!(!inverted.has(c2));
        assert!(inverted.has(c3));
    }

    #[test]
    fn test_operator_traits() {
        let mut set = CellSet::empty();
        set += Cell::new(0);
        set += Cell::new(1);
        assert!(set.has(Cell::new(0)));
        assert!(set.has(Cell::new(1)));

        let set2 = CellSet::empty() + Cell::new(1) + Cell::new(2);
        let union = set | set2;
        assert!(union.has(Cell::new(0)));
        assert!(union.has(Cell::new(1)));
        assert!(union.has(Cell::new(2)));

        let intersect = set & set2;
        assert!(intersect.has(Cell::new(1)));
        assert_eq!(intersect.len(), 1);

        let diff = set2 - set;
        assert!(diff.has(Cell::new(2)));
        assert!(!diff.has(Cell::new(1)));

        let inverted = !set;
        assert!(!inverted.has(Cell::new(0)));
        assert!(inverted.has(Cell::new(2)));
    }

    #[test]
    fn test_as_single_pair_triple_first_pop() {
        let single = CellSet::empty().with(Cell::new(5));
        assert_eq!(single.as_single(), Some(Cell::new(5)));
        assert_eq!(single.as_pair(), None);
        assert_eq!(single.as_triple(), None);

        let pair = CellSet::empty().with(Cell::new(1)).with(Cell::new(3));
        assert_eq!(pair.as_single(), None);
        assert_eq!(pair.as_pair(), Some((Cell::new(1), Cell::new(3))));
        assert_eq!(pair.as_triple(), None);

        let triple = CellSet::empty()
            .with(Cell::new(0))
            .with(Cell::new(2))
            .with(Cell::new(4));
        assert_eq!(
            triple.as_triple(),
            Some((Cell::new(0), Cell::new(2), Cell::new(4)))
        );

        let mut pop_set = triple;
        let popped = pop_set.pop();
        assert_eq!(popped, Some(Cell::new(0)));
        assert_eq!(pop_set.len(), 2);
    }

    #[test]
    fn test_iter_and_bit_iter() {
        let set = CellSet::empty()
            .with(Cell::new(1))
            .with(Cell::new(3))
            .with(Cell::new(5));

        let cells: Vec<_> = set.iter().collect();
        assert_eq!(cells, vec![Cell::new(1), Cell::new(3), Cell::new(5)]);

        let bits: Vec<_> = set.bit_iter().map(|b| b.cell()).collect();
        assert_eq!(bits, cells);
    }

    #[test]
    fn test_share_house_and_rows_columns_blocks() {
        let set = CellSet::empty().with(Cell::new(0)).with(Cell::new(1));
        assert!(set.share_row());
        assert!(!set.share_column());
        assert!(set.share_block());
        assert!(set.share_house(Shape::Row));
        assert!(!set.share_house(Shape::Column));

        let rows = set.rows();
        assert_eq!(rows.len(), 1);

        let columns = set.columns();
        assert_eq!(columns.len(), 2);
    }

    #[test]
    fn test_peers() {
        let cell = Cell::new(0);
        let set = CellSet::empty().with(cell);
        let peers = set.peers();
        assert!(!peers.has(cell));
        assert_eq!(peers.len(), 20);
    }

    #[test]
    fn test_pattern_string_display_debug() {
        let set = CellSet::empty().with(Cell::new(0)).with(Cell::new(80));
        let pattern = set.pattern_string();
        assert_eq!(pattern.chars().nth(0), Some('1'));
        assert_eq!(pattern.chars().nth(80), Some('1'));

        let display = format!("{}", set);
        assert!(display.contains("A1"));

        let dbg = set.debug();
        // dbg beginnt mit "{len:02}:"
        assert!(dbg.starts_with("02:"));
    }
}
