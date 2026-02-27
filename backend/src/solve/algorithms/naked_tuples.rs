use super::*;
use itertools::Itertools;

use crate::layout::cells::cell_set::CellIteratorUnion;
use crate::layout::values::known_set::KnownSetIteratorUnion;
use crate::layout::values::known_set::KnownSetLike;
use crate::layout::House;
use crate::puzzle::{Action, Board, Effects, KnownSet, Strategy, Verdict};

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

fn find_naked_tuples(
    board: &Board,
    single: bool,
    size: usize,
    strategy: Strategy,
) -> Option<Effects> {
    let mut effects = Effects::new();

    for house in House::iter() {
        let house_cells = house.cells();

        for candidates in house_cells
            .iter()
            .map(|cell| (cell, board.candidates(cell)))
            .filter(|(_, candidates)| 2 <= candidates.len() && candidates.len() <= size)
            .combinations(size)
        {
            let known_sets = candidates.iter().map(|(_, ks)| *ks).collect_vec();
            let tuple_knowns = known_sets.iter().copied().union_knowns();

            if tuple_knowns.len() != size
                || is_degenerate(&known_sets, size, 2)
                || is_degenerate(&known_sets, size, 3)
            {
                continue;
            }

            let tuple_cells = candidates.iter().map(|(c, _)| *c).union_cells();
            let erase_cells = house_cells - tuple_cells;
            let mut action = Action::new(strategy);

            tuple_knowns.iter().for_each(|k| {
                action.erase_cells(erase_cells & board.candidate_cells(k), k);
                action.clue_cells_for_known(
                    Verdict::Secondary,
                    tuple_cells & board.candidate_cells(k),
                    k,
                );
            });

            tuple_cells.iter().for_each(|c| {
                action.clue_cell_for_knowns(
                    Verdict::Related,
                    c,
                    KnownSet::full() - board.candidates(c),
                );
            });

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

pub fn is_degenerate(known_sets: &[KnownSet], size: usize, smaller_size: usize) -> bool {
    size > smaller_size
        && known_sets
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

    use crate::cell;
    use crate::cells;
    use crate::layout::values::known::Known;
    use crate::layout::values::known_set::{KnownSet, KnownSetLike};

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

        let removed = knowns!("1 2 3 4 5 6 7");
        board.remove_candidates_from_cells(cells!("A1 A2"), removed, &mut effects);

        let found = find_naked_tuples(&board, false, 2, Strategy::NakedPair).unwrap();
        found.apply_all(&mut board);

        assert_eq!(KnownSet::full() - removed, board.candidates(cell!("A1")));
        assert_eq!(KnownSet::full() - removed, board.candidates(cell!("A2")));

        assert_ne!(KnownSet::full(), board.candidates(cell!("A5")));
    }

    #[test]
    fn naked_triples() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let removed = knowns!("1 2 3 4 5 6");
        board.remove_candidates_from_cells(cells!("A1 A2 A5"), removed, &mut effects);

        let found = find_naked_tuples(&board, false, 3, Strategy::NakedTriple).unwrap();
        found.apply_all(&mut board);

        assert_eq!(KnownSet::full() - removed, board.candidates(cell!("A1")));
        assert_eq!(KnownSet::full() - removed, board.candidates(cell!("A2")));
        assert_eq!(KnownSet::full() - removed, board.candidates(cell!("A5")));

        assert_ne!(KnownSet::full(), board.candidates(cell!("A8")));
    }

    #[test]
    fn naked_quads() {
        let mut board = Board::new();
        let mut effects = Effects::new();

        let removed = knowns!("1 2 3 4 5");
        board.remove_candidates_from_cells(cells!("A1 A2 A5 A8"), removed, &mut effects);

        let found = find_naked_tuples(&board, false, 4, Strategy::NakedQuad).unwrap();
        found.apply_all(&mut board);

        assert_eq!(KnownSet::full() - removed, board.candidates(cell!("A1")));
        assert_eq!(KnownSet::full() - removed, board.candidates(cell!("A2")));
        assert_eq!(KnownSet::full() - removed, board.candidates(cell!("A5")));
        assert_eq!(KnownSet::full() - removed, board.candidates(cell!("A8")));

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
