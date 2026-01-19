use itertools::Itertools;
use super::*;

use crate::puzzle::{Action, CellSet, Known, Board, Effects, Strategy, Verdict};
use crate::layout::House;
use crate::layout::cells::cell_set::CellSetIteratorUnion;
use crate::layout::values::known_set::KnownIteratorUnion;
// Solver wrapper for Hidden Pair strategy
pub struct HiddenPairSolver;

impl Solver for HiddenPairSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::HiddenPair
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_hidden_pairs(board, single)
    }
}

// Solver wrapper for Hidden Triple strategy
pub struct HiddenTripleSolver;

impl Solver for HiddenTripleSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::HiddenTriple
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_hidden_triples(board, single)
    }
}

pub struct HiddenQuadSolver;

impl Solver for HiddenQuadSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::HiddenQuad
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_hidden_quads(board, single)
    }
}

// Entry function for Hidden Pair strategy
pub fn find_hidden_pairs(board: &Board, single: bool) -> Option<Effects> {
    find_hidden_tuples(board, single, 2, Strategy::HiddenPair)
}

// Entry function for Hidden Triple strategy
pub fn find_hidden_triples(board: &Board, single: bool) -> Option<Effects> {
    find_hidden_tuples(board, single, 3, Strategy::HiddenTriple)
}

// Entry function for Hidden Quad strategy
pub fn find_hidden_quads(board: &Board, single: bool) -> Option<Effects> {
    find_hidden_tuples(board, single, 4, Strategy::HiddenQuad)
}

/// Generic logic for detecting hidden tuples in a house
/// size: 2 for pairs, 3 for triples, 4 for quads
/// Filters degenerate sets to avoid partial overlaps
/// Returns Effects representing candidate erasures and clues
pub fn find_hidden_tuples(
    board: &Board,
    single: bool,
    size: usize,
    strategy: Strategy,
) -> Option<Effects> {
    let mut effects = Effects::new();

    for house in House::iter() {
// Collect all candidate sets for each known in the house
        for candidates in Known::iter()
            .map(|k| (k, house.cells() & board.candidate_cells(k)))
            .filter(|(_, cs)| 2 <= cs.len() && cs.len() <= size)
            .combinations(size)
        {
            let cell_sets: Vec<CellSet> = candidates.iter().map(|(_, cs)| *cs).collect();
            let tuple_cells = cell_sets.iter().copied().union_cells();

// Skip degenerate tuples or invalid sizes
            if tuple_cells.len() != size || is_degenerate(&cell_sets, size, 2) || is_degenerate(&cell_sets, size, 3)
            {
                continue;
            }

            let tuple_knowns = candidates.iter().map(|(k, _)| *k).union_knowns();
            let mut action = Action::new(strategy);

// Apply candidate erasures outside of hidden tuple
            tuple_cells.iter().for_each(|c| {
                let to_erase = board.candidates(c) - tuple_knowns;
                action.erase_knowns(c, to_erase);
            });

// Apply clues for knowns in the tuple
            tuple_knowns.iter().for_each(|k| {
                action.clue_cells_for_known(
                    Verdict::Secondary,
                    board.house_candidate_cells(house, k),
                    k,
                );
            });

// Apply related clues for cells outside the tuple
            (house.cells() - tuple_cells).iter().for_each(|c| {
                action.clue_cell_for_knowns(Verdict::Related, c, tuple_knowns);
            });

// Add action and early exit if only a single effect is desired
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

/// Determines whether a combination of candidate cell sets forms a degenerate tuple
/// Degenerate tuples are subsets of smaller sizes that would conflict with the tuple logic
pub fn is_degenerate(cell_sets: &[CellSet], size: usize, smaller_size: usize) -> bool {
    size > smaller_size && cell_sets
        .iter()
        .combinations(smaller_size)
        .any(|combo| combo.iter().map(|cs| **cs).union_cells().len() <= smaller_size)
}


#[cfg(test)]
mod tests {
    use super::*;

    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::Known;
    use crate::layout::values::known_set::{KnownSet, KnownSetLike};

    // Local helper macro: build a KnownSet from a string like "1 2 3"
    // (keeps tests independent from any missing global `knowns!` macro)
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
    fn hidden_pairs() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        // Make (1,2) a hidden pair in row A at A3 & A7 by removing {1,2}
        // from all other cells in the row.
        let cs = cells!("A1 A2 A4 A5 A6 A8 A9");
        let ks = knowns!("1 2");
        board.remove_candidates_from_cells(cs, ks, &mut effects);

        let found = find_hidden_pairs(&board, false).expect("expected hidden pair effects");
        found.apply_all(&mut board);

        assert_eq!(ks, board.candidates(cell!("A3")));
        assert_eq!(ks, board.candidates(cell!("A7")));

        // Other cells in the row must not contain 1 or 2 anymore
        for c in cells!("A1 A2 A4 A5 A6 A8 A9").iter() {
            assert!(!board.candidates(c).has_any(ks));
        }
    }

    #[test]
    fn hidden_triples() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        // Make (1,2,3) a hidden triple in row A at A3,A5,A7 by removing {1,2,3}
        // from all other cells in the row.
        let cs = cells!("A1 A2 A4 A6 A8 A9");
        let ks = knowns!("1 2 3");
        board.remove_candidates_from_cells(cs, ks, &mut effects);

        let found = find_hidden_triples(&board, false).expect("expected hidden triple effects");
        found.apply_all(&mut board);

        assert_eq!(ks, board.candidates(cell!("A3")));
        assert_eq!(ks, board.candidates(cell!("A5")));
        assert_eq!(ks, board.candidates(cell!("A7")));

        for c in cells!("A1 A2 A4 A6 A8 A9").iter() {
            assert!(!board.candidates(c).has_any(ks));
        }
    }

    #[test]
    fn hidden_quads() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        // Make (1,2,3,4) a hidden quad in row A at A1,A3,A5,A7 by removing {1,2,3,4}
        // from all other cells in the row.
        let cs = cells!("A2 A4 A6 A8 A9");
        let ks = knowns!("1 2 3 4");
        board.remove_candidates_from_cells(cs, ks, &mut effects);

        let found = find_hidden_quads(&board, false).expect("expected hidden quad effects");
        found.apply_all(&mut board);

        assert_eq!(ks, board.candidates(cell!("A1")));
        assert_eq!(ks, board.candidates(cell!("A3")));
        assert_eq!(ks, board.candidates(cell!("A5")));
        assert_eq!(ks, board.candidates(cell!("A7")));

        for c in cells!("A2 A4 A6 A8 A9").iter() {
            assert!(!board.candidates(c).has_any(ks));
        }
    }

    #[test]
    fn hidden_pair_single_true_returns_one_action() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let cs = cells!("A1 A2 A4 A5 A6 A8 A9");
        let ks = knowns!("1 2");
        board.remove_candidates_from_cells(cs, ks, &mut effects);

        let found = find_hidden_pairs(&board, true).expect("expected hidden pair effects");
        assert_eq!(found.actions().len(), 1);

        found.apply_all(&mut board);
        assert_eq!(ks, board.candidates(cell!("A3")));
        assert_eq!(ks, board.candidates(cell!("A7")));
    }

    #[test]
    fn is_degenerate_detects_smaller_subset() {
        // Construct a clearly-degenerate "quad": three sets already fit in 3 cells
        // (i.e., it degenerates to a triple).
        let a = cells!("A1 A2 A3");
        let b = cells!("A1 A2 A3");
        let c = cells!("A1 A2 A3");
        let d = cells!("A1 A2 A3 A4");

        let sets = vec![a, b, c, d];

        assert!(is_degenerate(&sets, 4, 3));
    }
}
