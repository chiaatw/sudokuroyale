use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter::FusedIterator;

use crate::layout::houses::house_set::HouseSetLike;
use crate::layout::{Coord, House};

use super::{Cell, CellSet};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Rectangle {
    Data {
        top_left: Cell,
        top_right: Cell,
        bottom_left: Cell,
        bottom_right: Cell,
        cells: CellSet,
        block_count: usize,
    },
}

impl Rectangle {
    #[inline]
    pub const fn cells(self) -> CellSet {
        match self {
            Rectangle::Data { cells, .. } => cells,
        }
    }

    #[inline]
    pub const fn block_count(self) -> usize {
        match self {
            Rectangle::Data { block_count, .. } => block_count,
        }
    }
    pub fn iter() -> RectangleIter {
        RectangleIter::default()
    }

    pub const fn new(top_left: Cell, bottom_right: Cell) -> Rectangle {
        let top_right = Cell::from_coords(top_left.row_coord(), bottom_right.column_coord());
        let bottom_left = Cell::from_coords(bottom_right.row_coord(), top_left.column_coord());
        let cells = CellSet::of(&[top_left, top_right, bottom_left, bottom_right]);

        let tl_block = top_left.block_coord().usize();
        let br_block = bottom_right.block_coord().usize();
        let block_count = if tl_block == br_block {
            1
        } else if tl_block % 3 == br_block % 3 || tl_block / 3 == br_block / 3 {
            2
        } else {
            4
        };

        Rectangle::Data {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
            cells,
            block_count,
        }
    }

    pub fn from(c1: Cell, c2: Cell, c3: Cell, c4: Cell) -> Rectangle {
        Rectangle::new(c1.min(c2).min(c3).min(c4), c1.max(c2).max(c3).max(c4))
    }

    pub fn with_origin(self, origin: Cell) -> Rectangle {
        match self {
            Rectangle::Data {
                top_left,
                top_right,
                bottom_left,
                bottom_right,
                cells,
                block_count,
            } => {
                if origin == bottom_right {
                    Rectangle::Data {
                        top_left: bottom_right,
                        top_right: bottom_left,
                        bottom_left: top_right,
                        bottom_right: top_left,
                        cells,
                        block_count,
                    }
                } else if origin == top_right {
                    Rectangle::Data {
                        top_left: top_right,
                        top_right: top_left,
                        bottom_left: bottom_right,
                        bottom_right: bottom_left,
                        cells,
                        block_count,
                    }
                } else if origin == bottom_left {
                    Rectangle::Data {
                        top_left: bottom_left,
                        top_right: bottom_right,
                        bottom_left: top_left,
                        bottom_right: top_right,
                        cells,
                        block_count,
                    }
                } else {
                    self
                }
            }
        }
    }

    #[inline]
    fn tl_br(&self) -> (Cell, Cell) {
        match *self {
            Rectangle::Data {
                top_left,
                bottom_right,
                ..
            } => (top_left, bottom_right),
        }
    }
    #[inline]
    pub const fn top_right(&self) -> Cell {
        match *self {
            Rectangle::Data { top_right, .. } => top_right,
        }
    }

    #[inline]
    pub const fn top_left(&self) -> Cell {
        match *self {
            Rectangle::Data { top_left, .. } => top_left,
        }
    }

    #[inline]
    pub const fn bottom_left(&self) -> Cell {
        match *self {
            Rectangle::Data { bottom_left, .. } => bottom_left,
        }
    }

    #[inline]
    pub const fn bottom_right(&self) -> Cell {
        match *self {
            Rectangle::Data { bottom_right, .. } => bottom_right,
        }
    }
}

impl Hash for Rectangle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let (tl, br) = self.tl_br();
        tl.hash(state);
        br.hash(state);
    }
}

impl TryFrom<Vec<Cell>> for Rectangle {
    type Error = ();

    fn try_from(cells: Vec<Cell>) -> Result<Rectangle, ()> {
        Rectangle::try_from(CellSet::from_iter(cells))
    }
}

impl TryFrom<CellSet> for Rectangle {
    type Error = ();

    fn try_from(cells: CellSet) -> Result<Rectangle, ()> {
        if cells.len() < 2 || 4 < cells.len() {
            return Err(());
        }

        let rows = cells.rows();
        let columns = cells.columns();

        if rows.len() != 2 || columns.len() != 2 {
            return Err(());
        }

        let (top, bottom) = rows.as_pair().unwrap();
        let (left, right) = columns.as_pair().unwrap();

        Ok(Rectangle::new(
            Cell::from_coords(top.coord(), left.coord()),
            Cell::from_coords(bottom.coord(), right.coord()),
        ))
    }
}

impl fmt::Debug for Rectangle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (tl, br) = self.tl_br();
        write!(f, "Rectangle({} {})", tl, br)
    }
}

impl fmt::Display for Rectangle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (tl, br) = self.tl_br();
        write!(
            f,
            "R{}{}C{}{}",
            tl.row().coord(),
            br.row().coord(),
            tl.column().coord(),
            br.column().coord(),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RectangleIter {
    State {
        horiz_vert: usize,
        block: usize,
        cell: usize,
    },
    Done,
}

impl Default for RectangleIter {
    fn default() -> Self {
        RectangleIter::State {
            horiz_vert: 0,
            block: 0,
            cell: 0,
        }
    }
}

impl Iterator for RectangleIter {
    type Item = Rectangle;

    fn next(&mut self) -> Option<Rectangle> {
        match self {
            RectangleIter::Done => None,
            RectangleIter::State {
                horiz_vert,
                block,
                cell,
            } => {
                if *horiz_vert == 2 {
                    *self = RectangleIter::Done;
                    return None;
                }

                let (from, to) = BLOCKS[*horiz_vert][*block];
                let ((tl, _), (_, br)) = CELL_COORDS[*horiz_vert][*cell];
                let rect = Rectangle::new(from.cell(tl), to.cell(br));

                *cell += 1;
                if *cell == 27 {
                    *cell = 0;
                    *block += 1;
                    if *block == 9 {
                        *block = 0;
                        *horiz_vert += 1;
                    }
                }

                Some(rect)
            }
        }
    }
}

impl FusedIterator for RectangleIter {}

type IndexPair = (u8, u8);
type CoordPair = (Coord, Coord);

const BLOCKS: [[(House, House); 9]; 2] = {
    const BLOCKS: [[IndexPair; 9]; 2] = [
        [
            (0, 1),
            (0, 2),
            (1, 2),
            (3, 4),
            (3, 5),
            (4, 5),
            (6, 7),
            (6, 8),
            (7, 8),
        ],
        [
            (0, 3),
            (0, 6),
            (3, 6),
            (1, 4),
            (1, 7),
            (4, 7),
            (2, 5),
            (2, 8),
            (5, 8),
        ],
    ];

    const DEFAULT: House = House::block(Coord::new(0));
    let mut blocks = [[(DEFAULT, DEFAULT); 9]; 2];
    let mut hv = 0;

    while hv < 2 {
        let mut i = 0;
        while i < 9 {
            let (f, t) = BLOCKS[hv][i];
            blocks[hv][i] = (House::block(Coord::new(f)), House::block(Coord::new(t)));
            i += 1;
        }
        hv += 1;
    }
    blocks
};

const CELL_COORDS: [[(CoordPair, CoordPair); 27]; 2] = {
    const COORDS: [[(IndexPair, IndexPair); 27]; 2] = [
        [
            ((0, 3), (0, 3)),
            ((0, 3), (1, 4)),
            ((0, 3), (2, 5)),
            ((0, 6), (0, 6)),
            ((0, 6), (1, 7)),
            ((0, 6), (2, 8)),
            ((3, 6), (3, 6)),
            ((3, 6), (4, 7)),
            ((3, 6), (5, 8)),
            ((1, 4), (0, 3)),
            ((1, 4), (1, 4)),
            ((1, 4), (2, 5)),
            ((1, 7), (0, 6)),
            ((1, 7), (1, 7)),
            ((1, 7), (2, 8)),
            ((4, 7), (3, 6)),
            ((4, 7), (4, 7)),
            ((4, 7), (5, 8)),
            ((2, 5), (0, 3)),
            ((2, 5), (1, 4)),
            ((2, 5), (2, 5)),
            ((2, 8), (0, 6)),
            ((2, 8), (1, 7)),
            ((2, 8), (2, 8)),
            ((5, 8), (3, 6)),
            ((5, 8), (4, 7)),
            ((5, 8), (5, 8)),
        ],
        [
            ((0, 1), (0, 1)),
            ((0, 1), (3, 4)),
            ((0, 1), (6, 7)),
            ((0, 2), (0, 2)),
            ((0, 2), (3, 5)),
            ((0, 2), (6, 8)),
            ((1, 2), (1, 2)),
            ((1, 2), (4, 5)),
            ((1, 2), (7, 8)),
            ((3, 4), (0, 1)),
            ((3, 4), (3, 4)),
            ((3, 4), (6, 7)),
            ((3, 5), (0, 2)),
            ((3, 5), (3, 5)),
            ((3, 5), (6, 8)),
            ((4, 5), (1, 2)),
            ((4, 5), (4, 5)),
            ((4, 5), (7, 8)),
            ((6, 7), (0, 1)),
            ((6, 7), (3, 4)),
            ((6, 7), (6, 7)),
            ((6, 8), (0, 2)),
            ((6, 8), (3, 5)),
            ((6, 8), (6, 8)),
            ((7, 8), (1, 2)),
            ((7, 8), (4, 5)),
            ((7, 8), (7, 8)),
        ],
    ];

    const D: Coord = Coord::new(0);
    let mut out = [[((D, D), (D, D)); 27]; 2];
    let mut hv = 0;

    while hv < 2 {
        let mut i = 0;
        while i < 27 {
            let ((a, b), (c, d)) = COORDS[hv][i];
            out[hv][i] = (
                (Coord::new(a), Coord::new(b)),
                (Coord::new(c), Coord::new(d)),
            );
            i += 1;
        }
        hv += 1;
    }
    out
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rectangle_new_basic() {
        let tl = Cell::from_str("A1");
        let br = Cell::from_str("B2");
        let rect = Rectangle::new(tl, br);

        assert_eq!(rect.tl_br().0, tl);
        assert_eq!(rect.tl_br().1, br);

        // A1-B2 liegt innerhalb eines Blocks
        assert_eq!(rect.block_count(), 1);
    }

    #[test]
    fn test_rectangle_new_block_count() {
        // Rechteck innerhalb eines Blocks
        let rect1 = Rectangle::new(Cell::from_str("A1"), Cell::from_str("B2"));
        assert_eq!(rect1.block_count(), 1);

        // Rechteck über 2 Blöcke horizontal
        let rect2 = Rectangle::new(Cell::from_str("A1"), Cell::from_str("A5"));
        assert_eq!(rect2.block_count(), 2);

        // Rechteck über 4 Blöcke
        let rect3 = Rectangle::new(Cell::from_str("A1"), Cell::from_str("E5"));
        assert_eq!(rect3.block_count(), 4);
    }

    #[test]
    fn test_rectangle_from_cells() {
        let c1 = Cell::from_str("A1");
        let c2 = Cell::from_str("A2");
        let c3 = Cell::from_str("B1");
        let c4 = Cell::from_str("B2");

        let rect = Rectangle::from(c1, c2, c3, c4);
        let (tl, br) = rect.tl_br();
        assert_eq!(tl, Cell::from_str("A1"));
        assert_eq!(br, Cell::from_str("B2"));
    }

    #[test]
    fn test_rectangle_with_origin() {
        let rect = Rectangle::new(Cell::from_str("A1"), Cell::from_str("B2"));
        let r1 = rect.with_origin(Cell::from_str("B2"));
        let r2 = rect.with_origin(Cell::from_str("A1"));

        assert_eq!(r2.tl_br(), rect.tl_br());
        assert_ne!(r1.tl_br(), rect.tl_br());
    }

    #[test]
    fn test_rectangle_try_from_vec_and_cells() {
        let cells = vec![
            Cell::from_str("A1"),
            Cell::from_str("A2"),
            Cell::from_str("B1"),
            Cell::from_str("B2"),
        ];

        let rect_from_vec = Rectangle::try_from(cells.clone()).unwrap();
        let rect_from_set = Rectangle::try_from(CellSet::from_iter(cells.clone())).unwrap();

        assert_eq!(rect_from_vec.tl_br(), rect_from_set.tl_br());

        // Fehlerfälle
        let too_few = vec![Cell::from_str("A1")];
        assert!(Rectangle::try_from(too_few).is_err());

        let too_many = vec![
            Cell::from_str("A1"),
            Cell::from_str("A2"),
            Cell::from_str("B1"),
            Cell::from_str("B2"),
            Cell::from_str("C3"),
        ];
        assert!(Rectangle::try_from(too_many).is_err());
    }

    #[test]
    fn test_rectangle_display_and_debug() {
        let rect = Rectangle::new(Cell::from_str("A1"), Cell::from_str("B2"));
        let debug_str = format!("{:?}", rect);
        let display_str = format!("{}", rect);

        assert!(debug_str.contains("Rectangle(A1 B2)"));
        assert!(display_str.starts_with("R"));
        assert!(display_str.contains("C"));
    }

    #[test]
    fn test_rectangle_iter() {
        let mut iter = Rectangle::iter();
        let mut count = 0;
        while let Some(rect) = iter.next() {
            let (tl, br) = rect.tl_br();
            // Alle Rechtecke müssen tl <= br sein
            assert!(tl.index() <= br.index());
            count += 1;
        }
        assert!(count > 0); // Iterator liefert Rechtecke
    }

    #[test]
    fn test_rectangle_iter_fused() {
        let mut iter = Rectangle::iter();
        while iter.next().is_some() {}
        // FusedIterator: nach Ende liefert None
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }
}
