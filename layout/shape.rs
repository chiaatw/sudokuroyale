use std::fmt

#[derive(Clone, Copy, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Form {
    #[default]
    Row,
    Column,
    Block,
}

pub trait ShapeTrait{
    fn label(&self) -> &str;
    fn index(&self) -> usize;
    fn houses(&self, house: Coord) -> CellSet;
    fn house(&self, house: Coord, index: usize) -> House;
}

impl ShapeTrait for Form {
    fn label(&self) -> &str {
        match self {
            Form::Row => "Row",
            Form::Column => "Col",
            Form::Block => "Box",
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