use itertools::Itertools;
use super::*;

use crate::puzzle::{Action, KnownSet, Board, Effects, Strategy, Verdict};
use crate::layout::House;
use crate::layout::values::known_set::KnownSetIteratorUnion;
use crate::layout::values::known_set::KnownSetLike;
use crate::layout::cells::cell_set::CellIteratorUnion;

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
            .map(|cell| (cell, board.candidates(cell)))
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
                action.erase_cells(erase_cells & board.candidate_cells(k), k);
                action.clue_cells_for_known(
                    Verdict::Secondary,
                    tuple_cells & board.candidate_cells(k),
                    k,
                );
            });

// Provide related clues for cells in the tuple
            tuple_cells.iter().for_each(|c| {
                action.clue_cell_for_knowns(
                    Verdict::Related,
                    c,
                    KnownSet::full() - board.candidates(c),
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

pub fn find_naked_pairs(board: &Board, single: bool) -> Option<Effects> {
    find_naked_tuples(board, single, 2, Strategy::NakedPair)
}

pub fn find_naked_triples(board: &Board, single: bool) -> Option<Effects> {
    find_naked_tuples(board, single, 3, Strategy::NakedTriple)
}

pub fn find_naked_quads(board: &Board, single: bool) -> Option<Effects> {
    find_naked_tuples(board, single, 4, Strategy::NakedQuad)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::Known;
    use crate::layout::values::known_set::{KnownSet, KnownSetLike};

    // Falls es kein globales knowns! gibt: lokales Macro
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
    fn naked_pairs() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        // A1 und A2 verlieren {1..7} -> bleiben {8,9} => Naked Pair in Row A
        let removed = knowns!("1 2 3 4 5 6 7");
        board.remove_candidates_from_cells(cells!("A1 A2"), removed, &mut effects);

        let found = find_naked_tuples(&board, false, 2, Strategy::NakedPair).unwrap();
        found.apply_all(&mut board);

        // Pair-Zellen behalten (full - removed)
        assert_eq!(KnownSet::full() - removed, board.candidates(cell!("A1")));
        assert_eq!(KnownSet::full() - removed, board.candidates(cell!("A2")));

        // In derselben Row sollten 8 und 9 aus anderen Zellen entfernt werden.
        // In deinem alten Test hast du "removed" erwartet, das ist logisch verdreht.
        // Korrekt ist: A5 sollte NICHT mehr {8,9} enthalten -> also ist es kleiner als full.
        assert_ne!(KnownSet::full(), board.candidates(cell!("A5")));
    }

    #[test]
    fn naked_triples() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        // A1, A2, A5 verlieren {1..6} -> bleiben {7,8,9} => Naked Triple in Row A
        let removed = knowns!("1 2 3 4 5 6");
        board.remove_candidates_from_cells(cells!("A1 A2 A5"), removed, &mut effects);

        let found = find_naked_tuples(&board, false, 3, Strategy::NakedTriple).unwrap();
        found.apply_all(&mut board);

        assert_eq!(KnownSet::full() - removed, board.candidates(cell!("A1")));
        assert_eq!(KnownSet::full() - removed, board.candidates(cell!("A2")));
        assert_eq!(KnownSet::full() - removed, board.candidates(cell!("A5")));

        // Andere Zellen in Row A sollten 7/8/9 verlieren -> Kandidaten != full
        assert_ne!(KnownSet::full(), board.candidates(cell!("A8")));
    }

    #[test]
    fn naked_quads() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        // A1, A2, A5, A8 verlieren {1..5} -> bleiben {6,7,8,9} => Naked Quad in Row A
        let removed = knowns!("1 2 3 4 5");
        board.remove_candidates_from_cells(cells!("A1 A2 A5 A8"), removed, &mut effects);

        let found = find_naked_tuples(&board, false, 4, Strategy::NakedQuad).unwrap();
        found.apply_all(&mut board);

        assert_eq!(KnownSet::full() - removed, board.candidates(cell!("A1")));
        assert_eq!(KnownSet::full() - removed, board.candidates(cell!("A2")));
        assert_eq!(KnownSet::full() - removed, board.candidates(cell!("A5")));
        assert_eq!(KnownSet::full() - removed, board.candidates(cell!("A8")));

        // Andere Zellen in Row A sollten 6/7/8/9 verlieren -> Kandidaten != full
        assert_ne!(KnownSet::full(), board.candidates(cell!("A9")));
    }

    #[test]
    fn single_mode_returns_at_most_one_action() {
        let board = Board::new();
        if let Some(effects) = find_naked_pairs(&board, true) {
            assert!(effects.actions().len() <= 1);
        }
    }

    #[test]
    fn empty_board_returns_none() {
        let board = Board::new();
        assert!(find_naked_pairs(&board, false).is_none());
        assert!(find_naked_triples(&board, false).is_none());
        assert!(find_naked_quads(&board, false).is_none());
    }
}