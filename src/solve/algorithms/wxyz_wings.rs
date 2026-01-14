use std::collections::HashMap;
use itertools::Itertools;

use super::*;

/// Solver interface implemented by all strategies
pub trait Solver {
/// Returns the strategy identifier
    fn strategy(&self) -> Strategy;

/// Applies the strategy to the board
/// 
/// If single is true, the solver stops after the first successful action
    fn apply(&self, board: &Board, single: bool) -> Option<Effects>;
}

/// WXYZ-Wing strategy solver
/// 
/// This solver detects WXYZ-Wing patters consisting of combinations of bi-value, tri-value and quad-value cells
/// and produces candidate eliminations based on restricted and non-restricted candidates
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

// Finds all WXYZ-Wing patterns on the board and returns their effects
pub fn find_wxyz_wings(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

// Group cells by exact candidate sets
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

// Quad-based WXYZ-Wing candidate sets
    let quad_sets = quads_by_candidates
        .iter()
        .map(|(candidates, cells)| {
            (
                *candidates,
                *cells,
                triples_by_candidates
                    .iter()
                    .filter(|(c, _)| c.is_subset_of(*candidates))
                    .map(|(_, cells)| *cells)
                    .union_cells()
                    | pairs_by_candidates
                        .iter()
                        .filter(|(c, _)| c.is_subset_of(*candidates))
                        .map(|(_, cells)| *cells)
                        .union_cells(),
            )
        })
        .collect_vec();

// Triple based WXYZ-Wing candidate sets with disjoint grouping
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

// Tracks bi-value cells that see each other
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

// Validates a WXYZ-wIng and applies its action if found
    let mut check_wing = |wing: CellSet| -> bool {
// Ignore XY chains
        if (wing & bi_values) == wing {
            return false;
        }

// Ignore naked quads
        if wing.share_any_house() {
            return false;
        }

// Ignore naked pairs
        if (wing & bi_values).iter().any(|cell| {
            seen_bi_values
                .get(&cell)
                .map_or(false, |seen| !(*seen & wing).is_empty())
        }) {
            return false;
        }

        let wing_knowns = wing.iter().fold(KnownSet::empty(), |set, cell| set | board.candidates(cell));

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

            for known in wing_knowns {
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
            let erase = cells.iter().fold(board.candidate_cells(candidate), |set, cell| {
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
    use crate::io::Parse;

    #[test]
    fn test_wxyz_wing_basic() {
        let parser = Parse::grid().stop_on_error();
        let (board, _, failed) = parser.parse(
            "
            +-------+-------+-------+
            | 12 34 | 56 78 | 9 . . |
            | .  .  | 12 34 | 56 78 |
            | .  .  | .  .  | 12 34 |
            +-------+-------+-------+
            "
        );
        assert_eq!(None, failed);

        let solver = WXYZWingSolver;
        let effects = solver.apply(&board, true);
        assert!(effects.is_some(), "WXYZ-Wing sollte gefunden werden");
        let effects = effects.unwrap();
        assert!(effects.has_actions(), "WXYZ-Wing-Effekte sollten Aktionen enthalten");
    }

    #[test]
    fn test_wxyz_wing_none() {
        let board = Board::new(); // komplett leeres Board
        let solver = WXYZWingSolver;
        let effects = solver.apply(&board, true);
        assert!(effects.is_none(), "Kein WXYZ-Wing sollte None zurückgeben");
    }

    #[test]
    fn test_wxyz_wing_multiple() {
        let parser = Parse::grid().stop_on_error();
        let (board, _, failed) = parser.parse(
            "
            +-------+-------+-------+
            | 12 34 | 56 78 | 9 . . |
            | 12 34 | 56 78 | 9 . . |
            | .  .  | 12 34 | 56 78 |
            +-------+-------+-------+
            "
        );
        assert_eq!(None, failed);

        let solver = WXYZWingSolver;
        let effects = solver.apply(&board, false);
        assert!(effects.is_some(), "Mehrere WXYZ-Wings sollten erkannt werden");
        let effects = effects.unwrap();
        assert!(effects.actions().len() > 1, "Mehrere Aktionen sollten vorhanden sein");
    }

    #[test]
    fn test_wxyz_wing_clues() {
        let parser = Parse::grid().stop_on_error();
        let (board, _, failed) = parser.parse(
            "
            +-------+-------+-------+
            | 12 34 | 56 78 | 9 . . |
            | .  .  | 12 34 | 56 78 |
            | .  .  | .  .  | 12 34 |
            +-------+-------+-------+
            "
        );
        assert_eq!(None, failed);

        let solver = WXYZWingSolver;
        let effects = solver.apply(&board, true).unwrap();
        let action = &effects.actions()[0];
        assert!(!action.secondary_clues().is_empty(), "Secondary clues sollten gesetzt sein");
        assert!(!action.primary_clues().is_empty(), "Primary clues sollten gesetzt sein");
    }
}
