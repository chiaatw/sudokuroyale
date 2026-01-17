use std::fmt;
use once_cell::sync::Lazy;
use std::array;

use crate::layout::{Cell, CellSet, Coord};
use crate::layout::houses::house::House;

#[derive(Clone, Copy, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Shape {
    #[default]
    Row,
    Column,
    Block,
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
            let house_u8 = house as u8;
            for coord in 0..9 {
                let coord_u8 = coord as u8;
                cells[house][coord] = match self {
                    Shape::Row => Cell::new(9 * house_u8 + coord_u8),
                    Shape::Column => Cell::new(house_u8 + 9 * coord_u8),
                    Shape::Block => Cell::new(
                        (house_u8 / 3) * 27
                        + (house_u8 % 3) * 3
                        + (coord_u8 / 3) * 9
                        + (coord_u8 % 3),
                    ),
                };
            }
        }
        cells
    }
    fn cell_sets(&self) -> [CellSet; 9] {
        let cells = ShapeCells::cells(self);
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
    ShapeCells::cells(&Shape::Row),
    ShapeCells::cells(&Shape::Column),
    ShapeCells::cells(&Shape::Block),
]);

pub static CELL_SETS: Lazy<[[CellSet; 9]; 3]> = Lazy::new(|| [
    Shape::Row.cell_sets(),
    Shape::Column.cell_sets(),
    Shape::Block.cell_sets(),
]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_label_and_index() {
        assert_eq!(Shape::Row.label(), "Row");
        assert_eq!(Shape::Column.label(), "Col");
        assert_eq!(Shape::Block.label(), "Box");

        assert_eq!(Shape::Row.index(), 0);
        assert_eq!(Shape::Column.index(), 1);
        assert_eq!(Shape::Block.index(), 2);
    }

    #[test]
    fn shape_from_char() {
        assert_eq!(Shape::from('R'), Shape::Row);
        assert_eq!(Shape::from('C'), Shape::Column);
        assert_eq!(Shape::from('B'), Shape::Block);
    }

    #[test]
    #[should_panic]
    fn shape_from_char_invalid() {
        let _ = Shape::from('X');
    }

    #[test]
    fn shape_new_index() {
        assert_eq!(Shape::new(0), Shape::Row);
        assert_eq!(Shape::new(1), Shape::Column);
        assert_eq!(Shape::new(2), Shape::Block);
    }

    #[test]
    #[should_panic]
    fn shape_new_index_invalid() {
        let _ = Shape::new(3);
    }

    #[test]
    fn cells_row_column_block() {
        let row_cells = Shape::Row.cells();
        let col_cells = Shape::Column.cells();
        let block_cells = Shape::Block.cells();

        // Row: first row should be 0..8
        for i in 0..9 {
            assert_eq!(row_cells[0][i].0, i as u8);
        }
        // Column: first column should be multiples of 9
        for i in 0..9 {
            assert_eq!(col_cells[0][i].0, (i * 9) as u8);
        }
        // Block: first block first row
        assert_eq!(block_cells[0][0].0, 0);
        assert_eq!(block_cells[0][1].0, 1);
        assert_eq!(block_cells[0][2].0, 2);
    }

    #[test]
    fn cell_sets_length() {
        let row_sets = Shape::Row.cell_sets();
        assert_eq!(row_sets.len(), 9);
        let col_sets = Shape::Column.cell_sets();
        assert_eq!(col_sets.len(), 9);
        let block_sets = Shape::Block.cell_sets();
        assert_eq!(block_sets.len(), 9);
    }

    #[test]
    fn house_and_house_iter() {
        let house = Shape::Row.house(Coord::new(3));
        assert_eq!(house.coord.0, 3);
        assert!(house.shape.is_row());

        let mut iter = Shape::Row.house_iter();
        for i in 0..9 {
            let h = iter.next().unwrap();
            assert_eq!(h.coord.0, i);
            assert!(h.shape.is_row());
        }
        assert!(iter.next().is_none());
    }

    #[test]
    fn shape_iter() {
        let mut iter = Shape::iter();
        assert_eq!(iter.next(), Some(Shape::Row));
        assert_eq!(iter.next(), Some(Shape::Column));
        assert_eq!(iter.next(), Some(Shape::Block));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn cell_at_correct() {
        // Row 1, index 5 → Cell should be 9*1 + 5 = 14
        let c = Shape::Row.cell_at(Coord::new(1), 5);
        assert_eq!(c.0, 14);
        // Column 2, index 3 → Cell should be 2 + 9*3 = 29
        let c = Shape::Column.cell_at(Coord::new(2), 3);
        assert_eq!(c.0, 29);
    }

    #[test]
    fn house_type_checks() {
        let row = Shape::Row;
        let col = Shape::Column;
        let block = Shape::Block;

        assert!(row.is_row());
        assert!(!row.is_column());
        assert!(!row.is_block());

        assert!(col.is_column());
        assert!(!col.is_row());
        assert!(!col.is_block());

        assert!(block.is_block());
        assert!(!block.is_row());
        assert!(!block.is_column());
    }

    #[test]
    fn display_debug() {
        let s = format!("{}", Shape::Row);
        assert_eq!(s, "Row");
        let d = format!("{:?}", Shape::Block);
        assert_eq!(d, "Shape::Box");
    }

    #[test]
    fn coord_and_cell_debug() {
        let coord = Coord::new(7);
        let c = Cell::new(42);
        let coord_dbg = format!("{:?}", coord);
        let cell_dbg = format!("{:?}", c);
        assert!(coord_dbg.contains("7"));
        assert!(cell_dbg.contains("42"));
    }
}













