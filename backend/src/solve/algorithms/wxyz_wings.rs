use itertools::Itertools;
use std::collections::HashMap;

use crate::layout::cells::cell_set::{CellIteratorUnion, CellSetIteratorUnion};
use crate::layout::values::known_set::KnownSetLike;
use crate::puzzle::{Action, Board, Cell, CellSet, Effects, Known, KnownSet, Strategy, Verdict};

pub trait Solver {
    fn strategy(&self) -> Strategy;

    fn apply(&self, board: &Board, single: bool) -> Option<Effects>;
}

pub struct WXYZWingSolver;

impl Solver for WXYZWingSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::WXYZWing
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_wxyz_wings(board, single)
    }
}

pub fn find_wxyz_wings(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    let pairs_by_candidates = board.cell_candidates_with_n_candidates(2).fold(
        HashMap::new(),
        |mut map: HashMap<KnownSet, CellSet>, (cell, candidates)| {
            *map.entry(candidates).or_default() += cell;
            map
        },
    );

    let triples_by_candidates = board.cell_candidates_with_n_candidates(3).fold(
        HashMap::new(),
        |mut map: HashMap<KnownSet, CellSet>, (cell, candidates)| {
            *map.entry(candidates).or_default() += cell;
            map
        },
    );

    let quads_by_candidates = board.cell_candidates_with_n_candidates(4).fold(
        HashMap::new(),
        |mut map: HashMap<KnownSet, CellSet>, (cell, candidates)| {
            *map.entry(candidates).or_default() += cell;
            map
        },
    );

    let quad_sets = quads_by_candidates
        .iter()
        .map(|(candidates, cells)| {
            (
                *candidates,
                *cells,
                triples_by_candidates
                    .iter()
                    .filter(|(c, _)| (**c).is_subset_of(*candidates))
                    .map(|(_, cells)| *cells)
                    .union_cells()
                    | pairs_by_candidates
                        .iter()
                        .filter(|(c, _)| (**c).is_subset_of(*candidates))
                        .map(|(_, cells)| *cells)
                        .union_cells(),
            )
        })
        .collect_vec();

    let triple_sets = triples_by_candidates
        .iter()
        .map(|(candidates, cells)| {
            let triples_with_two_common_candidates =
                triples_by_candidates
                    .iter()
                    .fold(HashMap::new(), |mut acc, (ks, cs)| {
                        let diff = *ks - *candidates;
                        if let Some(single) = diff.as_single() {
                            *acc.entry(single).or_insert_with(CellSet::empty) |= *cs;
                        }
                        acc
                    });

            (
                *candidates,
                *cells,
                pairs_by_candidates.iter().fold(
                    triples_with_two_common_candidates,
                    |mut acc, (ks, cs)| {
                        let diff = *ks - *candidates;
                        if let Some(single) = diff.as_single() {
                            *acc.entry(single).or_insert_with(CellSet::empty) |= *cs;
                        }
                        acc
                    },
                ),
                pairs_by_candidates
                    .iter()
                    .filter(|(ks, _)| ks.is_subset_of(*candidates))
                    .map(|(_, cells)| *cells)
                    .union_cells(),
            )
        })
        .collect_vec();

    let seen_bi_values: HashMap<Cell, CellSet> =
        pairs_by_candidates
            .iter()
            .fold(HashMap::new(), |mut map, (_, cells)| {
                cells.iter().combinations(2).for_each(|combo| {
                    let (c1, c2) = (combo[0], combo[1]);
                    if c1.sees(c2) {
                        if c1 < c2 {
                            *map.entry(c1).or_default() += c2;
                        } else {
                            *map.entry(c2).or_default() += c1;
                        }
                    }
                });
                map
            });

    let bi_values = board.cells_with_n_candidates(2);

    let mut check_wing = |wing: CellSet| -> bool {
        if (wing & bi_values) == wing {
            return false;
        }

        if wing.share_any_house() {
            return false;
        }

        if (wing & bi_values).iter().any(|cell| {
            seen_bi_values
                .get(&cell)
                .map_or(false, |seen| !(*seen & wing).is_empty())
        }) {
            return false;
        }

        let wing_knowns = wing
            .iter()
            .fold(KnownSet::empty(), |set, cell| set | board.candidates(cell));

        if wing_knowns.len() != 4 {
            return false;
        }

        if wing_knowns
            .iter()
            .any(|known| (wing & board.candidate_cells(known)).len() < 2)
        {
            return false;
        }

        let mut restricted: HashMap<Known, CellSet> = HashMap::new();
        let mut non_restricted: HashMap<Known, CellSet> = HashMap::new();

        for known in wing_knowns.iter() {
            let candidates = wing & board.candidate_cells(known);
            let is_restricted = candidates.iter().combinations(2).all(|c| c[0].sees(c[1]));

            if is_restricted {
                restricted.insert(known, candidates);
            } else {
                if !non_restricted.is_empty() {
                    return false;
                }
                non_restricted.insert(known, candidates);
            }
        }

        if non_restricted.is_empty() {
            return false;
        }

        let (candidate, cells) = non_restricted.into_iter().next().unwrap();
        let erase = cells
            .iter()
            .fold(board.candidate_cells(candidate), |set, cell| {
                set & cell.peers()
            });

        if erase.is_empty() {
            return false;
        }

        let mut action = Action::new_erase_cells(Strategy::WXYZWing, erase, candidate);
        action.clue_cells_for_known(Verdict::Secondary, cells, candidate);

        for (known, cells) in restricted {
            action.clue_cells_for_known(Verdict::Primary, cells, known);
        }

        effects.add_action(action)
    };

    // Quad-driven WXYZ-Wings
    for (_, quads, subsets) in quad_sets {
        for quad_combo in quads.iter().combinations(4) {
            if check_wing(quad_combo.iter().copied().union_cells()) && single {
                return Some(effects);
            }
        }

        for n in (2..4).rev() {
            for quad_combo in quads.iter().combinations(n) {
                let base = quad_combo.iter().copied().union_cells();
                for others in subsets.iter().combinations(4 - n) {
                    if check_wing(base | others.iter().copied().union_cells()) && single {
                        return Some(effects);
                    }
                }
            }
        }

        for quad in quads {
            for others in subsets.iter().combinations(3) {
                if check_wing(others.iter().copied().union_cells() + quad) && single {
                    return Some(effects);
                }
            }
        }
    }

    // Triple-driven WXYZ-Wings
    for (candidates, triples, disjoints, subsets) in triple_sets {
        for triple_combo in triples.iter().combinations(4) {
            if check_wing(triple_combo.iter().copied().union_cells()) && single {
                return Some(effects);
            }
        }

        for n in (1..4).rev() {
            for triple_combo in triples.iter().combinations(n) {
                let base = triple_combo.iter().copied().union_cells();
                for k in (!candidates).iter() {
                    if let Some(disjoint) = disjoints.get(&k) {
                        for others in (*disjoint | subsets).iter().combinations(4 - n) {
                            if check_wing(base | others.iter().copied().union_cells()) && single {
                                return Some(effects);
                            }
                        }
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
mod tests {
    use super::*;

    #[test]
    fn empty_board_returns_none() {
        let board = Board::new();
        assert!(find_wxyz_wings(&board, false).is_none());
    }

    #[test]
    fn solver_delegates_to_find() {
        let board = Board::new();
        let solver = WXYZWingSolver;

        let via_solver = solver.apply(&board, false);
        let via_fn = find_wxyz_wings(&board, false);

        assert_eq!(via_solver.is_some(), via_fn.is_some());
    }

    #[test]
    fn single_mode_never_returns_more_than_one_action() {
        let board = Board::new();

        if let Some(effects) = find_wxyz_wings(&board, true) {
            assert!(
                effects.actions().len() <= 1,
                "single=true darf höchstens eine Action liefern"
            );
        }
    }

    #[test]
    fn no_panic_on_empty_board_multiple_mode() {
        let board = Board::new();
        let _ = find_wxyz_wings(&board, false);
    }
}
