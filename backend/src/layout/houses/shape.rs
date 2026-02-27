use once_cell::sync::Lazy;
use std::array;
use std::fmt;

use crate::layout::houses::house::House;
use crate::layout::{Cell, CellSet, Coord};

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

pub trait ShapeTrait {
    fn label(&self) -> &str;
    fn index(&self) -> usize;

    fn cells(&self, house: Coord) -> CellSet; //Gibt alle Zellen eines Hauses zurück
    fn cell_at(&self, house: Coord, index: usize) -> Cell; //gibt eine einzelne Zelle eines Hauses zurück
    fn house(&self, coord: Coord) -> House; // Gibt die House-Struktur zurück: Zeile/Spalte/Block + Koordinate

    fn is_row(&self) -> bool;
    fn is_column(&self) -> bool;
    fn is_block(&self) -> bool;

    fn house_iter(&self) -> HouseIter; // Gibt einen Iterator über alle Häuser dieser Form zurück

    fn iter() -> ShapeIter
    where
        Self: Sized; //Gibt einen Iterator über alle Formtypen zurück
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
    pub const fn usize(self) -> usize {
        self as usize
    }

    pub const fn is_row(self) -> bool {
        matches!(self, Shape::Row)
    }
    pub const fn is_column(self) -> bool {
        matches!(self, Shape::Column)
    }
    pub const fn is_block(self) -> bool {
        matches!(self, Shape::Block)
    }

    pub fn cells(self, house: Coord) -> CellSet {
        <Shape as ShapeTrait>::cells(&self, house)
    }

    pub fn cell(self, house: Coord, coord: Coord) -> Cell {
        <Shape as ShapeTrait>::cell_at(&self, house, coord.usize())
    }

    pub fn house_iter(self) -> HouseIter {
        <Shape as ShapeTrait>::house_iter(&self)
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

pub static CELLS: Lazy<[[[Cell; 9]; 9]; 3]> = Lazy::new(|| {
    [
        ShapeCells::cells(&Shape::Row),
        ShapeCells::cells(&Shape::Column),
        ShapeCells::cells(&Shape::Block),
    ]
});

pub static CELL_SETS: Lazy<[[CellSet; 9]; 3]> = Lazy::new(|| {
    [
        Shape::Row.cell_sets(),
        Shape::Column.cell_sets(),
        Shape::Block.cell_sets(),
    ]
});

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
    fn cell_at_correct() {
        // Zeile 1, Index 5 -> 9*1 + 5 = 14
        let c = Shape::Row.cell(Coord::new(1), Coord::new(5));
        assert_eq!(c.index(), 14);

        // Spalte 2, Index 3 -> 2 + 9 * 3 = 29
        let c = Shape::Column.cell(Coord::new(2), Coord::new(3));
        assert_eq!(c.index(), 29);
    }

    #[test]
    fn cells_row_column_block_first_house() {
        // Zeile 0: Zellen 0..8
        for i in 0..9 {
            let cell = Shape::Row.cell(Coord::new(0), Coord::new(i));
            assert_eq!(cell.index(), i as u8);
        }

        // Spalte 0: Zellen 0,9,18,...
        for i in 0..9 {
            let cell = Shape::Column.cell(Coord::new(0), Coord::new(i));
            assert_eq!(cell.index(), (i * 9) as u8);
        }

        // Block 0: erste zeile des ersten Blocks
        assert_eq!(Shape::Block.cell(Coord::new(0), Coord::new(0)).index(), 0);
        assert_eq!(Shape::Block.cell(Coord::new(0), Coord::new(1)).index(), 1);
        assert_eq!(Shape::Block.cell(Coord::new(0), Coord::new(2)).index(), 2);
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
        assert_eq!(house.coord().index(), 3);
        assert!(house.shape().is_row());

        let mut iter = Shape::Row.house_iter();
        for i in 0..9 {
            let h = iter.next().unwrap();
            assert_eq!(h.coord().index(), i);
            assert!(h.shape().is_row());
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
    fn display_and_debug() {
        assert_eq!(format!("{}", Shape::Row), "Row");
        assert_eq!(format!("{:?}", Shape::Block), "Shape::Box");
    }

    #[test]
    fn coord_and_cell_debug() {
        let coord = Coord::new(7);
        let cell = Cell::new(42);

        let coord_dbg = format!("{:?}", coord);
        let cell_dbg = format!("{:?}", cell);

        assert!(coord_dbg.contains("7"));
        assert!(cell_dbg.contains("42"));
    }
}
