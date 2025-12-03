use std::fmt

#derive(Clone, Copy, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
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
}