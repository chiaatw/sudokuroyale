use super::*;

pub struct NakedSingleSolver;

impl Solver for NakedSingleSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::NakedSingle
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_naked_singles(board, single)
    }
}

pub fn find_naked_singles(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    for (cell, knowns) in board.cell_candidates_with_n_candidates(1) {
        let known = knowns.as_single().unwrap();

        let mut action = Action::new_set(Strategy::NakedSingle, cell, known);
        action.clue_cell_for_knowns(
            Verdict::Related,
            cell,
            KnownSet::full() - known,
        );

        if effects.add_action(action) && single {
            return Some(effects);
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}