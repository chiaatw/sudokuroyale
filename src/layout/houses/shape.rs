use std::fmt
use once_cell::sync::Lazy;
use std::array;

#[derive(Clone, Copy, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Shape {
    #[default]
    Row,
    Column,
    Block,
}

#[derive(Copy, Clone)]
pub struct Coord(u8);

impl Coord {
    const fn new(value: u8) -> Self {
        Coord(value)
    }
    const fn u8(self) -> u8 {
        self.0 
    }
    const fn usize(self) -> usize {
        self.0 as usize
    }
}

#[derive(Copy, Clone, Debug)]
pub struct House {
    pub shape: Shape,
    pub coord: Coord,
}

impl House {
    pub const fn new(shape: Shape, coord: Coord) -> Self {
        House { shape, coord }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct HouseIter {
    shape: Shape,
    index: u8,
}

impl HouseIter {
    pub const fn new(shape: Shape) -> Self {
        HouseIter { shape, index: 0 }
    }
}

impl Iterator for HouseIter {
    type Item = House;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < 9 {
            let h = House::new(self.shape, Coord::new(self.index));
            self.index += 1;
            Some(h)
        } else {
            None
            }
        }
    }

#[derive(Copy, Clone)]
struct Cell(u8);

impl Cell {
    const fn new(value: u8) -> Self {
        Cell(value)
    }
}

//TODO CellSet stores no data
#[derive(Copy, Clone)]
pub struct CellSet;

impl CellSet {
    pub const fn empty() -> Self {
        CellSet
    }
    pub const fn of <const N: usize>(_: &[Cell; N]) -> Self {
        CellSet
    }
}

pub trait ShapeTrait{
    fn label(&self) -> &str;
    fn index(&self) -> usize;

    fn cells(&self, house: Coord) -> CellSet; //returns all cells of one house
    fn cell_at(&self, house: Coord, index: usize) -> Cell; //returns a single cell of a house    
    fn house(&self, coord: Coord) -> House; //Returns the House struct: Row/Column/Block + Coordinate

    fn is_row(&self) -> bool;
    fn is_column(&self) -> bool;
    fn is_block(&self) -> bool;

    fn house_iter(&self) -> HouseIter; //Returns an iterator over all houses of this shape

    fn iter() -> ShapeIter 
    where 
        Self: Sized; //Returns an iterator over all shape types
}

pub trait ShapeCells {
    fn cells(&self) -> [[Cell; 9]; 9];
    fn cell_sets(&self) -> [CellSet; 9];
}

impl ShapeCells for Shape {
    fn cells(&self) -> [[Cell; 9]; 9] {
        let mut cells: [[Cell; 9]; 9] = [[Cell::new(0); 9]; 9];
        for house in 0..9 {
            for coord in 0..9 {
                cells[house][coord] = match self {
                    Shape::Row => Cell::new(9 * house + coord),
                    Shape::Column => Cell::new(house + 9 * coord),
                    Shape::Block => Cell::new(
                        (house / 3) * 27
                        + (house % 3) * 3
                        + (coord / 3) * 9
                        + (coord % 3),
                    ),
                };
            }
        }
        cells
    }
    fn cell_sets(&self) -> [CellSet; 9] {
        let cells = self.cells();
        array::from_fn(|i| CellSet::of::<9>(&cells[i]))
    }
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

    fn cells(&self, house: Coord) -> CellSet {
        CELL_SETS[self.index()][house.usize()]
    }
    fn cell_at(&self, house: Coord, index: usize) -> Cell {
        CELLS[self.index()][house.usize()][index]
    }
    fn house(&self, coord: Coord) -> House {
        House::new(*self, coord)
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
    fn iter() -> ShapeIter 
    where
        Self: Sized,
    {
        ShapeIter::new()
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
        pub const fn new(index: u8) -> Self {
            match index {
                0 => Shape::Row,
                1 => Shape::Column,
                2 => Shape::Block,
                _ => panic!("Invalid Shape index"),
            }
        }
    }

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

impl fmt::Debug for Coord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Coord").field(&self.0).finish()
    }
}

impl fmt::Debug for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Cell").field(&self.0).finish()
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

pub static CELLS: Lazy<[[[Cell; 9]; 9]; 3]> = Lazy::new(|| [
    Shape::Row.cells(),
    Shape::Column.cells(),
    Shape::Block.cells(),
]);

pub static CELL_SETS: Lazy<[[CellSet; 9]; 3]> = Lazy::new(|| [
    Shape::Row.cell_sets(),
    Shape::Column.cell_sets(),
    Shape::Block.cell_sets(),
]);













