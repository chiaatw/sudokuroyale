use super::*;
use itertools::Itertools;

use crate::layout::cells::cell_set::CellSetIteratorUnion;
use crate::layout::houses::house_set::HouseSetLike;
use crate::layout::HouseSet;
use crate::layout::Shape;
use crate::puzzle::{Action, Board, Effects, Known, Strategy, Verdict};
// X-Wing solver wrapper for the engine
pub struct XWingSolver;

impl Solver for XWingSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::XWing
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_x_wings(board, single)
    }
}

pub struct SwordfishSolver;

impl Solver for SwordfishSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::Swordfish
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_swordfish(board, single)
    }
}

// Jellyfish solver wrapper for the engine
pub struct JellyfishSolver;

impl Solver for JellyfishSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::Jellyfish
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_jellyfish(board, single)
    }
}

// Entry function for X-Wing strategy
pub fn find_x_wings(board: &Board, single: bool) -> Option<Effects> {
    find_fish(board, single, 2, Strategy::XWing)
}

// Entry function for Swordfish strategy
pub fn find_swordfish(board: &Board, single: bool) -> Option<Effects> {
    find_fish(board, single, 3, Strategy::Swordfish)
}

// Entry function for Jellyfish strategy
pub fn find_jellyfish(board: &Board, single: bool) -> Option<Effects> {
    find_fish(board, single, 4, Strategy::Jellyfish)
}

/// Generic Fish detection logic
/// Handles X-Wing (size 2), Swordfish (size 3), Jellyfish (size 4)
fn find_fish(board: &Board, single: bool, size: usize, strategy: Strategy) -> Option<Effects> {
    let mut effects = Effects::new();

    // First check rows, then columns if no single-action early exit
    if !check_houses(board, single, size, strategy, Shape::Row, &mut effects) {
        check_houses(board, single, size, strategy, Shape::Column, &mut effects);
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

// Core logic for detecting Fish patterns along a given shape (row or column)
fn check_houses(
    board: &Board,
    single: bool,
    size: usize,
    strategy: Strategy,
    shape: Shape,
    effects: &mut Effects,
) -> bool {
    for known in Known::iter() {
        let candidate_cells = board.candidate_cells(known);

        // Iterate over all combinations of houses of the given size
        for candidates in shape
            .house_iter()
            .map(|house| (house, house.shape().cells(house.coord()) & candidate_cells))
            // Only consider houses that have 2..=size candidates for this known
            .filter(|(_, cells)| 2 <= cells.len() && cells.len() <= size)
            .map(|(house, cells)| (house, cells, house.crossing_houses(cells)))
            .combinations(size)
        {
            // Union of all crossing houses (the other orientation)
            let crosses = candidates
                .iter()
                .map(|(_, _, crosses)| *crosses)
                .fold(HouseSet::empty(Shape::Row), |acc, hs| acc | hs);

            if crosses.len() != size {
                continue;
            }

            // Skip degenerate intermediate combinations for Swordfish and Jellyfish
            if size > 2
                && candidates
                    .iter()
                    .map(|(_, _, crosses)| *crosses)
                    .filter(|crosses| crosses.len() < 3)
                    .combinations(2)
                    .map(|pair| pair[0] | pair[1])
                    .any(|union| union.len() <= 2)
            {
                continue;
            }

            if size > 3
                && candidates
                    .iter()
                    .map(|(_, _, crosses)| *crosses)
                    .filter(|crosses| crosses.len() < 4)
                    .combinations(3)
                    .map(|pair| pair[0] | pair[1] | pair[2])
                    .any(|union| union.len() <= 3)
            {
                continue;
            }

            let main_cells = candidates.iter().map(|(_, cells, _)| *cells).union_cells();
            let cross_cells = crosses.cells() & candidate_cells;
            let erase = cross_cells - main_cells;

            if erase.is_empty() {
                continue;
            }

            // Construct the action to erase candidate cells and add clues
            let mut action = Action::new(strategy);
            action.erase_cells(erase, known);

            candidates.iter().for_each(|(house, cells, _)| {
                action.clue_cells_for_known(Verdict::Secondary, *cells, known);
                action.clue_cells_for_known(
                    Verdict::Related,
                    house.cells() - main_cells - board.knowns(),
                    known,
                );
            });

            // Early exit if only a single solution is required
            if effects.add_action(action) && single {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_board_returns_none_for_all_fish() {
        let board = Board::new();
        assert!(find_x_wings(&board, true).is_none());
        assert!(find_swordfish(&board, true).is_none());
        assert!(find_jellyfish(&board, true).is_none());
    }

    #[test]
    fn solvers_delegate_to_find_functions() {
        let board = Board::new();

        let x = XWingSolver.apply(&board, true);
        let x2 = find_x_wings(&board, true);
        assert_eq!(x.is_some(), x2.is_some());

        let s = SwordfishSolver.apply(&board, true);
        let s2 = find_swordfish(&board, true);
        assert_eq!(s.is_some(), s2.is_some());

        let j = JellyfishSolver.apply(&board, true);
        let j2 = find_jellyfish(&board, true);
        assert_eq!(j.is_some(), j2.is_some());
    }

    #[test]
    fn single_mode_never_returns_more_than_one_action() {
        let board = Board::new();

        if let Some(effects) = find_x_wings(&board, true) {
            assert!(effects.actions().len() <= 1);
        }
        if let Some(effects) = find_swordfish(&board, true) {
            assert!(effects.actions().len() <= 1);
        }
        if let Some(effects) = find_jellyfish(&board, true) {
            assert!(effects.actions().len() <= 1);
        }
    }

    #[test]
    fn no_false_positives_with_small_noise() {
        let mut board = Board::new();
        let mut eff = Effects::new();

        // Optional: falls set_known bei dir existiert, ok; sonst Test entfernen.
        board.set_known(
            crate::layout::cells::cell::cell!("A1"),
            crate::layout::values::known::known!("1"),
            &mut eff,
        );
        board.set_known(
            crate::layout::cells::cell::cell!("B2"),
            crate::layout::values::known::known!("2"),
            &mut eff,
        );
        board.set_known(
            crate::layout::cells::cell::cell!("C3"),
            crate::layout::values::known::known!("3"),
            &mut eff,
        );

        assert!(!eff.has_errors());

        assert!(find_x_wings(&board, false).is_none());
        assert!(find_swordfish(&board, false).is_none());
        assert!(find_jellyfish(&board, false).is_none());
    }
}
