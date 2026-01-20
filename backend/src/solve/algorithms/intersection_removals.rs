use super::*;

use crate::layout::houses::house::HouseLike;
use crate::layout::values::known_set::KnownSetLike;
use crate::layout::{House, HouseSet};
use crate::puzzle::{Action, Board, Effects, Known, Strategy, Verdict};

/// Solver for Intersection Removal strategies:
/// Pointing Pair/Triple
/// Box Line Reduction
///
/// Detects candidates confined to a row/column withing a block (Pointing)
/// and removes candidates from block/line disjoint ares(Box-Line Reduction)
pub struct IntersectionSolver;

impl Solver for IntersectionSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::IntersectionRemoval
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_intersection_removals(board, single)
    }
}

/// Finds intersection removal effects across all blocks
/// Returns Effects containing candidate erasures and clues
pub fn find_intersection_removals(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    for block in House::blocks_iter() {
        if check_intersection(board, single, block, block.rows(), &mut effects)
            || check_intersection(board, single, block, block.columns(), &mut effects)
        {
            break;
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

/// Checks intersections between a block and given houses (rows, columns)
/// Applies Pointing Pair/Triple or Box Line Reduction as appropriate
fn check_intersection(
    board: &Board,
    single: bool,
    block: House,
    houses: HouseSet,
    effects: &mut Effects,
) -> bool {
    for known in Known::iter() {
        for house in houses.iter() {
            let block_cells = block.cells();
            let intersection_cells = block_cells & house.cells();
            let box_cells = block_cells - intersection_cells;
            let box_candidates = board.all_candidates(box_cells);
            let line_cells = house.cells() - intersection_cells;
            let line_candidates = board.all_candidates(line_cells);

            let candidate_cells = board.candidate_cells(known);
            let intersection_candidate_cells = intersection_cells & candidate_cells;
            let intersection_candidate_cells_count = intersection_candidate_cells.len();

            if intersection_candidate_cells_count < 2 {
                // ignore hidden single
                continue;
            }

            // Case 1: Box Line Reduction
            if box_candidates.has(known) && !line_candidates.has(known) {
                let erase = box_cells & candidate_cells;
                if !erase.is_empty() {
                    let mut action = Action::new(Strategy::BoxLineReduction);
                    action.erase_cells(erase, known);
                    action.clue_cells_for_known(
                        Verdict::Secondary,
                        intersection_candidate_cells,
                        known,
                    );
                    action.clue_cells_for_known(
                        Verdict::Related,
                        line_cells - board.knowns(),
                        known,
                    );

                    if effects.add_action(action) && single {
                        return true;
                    }
                }
            }
            // Case 2: Pointing Pair/Triple
            else if line_candidates.has(known) {
                let erase = line_cells & candidate_cells;
                if !erase.is_empty() {
                    let strategy = if intersection_candidate_cells_count == 3 {
                        Strategy::PointingTriple
                    } else {
                        Strategy::PointingPair
                    };
                    let mut action = Action::new(strategy);
                    action.erase_cells(erase, known);
                    action.clue_cells_for_known(
                        Verdict::Secondary,
                        intersection_candidate_cells,
                        known,
                    );
                    action.clue_cells_for_known(
                        Verdict::Related,
                        block_cells - intersection_cells - board.knowns(),
                        known,
                    );

                    if effects.add_action(action) && single {
                        return true;
                    }
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_board_returns_none() {
        let board = Board::new();
        assert!(find_intersection_removals(&board, true).is_none());
    }

    #[test]
    fn solver_delegates_to_find_intersection_removals() {
        let board = Board::new();
        let solver = IntersectionSolver;

        let via_solver = solver.apply(&board, true);
        let via_fn = find_intersection_removals(&board, true);

        assert_eq!(via_solver.is_some(), via_fn.is_some());
    }

    #[test]
    fn single_mode_never_returns_more_than_one_action() {
        let board = Board::new();
        if let Some(effects) = find_intersection_removals(&board, true) {
            assert!(effects.actions().len() <= 1);
        }
    }

    #[test]
    fn no_false_positives_with_small_noise() {
        let mut board = Board::new();
        let mut eff = Effects::new();

        // Falls set_known in deinem Board existiert (wie in anderen Dateien), ok.
        // Wenn nicht, diesen Test einfach entfernen.
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

        // Ohne gezielt konstruierte Kandidatenlage sollte Intersection Removal nicht "zufällig" feuern
        assert!(find_intersection_removals(&board, false).is_none());
    }
}
