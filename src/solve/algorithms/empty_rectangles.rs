use super::*;
use std::collections::{HashSet, HashMap};

// Trait-based solver for Empty Rectangle strategy
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

// Core Empty Rectangle detection logic
pub fn find_empty_rectangles(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    for known in Known::iter() {
        for block in House::blocks_iter() {
            if let Some((cells, row, column)) = fit_row_column(board, block, known) {
                let mut erased = CellSet::empty();

// Iterate over both orientations: (row, column) and (column, row)
                for (top, left) in [(row, column), (column, row)] {
                    let candidates = board.house_candidate_cells(left, known) - cells;

// Iterate over possible start cells in the top house
                    for start in (board.house_candidate_cells(top, known) - cells).iter() {
                        if erased.has(start) {
                            continue;
                        }

                        let right = start.house(left.shape());
                        if let Some(pivot) = (board.house_candidate_cells(right, known) - start).as_single() {
                            if start.block() == pivot.block() {
// Skip if pivot is in the same block as start
                                continue;
                            }

                            let bottom = pivot.house(top.shape());
                            let ends = board.house_candidate_cells(bottom, known) - pivot;

                            if let Some(end) = (ends & candidates).as_single() {
                                erased += end;

                                let mut action = Action::new_erase(Strategy::EmptyRectangle, end, known);

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

/// Helper function: checks if all candidate cells for a known in a block
/// fit entirely within a single row or column, returning that orientation
fn fit_row_column(board: &Board, block: House, known: Known) -> Option<(CellSet, House, House)> {
    let cells = board.house_candidate_cells(block, known);
    if cells.len() < 3 {
// Not enough candidates to form a rectangle
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
    use crate::io::{Parse, Parser};
    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::known;

    use super::*;

    #[test]
    fn test_empty_rectangle() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse(
            "441181i402i4k4080h0g20g10884418411024c0c03o4100gs421g4p4o4410h09q403o030o6om0911a4o42go040p0og20o040031g0508g2g214a40ha409403020411403g108140g8188880g412411i402g4",
        );

        if let Some(got) = find_empty_rectangles(&board, true) {
            let mut action = Action::new(Strategy::EmptyRectangle);
            action.erase(cell!("J5"), known!("2"));
            action.clue_cells_for_known(Verdict::Primary, cells!("H7 J7 J9"), known!("2"));
            action.clue_cells_for_known(Verdict::Secondary, cells!("B5 B7"), known!("2"));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("No effects found");
        }
    }
}