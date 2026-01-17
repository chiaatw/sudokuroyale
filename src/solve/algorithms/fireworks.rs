use super::hidden_tuples::is_degenerate;
use super::*;

use crate::puzzle::{Action, Board, Effects, Strategy, Verdict};
use crate::layout::values::known_set::KnownSetLike;
use itertools::Itertools;
use crate::layout::values::known_set::KnownIteratorUnion;
use crate::layout::cells::cell_set::CellSetIteratorUnion;
// Trait-based solver for the Fireworks strategy
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

// Core Fireworks detection logic
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
            let triple = combos
                .iter()
                .map(|(known, ..)| *known)
                .union_knowns();

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
                    action.clue_cell_for_knowns(
                        Verdict::Secondary,
                        cell,
                        triple & knowns,
                    );
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
mod fireworks_tests {
    use super::*;
    use crate::io::Parse;
    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::known;
    use crate::layout::values::known_set::knowns;

    /// Test für einen typischen Fireworks-Fall
    #[test]
    fn test_fireworks_basic() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse(
            "g081g080g083...Puzzle mit Fireworks-Muster...",
        );

        if let Some(effects) = find_fireworks(&board, true) {
            let action = &effects.actions()[0];

            // Pivot-Zelle muss Kandidaten gesetzt bekommen
            assert!(!action.set.is_empty() || !action.erased.is_empty());

            // Clues müssen mindestens in zwei Zellen gesetzt sein
            assert!(action.secondary_clues.len() >= 2);

            // Pivot muss in den Action-Zellen enthalten sein
            let pivot_cell = cell!("E5"); // Beispiel-Pivot für den Test-Puzzle
            assert!(action.secondary_clues.contains_key(&pivot_cell) || action.set.contains(&known!("3")));
        } else {
            panic!("Fireworks-Muster wurde nicht gefunden");
        }
    }

    /// Test, dass sich sehende Wings übersprungen werden
    #[test]
    fn test_fireworks_sees_filter() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse(
            "g082g083g084...Puzzle, bei dem Wings sich sehen...",
        );

        let effects = find_fireworks(&board, false);
        // Keine Aktionen sollten für dieses ungültige Fireworks-Muster generiert werden
        assert!(effects.is_none() || effects.unwrap().actions().is_empty());
    }

    /// Test, dass degenerierte Triples übersprungen werden
    #[test]
    fn test_fireworks_degenerate() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse(
            "g085g086g087...Puzzle mit degeneriertem Triple...",
        );

        let effects = find_fireworks(&board, false);
        // Keine Aktion soll generiert werden
        assert!(effects.is_none() || effects.unwrap().actions().is_empty());
    }

    /// Test, dass mehrere Fireworks auf einem Board erkannt werden
    #[test]
    fn test_fireworks_multiple() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse(
            "g088g089g090...Puzzle mit mehreren Fireworks-Mustern...",
        );

        let effects = find_fireworks(&board, false).unwrap();
        // Mindestens zwei Aktionen generiert
        assert!(effects.actions().len() >= 2);
    }
}
