use itertools::Itertools;
use super::*;

// Solver for Naked Pair strategy
pub struct NakedPairSolver;

impl Solver for NakedPairSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::NakedPair
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_naked_tuples(board, single, 2, Strategy::NakedPair)
    }
}

// Solver for Naked Triple Strategy
pub struct NakedTripleSolver;

impl Solver for NakedTripleSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::NakedTriple
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_naked_tuples(board, single, 3, Strategy::NakedTriple)
    }
}

// Solver for Naked Quad strategy
pub struct NakedQuadSolver;

impl Solver for NakedQuadSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::NakedQuad
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_naked_tuples(board, single, 4, Strategy::NakedQuad)
    }
}

/// Generic logic for detecting naked tuples in a house
/// size: 2 for pairs, 3 for triples, 4 for quads
/// Returns Effects representing candidate erasure and clues
fn find_naked_tuples(board: &Board, single: bool, size: usize, strategy: Strategy) -> Option<Effects> {
    let mut effects = Effects::new();

    for house in House::iter() {
        let house_cells = house.cells();

// Generate all combinations of cells in the house that could form a naked tuple
        for candidates in house_cells
            .iter()
            .map(|cell| (*cell, board.candidates(cell)))
            .filter(|(_, candidates)| 2 <= candidates.len() && candidates.len() <= size)
            .combinations(size)
        {
            let known_sets = candidates.iter().map(|(_, ks)| *ks).collect_vec();
            let tuple_knowns = known_sets.iter().copied().union_knowns();

// Skip if the combined knowns don't match the tuple size or are degenerate
            if tuple_knowns.len() != size || is_degenerate(&known_sets, size, 2) || is_degenerate(&known_sets, size, 3)
            {
                continue;
            }

            let tuple_cells = candidates.iter().map(|(c, _)| *c).union_cells();
            let erase_cells = house_cells - tuple_cells;
            let mut action = Action::new(strategy);

// Erase knowns outside of the naked tuple and mark clues
            tuple_knowns.iter().for_each(|k| {
                action.erase_cells(erase_cells & board.candidate_cells(k), *k);
                action.clue_cells_for_known(
                    Verdict::Secondary,
                    tuple_cells & board.candidate_cells(*k),
                    *k,
                );
            });

// Provide related clues for cells in the tuple
            tuple_cells.iter().for_each(|c| {
                action.clue_cell_for_knowns(
                    Verdict::Related,
                    *c,
                    KnownSet::full() - board.candidates(*c),
                );
            });

// Add action and optionally return early if only a single effect is desired
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

/// Determine whether a set of candidate sets forms a degenerate tuple 
/// Degenerate tuples are subsets of smaller sizes that conflict with the tuple logic
pub fn is_degenerate(known_sets: &[KnownSet], size: usize, smaller_size: usize) -> bool {
    size > smaller_size && known_sets
        .iter()
        .combinations(smaller_size)
        .map(|sets| sets.into_iter().copied().union_knowns())
        .any(|set| set.len() <= smaller_size)
}

#[cfg(test)]
mod tests {
    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known_set::knowns;
    use super::*;

    #[test]
    fn naked_pairs() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let knowns = knowns!("1 2 3 4 5 6 7");
        board.remove_candidates_from_cells(cells!("A1 A2"), knowns, &mut effects);

        find_naked_tuples(&board, false, 2, Strategy::NakedPair)
            .unwrap()
            .apply_all(&mut board);

        assert_eq!(!knowns, board.candidates(cell!("A1")));
        assert_eq!(!knowns, board.candidates(cell!("A2")));
        assert_eq!(knowns, board.candidates(cell!("A5")));
        assert_eq!(knowns, board.candidates(cell!("B3")));
        assert_eq!(knowns, board.candidates(cell!("C2")));
    }

    #[test]
    fn naked_triples() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let knowns = knowns!("1 2 3 4 5 6");
        board.remove_candidates_from_cells(cells!("A1 A2 A5"), knowns, &mut effects);

        find_naked_tuples(&board, false, 3, Strategy::NakedTriple)
            .unwrap()
            .apply_all(&mut board);

        assert_eq!(!knowns, board.candidates(cell!("A1")));
        assert_eq!(knowns, board.candidates(cell!("A8")));
        assert_eq!(KnownSet::full(), board.candidates(cell!("B3")));
        assert_eq!(KnownSet::full(), board.candidates(cell!("C2")));
    }

    #[test]
    fn naked_quads() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let knowns = knowns!("1 2 3 4 5");
        board.remove_candidates_from_cells(cells!("A1 A2 A5 A8"), knowns, &mut effects);

        find_naked_tuples(&board, false, 4, Strategy::NakedQuad)
            .unwrap()
            .apply_all(&mut board);

        assert_eq!(!knowns, board.candidates(cell!("A1")));
        assert_eq!(!knowns, board.candidates(cell!("A2")));
        assert_eq!(knowns, board.candidates(cell!("A9")));
        assert_eq!(KnownSet::full(), board.candidates(cell!("B3")));
        assert_eq!(KnownSet::full(), board.candidates(cell!("C2")));
    }
    #[test]
    fn naked_pair_detection() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        // Set up a naked pair in a row: A1 and A2 with only candidates 1 and 2
        board.set_candidates(cell!("A1"), knowns!("1 2"), &mut effects);
        board.set_candidates(cell!("A2"), knowns!("1 2"), &mut effects);
        board.set_candidates(cell!("A3"), knowns!("1 2 3"), &mut effects);

        let found = find_naked_tuples(&board, false, 2, Strategy::NakedPair).unwrap();
        found.apply_all(&mut board);

        // Only A1 and A2 form the naked pair, so 1 and 2 should be removed from A3
        assert_eq!(knowns!("3"), board.candidates(cell!("A3")));
    }

    #[test]
    fn naked_triple_detection() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        // Set up a naked triple: B1, B2, B3 with candidates 4,5,6
        board.set_candidates(cell!("B1"), knowns!("4 5"), &mut effects);
        board.set_candidates(cell!("B2"), knowns!("4 6"), &mut effects);
        board.set_candidates(cell!("B3"), knowns!("5 6"), &mut effects);
        board.set_candidates(cell!("B4"), knowns!("4 5 6 7"), &mut effects);

        let found = find_naked_tuples(&board, false, 3, Strategy::NakedTriple).unwrap();
        found.apply_all(&mut board);

        // The naked triple should remove 4,5,6 from B4
        assert_eq!(knowns!("7"), board.candidates(cell!("B4")));
    }

    #[test]
    fn naked_quad_detection() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        // Set up a naked quad in column C
        board.set_candidates(cell!("C1"), knowns!("1 2"), &mut effects);
        board.set_candidates(cell!("C2"), knowns!("1 3"), &mut effects);
        board.set_candidates(cell!("C3"), knowns!("2 4"), &mut effects);
        board.set_candidates(cell!("C4"), knowns!("3 4"), &mut effects);
        board.set_candidates(cell!("C5"), knowns!("1 2 3 4 5"), &mut effects);

        let found = find_naked_tuples(&board, false, 4, Strategy::NakedQuad).unwrap();
        found.apply_all(&mut board);

        // The naked quad should remove 1,2,3,4 from C5
        assert_eq!(knowns!("5"), board.candidates(cell!("C5")));
    }
}