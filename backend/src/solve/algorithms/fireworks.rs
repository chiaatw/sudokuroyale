use super::hidden_tuples::is_degenerate;
use super::*;

use crate::layout::cells::cell_set::CellSetIteratorUnion;
use crate::layout::values::known_set::KnownIteratorUnion;
use crate::layout::values::known_set::KnownSetLike;
use crate::puzzle::{Action, Board, Effects, Strategy, Verdict};
use itertools::Itertools;
pub struct FireworksSolver;

impl Solver for FireworksSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::Fireworks
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_fireworks(board, single)
    }
}

pub fn find_fireworks(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    for pivot in board.unknowns() {
        let row_cells = pivot.row().cells();
        let column_cells = pivot.column().cells();
        let block_cells = pivot.block().cells();

        let disjoint_cells = (row_cells | column_cells) - block_cells;
        let full_cells = disjoint_cells + pivot;

        let candidates = board.all_candidates(row_cells) & board.all_candidates(column_cells);

        for combos in candidates
            .iter()
            .filter_map(|known| {
                let set = board.candidate_cells(known);
                if set.has_any(row_cells) && set.has_any(column_cells) {
                    Some((known, set))
                } else {
                    None
                }
            })
            .map(|(known, set)| {
                (
                    known,
                    set & block_cells,
                    set & disjoint_cells,
                    set & full_cells,
                )
            })
            .filter(|(_, block_set, disjoint_set, _)| {
                !block_set.is_empty() && disjoint_set.len() <= 2
            })
            .combinations(3)
        {
            let triple = combos.iter().map(|(known, ..)| *known).union_knowns();

            if triple.len() != 3 {
                continue;
            }

            let wings = combos
                .iter()
                .map(|(_, _, disjoint_set, _)| *disjoint_set)
                .union_cells();

            if let Some((wing1, wing2)) = wings.as_pair() {
                if wing1.sees(wing2) {
                    continue;
                }

                let cells = wings + pivot;
                let all_knowns = board.all_candidates(cells);

                if !all_knowns.has_all(triple) {
                    continue;
                }

                let full_sets = combos
                    .iter()
                    .map(|(_, _, _, full_set)| *full_set)
                    .collect_vec();

                if is_degenerate(&full_sets, 3, 2) {
                    continue;
                }

                let mut action = Action::new(Strategy::Fireworks);

                cells.iter().for_each(|cell| {
                    let knowns = board.candidates(cell);
                    action.erase_knowns(cell, knowns - triple);
                    action.clue_cell_for_knowns(Verdict::Secondary, cell, triple & knowns);
                });

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_board_returns_none() {
        let board = Board::new();
        assert!(find_fireworks(&board, true).is_none());
    }

    #[test]
    fn solver_delegates_to_find_fireworks() {
        let board = Board::new();
        let solver = FireworksSolver;

        let via_solver = solver.apply(&board, true);
        let via_fn = find_fireworks(&board, true);

        assert_eq!(via_solver.is_some(), via_fn.is_some());
    }

    #[test]
    fn single_mode_never_returns_more_than_one_action() {
        let board = Board::new();
        if let Some(effects) = find_fireworks(&board, true) {
            assert!(effects.actions().len() <= 1);
        }
    }

    #[test]
    fn no_false_positives_with_small_noise() {
        let mut board = Board::new();
        let mut eff = Effects::new();

        board.set_known(
            crate::cell!("A1"),
            crate::layout::values::known::known!("1"),
            &mut eff,
        );
        board.set_known(
            crate::cell!("B2"),
            crate::layout::values::known::known!("2"),
            &mut eff,
        );
        board.set_known(
            crate::cell!("C3"),
            crate::layout::values::known::known!("3"),
            &mut eff,
        );

        assert!(!eff.has_errors());
        assert!(find_fireworks(&board, false).is_none());
    }
}
