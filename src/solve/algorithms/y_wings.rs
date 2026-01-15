use super::*;

use crate::puzzle::{Action, Board, Effects, Strategy, Verdict};
// Solve wrapper for the Y-Wing strategy
pub struct YWingSolver;

impl Solver for YWingSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::YWing
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_y_wings(board, single)
    }
}

// Finds Y-Wing patterns and returns the corresponding effects
pub fn find_y_wings(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    // Bi-value cells are potential pivots
    let bi_values = board.cells_with_n_candidates(2);

    for pivot in bi_values {
        let (k1, k2) = board.candidates(pivot).as_pair().unwrap();
        let peers = pivot.peers() & bi_values;

        if peers.len() < 2 {
// Need at least two pivot peers to form a Y-Wing
            continue; 
        }

        let k1_peers = peers & board.candidate_cells(k1);
        let k2_peers = peers & board.candidate_cells(k2);

        for c1 in k1_peers {
            let k1_other = board.candidates(c1) - k1;

            for c2 in k2_peers {
                let k2_other = board.candidates(c2) - k2;

// Skip if c1 and c2 see each other or the other candidates don't match
                if k1_other != k2_other || c1.sees(c2) {
                    continue;
                }

                let k = k1_other.iter().next().unwrap();
                let erase = c1.peers() & c2.peers() & board.candidate_cells(k);

                if erase.is_empty() {
                    continue;
                }

// Construct the action for this Y-Wing
                let mut action = Action::new(Strategy::YWing);
                action.erase_cells(erase, k);
                action.clue_cell_for_known(Verdict::Secondary, pivot, k1);
                action.clue_cell_for_known(Verdict::Tertiary, pivot, k2);
                action.clue_cell_for_known(Verdict::Tertiary, c1, k1);
                action.clue_cell_for_known(Verdict::Secondary, c1, k);
                action.clue_cell_for_known(Verdict::Secondary, c2, k2);
                action.clue_cell_for_known(Verdict::Tertiary, c2, k);

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
    use crate::io::{Parse, Parser};
    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::known;
    use super::*;

    #[test]
    fn test_y_wing_basic() {
        let parser = Parse::wiki().stop_on_error();
        let (board, _, failed) = parser.parse(
            "814kg10s2u246c116e110922812m41i42mg42i4k621sg134812m6e05g10h215081030950418128g11c0334240h2803114c4c0h64g181gq4g055g81j0j822jagg1181032k09g441i4ga214a5454h40h81he"
        );
        assert_eq!(None, failed);

        let solver = YWingSolver;
        if let Some(got) = solver.apply(&board, true) {
            let mut expected = Action::new(Strategy::YWing);

            expected.erase_cells(cells!("C4"), known!("9"));
            expected.clue_cell_for_known(Verdict::Secondary, cell!("D1"), known!("1"));
            expected.clue_cell_for_known(Verdict::Tertiary, cell!("D1"), known!("5"));
            expected.clue_cell_for_known(Verdict::Tertiary, cell!("D9"), known!("2"));
            expected.clue_cell_for_known(Verdict::Secondary, cell!("D9"), known!("1"));
            expected.clue_cell_for_known(Verdict::Secondary, cell!("F1"), known!("5"));
            expected.clue_cell_for_known(Verdict::Tertiary, cell!("F1"), known!("1"));

            assert_eq!(format!("{:?}", expected), format!("{:?}", got.actions()[0]));
        } else {
            panic!("Y-Wing solver found no effects");
        }
    }

    #[test]
    fn test_y_wing_none() {
        let board = crate::layout::Board::new(); // leeres Board
        let solver = YWingSolver;
        let effects = solver.apply(&board, true);
        assert!(effects.is_none(), "Leeres Board sollte keine Y-Wing Effekte liefern");
    }

    #[test]
    fn test_y_wing_multiple() {
        let parser = Parse::grid().stop_on_error();
        let (board, _, failed) = parser.parse(
            "
            +-------+-------+-------+
            | 12  23  13 | 4  5  6 | 7  8  9 |
            | 1   2   3  | 4  5  6 | 7  8  9 |
            | 1   2   3  | 4  5  6 | 7  8  9 |
            +-------+-------+-------+
            "
        );
        assert_eq!(None, failed);

        let solver = YWingSolver;
        let effects = solver.apply(&board, false);
        assert!(effects.is_some(), "Y-Wing solver sollte Effekte finden");
        let effects = effects.unwrap();
        assert!(effects.actions().len() >= 1, "Es sollten mindestens eine Aktion erzeugt werden");
    }
}
