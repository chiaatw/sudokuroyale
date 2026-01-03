//! This module defines the individual pieces that make up a Sudoku [Board] which holds 81 cells arranged in a 9x9 grid
//! 
//! Each [Cell] contains a single [Value] which is [Known] if given as a starting clue
//! or later solved to a digit (1-9) Until then it is considered unknown
//! 
//! The board tracks sets of cells using [CellSet]s These sets represent cells that are
//! given, known, candidates for each value have a specific number of candidates remaining
//! or have been solved/given to each known value
//! A [CellSet] is an 81 bit bitset stored in a 128 bit integer for efficiency supporting standard
//! set operations used by the board and solving strategies
//! 
//! [Rectangle] represents four cells and is used in detecting deadly and avoiding rectangles in the Unique Rectangle strategy
//! 
//! Each unknown cells remaining candidates are tracked with [KnownSet]s
//! This is a 9 bit bitset (one bit per value) and offers a similar interface to [CellSet]
//! 
//! Cells are grouped into [House]s of 9 cells define by a [Shape] (row, column or block)
//! Blocks are the standard 3x3 squares note that box is a reserved word in Rust
//! There are 9 houses of each shape
//! 
//! Many strategies iterate over [HouseSet]s to work with rows, columns or blocks
//! These use [CoordSet]s along with a [Shape] to identify which houses are included
//! [HouseSet] provides a similar interface to the other sets
//! 
//! [Coord] tracks the position of a cell within each of its houses and the position of each house on the board
//! Coordinates range from 1 to 9 in all cases
//! 
//! [HouseSet] relies on [CoordSet] to track which houses it contains
//! [CoordSet] is a 9 bit bitset with one bit per coordinate with an interface similar to the other sets
 
pub use cells::{
    Cell, CellIteratorUnion, CellSet, CellSetIteratorIntersection, CellSetIteratorUnion, Rectangle,
};
pub use houses::{
    Coord, CoordSet, House, HouseIteratorUnion, HouseSet, HouseSetIteratorIntersection,
    HouseSetIteratorUnion, Shape,
};
pub use values::{
    Known, KnownIteratorUnion, KnownSet, KnownSetIteratorIntersection, KnownSetIteratorUnion, Value,
};

pub mod cells;
pub mod houses;
pub mod values;