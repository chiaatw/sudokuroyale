use std::fmt;
use std::ops::{Add, Neg};

use crate::layout::{Coord, House, Shape};

use super::label::{index_from_label, label_from_index, try_index_from_label};
use super::{Bit, CellSet};

/// Represents a single grid cell identified by a linear index
/// running left-to-right, top-to-bottom.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Cell {
    Index(u8),
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Index(0)
    }
}

impl Cell {
    /// Total number of cells in the grid.
    pub const COUNT: u8 = 81;

    /// Returns an iterator over all cells in index order.
    pub fn iter() -> CellIter {
        CellIter::new()
    }

    /// Creates a cell from its linear index.
    pub const fn new(index: u8) -> Self {
        debug_assert!(index < Cell::COUNT);
        Cell::Index(index)
    }

    /// Creates a cell from row and column coordinates.
    pub const fn from_coords(row: Coord, column: Coord) -> Self {
        Self::new(row.u8() * 9 + column.u8())
    }

    /// Creates a cell from row and column houses.
    pub const fn from_row_column(row: House, column: House) -> Self {
        Self::from_coords(row.coord(), column.coord())
    }

    /// Parses a cell from its textual label (e.g. "A1").
    pub fn from_str(label: &str) -> Self {
        Self::new(index_from_label(label))
    }

    /// Parses a cell from an owned string label.
    pub fn from_string(label: String) -> Self {
        Self::from_str(label.as_str())
    }

    /// Returns the internal linear index.
    pub const fn index(&self) -> u8 {
        match *self {
            Cell::Index(i) => i,
        }
    }

    /// Returns the index as `usize`.
    pub const fn usize(&self) -> usize {
        self.index() as usize
    }

    /// Returns the bit representation of this cell.
    pub const fn bit(&self) -> Bit {
        Bit::new(1 << self.index())
    }

    /// Returns all houses this cell belongs to.
    pub const fn houses(&self) -> [House; 3] {
        HOUSES[self.usize()]
    }

    /// Returns the house of the given shape.
    pub const fn house(&self, shape: Shape) -> House {
        HOUSES[self.usize()][shape.usize()]
    }

    /// Returns the row house.
    pub const fn row(&self) -> House {
        self.house(Shape::Row)
    }

    /// Returns the row coordinate.
    pub const fn row_coord(&self) -> Coord {
        HOUSE_COORDS[self.usize()][Shape::Row.usize()]
    }

    /// Returns the column house.
    pub const fn column(&self) -> House {
        self.house(Shape::Column)
    }

    /// Returns the column coordinate.
    pub const fn column_coord(&self) -> Coord {
        HOUSE_COORDS[self.usize()][Shape::Column.usize()]
    }

    /// Returns the block house.
    pub const fn block(&self) -> House {
        self.house(Shape::Block)
    }

    /// Returns the block coordinate.
    pub const fn block_coord(&self) -> Coord {
        HOUSE_COORDS[self.usize()][Shape::Block.usize()]
    }

    /// Returns this cell’s coordinate inside its row.
    pub const fn coord_in_row(&self) -> Coord {
        COORDS_IN_HOUSES[self.usize()][Shape::Row.usize()]
    }

    /// Returns this cell’s coordinate inside its column.
    pub const fn coord_in_column(&self) -> Coord {
        COORDS_IN_HOUSES[self.usize()][Shape::Column.usize()]
    }

    /// Returns this cell’s coordinate inside its block.
    pub const fn coord_in_block(&self) -> Coord {
        COORDS_IN_HOUSES[self.usize()][Shape::Block.usize()]
    }

    /// Returns all houses shared with another cell.
    pub fn common_houses(&self, other: Cell) -> Vec<House> {
        [self.row(), self.column(), self.block()]
            .into_iter()
            .filter(|h| h.has(other))
            .collect()
    }

    /// Returns the set of all peer cells.
    pub const fn peers(&self) -> CellSet {
        PEERS[self.usize()]
    }

    /// Returns true if this cell sees another cell.
    pub const fn sees(&self, other: Cell) -> bool {
        self.peers().has(other)
    }

    /// Returns the canonical label for this cell.
    pub const fn label(&self) -> &'static str {
        label_from_index(self.index())
    }

    /// Formats a list of cells as a labeled string.
    pub fn labels(cells: &Vec<Cell>) -> String {
        let mut s = String::from("(");
        for cell in cells {
            s.push(' ');
            s.push_str(cell.label());
        }
        s.push_str(" )");
        s
    }
}


impl TryFrom<&str> for Cell {
    type Error = String;

    fn try_from(label: &str) -> Result<Self, Self::Error> {
        try_index_from_label(label).map(Cell::new)
    }
}

impl TryFrom<String> for Cell {
    type Error = String;

    fn try_from(label: String) -> Result<Self, Self::Error> {
        Self::try_from(label.as_str())
    }
}


impl Add<Cell> for Cell {
    type Output = CellSet;

    fn add(self, rhs: Cell) -> CellSet {
        CellSet::empty() + self + rhs
    }
}

impl Neg for Cell {
    type Output = CellSet;

    fn neg(self) -> CellSet {
        CellSet::full() - self
    }
}


impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}


pub struct CellIter(u8);

impl CellIter {
    pub const fn new() -> Self {
        Self(0)
    }
}

impl Iterator for CellIter {
    type Item = Cell;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 < Cell::COUNT {
            let cell = Cell::new(self.0);
            self.0 += 1;
            Some(cell)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for CellIter {
    fn len(&self) -> usize {
        (Cell::COUNT - self.0) as usize
    }
}


#[allow(unused_macros)]
macro_rules! cell {
    ($l:expr) => {
        Cell::from_str($l)
    };
}

#[allow(unused_imports)]
pub(crate) use cell;


/// Precomputed row, column and block coordinates for every cell.
const HOUSE_COORDS: [[Coord; 3]; 81] = {
    let mut coords = [[Coord::new(0); 3]; 81];
    let mut i = 0;
    while i < 81 {
        let r = i / 9;
        let c = i % 9;
        let b = (r / 3) * 3 + (c / 3);
        coords[i] = [Coord::new(r), Coord::new(c), Coord::new(b)];
        i += 1;
    }
    coords
};

/// Precomputed row, column and block houses for every cell.
const HOUSES: [[House; 3]; 81] = {
    let mut houses = [[House::new(Shape::Row, Coord::new(0)); 3]; 81];
    let mut i = 0;
    while i < 81 {
        houses[i] = [
            House::row(HOUSE_COORDS[i][Shape::Row.usize()]),
            House::column(HOUSE_COORDS[i][Shape::Column.usize()]),
            House::block(HOUSE_COORDS[i][Shape::Block.usize()]),
        ];
        i += 1;
    }
    houses
};

/// Coordinates of each cell relative to its houses.
const COORDS_IN_HOUSES: [[Coord; 3]; 81] = {
    let mut coords = [[Coord::new(0); 3]; 81];
    let mut i = 0;
    while i < 81 {
        let r = i / 9;
        let c = i % 9;
        let b = 3 * (r % 3) + (c % 3);
        coords[i] = [Coord::new(c), Coord::new(r), Coord::new(b)];
        i += 1;
    }
    coords
};

/// Cached peer sets for every cell.
const PEERS: [CellSet; 81] = {
    let mut sets = [CellSet::empty(); 81];
    let mut i = 0;
    while i < 81 {
        let cell = Cell::new(i as u8);
        sets[i] = CellSet::empty()
            .union(cell.row().cells())
            .union(cell.column().cells())
            .union(cell.block().cells())
            .without(cell);
        i += 1;
    }
    sets
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_index_and_new() {
        let c = Cell::new(0);
        assert_eq!(c.index(), 0);
        assert_eq!(c.usize(), 0);

        let c2 = Cell::from_coords(Coord::new(0), Coord::new(0));
        assert_eq!(c2.index(), 0);

        let c3 = Cell::from_row_column(House::row(Coord::new(0)), House::column(Coord::new(0)));
        assert_eq!(c3.index(), 0);

        let c_last = Cell::new(80);
        assert_eq!(c_last.index(), 80);
    }

    #[test]
    fn test_from_str_and_label() {
        let c = Cell::from_str("A1");
        assert_eq!(c.label(), "A1");

        let c2: Cell = "B3".try_into().unwrap();
        assert_eq!(c2.label(), "B3");

        let result: Result<Cell, _> = "Z9".try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_peers() {
        let cell = Cell::new(0); // A1
        let peers = cell.peers();
        assert!(!peers.has(cell));
        assert!(peers.has(Cell::new(1)));  // A2
        assert!(peers.has(Cell::new(9)));  // B1
        assert!(peers.has(Cell::new(10))); // B2
        assert_eq!(peers.len(), 20);       // 20 Peers für A1
    }

    #[test]
    fn test_add_and_neg_operator() {
        let c1 = Cell::new(0);
        let c2 = Cell::new(1);
        let set = c1 + c2;
        assert!(set.has(c1));
        assert!(set.has(c2));
        assert_eq!(set.len(), 2);

        let inv = -c1;
        assert!(!inv.has(c1));
        assert!(inv.has(c2));
        assert_eq!(inv.len(), 80);
    }

    #[test]
    fn test_iter() {
        let mut iter = Cell::iter();
        for i in 0..Cell::COUNT {
            let c = iter.next().unwrap();
            assert_eq!(c.index(), i);
        }
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_exact_size_iterator_len() {
        let iter = Cell::iter();
        assert_eq!(iter.len(), 81);
        let mut iter2 = Cell::iter();
        iter2.next();
        assert_eq!(iter2.len(), 80);
    }

    #[test]
    fn test_common_houses() {
        let c1 = Cell::from_str("A1");
        let c2 = Cell::from_str("A2");
        let houses = c1.common_houses(c2);
        assert!(houses.iter().any(|h| h == c1.row()));
        assert!(houses.iter().any(|h| h == c1.block()));
        assert_eq!(houses.len(), 2);

        let c3 = Cell::from_str("B1");
        let houses2 = c1.common_houses(c3);
        assert_eq!(houses2.len(), 2); // gleiche Spalte + Block
    }

    #[test]
    fn test_houses_and_coords() {
        let c = Cell::from_str("C3");
        let houses = c.houses();
        assert_eq!(houses[Shape::Row.usize()], c.row());
        assert_eq!(houses[Shape::Column.usize()], c.column());
        assert_eq!(houses[Shape::Block.usize()], c.block());

        assert_eq!(c.row_coord(), Coord::new(0));
        assert_eq!(c.column_coord(), Coord::new(2));
        assert_eq!(c.block_coord(), Coord::new(0));

        assert_eq!(c.coord_in_row(), Coord::new(2));
        assert_eq!(c.coord_in_column(), Coord::new(0));
        assert_eq!(c.coord_in_block(), Coord::new(2));
    }

    #[test]
    fn test_labels_formatting() {
        let cells = vec![Cell::from_str("A1"), Cell::from_str("B2"), Cell::from_str("C3")];
        let s = Cell::labels(&cells);
        assert_eq!(s, "( A1 B2 C3 )");
    }

    #[test]
    fn test_display_trait() {
        let c = Cell::from_str("A1");
        let s = format!("{}", c);
        assert_eq!(s, "A1");
    }

    #[test]
    fn test_cell_macro() {
        let c = cell!("A1");
        assert_eq!(c.index(), 0);
        let c2 = cell!("C3");
        assert_eq!(c2.label(), "C3");
    }
}
