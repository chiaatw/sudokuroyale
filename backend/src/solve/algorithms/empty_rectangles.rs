use super::*;

use crate::layout::houses::house::HouseLike;
use crate::layout::House;
use crate::puzzle::{Action, Board, CellSet, Effects, Known, Strategy, Verdict};
/// Solver for the Empty Rectangle strategy
///
/// Detects Empty Rectangles on the board and produces candidate eliminations
pub struct EmptyRectangleSolver;

impl Solver for EmptyRectangleSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::EmptyRectangle
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_empty_rectangles(board, single)
    }
}

/// Finds all Empty Rectangles on the board and returns their effects
pub fn find_empty_rectangles(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    // Iterate over all possible candidates
    for known in Known::iter() {
        // Iterate over all blocks
        for block in House::blocks_iter() {
            if let Some((cells, row, column)) = fit_row_column(board, block, known) {
                let mut erased = CellSet::empty();

                // Consider both orientations: (row, column) and (column, row)
                for (top, left) in [(row, column), (column, row)] {
                    // Candidates in the left house not part of the rectangle
                    let candidates = board.house_candidate_cells(left, known) - cells;

                    for start in (board.house_candidate_cells(top, known) - cells).iter() {
                        if erased.has(start) {
                            continue;
                        }

                        let right = start.house(left.shape());

                        if let Some(pivot) =
                            (board.house_candidate_cells(right, known) - start).as_single()
                        {
                            // Cannot remove candidates in the starting block
                            if start.block() == pivot.block() {
                                continue;
                            }

                            let bottom = pivot.house(top.shape());
                            let ends = board.house_candidate_cells(bottom, known) - pivot;

                            if let Some(end) = (ends & candidates).as_single() {
                                erased += end;

                                let mut action =
                                    Action::new_erase(Strategy::EmptyRectangle, end, known);

                                // Determine clues or direct erase based on context
                                if ends.len() == 1 {
                                    action.erase(start, known);
                                } else {
                                    action.clue_cell_for_known(Verdict::Secondary, start, known);
                                }

                                action.clue_cells_for_known(Verdict::Primary, cells, known);
                                action.clue_cell_for_known(Verdict::Secondary, pivot, known);

                                if effects.add_action(action) && single {
                                    return Some(effects);
                                }
                            }
                        }
                    }
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

/// Checks if all candidate cells in a block can fit in a single row or column
fn fit_row_column(board: &Board, block: House, known: Known) -> Option<(CellSet, House, House)> {
    let cells = board.house_candidate_cells(block, known);

    if cells.len() < 3 {
        // Degenerate cases (1-2 candidates) are ignored
        return None;
    }

    for row in block.rows().iter() {
        for column in block.columns().iter() {
            if cells.is_subset_of(row.cells() | column.cells()) {
                return Some((cells, row, column));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_board_returns_none() {
        let board = Board::new();
        assert!(find_empty_rectangles(&board, true).is_none());
    }

    #[test]
    fn solver_delegates_to_find() {
        let board = Board::new();
        let solver = EmptyRectangleSolver;

        let a = solver.apply(&board, true);
        let b = find_empty_rectangles(&board, true);

        // Beide sollten gleich sein (hier: None)
        assert_eq!(a.is_some(), b.is_some());
    }

    #[test]
    fn fit_row_column_returns_none_for_degenerate_candidate_count() {
        // Ohne Kandidaten-Setup sollte ein leeres Board im Block für irgendein Known
        // entweder 0 Kandidaten liefern oder jedenfalls < 3 in der Blockmenge.
        // Damit muss fit_row_column None liefern.
        let board = Board::new();

        // Nimm eine beliebige Zahl und einen Block
        let known = Known::iter().next().unwrap();
        let block = House::blocks_iter().next().unwrap();

        assert!(super::fit_row_column(&board, block, known).is_none());
    }

    #[test]
    fn single_mode_never_returns_more_than_one_action() {
        // Auch wenn wir hier voraussichtlich None bekommen:
        // Diese Eigenschaft ist wichtig und bleibt korrekt, wenn später mal Aktionen gefunden werden.
        let board = Board::new();
        if let Some(effects) = find_empty_rectangles(&board, true) {
            assert!(effects.actions().len() <= 1);
        }
    }
}
