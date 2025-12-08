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
    fn shape(&self) -> Shape { Shape::Block}

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

    pub const fn is_top(&self) -> bool {
        self.is_row() && self.coord.u8() == 0
    }

    pub const fn is_bottom(&self) -> bool {
        self.is_row() && self.coord.u8() == 8
    }

    pub const fn is_left(&self) -> bool {
        self.is_column() && self.coord.u8() == 0
    }

    pub const fn is_right(&self) -> bool {
        self.is_column() && self.coord.u8() == 8
    }

    pub const fn is_block_top(&self) -> bool {
        self.is_row() && self.coord.u8() % 3 == 0
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


impl HouseIterator for AnyHouse {
    type Item = AnyHouse;

    fn iter() -> Box<dyn Iterator<Item = Self::Item>> {
        Box::new(AnyHouseIter { index:0 })
    }
}

    pub fn iter() -> HousesIter {
        HousesIter::new()
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
        let coord = chars.next().unwarp() as u8 - b'1';
        if coord > 9 {
            panic!("Invalid house coord: \"{}\"; must be 1-9", label);
        }

        Self {
            shape: Shape::from(shape),
            coord: Coord::from(coord),
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt:Result {
        write!(f, "{}", self.label())
    }
}

    pub const fn houses(&self, shape: Shape) -> HouseSet {
        match shape {
            Shape::Row => self.rows(),
            Shape::Column => self.columns(),
            Shape::Block => self.blocks(),
        }
    }

    pub const fn rows(&self) -> HouseSet {
        match self.shape {
            Shape::Row => ROW_ROWS[self.coord.usize()],
            Shape::Column => COLUMN_ROWS[self.coord.usize()],
            Shape::Block => BLOCK_ROWS[self.coord.usize()],
        }
    }

    pub const fn columns(&self) -> HouseSet {
        match self.shape {
            Shape::Row => ROW_COLUMNS[self.coord.usize()],
            Shape::Column => COLUMN_COLUMNS[self.coord.usize()],
            Shape::Block => BLOCK_COLUMNS[self.coord.usize()],
        }
    }

    pub const fn blocks(&self) -> HouseSet {
        match self.shape {
            Shape::Row => ROW_BLOCKS[self.coord.usize()],
            Shape::Column => COLUMN_BLOCKS[self.coord.usize()],
            Shape::Block => BLOCK_BLOCKS[self.coord.usize()],
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



const fn make_houses(shape: Shape) -> [House; 9] {
    let mut houses: [House; 9 ] = [House::new(Shape::Row, Coord::new(0)); 9];
    let mut i = 0;

    while i < 9 {
        houses[i] = House::new(shape, Coord::new(i as u8));
        i += 1;
    }
    houses
}

pub const INTERSECTIONS: [[[[CellSet; 9]; 9]; 3]; 3] = {
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

