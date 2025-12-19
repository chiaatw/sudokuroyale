use super::*;

pub struct TwoStringKiteSolver;

impl Solver for TwoStringKiteSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::TwoStringKite
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        self.find_two_string_kites(board, single)
    }
}

impl TwoStringKiteSolver {
    fn find_two_string_kites(&self, board: &Board, single: bool) -> Option<Effects> {
        let mut effects = Effects::new();

        for known in Known::iter() {
            let candidates = board.candidate_cells(known);
            if candidates.len() < 5 {
                continue;
            }

            for row in House::rows_iter() {
                let row_cells = board.house_candidate_cells(row, known);
                if row_cells.len() != 2 || row_cells.blocks().len() == 1 {
                    continue;
                }

                for column in House::columns_iter() {
                    let column_cells = board.house_candidate_cells(column, known);
                    if column_cells.len() != 2
                        || !(row_cells & column_cells).is_empty()
                        || column_cells.blocks().len() == 1
                    {
                        continue;
                    }

                    let (row_cell_left, row_cell_right) = row_cells.as_pair().unwrap();
                    let (column_cell_high, column_cell_low) = column_cells.as_pair().unwrap();

                    let (pivots, ends) = 
                        if row_cell_left.block() == column_cell_high.block() {
                            (row_cell_left + column_cell_high,
                             row_cell_right + column_cell_low)
                        } else if row_cell_left.block() == column_cell_low.block() {
                            (row_cell_left + column_cell_low,
                             row_cell_right + column_cell_high)
                        } else if row_cell_right.block() == column_cell_high.block() {
                            (row_cell_right + column_cell_high,
                             row_cell_left + column_cell_low)
                        } else if row_cell_right.block() == column_cell_low.block() {
                            (row_cell_right + column_cell_low,
                             row_cell_left + column_cell_high)
                        } else {
                            continue;
                        };

                    let erase = ends.peers() & candidates;
                    if erase.is_empty() {
                        continue;
                    }

                    let mut action = 
                        Action::new_erase_cells(Strategy::TwoStringKite, erase, known);
                    action.clue_cells_for_known(Verdict::Secondary, ends, known);
                    action.clue_cells_for_known(Verdict::Primary, pivots, known);

                    if effects.add_action(action) && single {
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
}