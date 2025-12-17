use super::*;

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
        for known in knowns {
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