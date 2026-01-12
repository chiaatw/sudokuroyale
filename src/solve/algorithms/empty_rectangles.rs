use super::*;

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

                        if let Some(pivot) = (board.house_candidate_cells(right, known) - start).as_single()
                        {
// Cannot remove candidates in the starting block
                            if start.block() == pivot.block() {
                                continue;
                            }

                            let bottom = pivot.house(top.shape());
                            let ends = board.house_candidate_cells(bottom, known) - pivot;

                            if let Some(end) = (ends & candidates).as_single() {
                                erased += end;

                                let mut action = Action::new_erase(Strategy::EmptyRectangle, end, known);

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
    use crate::io::{Parse, Parser};
    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::known;

    use super::*;

    #[test]
    fn test_empty_rectangle_solver() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse(
            "441181i402i4k4080h0g20g10884418411024c0c03o4100gs421g4p4o4410h09q403o030o6om0911a4o42go040p0og20o040031g0508g2g214a40ha409403020411403g108140g8188880g412411i402g4",
        );

        let solver = EmptyRectangleSolver;

        if let Some(got) = solver.apply(&board, true) {
            let mut expected = Action::new(Strategy::EmptyRectangle);

            expected.erase(cell!("J5"), known!("2"));
            expected.clue_cells_for_known(Verdict::Primary, cells!("H7 J7 J9"), known!("2"));
            expected.clue_cells_for_known(Verdict::Secondary, cells!("B5 B7"), known!("2"));

            assert_eq!(format!("{:?}", expected), format!("{:?}", got.actions()[0]));
        } else {
            panic!("Empty Rectangle solver found no effects");
        }
    }
     #[test]
    fn test_no_candidates_returns_none() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse("Puzzle ohne Kandidaten für die getestete Zahl");
        assert!(EmptyRectangleSolver.apply(&board, true).is_none());
    }

    /// Degenerierte Fälle (1–2 Kandidaten) → keine Aktion
    #[test]
    fn test_degenerate_cases() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse("Puzzle mit nur 1-2 Kandidaten in einem Block");
        assert!(EmptyRectangleSolver.apply(&board, true).is_none());
    }

    /// Mehrere Empty Rectangles → alle Aktionen gesammelt
    #[test]
    fn test_multiple_empty_rectangles() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse("Puzzle mit mehreren Empty Rectangles");
        let effects = EmptyRectangleSolver.apply(&board, false).unwrap();
        assert!(effects.actions().len() > 1);
    }

    /// Prüfen, dass Primary und Secondary Clues korrekt gesetzt sind
    #[test]
    fn test_clues_assignment() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse(
            "441181i402i4k4080h0g20g10884418411024c0c03o4100gs421g4p4o4410h09q403o030o6om0911a4o42go040p0og20o040031g0508g2g214a40ha409403020411403g108140g8188880g412411i402g4",
        );

        let effects = EmptyRectangleSolver.apply(&board, true).unwrap();
        for action in effects.actions() {
            assert!(!action.primary_clues.is_empty() || !action.secondary_clues.is_empty() || !action.set.is_empty());
        }
    }
}