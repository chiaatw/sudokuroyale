pub trait HouseLike {
    fn coord(&self) -> Coord;
    fn shape(&self) -> Shape;

    fn cells(&self) -> CellSet;
    fn cell(&self, coord: Coord) -> Cell;

    fn has(&self, cell: Cell) -> bool {
        self.cells.has(cell)
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

pub struct {
    coord: Coord,
}