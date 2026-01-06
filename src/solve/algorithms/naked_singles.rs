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

#[cfg(test)]
mod naked_single_tests {
    use super::*;
    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::known;
    use crate::layout::values::known_set::knowns;

    #[test]
    fn test_single_naked_single() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        // Set up a Naked Single: only candidate "5" in cell A1
        board.set_candidates(cell!("A1"), knowns!("5"), &mut effects);

        let found = find_naked_singles(&board, true).unwrap();
        let action = &found.actions()[0];

        // Should set A1 to 5
        assert_eq!(cell!("A1"), action.cell());
        assert_eq!(known!("5"), action.set_value().unwrap());
    }

    #[test]
    fn test_multiple_naked_singles() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        // Two naked singles
        board.set_candidates(cell!("A1"), knowns!("3"), &mut effects);
        board.set_candidates(cell!("B2"), knowns!("7"), &mut effects);

        let found = find_naked_singles(&board, false).unwrap();
        let mut set_cells = vec![];
        let mut set_values = vec![];

        for action in found.actions() {
            set_cells.push(action.cell());
            set_values.push(action.set_value().unwrap());
        }

        assert!(set_cells.contains(&cell!("A1")));
        assert!(set_cells.contains(&cell!("B2")));
        assert!(set_values.contains(&known!("3")));
        assert!(set_values.contains(&known!("7")));
    }

    #[test]
    fn test_no_naked_singles() {
        let board = Board::new(); // empty board with all candidates
        let found = find_naked_singles(&board, false);

        // No naked singles should be detected
        assert!(found.is_none());
    }
}
