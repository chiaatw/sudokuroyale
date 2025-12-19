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