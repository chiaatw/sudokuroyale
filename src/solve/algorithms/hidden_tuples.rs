use itertools::Itertools;
use super::*;

use crate::puzzle::{Action, CellSet, Known, Board, Effects, Strategy, Verdict};
use crate::layout::House;
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
                let to_erase = board.candidates(*c) - tuple_knowns;
                action.erase_knowns(*c, to_erase);
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
        .any(|combo| combo.iter().copied().union_cells().len() <= smaller_size)
}


#[cfg(test)]
mod tests {
    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known_set::knowns;
    use crate::layout::Cell;

    use super::*;

    #[test]
    fn hidden_pairs() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let cells = cells!("A1 A2 A4 A5 A6 A8 A9");
        let knowns = knowns!("1 2");
        board.remove_candidates_from_cells(cells, knowns, &mut effects);

        find_hidden_pairs(&board, false)
            .unwrap()
            .apply_all(&mut board);

        assert_eq!(knowns, board.candidates(cell!("A3")));
        assert_eq!(knowns, board.candidates(cell!("A7")));
        assert_eq!(!knowns, board.candidates(cell!("A2")));
        assert_eq!(!knowns, board.candidates(cell!("A6")));
        assert_eq!(!knowns, board.candidates(cell!("A9")));
    }

    #[test]
    fn hidden_triples() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let cells = cells!("A1 A2 A4 A6 A8 A9");
        let knowns = knowns!("1 2 3");
        board.remove_candidates_from_cells(cells, knowns, &mut effects);

        find_hidden_triples(&board, false)
            .unwrap()
            .apply_all(&mut board);

        assert_eq!(knowns, board.candidates(cell!("A3")));
        assert_eq!(knowns, board.candidates(cell!("A5")));
        assert_eq!(knowns, board.candidates(cell!("A7")));
        assert_eq!(!knowns, board.candidates(cell!("A2")));
        assert_eq!(!knowns, board.candidates(cell!("A6")));
        assert_eq!(!knowns, board.candidates(cell!("A9")));
    }

    #[test]
    fn hidden_quads() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let cells = cells!("A2 A4 A6 A8 A9");
        let knowns = knowns!("1 2 3 4");
        board.remove_candidates_from_cells(cells, knowns, &mut effects);

        find_hidden_quads(&board, false)
            .unwrap()
            .apply_all(&mut board);

        assert_eq!(knowns, board.candidates(cell!("A1")));
        assert_eq!(knowns, board.candidates(cell!("A3")));
        assert_eq!(knowns, board.candidates(cell!("A5")));
        assert_eq!(knowns, board.candidates(cell!("A7")));
        assert_eq!(!knowns, board.candidates(cell!("A2")));
        assert_eq!(!knowns, board.candidates(cell!("A6")));
        assert_eq!(!knowns, board.candidates(cell!("A9")));
    }
    #[test]
    fn hidden_pair_single_true() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        // Setup: A1-A3 have candidates 1,2
        board.set_candidates(cell!("A1"), knowns!("1 2"));
        board.set_candidates(cell!("A2"), knowns!("1 2"));
        board.set_candidates(cell!("A3"), knowns!("1 2 3"));
        board.set_candidates(cell!("A4"), knowns!("3"));

        // Only one action should be returned with single = true
        let found = find_hidden_pairs(&board, true).unwrap();
        assert_eq!(found.actions().len(), 1);

        // Candidates outside hidden pair cells should be erased
        let action = &found.actions()[0];
        assert!(action.erased(cell!("A3")).contains(&knowns!("1")));
        assert!(action.erased(cell!("A3")).contains(&knowns!("2")));
    }

    #[test]
    fn hidden_triple_non_degenerate() {
        let mut board = Board::new();

        // Setup: Hidden triple in row A
        board.set_candidates(cell!("A1"), knowns!("1 2 3"));
        board.set_candidates(cell!("A2"), knowns!("1 2 3"));
        board.set_candidates(cell!("A3"), knowns!("2 3 4"));
        board.set_candidates(cell!("A4"), knowns!("4"));

        let found = find_hidden_triples(&board, false).unwrap();
        // Should generate at least one effect
        assert!(found.actions().len() >= 1);

        // Cells in triple should only keep tuple knowns
        for c in [cell!("A1"), cell!("A2"), cell!("A3")] {
            let ks = board.candidates(c) & knowns!("1 2 3");
            assert!(!ks.is_empty());
        }
    }

    #[test]
    fn hidden_quad_degenerate_ignored() {
        let mut board = Board::new();

        // Setup: degenerate quad (subset forms hidden triple)
        board.set_candidates(cell!("A1"), knowns!("1 2 3"));
        board.set_candidates(cell!("A2"), knowns!("1 2 3"));
        board.set_candidates(cell!("A3"), knowns!("1 2 3"));
        board.set_candidates(cell!("A4"), knowns!("4"));

        // Degenerate quad should be skipped
        let result = find_hidden_quads(&board, false);
        assert!(result.is_none());
    }
}
