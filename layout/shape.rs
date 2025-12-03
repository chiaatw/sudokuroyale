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
        pub fn iter() -> ShapeIter {
        ShapeIter::new()
        }
    }
}

