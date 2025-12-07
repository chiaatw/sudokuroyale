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

pub trait HouseIterator {
    type Item: HouseLike;

    fn iter() -> Box<dyn Iterator<Item = Self::Item>>;
}