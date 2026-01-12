pub mod bit;
pub mod cell_set;
pub mod cell;
pub mod label;
pub mod rectangle;

pub mod grid;

pub mod coord_set;
pub mod coord;
pub mod house_set;
pub mod house;
pub mod shape;

pub mod known_set;
pub mod known;
pub mod value;

pub use cell::{Cell, CellIter};
pub use bit::Bit;
pub use cell_set::CellSet;

pub use label::{
    CellIndex,
    index_from_label,
    try_index_from_label,
    label_from_index,
};

pub use rectangle::{Rectangle, RectangleIter};

pub use grid::Grid;

pub use coord_set::{CoordSet, Iter};
pub use coord_set::{CoordIteratorUnion, CoordSetIteratorUnion, CoordSetIteratorIntersection};
pub use coord::Coord;
pub use house_set::HouseSet;
pub use house_set::HouseSetLike;
pub use house::{House, Row, Column, Block, HouseLike};

pub use shape::{Shape, ShapeIter, ShapeTrait, ShapeCells, CELLS, CELL_SETS};

pub use known::{Known, KnownLike, KnownIter};
pub use known_set::{KnownSet, KnownSetLike, KnownSetIter, KnownIteratorUnion, KnownSetIteratorUnion, KnownSetIteratorIntersection};

pub use value::{Value, ValueLike};