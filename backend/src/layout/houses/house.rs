use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Neg};
use std::sync::OnceLock;

use crate::layout::houses::house_set::{blocks, cols, rows};
use crate::layout::{Cell, CellSet, Coord};

use super::{HouseSet, Shape};

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
        self.is_column() && self.coord().u8() == 0
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

    pub fn cell(&self, coord: Coord) -> Cell {
        self.shape.cell(self.coord, coord)
    }

    pub fn cells(&self) -> CellSet {
        self.shape.cells(self.coord)
    }

    pub fn has(&self, cell: Cell) -> bool {
        self.cells().has(cell)
    }

    pub fn crossing_houses(&self, cells: CellSet) -> HouseSet {
        match self.shape() {
            Shape::Row => cells
                .iter()
                .fold(HouseSet::empty(Shape::Column), |acc, cell| {
                    acc + cell.column_coord()
                }),
            Shape::Column => cells.iter().fold(HouseSet::empty(Shape::Row), |acc, cell| {
                acc + cell.row_coord()
            }),
            Shape::Block => {
                let mut acc = HouseSet::empty(Shape::Row);
                for c in cells.iter() {
                    acc = acc + c.row_coord();
                }
                acc
            }
        }
    }

    pub fn intersect(&self, other: House) -> CellSet {
        self.cells().intersect(other.cells())
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

    pub fn all_blocks() -> HouseSet {
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
        match self.shape() {
            Shape::Row => match shape {
                Shape::Row => ROW_ROWS[self.coord().usize()],
                Shape::Column => ROW_COLUMNS[self.coord().usize()],
                Shape::Block => ROW_BLOCKS[self.coord().usize()],
            },
            Shape::Column => match shape {
                Shape::Row => COLUMN_ROWS[self.coord().usize()],
                Shape::Column => COLUMN_COLUMNS[self.coord().usize()],
                Shape::Block => COLUMN_BLOCKS[self.coord().usize()],
            },
            Shape::Block => match shape {
                Shape::Row => BLOCK_ROWS[self.coord().usize()],
                Shape::Column => BLOCK_COLUMNS[self.coord().usize()],
                Shape::Block => BLOCK_BLOCKS[self.coord().usize()],
            },
        }
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
    fn coord(&self) -> Coord {
        self.coord
    }
    fn shape(&self) -> Shape {
        Shape::Row
    }

    fn cells(&self) -> CellSet {
        house_cells(Shape::Row)[self.coord.usize()]
    }

    fn cell(&self, coord: Coord) -> Cell {
        Cell::from_row(self.coord, coord)
    }

    fn label(&self) -> &str {
        &LABELS[0][self.coord.usize()]
    }
    fn console_label(&self) -> char {
        CONSOLE_LABELS[0][self.coord.usize()]
    }

    fn crossing_houses(&self, cells: CellSet) -> HouseSet {
        cells.iter().fold(HouseSet::empty(Shape::Column), |acc, c| {
            acc + c.column_coord()
        })
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
    fn coord(&self) -> Coord {
        self.coord
    }
    fn shape(&self) -> Shape {
        Shape::Column
    }

    fn cells(&self) -> CellSet {
        house_cells(Shape::Column)[self.coord.usize()]
    }

    fn cell(&self, coord: Coord) -> Cell {
        Cell::from_column(self.coord, coord)
    }

    fn label(&self) -> &str {
        &LABELS[1][self.coord.usize()]
    }
    fn console_label(&self) -> char {
        CONSOLE_LABELS[1][self.coord.usize()]
    }

    fn crossing_houses(&self, cells: CellSet) -> HouseSet {
        cells
            .iter()
            .fold(HouseSet::empty(Shape::Row), |acc, c| acc + c.row_coord())
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
pub struct Block {
    coord: Coord,
}

impl Block {
    pub const fn new(coord: Coord) -> Self {
        Self { coord }
    }
}

impl HouseLike for Block {
    fn coord(&self) -> Coord {
        self.coord
    }
    fn shape(&self) -> Shape {
        Shape::Block
    }

    fn cells(&self) -> CellSet {
        house_cells(Shape::Block)[self.coord.usize()]
    }

    fn cell(&self, coord: Coord) -> Cell {
        Cell::from_block(House::block(self.coord), coord)
    }

    fn label(&self) -> &str {
        &LABELS[2][self.coord.usize()]
    }
    fn console_label(&self) -> char {
        CONSOLE_LABELS[2][self.coord.usize()]
    }

    fn crossing_houses(&self, cells: CellSet) -> HouseSet {
        let mut acc = HouseSet::empty(Shape::Row);
        for c in cells.iter() {
            acc = acc + c.row_coord();
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
        Self { shape, coord: 0 }
    }
    pub const fn all() -> Self {
        Self {
            shape: Shape::Row,
            coord: 0,
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
            Shape::Row => 18 + 9 - self.coord as usize,
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

pub fn intersections() -> &'static [[[[CellSet; 9]; 3]; 9]; 3] {
    static INT: OnceLock<[[[[CellSet; 9]; 3]; 9]; 3]> = OnceLock::new();
    INT.get_or_init(|| {
        let mut sets: [[[[CellSet; 9]; 3]; 9]; 3] = [[[[CellSet::empty(); 9]; 3]; 9]; 3];

        let mut i = 0usize;
        while i < 3 {
            let mut ii = 0usize;
            while ii < 9 {
                let mut j = 0usize;
                while j < 3 {
                    let mut jj = 0usize;
                    while jj < 9 {
                        let a = House::new(Shape::new(i as u8), Coord::new(ii as u8)).cells();
                        let b = House::new(Shape::new(j as u8), Coord::new(jj as u8)).cells();
                        sets[i][ii][j][jj] = a.intersect(b);
                        jj += 1;
                    }
                    j += 1;
                }
                ii += 1;
            }
            i += 1;
        }
        sets
    })
}

fn house_cells(shape: Shape) -> &'static [CellSet; 9] {
    static ROW: OnceLock<[CellSet; 9]> = OnceLock::new();
    static COL: OnceLock<[CellSet; 9]> = OnceLock::new();
    static BLK: OnceLock<[CellSet; 9]> = OnceLock::new();

    let slot = match shape {
        Shape::Row => &ROW,
        Shape::Column => &COL,
        Shape::Block => &BLK,
    };

    slot.get_or_init(|| {
        let mut out = [CellSet::empty(); 9];
        let mut i = 0usize;
        while i < 9 {
            out[i] = shape.cells(Coord::new(i as u8));
            i += 1;
        }
        out
    })
}

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
    rows!(123),
    rows!(123),
    rows!(123),
    rows!(456),
    rows!(456),
    rows!(456),
    rows!(789),
    rows!(789),
    rows!(789),
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

    #[test]
    fn house_creation_and_basic_properties() {
        let r = House::row(Coord::from(0));
        let c = House::column(Coord::from(1));
        let b = House::block(Coord::from(2));

        assert!(r.is_row());
        assert!(!r.is_column());
        assert!(!r.is_block());
        assert_eq!(r.coord(), Coord::from(0));

        assert!(c.is_column());
        assert_eq!(c.coord(), Coord::from(1));

        assert!(b.is_block());
        assert_eq!(b.coord(), Coord::from(2));
    }

    #[test]
    fn labels_and_console_labels() {
        let r = House::row(Coord::from(0));
        let c = House::column(Coord::from(1));
        let b = House::block(Coord::from(2));

        assert_eq!(r.label(), "Row A");
        assert_eq!(r.console_label(), 'A');

        assert_eq!(c.label(), "Col 2");
        assert_eq!(c.console_label(), '2');

        assert_eq!(b.label(), "Box 3");
        assert_eq!(b.console_label(), '❸');
    }

    #[test]
    fn row_cells_are_correct() {
        let r = House::row(Coord::from(1)); // Row B
        let mut expected = CellSet::empty();

        for col in 0..9 {
            expected += Cell::from_row(Coord::from(1), Coord::from(col));
        }

        assert_eq!(r.cells(), expected);
    }

    #[test]
    fn column_cells_are_correct() {
        let c = House::column(Coord::from(2)); // Column 3
        let mut expected = CellSet::empty();

        for row in 0..9 {
            expected += Cell::from_column(Coord::from(2), Coord::from(row));
        }

        assert_eq!(c.cells(), expected);
    }

    #[test]
    fn block_cells_are_correct() {
        let b = House::block(Coord::from(0)); // Block 1
        let mut expected = CellSet::empty();

        for i in 0..9 {
            expected += Cell::from_block(b, Coord::from(i));
        }

        assert_eq!(b.cells(), expected);
    }

    #[test]
    fn cell_access_matches_cells() {
        let r = House::row(Coord::from(3));

        for i in 0..9 {
            let cell = r.cell(Coord::from(i));
            assert!(r.has(cell));
        }
    }

    #[test]
    fn house_from_str() {
        let r: House = "R1".into();
        let c: House = "C5".into();
        let b: House = "B9".into();

        assert_eq!(r, House::row(Coord::from(0)));
        assert_eq!(c, House::column(Coord::from(4)));
        assert_eq!(b, House::block(Coord::from(8)));
    }

    #[test]
    #[should_panic]
    fn house_from_str_invalid_shape() {
        let _: House = "X1".into();
    }

    #[test]
    #[should_panic]
    fn house_from_str_invalid_coord() {
        let _: House = "R0".into();
    }

    #[test]
    fn house_intersections() {
        let row = House::row(Coord::from(0));
        let block = House::block(Coord::from(0));
        let col = House::column(Coord::from(0));

        let rb = row.intersect(block);
        let mut expected_rb = CellSet::empty();
        expected_rb += Cell::from_row(Coord::from(0), Coord::from(0));
        expected_rb += Cell::from_row(Coord::from(0), Coord::from(1));
        expected_rb += Cell::from_row(Coord::from(0), Coord::from(2));

        assert_eq!(rb, expected_rb);

        let cb = col.intersect(block);
        let mut expected_cb = CellSet::empty();
        expected_cb += Cell::from_column(Coord::from(0), Coord::from(0));
        expected_cb += Cell::from_column(Coord::from(0), Coord::from(1));
        expected_cb += Cell::from_column(Coord::from(0), Coord::from(2));

        assert_eq!(cb, expected_cb);
    }

    #[test]
    fn crossing_houses_row_to_columns() {
        let row = House::row(Coord::from(1)); // Row B
        let mut cells = CellSet::empty();
        cells += Cell::from_row(Coord::from(1), Coord::from(0));
        cells += Cell::from_row(Coord::from(1), Coord::from(1));

        let houses = row.crossing_houses(cells);

        let mut expected = HouseSet::empty(Shape::Column);
        expected += House::column(Coord::from(0));
        expected += House::column(Coord::from(1));

        assert_eq!(houses, expected);
    }

    #[test]
    fn crossing_houses_column_to_rows() {
        let col = House::column(Coord::from(5)); // Column 6
        let mut cells = CellSet::empty();
        cells += Cell::from_column(Coord::from(5), Coord::from(2));
        cells += Cell::from_column(Coord::from(5), Coord::from(5));

        let houses = col.crossing_houses(cells);

        let mut expected = HouseSet::empty(Shape::Row);
        expected += House::row(Coord::from(2));
        expected += House::row(Coord::from(5));

        assert_eq!(houses, expected);
    }

    #[test]
    fn house_iterators() {
        let mut iter = HouseIter::new(Shape::Row);

        for i in 0..9 {
            let h = iter.next().unwrap();
            assert!(h.is_row());
            assert_eq!(h.coord(), Coord::from(i));
        }

        assert!(iter.next().is_none());

        let mut all = HousesIter::new();
        let mut count = 0;

        while let Some(h) = all.next() {
            assert!(matches!(
                h.shape(),
                Shape::Row | Shape::Column | Shape::Block
            ));
            count += 1;
        }

        assert_eq!(count, 27);
    }

    #[test]
    fn ordering_of_houses() {
        let r0 = House::row(Coord::from(0));
        let r1 = House::row(Coord::from(1));
        let c0 = House::column(Coord::from(0));

        assert!(r0 < r1);
        assert!(r1 < c0); 
    }
}
