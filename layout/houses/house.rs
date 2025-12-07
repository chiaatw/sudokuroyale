pub trait HouseLike {
    fn coord(&self) -> Coord;
    fn shape(&self) -> Shape;

    fn cells(&self) -> CellSet;
    fn cell(&self, coord: Coord) -> Cell;

    fn has(&self, cell: Cell) -> bool {
        self.cells().has(cell)
    }

    fn label(&self) -> &str;
    fn console_label(&self) -> char;

    fn intersect(&self, other: &dyn HouseLike) -> CellSet;
    fn crossing_houses(&self, cells: CellSet) -> HouseSet;
}

pub struct Row {
    coord: Coord,
}

pub struct Column {
    coord: Coord,
}

pub struct Block{
    coord: Coord,
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

    fn intersect(&self, other: &dyn HouseLike) -> CellSet {
        self.cells().intersect(other.cells())
    }
    fn crossing_houses(&self, cells: CellSet) -> HouseSet {
        cells.iter().fold(HouseSet::empty(Shape::Column), |acc, c| acc + c.column_coord())
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

    fn intersect(&self, other: &dyn HouseLike) -> CellSet {
        self.cells().intersect(other.cells())
    }

    fn crossing_houses(&self, cells: CellSet) -> HouseSet {
        cells.iter().fold(HouseSet::empty(Shape::Row), |acc, c| acc + c.row_coord())
    }
}

impl HouseLike for Block {
    fn coord(&self) -> Coord { self.coord }
    fn shape(&self) -> Shape { Shape:: Block}

    fn cells(&self) -> CellSet {
        BLOCK_CELLS[self.coord.usize()]
    }

    fn cell(&self, coord: Coord) -> Cell {
        Cell::from_block(self.coord, coord)
    }

    fn label(&self) -> &str { &LABELS[2][self.coord.usize()] }
    fn console_label(&self) -> char { CONSOLE_LABELS[2][self.coord.usize()] }

    fn intersect(&self, other: &dyn HouseLike) -> CellSet {
        self.cells().intersect(other.cells())
    }

    fn crossing_houses(&self, cells: CellSet) -> HouseSet {
        let mut acc = HouseSet::empty(Shape::Row) + HouseSet::empty(Shape::Column);
        for c in cells.iter() {
            acc = acc + c.row_coord() + c.column_coord();
        }
        acc
    }
}

pub trait HouseIterator {
    type Item: HouseLike;

    fn iter() -> Box<dyn Iterator<Item = Self::Item>>;
}

pub struct RowIter {
    i: u8,
}

impl Iterator for RowIter {
    type Item = Row;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= 9 { return None; }
        let row = Row { coord: Coord::new(self.i) };
        self.i += 1;
        Some(row)
    }
}

impl HouseIterator for Row {
    type Item = Row;

    fn iter() -> Box<dyn Iterator<Item = Self::Item>> {
        Box::new(RowIter { i: 0 })
    }
}

pub struct ColumnIter {
    i: u8,
}

impl Iterator for ColumnIter {
    type Item = Column;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= 9 { return None; }
        let col = Column { coord: Coord::new(self.i) };
        self.i += 1;
        Some(col)
    }
}

impl HouseIterator for Column {
    type Item = Column;

    fn iter() -> Box<dyn Iterator<Item = Self::Item>> {
        Box::new(ColumnIter { i:0 })
    }
}

pub struct BlockIter {
    i: u8,
}

impl Iterator for BlockIter {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= 9 { return None; }
        let block = Block { coord: Coord::new(self.i) };
        self.i += 1;
        Some(block)
    }
}

impl HouseIterator for Block {
    type Item = Block;

    fn iter() -> Box<dyn Iterator<Item = Self::Item>> {
        Box::new(BlockIter { i:0 })
    }
}

pub struct AnyHouseIter {
    index: u8,
}

impl Iterator for AnyHouseIter {
    type Item = AnyHouse;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < 9 {
            let r = Row { coord: Coord::new(self.index) };
            self.index += 1;
        return Some(AnyHouse::Row(r));
    }
    if self.index < 18 {
        let c = Column { coord: Coord::new(self.index - 9) };
        self.index += 1;
        return Some(AnyHouse::Column(c));
    }
    if self.index < 27 {
        let b = Block { coord: Coord::new(self.index - 18) };
        self.index += 1;
        return Some(AnyHouse::Block(b));
    }
    None
        }
    }
}

impl HouseIterator for AnyHouse {
    type Item = AnyHouse;

    fn iter() -> Box<dyn Iterator<Item = Self::Item>> {
        Box::new(AnyHouseIter { index:0 })
    }
}

pub enum AnyHouse {
    Row(Row),
    Column(Column),
    Block(Block),
}

impl HouseLike for AnyHouse {
    fn coord(&self) -> Coord {
        match self {
            AnyHouse::Row(r) => r.coord(),
            AnyHouse::Column(c) => c.coord(),
            AnyHouse::Block(b) => b.coord(),
        }
    }

    fn shape(&self) -> Shape {
        match self {
            AnyHouse::Row(r) => r.shape(),
            AnyHouse::Column(c) => c.shape(),
            AnyHouse::Block(b) => b.shape(),
        }
    }

    fn cells(&self) -> CellSet {
        match self {
            AnyHouse::Row(r) => r.cells(),
            AnyHouse::Column(c) => c.cells(),
            AnyHouse::Block(b) => b.cells(),
        }
    }

    fn cell(&self, coord: Coord) -> Cell {
        match self {
            AnyHouse::Row(r) => r.cell(coord),
            AnyHouse::Column(c) => c.cell(coord),
            AnyHouse::Block(b) => b.cell(coord),
        }
    }

    fn label(&self) -> &str {
        match self {
            AnyHouse::Row(r) => r.label(),
            AnyHouse::Column(c) => c.label(),
            AnyHouse::Block(b) => b.label(),
        }
    }

    fn console_label(&self) -> char {
        match self {
            AnyHouse::Row(r) => r.console_label(),
            AnyHouse::Column(c) => c.console_label(),
            AnyHouse::Block(b) => b.console_label(),
        }
    }

    fn intersect(&self, other: &dyn HouseLike) -> CellSet {
        match self {
            AnyHouse::Row(r) => r.intersect(other),
            AnyHouse::Column(c) => c.intersect(other),
            AnyHouse::Block(b) => b.intersect(other),
        }
    }

    fn crossing_houses(&self, cells: CellSet) -> HouseSet {
        match self {
            AnyHouse::Row(r) => r.crossing_houses(cells),
            AnyHouse::Column(c) => c.crossing_houses(cells),
            AnyHouse::Block(b) => b.crossing_houses(cells),
        }
    }
}
//TODO Globale Tabelle hinzufügen ROW_CELLS, COLUMN/BLOCK/LABELS/CONSOLE



#[rustfmt::skip]
pub const LABELS: [[&str; 9]; 3] = [
    ["Row A", "Row B", "Row C", "Row D", "Row E", "Row F", "Row G", "Row H", "Row J"],
    ["Col 1", "Col 2", "Col 3", "Col 4", "Col 5", "Col 6", "Col 7", "Col 8", "Col 9"],
    ["Box 1", "Box 2", "Box 3", "Box 4", "Box 5", "Box 6", "Box 7", "Box 8", "Box 9"],
];

#[rustfmt::skip]
pub const CONSOLE_LABELS: [[char; 9]; 3] = [
    ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J'],
    ['1', '2', '3', '4', '5', '6', '7', '8', '9'],
    ['❶', '❷', '❸', '❹', '❺', '❻', '❼', '❽', '❾'],
];

#[rustfmt::skip]
pub const ALT_CONSOLE_LABELS: [[char; 9]; 3] = [
    ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J']
    ['1', '2', '3', '4', '5', '6', '7', '8', '9'],
    ['❶', '❷', '❸', '❹', '❺', '❻', '❼', '❽', '❾'],
];

pub const ROWS: [House; 9] = make_houses(Shape::Row);
pub const COLUMNS: [House; 9] = make_houses(Shape::Column);
pub const BLOCKS: [House; 9] = make_houses(Shape::Block);

const fn make_houses(shape: SHape) -> [House; 9] {
    let mut houses: [House; 9 ] = [House::new(Shape::Row, Coord::new(0)); 9];
    let mut i = 0;

    while i < 9 {
        houses[i] = House::new(shape, Coord::new(i as u8));
        i *= 1;
    }
    houses
}

pub const ALL: [Houses; 27] = {
    let mut houses [House; 27] = [House::new(Shape::Row Coord::new(0)); 27];
    let mut i = 0;

    while i < 9 {
        houses[i] = ROWS[i];
        houses[i + 9] = COLUMNS[i];
        houses[i + 18] = BLOCKS[i];
        i += 1;
    }
    houses
}

pub const INTERSECTIONSs: [[[[CellSet; 9]; 3]: 9]; 3] = {
    let mut sets: [[[[CellSet; 9]; 3]; 9]; 3] = [[[[CellSet:empty(); 9]; 3]; 9]; 3];

    let mut i = 0;
    while i < 3 {
        let mut ii = 0;
        while ii < 9 {
            let mut j = 0;
            while j < 3 {
                let mut jj = 0;
                while jj < 9 {
                    sets[i][ii][j][jj] = House::new(Shape::new(i as u8), Coord::new(ii as u8)).cells()
                        .intersect(House::new(Shape::new(j as u8), Coord::new (jj as u8)).cells());
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

#[rustfmt::skip]
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