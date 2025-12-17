use std::collections::HashMap;
use super::*;

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

    // Group cells by number of candidates
    let pairs = group_cells_by_candidate_count(board, 2);
    let triples = group_cells_by_candidate_count(board, 3);
    let quads = group_cells_by_candidate_count(board, 4);

    // Track bi-value cells that see each other
    let seen_bi_values = seen_bi_value_cells(&pairs_by_candidates);
    let bi_values = board.cells_with_n_candidates(2);

    // Closure to check a potential WXYZ wing
    let mut check_wing = |wing: CellSet| -> bool {
        if !is_valid_wing(board, wing, &seen_bi_values, bi_values) {
            return false;
        }
        apply_wing_action(board, wing, &mut effects)
    };

    // Process quads
    for (_, quad_cells) in &quads {
        process_quads(quad_cells, &pairs, &triples, &mut check_wing, single, &mut effects);
    }

    // Process triples
    for (candidates, triple_cells) in &triples {
        process_triples(*candidates, triple_cells, &pairs, &triples, &mut check_wing, single, &mut effects);
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

// Groups cells by their candidate set size
fn group_cells_by_candidate_count(board: &Board, n: usize) -> HashMap<KnownSet, CellSet> {
    board.cell_candidates_with_n_candidates(n).fold(HashMap::new(), |mut map, (cell, candidates)| {
        *map.entry(candidates).or_default() += cell;
        map
    })
}

// Returns a map of bi-value cells that see each other
fn seen_bi_value_cells(pairs: &HashMap<KnownSet, CellSet>) -> HashMap<Cell, CellSet> {
    let mut seen: HashMap<Cell, CellSet> = HashMap::new();
    for cells in pairs.values() {
        for combo in cells.iter().combinations(2) {
            let (c1, c2) = (combo[0], combo[1]);
            if c1.sees(c2) {
                if c1 < c2 {
                    *seen.entry(c1).or_default() += c2;
                } else {
                    *seen.entry(c2).or_default() += c1;
                }
            }
        }
    }
    seen
}

// Determines if a cell set is a valid WXYZ wing
fn is_valid_wing(board: &Board, wing: CellSet, seen_bi_value_cells: &HashMap<Cell, CellSet>) -> bool {
    let bi_values = board.cells_with_n_candidates(2);
    if (wing & bi_values) == wing {
        return false;
    }
    if wing.share_any_house() {
        return false;
    }

    if (wing & bi_values).iter().any(|cell| {
        seen_bi_values.get(cell).map_or(false, |seen| !(*seen & wing).is_empty())
    }) {
        return false;
    }

    let wing_knowns = wing.iter().fold(KnownSet::empty(), |acc, cell| acc| board.candidates(cell));
    if wing_knowns.len() != 4 {
        return false;
    }
    if wing_knowns.iter().any(|k| (wing & board.candidate_cells(*k)).len() < 2) {
        return false;
    }

    true
}

// Applies a WXYZ wing action for a given cell set
fn apply_wing_action(board: &Board, wing: CellSet, effects: &mut Effects) -> bool {
    let wing_knowns = wing.iter().fold(KnownSet::empty(), |acc, cell| acc | board.candidates(cell));
    let mut restricted: HashMap<Known, CellSet> = HashMap::new();
    let mut non_restricted: HashMap<Known, CellSet> = HashMap::new();

    for k in wing_knowns {
        let candidates = wing & board.candidate_cells(k);
        let is_restricted = candidates.iter().combinations(2).all(|combo| combo[0].sees(combo[1]));
        if is_restricted {
            restricted.insert(k, candidates);
        } else {
            if !non_restricted.is_empty() {
                return false;
            }
            non_restricted.insert(k, candidates);
        }
    }
    if non_restricted.is_empty() {
        return false;
    }

    let (candidate, cells) = non_restricted.into_iter().next().unwrap();
    let erase = cells.iter().fold(board.candidate_cells(candidate), |set, cell| set & cell.peers());
    if erase.is_empty() {
        return false;
    }

    let mut action = Action::new_erase_cells(Strategy::WXYZWing, erase, candidate);
    action.clue_cells_for_known(Verdict::Secondary, cells, candidate);
    for (k, cells) in restricted {
        action.clue_cells_for_known(Verdict::Primary, cells, k);
    }

    effects.add_action(action)
}

// Process quad combinations for WXYZ wings
fn process_quads(
    quads: &CellSet,
    pairs: &HashMap<KnownSet, CellSet>,
    triples: &HashMap<KnownSet, CellSet>,
    check_wing: &mut impl FnMut(CellSet) -> bool,
    single: bool,
    effects: &mut Effects,
) {
    for quad_combo in quads.iter().combinations(4) {
        if check_wing(quad_combo.iter().copied().union_cells()) && single {
            return;
        }
    }

    // Processes triple combinations for WXYZ wings
    fn process_triples(
        _candidates: KnownSet,
        _triples: &CellSet,
        _pairs: &HashMap<KnownSet, CellSet>,
        _triples_by_candidates: &HashMap<KnownSet, CellSet>,
        _check_wing: &mut impl FnMut(CellSet) -> bool,
        _single: bool,
        _effects: &mut Effects,
    ) {
        
    }
}
