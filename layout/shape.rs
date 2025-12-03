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