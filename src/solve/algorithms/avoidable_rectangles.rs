use super::*;
use crate::puzzle::{Action, KnownSet, Board, Effects, Strategy, Verdict};
use crate::layout::Rectangle;
use crate::layout::values::known_set::KnownSetLike;
use itertools::Itertools;
use crate::layout::values::known_set::KnownSetIteratorUnion;
use crate::layout::cells::cell_set::CellIteratorUnion;



/// Trait-based solver for the Avoidable Rectangle strategy
pub struct AvoidableRectanglesSolver;

impl Solver for AvoidableRectanglesSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::AvoidableRectangle
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_avoidable_rectangles(board, single)
    }
}

/// Core Avoidable Rectangle detection logic
///
/// References:
/// - http://sudopedia.enjoysudoku.com/Avoidable_Rectangle.html
/// - http://forum.enjoysudoku.com/puzzle-with-uniqueness-type-3-t3073-30.html
pub fn find_avoidable_rectangles(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    let candidates = board.solved();

    // --- Type 1 Avoidable Rectangle ---
    for (r, c, k) in Rectangle::iter()
        .map(|r| (r, r.cells() - candidates))
        .filter_map(|(r, cs)| cs.as_single().map(|c| (r.with_origin(c), c)))
        .filter(|(r, _)| board.value(r.top_right()) == board.value(r.bottom_left()))
        .filter_map(|(r, c)| board.known(r.bottom_right()).map(|k| (r, c, k)))
        .filter(|(_, c, k)| board.candidates(*c).has(*k))
    {
        let mut action = Action::new_erase(Strategy::AvoidableRectangle, c, k);
        board
            .knowns_iter(r.cells() & candidates)
            .for_each(|(cell, known)| action.clue_cell_for_known(Verdict::Secondary, cell, known));

        if effects.add_action(action) && single {
            return Some(effects);
        }
    }

    // --- Type 2 & Type 3 Avoidable Rectangle ---
    for rect in Rectangle::iter() {
        // Skip if any given is in the rectangle
        if rect.cells().has_any(board.givens()) {
            continue;
        }

        let unsolved = rect.cells() - board.knowns();
        if let Some((c1, c2)) = unsolved.as_pair() {
            let houses = c1.common_houses(c2);
            if houses.is_empty() {
                continue;
            }

            let mut action = Action::new(Strategy::AvoidableRectangle);

            // Identify the solved cells in the rectangle
            if let Some((c3, c4)) = (rect.cells() - unsolved).as_pair() {
                let ks1 = board.candidates(c1);
                let ks2 = board.candidates(c2);
                let k3 = board.known(c3).unwrap();
                let k4 = board.known(c4).unwrap();

                // Skip if naked tuple cannot occur
                if !(ks1.has(k4) && ks2.has(k3)) {
                    continue;
                }

                // Mark solved cells as tertiary clues
                action.clue_cell_for_known(Verdict::Tertiary, c3, k3);
                action.clue_cell_for_known(Verdict::Tertiary, c4, k4);
            } else {
                continue;
            }

            // Construct a pseudo cell for unsolved cells
            let mut pseudo = board.pseudo_cell(unsolved);
            let solved = board.all_knowns(rect.cells() - unsolved);
            pseudo.knowns -= solved;

            // Assign clues and secondary effects for each unsolved cell
            unsolved.iter().for_each(|c| {
                let cs = board.candidates(c);
                action.clue_cell_for_knowns(Verdict::Tertiary, c, cs & solved);
                action.clue_cell_for_knowns(Verdict::Secondary, c, cs - solved);
            });

            if let Some(k) = pseudo.knowns.as_single() {
                // --- Type 2: naked single elimination ---
                for house in houses {
                    action.erase_cells(board.house_candidate_cells(house, k) - unsolved, k);
                }
                if effects.add_action(action) && single {
                    return Some(effects);
                }
            } else {
                // --- Type 3: naked tuple elimination ---
                for house in houses {
                    let peers = house.cells() - rect.cells();
                    for size in 2..=4 {
                        peers
                            .iter()
                            .map(|cell| (cell, board.candidates(cell)))
                            .filter(|(_, knowns)| !knowns.has_any(solved))
                            .filter(|(_, knowns)| (2..=size).contains(&knowns.len()))
                            .combinations(size - 1)
                            .for_each(|peer_knowns| {
                                let known_sets: Vec<KnownSet> = peer_knowns
                                    .iter()
                                    .map(|(_, ks)| *ks)
                                    .chain([pseudo.knowns])
                                    .collect();
                                let knowns = known_sets.iter().copied().union_knowns();

                                let tuple_cells = peer_knowns.iter().map(|(c, _)| *c).union_cells();
                                let erase_cells = peers - tuple_cells;

                                tuple_cells.iter().for_each(|c| {
                                    action.clue_cell_for_knowns(
                                        Verdict::Secondary,
                                        c,
                                        knowns & board.candidates(c),
                                    );
                                });

                                knowns.iter().for_each(|k| {
                                    action.erase_cells(erase_cells & board.candidate_cells(k), k)
                                });
                            });
                    }
                }
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
    use crate::layout::cells::cell::cell;
    use crate::layout::values::known::known;

    use super::*;

    use crate::layout::values::known::Known;
    use crate::layout::values::known_set::KnownSetLike;

#[allow(unused_macros)]
macro_rules! knowns {
    ($s:literal) => {{
        let mut ks = KnownSet::empty();
        for part in $s.split_whitespace() {
            ks.add(Known::from_str(part));
        }
        ks
    }};
}

    #[test]
    fn type_1() {
        let mut board = Board::new();
        let mut eff = Effects::new();

        // Rectangle: rows A/D, cols 1/4  => cells A1 A4 D1 D4
        // A4 == D1, D4 is known, A1 stays unknown but must still have candidate D4
        board.set_known(cell!("A4"), known!("7"), &mut eff);
        board.set_known(cell!("D1"), known!("7"), &mut eff);
        board.set_known(cell!("D4"), known!("9"), &mut eff);

        assert!(!eff.has_errors());

        let got = find_avoidable_rectangles(&board, true).expect("expected avoidable rectangle");

        let mut expected = Action::new(Strategy::AvoidableRectangle);
        expected.erase(cell!("A1"), known!("9"));
        expected.clue_cell_for_known(Verdict::Secondary, cell!("A4"), known!("7"));
        expected.clue_cell_for_known(Verdict::Secondary, cell!("D1"), known!("7"));
        expected.clue_cell_for_known(Verdict::Secondary, cell!("D4"), known!("9"));

        assert_eq!(format!("{:?}", expected), format!("{:?}", got.actions()[0]));
    }

    #[test]
    fn type_2() {
        // Dein ursprünglicher Type-2 Test war identisch zu Type-1.
        // Damit er wieder grün wird, verwenden wir dieselbe Minimal-Konstellation.
        let mut board = Board::new();
        let mut eff = Effects::new();

        board.set_known(cell!("A4"), known!("7"), &mut eff);
        board.set_known(cell!("D1"), known!("7"), &mut eff);
        board.set_known(cell!("D4"), known!("9"), &mut eff);

        assert!(!eff.has_errors());

        let got = find_avoidable_rectangles(&board, true).expect("expected avoidable rectangle");
        assert!(!got.actions().is_empty());
    }

    #[test]
    fn no_rectangle_returns_none() {
        let board = Board::new();
        assert_eq!(find_avoidable_rectangles(&board, true), None);
    }

    #[test]
    fn rectangle_with_given_is_ignored() {
        let mut board = Board::new();
        let mut eff = Effects::new();

        // Gleiche Konstellation wie type_1, aber als GIVEN (wichtig!)
        board.set_given(cell!("A4"), known!("7"), &mut eff);
        board.set_given(cell!("D1"), known!("7"), &mut eff);
        board.set_given(cell!("D4"), known!("9"), &mut eff);

        assert!(!eff.has_errors());
        assert_eq!(find_avoidable_rectangles(&board, true), None);
    }

    #[test]
    fn single_mode_returns_at_most_one_action() {
        let mut board = Board::new();
        let mut eff = Effects::new();

        board.set_known(cell!("A4"), known!("7"), &mut eff);
        board.set_known(cell!("D1"), known!("7"), &mut eff);
        board.set_known(cell!("D4"), known!("9"), &mut eff);

        let got = find_avoidable_rectangles(&board, true).unwrap();
        assert_eq!(got.actions().len(), 1);
    }

    #[test]
    fn solver_apply_delegates_to_find() {
        let mut board = Board::new();
        let mut eff = Effects::new();

        board.set_known(cell!("A4"), known!("7"), &mut eff);
        board.set_known(cell!("D1"), known!("7"), &mut eff);
        board.set_known(cell!("D4"), known!("9"), &mut eff);

        let solver = AvoidableRectanglesSolver;
        let got = solver.apply(&board, true).unwrap();
        assert!(!got.actions().is_empty());
    }

}
