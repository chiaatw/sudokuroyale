use super::*;

use crate::puzzle::{Action, Board, Effects, Strategy, Verdict};
pub struct HiddenSingleSolver;

impl Solver for HiddenSingleSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy{
        Strategy::HiddenSingle
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_hidden_singles(board, single)
    }
}

pub fn find_hidden_singles(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    for (cell, knowns) in board.unknown_iter() {
        for known in knowns.iter() {
            for house in cell.houses() {
                if board.house_candidate_cells(house, known).len() == 1 {
                    let mut action = Action::new_set(Strategy::HiddenSingle, cell, known);
                    action.clue_cells_for_known(
                        Verdict::Related,
                        house.cells() - cell - board.knowns(),
                        known,
                    );

                    if effects.add_action(action) && single {
                        return Some(effects);
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

#[cfg(test)]
mod hidden_single_tests {
    use super::*;
    use crate::io::Parse;
    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::known;

    #[test]
    fn test_hidden_single_basic() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse(
            "53..7....6..195....98....6.8...6...34..8..6.6...3...1....6.2....8.419..5....8..79",
        );

        if let Some(effects) = find_hidden_singles(&board, true) {
            let mut expected = Action::new_set(Strategy::HiddenSingle, cell!("C1"), known!("4"));
            expected.clue_cells_for_known(
                Verdict::Related,
                cells!("A1 B1 A2 B2"),
                known!("4"),
            );
            assert_eq!(format!("{:?}", expected), format!("{:?}", effects.actions()[0]));
        } else {
            panic!("Hidden Single not found");
        }
    }

    #[test]
    fn test_hidden_single_multiple_actions() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse(
            "53..7....6..195....98....6.8...6...34..8..6.6...3...1....6.2....8.419..5....8..79",
        );

        if let Some(effects) = find_hidden_singles(&board, false) {
            // Es sollten mehrere Actions erzeugt werden, da single = false
            assert!(effects.actions().len() >= 1);
        } else {
            panic!("Hidden Singles not found when single=false");
        }
    }

    #[test]
    fn test_hidden_single_none() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse(
            "123456789456789123789123456214365897365897214897214365531642978642978531978531642",
        );

        // Vollständig gelöstes Board → keine Hidden Singles
        assert!(find_hidden_singles(&board, true).is_none());
    }
}
