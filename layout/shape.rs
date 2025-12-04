use std::fmt

#[derive(Clone, Copy, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Shape {
    #[default]
    Row,
    Column,
    Block,
};

pub trait ShapeTrait{
    fn label(&self) -> &str;
    fn index(&self) -> usize;
    fn houses(&self, house: Coord) -> CellSet;
    fn house(&self, house: Coord, index: usize) -> House;
    fn is_row(&self) -> bool;
    fn is_column(&self) -> bool;
    fn is_block(&self) -> bool;
    fn house_iter(&self) -> HouseIter;
    fn iter() -> ShapeIter;
    fn cells(&self, house: Coord) -> CellSet;
}

impl ShapeTrait for Shape {
    fn label(&self) -> &str {
        match self {
            Shape::Row => "Row",
            Shape::Column => "Col",
            Shape::Block => "Box",
        }
    }

    fn index(&self) -> usize {
        *self as usize
    }

    fn houses(&self, house: Coord) -> CellSet {
        CELL_SETS[self.index()][house.usize()]
    }

    fn house(&self, house: Coord, index: usize) -> House {
        House::new(*self, house)
    }
    fn is_row(&self) -> bool {
        matches!(self, Shape::Row)
    }
    fn is_column(&self) -> bool {
        matches!(self, Shape::Column)
    }
    fn is_block(&self) -> bool {
        matches!(self, Shape::Block)
    }
    fn house_iter(&self) -> HouseIter {
        HouseIter::new(*self)
    }
    fn iter() -> ShapeIter {
        ShapeIter::new()
    }
    fn cells(&self, house: Coord) -> CellSet {
        CELL_SETS[self.index()][house.usize()]
    }
}

    impl From<char> for Shape {
        fn from(c: char) -> Self {
            match c {
                'R' => Shape::Row,
                'C' => Shape::Column,
                'B' => Shape::Block,
                _ => panic!("Invalid character for Shape"),
            }
        }
    }

    impl Shape {
        pub const fn house_const(&self, house: Coord) -> House {
            House::new(*self, house)
        }
    }

// Change to Trait Implementation
impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl fmt::Debug for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Shape::{}", self.label())
    }
}

pub struct ShapeIter(u8);

impl ShapeIter {
    pub const fn new() -> Self {
        Self(0)
    }
}

impl Iterator for ShapeIter {
    type Item = Shape;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 < 3 {
            let shape = Shape::new(self.0);
            self.0 += 1;
            Some(shape)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for ShapeIter {
    fn len(&self) -> usize {
        3 - self.0 as usize
    }
}

const CELLS: [[[Cell; 9]; 9]; 3] = {
    let mut cells: [[[Cell; 9]; 9]; 3] = [[[Cell::new(0); 9]; 9]; 3];

    const fn shape_cells(shape: Shape) -> [[Cell; 9]; 9] {
        let mut cells: [[Cell; 9]; 9] = [[Cell::new(0); 9]; 9];
        let mut house = 0;

        while house < 9 {
            cells[house] = house_cells(shape, Coord::new(house as u8));
            house += 1;
        }
        cells
    }

    const fn house_cells(shape: Shape, house: Coord) -> [Cell; 9] {
        let mut cells: [Cell; 9] = [Cell::new(0); 9];
        let mut coord = 0;

        while coord < 9 {
            cells[coord] = house_cell(shape, house, Coord::new(coord as u8));
            coord += 1;
        }
        cells
    }

    const fn house_cell(shape: Shape, house: Coord, coord: Coord) -> Cell {
        match shape {
            Shape::Row => Cell::new(9 * house.u8() + coord.u8()),
            Shape::Column => Cell::new(house.u8() + 9 * coord.u8()),
            Shape::Block => Cell::new(
                (house.u8() / 3) * 27
                    + (house.u8() % 3) * 3
                    + (coord.u8() / 3) * 9
                    + (coord.u8() % 3),
            ),
        }
    }

    cells[Shape::Row.usize()] = shape_cells(Shape::Row);
    cells[Shape::Column.usize()] = shape_cells(Shape::Column);
    cells[Shape::Block.usize()] = shape_cells(Shape::Block);
    cells
};

const CELL_SETS: [[CellSet; 9]; 3] = {
    let mut sets: [[CellSet; 9]; 3] = [[CellSet::empty(); 9]; 3];

    const fn cell_sets(shape: Shape) -> [CellSet; 9] {
        let mut cell_sets: [CellSet; 9] = [CellSet::empty(); 9];
        let mut house = 0;

        while house < 9 {
            cell_sets[house] = CellSet::of::<9>(&CELLS[shape.usize()][house]);
            house += 1;
        }
        cell_sets
    }

    sets[Shape::Row.usize()] = cell_sets(Shape::Row);
    sets[Shape::Column.usize()] = cell_sets(Shape::Column);
    sets[Shape::Block.usize()] = cell_sets(Shape::Block);
    sets
};