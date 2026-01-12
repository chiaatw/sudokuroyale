use super::*;

// Solver wrapper for the XYZ-Wing strategy
pub struct XYZWingSolver;

impl Solver for XYZWingSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::XYZWing
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        self.find_xyz_wings(board, single)
    }
}

impl XYZWingSolver {
    fn find_xyz_wings(&self, board: &Board, single: bool) -> Option<Effects> {
        let mut effects = Effects::new();

        let tri_values = board.cells_with_n_candidates(3);
        if tri_values.is_empty() {
            return None;
        }

        let bi_values = board.cells_with_n_candidates(2);
        if bi_values.is_empty() {
            return None;
        }

        for pivot in tri_values {
            let pivot_peers = pivot.peers();

            for pair in (pivot_peers & bi_values)
                .iter()
                .combinations(2)
                .map(|pair| pair.iter().copied().union_cells())
            {
                let (c1, c2) = pair.as_pair().expect("cell pair");

                let candidates = pivot_peers & c1.peers() & c2.peers();
                if candidates.len() != 2 {
// degenerate naked triple
                    continue;
                }

                let ks = board.candidates(pivot);
                let ks1 = board.candidates(c1);
                let ks2 = board.candidates(c2);

                if ks1 | ks2 != ks {
// degenerate naked pair or unrelated candidates
                    continue;
                }

                let k = (ks1 & ks2)
                    .as_single()
                    .expect("one candidate in common");

                let mut action = Action::new(Strategy::XYZWing);
                action.erase_cells(candidates & board.candidate_cells(k), k);
                action.clue_cells_for_known(Verdict::Secondary, pair + pivot, k);
                action.clue_cell_for_knowns(Verdict::Primary, pivot, ks1 - k);
                action.clue_cell_for_knowns(Verdict::Primary, pivot, ks2 - k);
                action.clue_cell_for_knowns(Verdict::Primary, c1, ks1 - k);
                action.clue_cell_for_knowns(Verdict::Primary, c2, ks2 - k);

                if effects.add_action(action) && single {
                    return Some(effects);
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

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::known;
    use super::*;

    #[test]
    fn test_xyz_wing_basic() {
        let parser = Parse::wiki().stop_on_error();
        let (board, _, failed) = parser.parse(
            "814kg10s2u246c116e110922812m41i42mg42i4k621sg134812m6e05g10h215081030950418128g11c0334240h2803114c4c0h64g181gq4g055g81j0j822jagg1181032k09g441i4ga214a5454h40h81he"
        );
        assert_eq!(None, failed);

        let solver = XYZWingSolver;
        if let Some(got) = solver.apply(&board, true) {
            let mut expected = Action::new(Strategy::XYZWing);
            expected.erase_cells(cells!("B7 H9"), known!("6"));
            expected.clue_cells_for_known(Verdict::Secondary, cells!("A9 C9 G7 B7 H9"), known!("6"));
            expected.clue_cell_for_knowns(Verdict::Primary, cell!("A9"), knowns!("2 9"));
            expected.clue_cell_for_knowns(Verdict::Primary, cell!("C9"), knowns!("2 9"));
            expected.clue_cell_for_knowns(Verdict::Primary, cell!("G7"), knowns!("2 9"));
            expected.clue_cell_for_knowns(Verdict::Primary, cell!("B7"), knowns!("2 9"));
            expected.clue_cell_for_knowns(Verdict::Primary, cell!("H9"), knowns!("2 9"));

            assert_eq!(format!("{:?}", expected), format!("{:?}", got.actions()[0]));
        } else {
            panic!("XYZ-Wing solver found no effects");
        }
    }

    #[test]
    fn test_xyz_wing_none() {
        let board = crate::layout::Board::new(); // leeres Board
        let solver = XYZWingSolver;
        let effects = solver.apply(&board, true);
        assert!(effects.is_none(), "Leeres Board sollte keine XYZ-Wing Effekte liefern");
    }

    #[test]
    fn test_xyz_wing_multiple() {
        let parser = Parse::grid().stop_on_error();
        let (board, _, failed) = parser.parse(
            "
            +-------+-------+-------+
            | 123  23  13 | 45  45  6 | 7  8  9 |
            | 12   3   13 | 45  45  6 | 7  8  9 |
            | 1    2   3  | 4   5   6 | 7  8  9 |
            +-------+-------+-------+
            "
        );
        assert_eq!(None, failed);

        let solver = XYZWingSolver;
        let effects = solver.apply(&board, false);
        assert!(effects.is_some(), "XYZ-Wing solver sollte Effekte finden");
        let effects = effects.unwrap();
        assert!(effects.actions().len() >= 1, "Es sollten mindestens eine Aktion erzeugt werden");
    }
}
