use std::collections::{HashMap, HashSet};

use super::naked_tuples;
use super::*;
use crate::layout::cells::cell_set::CellIteratorUnion;
use crate::layout::houses::house::HouseLike;
use crate::layout::houses::house_set::HouseSetLike;
use crate::layout::houses::shape::ShapeTrait;
use crate::layout::values::known_set::KnownSetIteratorUnion;
use crate::layout::values::known_set::KnownSetLike;
use crate::layout::{House, HouseSet, Rectangle, Shape};
use crate::puzzle::{Action, Board, Cell, CellSet, Effects, Known, KnownSet, Strategy, Verdict};
use itertools::Itertools;

pub struct UniqueRectangleSolver;

impl Solver for UniqueRectangleSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::UniqueRectangle
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_unique_rectangles(board, single)
    }
}

pub fn find_unique_rectangles(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    let bi_values =
        board
            .cells_with_n_candidates(2)
            .iter()
            .fold(HashMap::new(), |mut acc, cell| {
                acc.entry(board.candidates(cell))
                    .or_insert(CellSet::empty())
                    .add(cell);
                acc
            });

    for (pair, cells) in bi_values.iter().filter(|(_, cells)| cells.len() >= 2) {
        let mut found_type_ones: HashSet<Rectangle> = HashSet::new();

        for corners in cells.iter().combinations(3).map(CellSet::from_iter) {
            if let Ok(rectangle) = Rectangle::try_from(corners) {
                if check_type_one(
                    board,
                    single,
                    corners,
                    rectangle,
                    *pair,
                    &mut found_type_ones,
                    &mut effects,
                ) {
                    return Some(effects);
                }
            }
        }

        for corners in cells.iter().combinations(2).map(CellSet::from_iter) {
            let (first, second) = corners.as_pair().unwrap();

            if first.row() == second.row() {
                if check_neighbors(
                    board,
                    single,
                    *pair,
                    first,
                    second,
                    Shape::Row,
                    &found_type_ones,
                    &mut effects,
                ) {
                    return Some(effects);
                }
            } else if first.column() == second.column() {
                if check_neighbors(
                    board,
                    single,
                    *pair,
                    first,
                    second,
                    Shape::Column,
                    &found_type_ones,
                    &mut effects,
                ) {
                    return Some(effects);
                }
            } else {
                if check_diagonals(
                    board,
                    single,
                    *pair,
                    first,
                    second,
                    &found_type_ones,
                    &mut effects,
                ) {
                    return Some(effects);
                }
            }
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

fn check_type_one(
    board: &Board,
    single: bool,
    corners: CellSet,
    rectangle: Rectangle,
    pair: KnownSet,
    found_type_ones: &mut HashSet<Rectangle>,
    effects: &mut Effects,
) -> bool {
    if rectangle.block_count() != 2 || found_type_ones.contains(&rectangle) {
        return false;
    }

    let fourth = (rectangle.cells() - corners).as_single().unwrap();
    let candidates = board.candidates(fourth);
    if !candidates.has_all(pair) {
        return false;
    }

    found_type_ones.insert(rectangle);
    let mut action = Action::new(Strategy::UniqueRectangle);
    action.erase_knowns(fourth, pair);
    action.clue_cells_for_knowns(Verdict::Primary, corners, pair);
    action.clue_cell_for_knowns(Verdict::Secondary, fourth, candidates - pair);

    effects.add_action(action) && single
}

fn check_neighbors(
    board: &Board,
    single: bool,
    pair: KnownSet,
    floor_left: Cell,
    floor_right: Cell,
    shape: Shape,
    found_type_ones: &HashSet<Rectangle>,
    effects: &mut Effects,
) -> bool {
    let floor_left_block = floor_left.block();
    let houses = if floor_left_block == floor_right.block() {
        HouseSet::full(shape).minus(floor_left_block.houses(shape))
    } else {
        floor_left_block.houses(shape) - floor_left.house(shape)
    };

    for house in houses {
        if let Ok(candidate) =
            Candidate::try_from_neighbors(board, pair, floor_left, floor_right, house)
        {
            if !found_type_ones.contains(&candidate.rectangle) {
                if candidate.check(board, single, effects) {
                    return true;
                }
            }
        }
    }

    false
}

fn check_diagonals(
    board: &Board,
    single: bool,
    pair: KnownSet,
    top: Cell,
    bottom: Cell,
    found_type_ones: &HashSet<Rectangle>,
    effects: &mut Effects,
) -> bool {
    if let Ok(candidate) = Candidate::try_from_diagonals(board, pair, top, bottom) {
        if !found_type_ones.contains(&candidate.rectangle) {
            if candidate.check(board, single, effects) {
                return true;
            }
        }
    }

    false
}

struct Candidate {
    rectangle: Rectangle,
    pair: KnownSet,
    pair1: Known,
    pair2: Known,

    diagonal: bool,
    floor: CellSet,
    floor_left: Cell,
    floor_right: Cell,

    roof: CellSet,
    roof_extras: KnownSet,
    roof_left: Cell,
    roof_left_extras: KnownSet,
    roof_right: Cell,
    roof_right_extras: KnownSet,
}

impl Candidate {
    fn try_from_neighbors(
        board: &Board,
        pair: KnownSet,
        floor_left: Cell,
        floor_right: Cell,
        roof_house: House,
    ) -> Result<Self, ()> {
        match roof_house.shape() {
            Shape::Row => Self::try_from_corners(
                board,
                pair,
                floor_left,
                floor_right,
                Cell::from_coords(roof_house.coord(), floor_left.column_coord()),
                Cell::from_coords(roof_house.coord(), floor_right.column_coord()),
            ),
            Shape::Column => Self::try_from_corners(
                board,
                pair,
                floor_left,
                floor_right,
                Cell::from_coords(floor_left.row_coord(), roof_house.coord()),
                Cell::from_coords(floor_right.row_coord(), roof_house.coord()),
            ),
            Shape::Block => Err(()),
        }
    }

    fn try_from_corners(
        board: &Board,
        pair: KnownSet,
        floor_left: Cell,
        floor_right: Cell,
        roof_left: Cell,
        roof_right: Cell,
    ) -> Result<Self, ()> {
        let roof_left_candidates = board.candidates(roof_left);
        if !roof_left_candidates.has_all(pair) {
            return Err(());
        }
        let roof_right_candidates = board.candidates(roof_right);
        if !roof_right_candidates.has_all(pair) {
            return Err(());
        }
        let roof_left_extras = board.candidates(roof_left) - pair;
        let roof_right_extras = board.candidates(roof_right) - pair;

        let rectangle = Rectangle::from(floor_left, floor_right, roof_left, roof_right);
        if rectangle.block_count() != 2 {
            return Err(());
        }

        let (pair1, pair2) = pair.as_pair().unwrap();

        Ok(Self {
            rectangle,
            pair,
            pair1,
            pair2,
            diagonal: false,
            floor: CellSet::from_iter([floor_left, floor_right]),
            floor_left,
            floor_right,
            roof: CellSet::from_iter([roof_left, roof_right]),
            roof_extras: roof_left_extras | roof_right_extras,
            roof_left,
            roof_left_extras,
            roof_right,
            roof_right_extras,
        })
    }

    fn try_from_diagonals(
        board: &Board,
        pair: KnownSet,
        floor1: Cell,
        floor2: Cell,
    ) -> Result<Self, ()> {
        let block1 = floor1.block();
        let block2 = floor2.block();
        if block1 == block2
            || (block1.rows() != block2.rows() && block1.columns() != block2.columns())
        {
            return Err(());
        }

        let floor = CellSet::from_iter([floor1, floor2]);
        let rectangle = Rectangle::try_from(floor)?;
        if rectangle.block_count() != 2 {
            return Err(());
        }

        let roof = rectangle.cells() - floor;
        let roof_pair = roof.as_pair().unwrap();

        let (floor_left, floor_right) = sort_by_column(floor1, floor2);
        let (roof_left, roof_right) = sort_by_column(roof_pair.0, roof_pair.1);

        let roof_left_candidates = board.candidates(roof_left);
        if !roof_left_candidates.has_all(pair) {
            return Err(());
        }
        let roof_right_candidates = board.candidates(roof_right);
        if !roof_right_candidates.has_all(pair) {
            return Err(());
        }
        let roof_left_extras = board.candidates(roof_left) - pair;
        let roof_right_extras = board.candidates(roof_right) - pair;

        let (pair1, pair2) = pair.as_pair().unwrap();

        Ok(Self {
            rectangle,
            pair,
            pair1,
            pair2,
            diagonal: true,
            floor,
            floor_left,
            floor_right,
            roof,
            roof_extras: roof_left_extras | roof_right_extras,
            roof_left,
            roof_left_extras,
            roof_right,
            roof_right_extras,
        })
    }

    fn check(&self, board: &Board, single: bool, effects: &mut Effects) -> bool {
        if self.diagonal {
            if self.check_type_five(board, effects) && single {
                return true;
            }
        }
        if self.check_type_two(board, effects) && single {
            return true;
        }
        if self.check_type_three(board, effects) && single {
            return true;
        }
        if self.check_type_four(board, effects) && single {
            return true;
        }

        false
    }

    fn check_type_two(&self, board: &Board, effects: &mut Effects) -> bool {
        if self.roof_left_extras.len() != 1 || self.roof_left_extras != self.roof_right_extras {
            return false;
        }

        let extra = self.roof_left_extras.as_single().unwrap();
        let cells = board.candidate_cells(extra) & self.roof_left.peers() & self.roof_right.peers();
        if cells.is_empty() {
            return false;
        }

        let mut action = Action::new(Strategy::UniqueRectangle);
        action.erase_cells(cells, extra);
        action.clue_cells_for_knowns(Verdict::Primary, self.rectangle.cells(), self.pair);
        action.clue_cells_for_known(Verdict::Secondary, self.roof, extra);

        effects.add_action(action)
    }

    fn check_type_three(&self, board: &Board, effects: &mut Effects) -> bool {
        if !(2..=4).contains(&self.roof_extras.len()) {
            return false;
        }

        let mut action = Action::new(Strategy::UniqueRectangle);
        action.clue_cells_for_knowns(Verdict::Primary, self.rectangle.cells(), self.pair);
        action.clue_cell_for_knowns(Verdict::Secondary, self.roof_left, self.roof_left_extras);
        action.clue_cell_for_knowns(Verdict::Secondary, self.roof_right, self.roof_right_extras);

        for house in self.roof_left.common_houses(self.roof_right) {
            let peers = house.cells() - self.roof;
            let peer_knowns: Vec<(Cell, KnownSet)> = peers
                .iter()
                .map(|cell| (cell, board.candidates(cell)))
                .collect();

            for size in 2..=4 {
                if size < self.roof_extras.len() {
                    continue;
                }

                for peer_knowns_combo in peer_knowns
                    .iter()
                    .filter(|(_, knowns)| (2..=size).contains(&(*knowns).len()))
                    .combinations(size - 1)
                {
                    let known_sets: Vec<KnownSet> = peer_knowns_combo
                        .iter()
                        .map(|(_, ks)| *ks)
                        .chain([self.roof_extras])
                        .collect();
                    let knowns = known_sets.iter().copied().union_knowns();
                    if knowns.len() != size
                        || naked_tuples::is_degenerate(&known_sets, size, 2)
                        || naked_tuples::is_degenerate(&known_sets, size, 3)
                    {
                        continue;
                    }

                    let cells = peers - peer_knowns_combo.iter().map(|(c, _)| *c).union_cells();

                    let mut found = false;
                    for known in knowns.iter() {
                        let erase = cells & board.candidate_cells(known);
                        if !erase.is_empty() {
                            found = true;
                            action.erase_cells(erase, known)
                        }
                    }
                    if found {
                        for (cell, knowns) in peer_knowns_combo {
                            action.clue_cell_for_knowns(Verdict::Secondary, *cell, *knowns);
                        }
                    }
                    break;
                }
            }
        }

        effects.add_action(action)
    }

    fn check_type_four(&self, board: &Board, effects: &mut Effects) -> bool {
        for shape in Shape::iter() {
            let house = self.roof_left.house(shape);
            if house != self.roof_right.house(shape) {
                continue;
            }

            let pair1_required = board.house_candidate_cells(house, self.pair1) == self.roof;
            let pair2_required = board.house_candidate_cells(house, self.pair2) == self.roof;
            if pair1_required == pair2_required {
                continue;
            }

            let (required, erase) = if pair1_required {
                (self.pair1, self.pair2)
            } else {
                (self.pair2, self.pair1)
            };

            let mut action = Action::new(Strategy::UniqueRectangle);
            action.erase_cells(self.roof, erase);
            action.clue_cells_for_knowns(Verdict::Primary, self.floor, self.pair);
            action.clue_cells_for_known(Verdict::Secondary, self.roof, required);

            if effects.add_action(action) {
                return true;
            }
        }

        false
    }

    fn check_type_five(&self, board: &Board, effects: &mut Effects) -> bool {
        let mut erase = None;

        for (shape, pair_check, pair_erase) in [
            (Shape::Row, self.pair1, self.pair2),
            (Shape::Row, self.pair2, self.pair1),
            (Shape::Column, self.pair1, self.pair2),
            (Shape::Column, self.pair2, self.pair1),
        ] {
            let house_left = self.floor_left.house(shape);
            let house_right = self.floor_right.house(shape);
            if board.house_candidate_cells(house_left, pair_check).len() == 2
                && board.house_candidate_cells(house_right, pair_check).len() == 2
            {
                erase = Some(pair_erase);
            }
        }

        if let Some(erase) = erase {
            let mut action = Action::new(Strategy::UniqueRectangle);
            action.erase_cells(CellSet::of(&[self.floor_left, self.floor_right]), erase);
            action.clue_cells_for_knowns(Verdict::Primary, self.roof, self.pair);
            action.clue_cells_for_knowns(Verdict::Primary, self.floor, self.pair - erase);

            effects.add_action(action)
        } else {
            false
        }
    }
}

fn sort_by_column(first: Cell, second: Cell) -> (Cell, Cell) {
    if first.column_coord() < second.column_coord() {
        (first, second)
    } else {
        (second, first)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::cell;
    use crate::layout::values::known::known;

    #[test]
    fn empty_board_returns_none() {
        let board = Board::new();
        assert!(find_unique_rectangles(&board, false).is_none());
    }

    #[test]
    fn solver_delegates_to_find_unique_rectangles() {
        let board = Board::new();
        let solver = UniqueRectangleSolver;

        let via_solver = solver.apply(&board, false);
        let via_fn = find_unique_rectangles(&board, false);

        assert_eq!(via_solver.is_some(), via_fn.is_some());
    }

    #[test]
    fn single_mode_never_returns_more_than_one_action() {
        let board = Board::new();

        if let Some(effects) = find_unique_rectangles(&board, true) {
            assert!(effects.actions().len() <= 1);
        }
    }

    #[test]
    fn no_false_positive_with_some_knowns() {
        let mut board = Board::new();
        let mut eff = Effects::new();

        board.set_known(cell!("A1"), known!("1"), &mut eff);
        board.set_known(cell!("B2"), known!("2"), &mut eff);
        board.set_known(cell!("C3"), known!("3"), &mut eff);
        board.set_known(cell!("D4"), known!("4"), &mut eff);

        assert!(!eff.has_errors());
        assert!(find_unique_rectangles(&board, false).is_none());
    }
}
