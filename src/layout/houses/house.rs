use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Neg};

use crate::layout::houses::house_set::{blocks, cols, rows};
use crate::layout::{Cell, CellSet, Coord};

use super::{HouseSet, Shape};
use super::house_set::Iter;


//Trait, describes House-API
pub trait HouseLike: Copy + Clone + Eq + PartialEq + Sized {
    fn coord(&self) -> Coord;
    fn shape(&self) -> Shape;

    fn usize(&self) -> usize {
        self.coord().usize()
    }

    fn cells(&self) -> CellSet;
    fn cell(&self, coord: Coord) -> Cell;

    fn has(&self, cell: Cell) -> bool {
        self.cells().has(cell)
    }

    fn label(&self) -> &str;
    fn console_label(&self) -> char;

    fn is_row(&self) -> bool {
        matches!(self.shape(), Shape::Row)
    }
    fn is_column(&self) -> bool {
        matches!(self.shape(), Shape::Column)
    }
    fn is_block(&self) -> bool {
        matches!(self.shape(), Shape::Block)
    }

    fn is_top(&self) -> bool {
        self.is_row() && self.coord().u8() == 0
    }
    fn is_bottom(&self) -> bool {
        self.is_row() && self.coord().u8() == 8
    }
    fn is_left(&self) -> bool {
        self.is_column() &&self.coord().u8() == 0
    }
    fn is_right(&self) -> bool {
        self.is_column() && self.coord().u8() == 8
    }
    fn is_block_top(&self) -> bool {
        self.is_row() && self.coord().u8() % 3 == 0
    }
    fn is_block_bottom(&self) -> bool {
        self.is_row() && self.coord().u8() % 3 == 2
    }
    fn is_block_left(&self) -> bool {
        self.is_column() && self.coord().u8() % 3 == 0
    }
    fn is_block_right(&self) -> bool {
        self.is_column() && self.coord().u8() % 3 == 2
    }


    fn intersect<H: HouseLike>(&self, other: H) -> CellSet {
        self.cells().intersect(other.cells())
    }

    fn crossing_houses(&self, cells: CellSet) -> HouseSet;


    //Returns the set of houses of a given shape that belong to this house
    fn houses(&self, shape: Shape) -> HouseSet;

    fn rows(&self) -> HouseSet {
        self.houses(Shape::Row)
    }
    fn columns(&self) -> HouseSet {
        self.houses(Shape::Column)
    }
    fn blocks(&self) -> HouseSet {
        self.houses(Shape::Block)
    }
}

#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq)]
pub struct House {
    shape: Shape,
    coord: Coord,
}

impl House {
    pub const COUNT: u8 = 9;

    pub const fn new(shape: Shape, coord: Coord) -> Self {
        Self { shape, coord }
    }

    pub const fn shape(&self) -> Shape {
        self.shape
    }
    pub const fn coord(&self) -> Coord {
        self.coord
    }
    pub const fn usize(&self) -> usize {
        self.coord.usize()
    }
    pub const fn label(&self) -> &str {
        LABELS[self.shape.usize()][self.coord.usize()]
    }
    pub const fn console_label(&self) -> char {
        CONSOLE_LABELS[self.shape.usize()][self.coord.usize()]
    }

    pub const fn is_row(&self) -> bool {
        self.shape.is_row()
    }

    pub const fn is_column(&self) -> bool {
        self.shape.is_column()
    }

    pub const fn is_block(&self) -> bool {
        self.shape.is_block()
    }

    pub const fn cell(&self, coord: Coord) -> Cell {
        self.shape.cell(self.coord, coord)
    }

    pub const fn cells(&self) -> CellSet {
        self.shape.cells(self.coord)
    }

    pub const fn has(&self, cell: Cell) -> bool {
        self.cells().has(cell)
    }

    pub fn crossing_houses(&self, cells: CellSet) -> HouseSet {
        match self.shape() {
            Shape::Row => cells
                .iter()
                .fold(HouseSet::empty(Shape::Column), | acc, cell | {
                    acc+ cell.column_coord()
                }),
            Shape::Column => cells.iter().fold(HouseSet::empty(Shape::Row), | acc, cell | {
                acc + cell.row_coord()
            }),
            Shape::Block => {
                let mut acc = HouseSet::empty(Shape::Row) + HouseSet::empty(Shape::Column);
                for c in cells.iter() {
                    acc = acc + c.row_coord() + c.column_coord();
                }
                acc
            }
        }
    }

    pub const fn intersect(&self, other: House) -> CellSet {
        INTERSECTIONS[self.shape.usize()][self.coord.usize()][other.shape.usize()]
            [other.coord.usize()]
    }

    pub fn iter() -> HouseIter {
        HouseIter::all()
    }

    pub const fn all_rows() -> HouseSet {
        HouseSet::full(Shape::Row)
    }

    pub fn rows_iter() -> HouseIter {
        HouseIter::new(Shape::Row)
    }

    pub const fn all_columns() -> HouseSet {
        HouseSet::full(Shape::Column)
    }

    pub fn columns_iter() -> HouseIter {
        HouseIter::new(Shape::Column)
    }

    pub const fn all_blocks() -> HouseSet {
        HouseSet::full(Shape::Block)
    }

    pub fn blocks_iter() -> HouseIter {
        HouseIter::new(Shape::Block)
    }
}

impl HouseLike for House {
    fn shape(&self) -> Shape {
        House::shape(self)
    }
    fn coord(&self) -> Coord {
        House::coord(self)
    }
    fn label(&self) -> &str {
        House::label(self)
    }
    fn console_label(&self) -> char {
        House::console_label(self)
    }
    fn cells(&self) -> CellSet {
        House::cells(self)
    }
    fn cell(&self, coord: Coord) -> Cell {
        House::cell(self, coord)
    }
    fn crossing_houses(&self, cells: CellSet) -> HouseSet {
        House::crossing_houses(self, cells)
    }
    fn houses(&self, shape: Shape) -> HouseSet {
        House::houses(self, shape)
    }
}


impl From<&str> for House {
    fn from(label: &str) -> Self {
        if label.len() != 2 {
            panic!(
                "Invalid house: \"{}\"; must be (R | C | B) and a digit",
                label
            );
        }
        let mut chars = label.chars();
        let shape = chars.next().unwrap();
        if shape != 'R' && shape != 'C' && shape != 'B' {
            panic!("Invalid house shape: \"{}\"; must be (R | C | B)", label);
        }
        let coord = chars.next().unwrap() as u8 - b'1';
        if coord >= 9 {
            panic!("Invalid house coord: \"{}\"; must be 1-9", label);
        }

        Self {
            shape: Shape::from(shape),
            coord: Coord::from(coord),
        }
    }
}

impl PartialOrd for House {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.shape.partial_cmp(&other.shape) {
            Some(Ordering::Equal) => self.coord.partial_cmp(&other.coord),
            result => result,
        }
    }
}

impl Add<House> for House {
    type Output = HouseSet;

    fn add(self, rhs: House) -> HouseSet {
        HouseSet::empty(self.shape) + self + rhs
    }
}

impl Neg for House {
    type Output = HouseSet;

    fn neg(self) -> HouseSet {
        HouseSet::full(self.shape) - self
    }
}

impl fmt::Display for House {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Row {
    coord: Coord,
}

impl Row {
    pub const fn new(coord: Coord) -> Self {
        Self { coord }
    }
    pub const fn from_coord(coord: Coord) -> Self {
        Self::new(coord)
    }
}

impl HouseLike for Row {
    fn coord(&self) -> Coord { self.coord }
    fn shape(&self) -> Shape { Shape::Row }

    fn cells(&self) -> CellSet {
        ROW_CELLS[self.coord.usize()]
    }

    fn cell(&self, coord: Coord) -> Cell {
        Cell::from_row(self.coord, coord)
    }

    fn label(&self) -> &str { &LABELS[0][self.coord.usize()]}
    fn console_label(&self) -> char { CONSOLE_LABELS[0][self.coord.usize()]}

    fn crossing_houses(&self, cells: CellSet) -> HouseSet {
        cells.iter().fold(HouseSet::empty(Shape::Column), |acc, c| acc + c.column_coord())
    }
    fn houses(&self, shape: Shape) -> HouseSet {
        match shape {
            Shape::Row => ROW_ROWS[self.coord.usize()],
            Shape::Column => ROW_COLUMNS[self.coord.usize()],
            Shape::Block => ROW_BLOCKS[self.coord.usize()],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Column {
    coord: Coord,
}

impl Column {
    pub const fn new(coord: Coord) -> Self {
        Self { coord }
    }
}

impl HouseLike for Column {
    fn coord(&self) -> Coord { self.coord }
    fn shape(&self) -> Shape { Shape::Column }

    fn cells(&self) -> CellSet {
        COLUMN_CELLS[self.coord.usize()]
    }

    fn cell(&self, coord: Coord) -> Cell {
        Cell::from_column(self.coord, coord)
    }

    fn label(&self) -> &str { &LABELS[1][self.coord.usize()] }
    fn console_label(&self) -> char { CONSOLE_LABELS[1][self.coord.usize()] }

    fn crossing_houses(&self, cells: CellSet) -> HouseSet {
        cells.iter().fold(HouseSet::empty(Shape::Row), |acc, c| acc + c.row_coord())
    }
    fn houses(&self, shape: Shape) -> HouseSet {
        match shape {
            Shape::Row => COLUMN_ROWS[self.coord.usize()],
            Shape::Column => COLUMN_COLUMNS[self.coord.usize()],
            Shape::Block => COLUMN_BLOCKS[self.coord.usize()],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Block{
    coord: Coord,
}

impl Block {
    pub const fn new(coord: Coord) -> Self {
        Self { coord }
    }
}

impl HouseLike for Block {
    fn coord(&self) -> Coord { self.coord }
    fn shape(&self) -> Shape { Shape::Block}

    fn cells(&self) -> CellSet {
        BLOCK_CELLS[self.coord.usize()]
    }

    fn cell(&self, coord: Coord) -> Cell {
        Cell::from_block(self.coord, coord)
    }

    fn label(&self) -> &str { &LABELS[2][self.coord.usize()] }
    fn console_label(&self) -> char { CONSOLE_LABELS[2][self.coord.usize()] }

    fn crossing_houses(&self, cells: CellSet) -> HouseSet {
        let mut acc = HouseSet::empty(Shape::Row) + HouseSet::empty(Shape::Column);
        for c in cells.iter() {
            acc = acc + c.row_coord() + c.column_coord();
        }
        acc
    }
    fn houses(&self, shape: Shape) -> HouseSet {
        match shape {
            Shape::Row => BLOCK_ROWS[self.coord.usize()],
            Shape::Column => BLOCK_COLUMNS[self.coord.usize()],
            Shape::Block => BLOCK_BLOCKS[self.coord.usize()],
        }
    }
}

pub struct HouseIter {
    shape: Shape,
    coord: u8,
}

impl HouseIter {
    pub const fn new(shape: Shape) -> Self {
        Self { shape, coord: 0}
    }
    pub const fn all() -> Self {
        Self {
            shape: Shape::Row,
            coord:0,
        }
    }
}

impl Iterator for HouseIter {
    type Item = House;

    fn next(&mut self) -> Option<Self::Item> {
        if self.coord == 9 {
            None
        } else {
            let house = House::new(self.shape, self.coord.into());
            self.coord += 1;
            Some(house)
        }
    }
}

impl ExactSizeIterator for HouseIter {
    fn len(&self) -> usize {
        9 - self.coord as usize
    }
}

pub struct HousesIter {
    shape: Shape, 
    coord: u8,
}

impl HousesIter {
    pub const fn new() -> Self {
        Self {
            shape: Shape::Row,
            coord: 0,
        }
    }
}

impl Iterator for HousesIter {
    type Item = House;

    fn next(&mut self) -> Option<Self::Item> {
        if self.coord == 9 {
            match self.shape {
                Shape::Row => {
                    self.shape = Shape::Column;
                    self.coord = 0;
                }
                Shape::Column => {
                    self.shape = Shape::Block;
                    self.coord = 0;
                }
                Shape::Block => return None,
            }
        }
        let house = House::new(self.shape, self.coord.into());
        self.coord += 1;
        Some(house)
    }
}

impl ExactSizeIterator for HousesIter {
    fn len(&self) -> usize {
        match self.shape {
            Shape::Row => 18 +9 - self.coord as usize,
            Shape::Column => 9 + 9 - self.coord as usize,
            Shape::Block => 9 - self.coord as usize,
        }
    }
}

#[rustfmt::skip]
pub const LABELS: [[&str; 9]; 3] = [
    ["Row A", "Row B", "Row C", "Row D", "Row E", "Row F", "Row G", "Row H", "Row I"],
    ["Col 1", "Col 2", "Col 3", "Col 4", "Col 5", "Col 6", "Col 7", "Col 8", "Col 9"],
    ["Box 1", "Box 2", "Box 3", "Box 4", "Box 5", "Box 6", "Box 7", "Box 8", "Box 9"],
];

#[rustfmt::skip]
pub const CONSOLE_LABELS: [[char; 9]; 3] = [
    ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I'],
    ['1', '2', '3', '4', '5', '6', '7', '8', '9'],
    ['❶', '❷', '❸', '❹', '❺', '❻', '❼', '❽', '❾'],
];

#[rustfmt::skip]
pub const ALT_CONSOLE_LABELS: [[char; 9]; 3] = [
    ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I'],
    ['1', '2', '3', '4', '5', '6', '7', '8', '9'],
    ['❶', '❷', '❸', '❹', '❺', '❻', '❼', '❽', '❾'],
];

pub const ROWS: [House; 9] = make_houses(Shape::Row);
pub const COLUMNS: [House; 9] = make_houses(Shape::Column);
pub const BLOCKS: [House; 9] = make_houses(Shape::Block);

const fn make_houses(shape: Shape) -> [House; 9] {
    let mut houses: [House; 9] = [House::new(Shape::Row, Coord::new(0)); 9];
    let mut i = 0;

    while i < 9 {
        houses[i] = House::new(shape, Coord::new(i as u8));
        i += 1;
    }
    houses
}

pub const ALL: [House; 27] = {
    let mut houses: [House; 27] = [House::new(Shape::Row, Coord::new(0)); 27];
    let mut i = 0;

    while i < 9 {
        houses[i] = ROWS[i];
        houses[i + 9] = COLUMNS[i];
        houses[i + 18] = BLOCKS[i];
        i += 1;
    }
    houses
};

pub const INTERSECTIONS: [[[[CellSet; 9]; 3]; 9]; 3] = {
    let mut sets: [[[[CellSet; 9]; 3]; 9]; 3] = [[[[CellSet::empty(); 9]; 3]; 9]; 3];

    let mut i = 0;
    while i < 3 {
        let mut ii = 0;
        while ii < 9 {
            let mut j = 0;
            while j < 3 {
                let mut jj = 0;
                while jj < 9 {
                    sets[i][ii][j][jj] = House::new(Shape::new(i as u8), Coord::new(ii as u8))
                        .cells()
                        .intersect(House::new(Shape::new(j as u8), Coord::new(jj as u8)).cells());
                    jj += 1;
                }
                j += 1;
            }
            ii += 1;
        }
        i += 1;
    }
    sets
};

const ROW_ROWS: [HouseSet; 9] = [
    rows!(1),
    rows!(2),
    rows!(3),
    rows!(4),
    rows!(5),
    rows!(6),
    rows!(7),
    rows!(8),
    rows!(9),
];

const COLUMN_ROWS: [HouseSet; 9] = [House::all_rows(); 9];

const BLOCK_ROWS: [HouseSet; 9] = [
    rows!(123), rows!(123), rows!(123),
    rows!(456), rows!(456), rows!(456),
    rows!(789), rows!(789), rows!(789),
];

const ROW_COLUMNS: [HouseSet; 9] = [House::all_columns(); 9];

const COLUMN_COLUMNS: [HouseSet; 9] = [
    cols!(1),
    cols!(2),
    cols!(3),
    cols!(4),
    cols!(5),
    cols!(6),
    cols!(7),
    cols!(8),
    cols!(9),
];

#[rustfmt::skip]
const BLOCK_COLUMNS: [HouseSet; 9] = [
    cols!(123), cols!(456), cols!(789),
    cols!(123), cols!(456), cols!(789),
    cols!(123), cols!(456), cols!(789),
];

const ROW_BLOCKS: [HouseSet; 9] = [
    blocks!(123),
    blocks!(123),
    blocks!(123),
    blocks!(456),
    blocks!(456),
    blocks!(456),
    blocks!(789),
    blocks!(789),
    blocks!(789),
];

const COLUMN_BLOCKS: [HouseSet; 9] = [
    blocks!(147),
    blocks!(147),
    blocks!(147),
    blocks!(258),
    blocks!(258),
    blocks!(258),
    blocks!(369),
    blocks!(369),
    blocks!(369),
];

const BLOCK_BLOCKS: [HouseSet; 9] = [
    blocks!(1),
    blocks!(2),
    blocks!(3),
    blocks!(4),
    blocks!(5),
    blocks!(6),
    blocks!(7),
    blocks!(8),
    blocks!(9),
];

#[allow(unused_macros)]
macro_rules! row {
    ($c:expr) => {
        House::row(coord!($c))
    };
}

#[allow(unused_macros)]
macro_rules! col {
    ($c:expr) => {
        House::column(coord!($c))
    };
}

#[allow(unused_macros)]
macro_rules! block {
    ($c:expr) => {
        House::block(coord!($c))
    };
}

#[allow(unused_imports)]
pub(crate) use {block, col, row};

impl House {
    pub const fn row(coord: Coord) -> Self {
        ROWS[coord.usize()]
    }
    pub const fn column(coord: Coord) -> Self {
        COLUMNS[coord.usize()]
    }
    pub const fn block(coord: Coord) -> Self {
        BLOCKS[coord.usize()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::houses::coord::coord;
    use crate::layout::houses::house_set::houses;

    #[test]
    fn houses() {
        let house_sets = [House::all_rows(), House::all_columns(), House::all_blocks()];

        for houses in house_sets {
            let mut all = CellSet::empty();

            for (i, house) in houses.iter().enumerate() {
                assert_eq!(house, House::new(houses.shape(), Coord::new(i as u8)));
                assert_eq!(houses.shape(), house.shape());
                assert_eq!(Coord::new(i as u8), house.coord());
                assert_eq!(i, house.usize());
                if !matches!(houses.shape(), Shape::Row) {
                    assert_eq!(format!("{} {}", houses.shape(), i + 1), house.label());
                }

                let mut house_cells = CellSet::empty();
                (0..9).for_each(|c| {
                    let cell = house.cell(c.into());
                    assert_eq!(house, cell.house(houses.shape()));
                    house_cells += cell
                });
                assert_eq!(house.cells(), house_cells);

                all |= house.cells();
            }

            assert_eq!(CellSet::full(), all);
        }
    }

    #[test]
    fn intersect() {
        assert_eq!(cells!("A1 A2 A3"), House::row(coord!(1)).intersect(House::block(coord!(1))));
    }

    #[test]
    fn row_cells() {
        assert_eq!(cells!("A1 A2 A3 A4 A5 A6 A7 A8 A9"), House::row(coord!(1)).cells());
        assert_eq!(cells!("B1 B2 B3 B4 B5 B6 B7 B8 B9"), House::row(coord!(2)).cells());
        assert_eq!(cells!("C1 C2 C3 C4 C5 C6 C7 C8 C9"), House::row(coord!(3)).cells());
        assert_eq!(cells!("D1 D2 D3 D4 D5 D6 D7 D8 D9"), House::row(coord!(4)).cells());
        assert_eq!(cells!("E1 E2 E3 E4 E5 E6 E7 E8 E9"), House::row(coord!(5)).cells());
        assert_eq!(cells!("F1 F2 F3 F4 F5 F6 F7 F8 F9"), House::row(coord!(6)).cells());
        assert_eq!(cells!("G1 G2 G3 G4 G5 G6 G7 G8 G9"), House::row(coord!(7)).cells());
        assert_eq!(cells!("H1 H2 H3 H4 H5 H6 H7 H8 H9"), House::row(coord!(8)).cells());
        assert_eq!(cells!("I1 I2 I3 I4 I5 I6 I7 I8 I9"), House::row(coord!(9)).cells());
    }

    #[test]
    fn column_cells() {
        assert_eq!(cells!("A1 B1 C1 D1 E1 F1 G1 H1 I1"), House::column(coord!(1)).cells());
        assert_eq!(cells!("A2 B2 C2 D2 E2 F2 G2 H2 I2"), House::column(coord!(2)).cells());
        assert_eq!(cells!("A3 B3 C3 D3 E3 F3 G3 H3 I3"), House::column(coord!(3)).cells());
        assert_eq!(cells!("A4 B4 C4 D4 E4 F4 G4 H4 I4"), House::column(coord!(4)).cells());
        assert_eq!(cells!("A5 B5 C5 D5 E5 F5 G5 H5 I5"), House::column(coord!(5)).cells());
        assert_eq!(cells!("A6 B6 C6 D6 E6 F6 G6 H6 I6"), House::column(coord!(6)).cells());
        assert_eq!(cells!("A7 B7 C7 D7 E7 F7 G7 H7 I7"), House::column(coord!(7)).cells());
        assert_eq!(cells!("A8 B8 C8 D8 E8 F8 G8 H8 I8"), House::column(coord!(8)).cells());
        assert_eq!(cells!("A9 B9 C9 D9 E9 F9 G9 H9 I9"), House::column(coord!(9)).cells());
    }

    #[test]
    fn block_cells() {
        assert_eq!(cells!("A1 A2 A3 B1 B2 B3 C1 C2 C3"), House::block(coord!(1)).cells());
        assert_eq!(cells!("A4 A5 A6 B4 B5 B6 C4 C5 C6"), House::block(coord!(2)).cells());
        assert_eq!(cells!("A7 A8 A9 B7 B8 B9 C7 C8 C9"), House::block(coord!(3)).cells());
        assert_eq!(cells!("D1 D2 D3 E1 E2 E3 F1 F2 F3"), House::block(coord!(4)).cells());
        assert_eq!(cells!("D4 D5 D6 E4 E5 E6 F4 F5 F6"), House::block(coord!(5)).cells());
        assert_eq!(cells!("D7 D8 D9 E7 E8 E9 F7 F8 F9"), House::block(coord!(6)).cells());
        assert_eq!(cells!("G1 G2 G3 H1 H2 H3 I1 I2 I3"), House::block(coord!(7)).cells());
        assert_eq!(cells!("G4 G5 G6 H4 H5 H6 I4 I5 I6"), House::block(coord!(8)).cells());
        assert_eq!(cells!("G7 G8 G9 H7 H8 H9 I7 I8 I9"), House::block(coord!(9)).cells());
    }

    #[test]
    fn columns_cross_rows() {
        let main = House::row(coord!(2));
        let cells = cells!("B1 B2");
        let got = main.crossing_houses(cells);

        assert_eq!(houses!("C1 C2"), got);
    }

    #[test]
    fn rows_cross_columns() {
        let main = House::column(coord!(6));
        let cells = cells!("C6 F6");
        let got = main.crossing_houses(cells);

        assert_eq!(houses!("R3 R6"), got);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::houses::coord::coord;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::houses::house_set::houses;

    #[test]
    fn house_creation_and_properties() {
        let r = House::row(coord!(0));
        let c = House::column(coord!(1));
        let b = House::block(coord!(2));

        // Shape & Coord
        assert!(r.is_row());
        assert!(!r.is_column());
        assert!(!r.is_block());
        assert_eq!(r.coord(), coord!(0));

        assert!(c.is_column());
        assert_eq!(c.coord(), coord!(1));

        assert!(b.is_block());
        assert_eq!(b.coord(), coord!(2));

        // Label & console_label
        assert_eq!(r.label(), "Row A");
        assert_eq!(r.console_label(), 'A');
        assert_eq!(c.label(), "Col 2");
        assert_eq!(c.console_label(), '2');
        assert_eq!(b.label(), "Box 3");
        assert_eq!(b.console_label(), '❸');
    }

    #[test]
    fn house_from_str() {
        let r: House = "R1".into();
        assert_eq!(r, House::row(coord!(0)));

        let c: House = "C5".into();
        assert_eq!(c, House::column(coord!(4)));

        let b: House = "B9".into();
        assert_eq!(b, House::block(coord!(8)));
    }

    #[test]
    #[should_panic]
    fn house_from_str_invalid_shape() {
        let _ : House = "X1".into();
    }

    #[test]
    #[should_panic]
    fn house_from_str_invalid_coord() {
        let _ : House = "R0".into();
    }

    #[test]
    fn cells_and_cell_access() {
        let r = House::row(coord!(1));
        let expected = cells!("B1 B2 B3 B4 B5 B6 B7 B8 B9");
        assert_eq!(r.cells(), expected);
        assert_eq!(r.cell(coord!(0)), Cell::from_row(coord!(1), coord!(0)));

        let c = House::column(coord!(2));
        let expected_c = cells!("A3 B3 C3 D3 E3 F3 G3 H3 I3");
        assert_eq!(c.cells(), expected_c);
        assert_eq!(c.cell(coord!(1)), Cell::from_column(coord!(2), coord!(1)));

        let b = House::block(coord!(3));
        let expected_b = cells!("D1 D2 D3 E1 E2 E3 F1 F2 F3");
        assert_eq!(b.cells(), expected_b);
        assert_eq!(b.cell(coord!(0)), Cell::from_block(coord!(3), coord!(0)));
    }

    #[test]
    fn house_intersections() {
        let row = House::row(coord!(0));
        let block = House::block(coord!(0));
        let expected = cells!("A1 A2 A3");
        assert_eq!(row.intersect(block), expected);

        let col = House::column(coord!(0));
        assert_eq!(col.intersect(block), cells!("A1 D1 G1"));
    }

    #[test]
    fn crossing_houses_test() {
        let row = House::row(coord!(1));
        let selected_cells = cells!("B1 B2");
        let crossing = row.crossing_houses(selected_cells);
        assert_eq!(crossing, houses!("C1 C2"));

        let col = House::column(coord!(5));
        let selected_cells = cells!("C6 F6");
        let crossing_col = col.crossing_houses(selected_cells);
        assert_eq!(crossing_col, houses!("R3 R6"));
    }

    #[test]
    fn house_iterators() {
        let mut iter = HouseIter::new(Shape::Row);
        for i in 0..9 {
            let house = iter.next().unwrap();
            assert_eq!(house.coord(), Coord::new(i));
            assert!(house.is_row());
        }
        assert!(iter.next().is_none());

        let mut all_iter = HousesIter::new();
        let mut count = 0;
        while let Some(h) = all_iter.next() {
            count += 1;
            assert!(matches!(h.shape(), Shape::Row | Shape::Column | Shape::Block));
        }
        assert_eq!(count, 27);
    }

    #[test]
    fn add_and_neg_operators() {
        let r1 = House::row(coord!(0));
        let r2 = House::row(coord!(1));
        let hs = r1 + r2; // HouseSet
        assert_eq!(hs.len(), 2);
        assert!(hs.contains(r1));
        assert!(hs.contains(r2));

        let hs_neg = -r1; // HouseSet without r1
        assert_eq!(hs_neg.len(), 8);
        assert!(!hs_neg.contains(r1));
    }

    #[test]
    fn house_boundaries() {
        let top_row = House::row(coord!(0));
        let bottom_row = House::row(coord!(8));
        let left_col = House::column(coord!(0));
        let right_col = House::column(coord!(8));

        assert!(top_row.is_top());
        assert!(!bottom_row.is_top());
        assert!(bottom_row.is_bottom());
        assert!(left_col.is_left());
        assert!(right_col.is_right());

        // Block boundaries
        let block_row_top = House::row(coord!(0));
        let block_row_bottom = House::row(coord!(2));
        let block_col_left = House::column(coord!(0));
        let block_col_right = House::column(coord!(2));

        assert!(block_row_top.is_block_top());
        assert!(block_row_bottom.is_block_bottom());
        assert!(block_col_left.is_block_left());
        assert!(block_col_right.is_block_right());
    }

    #[test]
    fn partial_ord_house() {
        let h1 = House::row(coord!(0));
        let h2 = House::row(coord!(1));
        let h3 = House::column(coord!(0));
        assert!(h1 < h2);
        assert!(h2 > h1);
        assert!(h1 < h3); // Rows < Columns
    }
}