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
    fn shape(&sef) -> Shape { Shape::Column }

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

    fn crossing houses(&self, cells: CellSet) -> HouseSet {
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
    i = u8,
}

impl Iterator for ColumnIter {

}

impl HouseIterator for Column {

}

pub struct BlockIter {
    i = u8,
}

impl Iterator for BlockIter {

}

impl HouseIterator for Block {

}

pub enum AnyHouse {
    Row(Row),
    Column(Column),
    Block(Block),
}

impl HouseLike for AnyHouse {

}
//TODO Globale Tabelle hinzufügen ROW_CELLS, COLUMN/BLOCK/LABELS/CONSOLE