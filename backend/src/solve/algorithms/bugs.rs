use super::*;

use crate::layout::values::known_set::KnownSetLike;
use crate::puzzle::{Action, Board, Effects, KnownSet, Strategy, Verdict};

pub struct BugSolver;

impl Solver for BugSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::Bug
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_bugs(board, single)
    }
}

pub fn find_bugs(board: &Board, _single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    let pairs = board.cells_with_n_candidates(2);
    let triples = board.cells_with_n_candidates(3);

    if pairs.is_empty() || triples.len() != 1 {
        return None;
    }

    for count in [1, 4, 5, 6, 7, 8, 9] {
        if !board.cells_with_n_candidates(count).is_empty() {
            return None;
        }
    }

    let triple = triples.as_single().unwrap();
    let candidates = board.candidates(triple);
    let mut eliminated = KnownSet::empty();

    for known in candidates.iter() {
        for house in triple.houses() {
            if board.house_candidate_cells(house, known).len() == 2 {
                eliminated += known;
                break;
            }
        }
    }

    if eliminated.len() == 2 {
        let solution = (candidates - eliminated).as_single().unwrap();
        let mut action = Action::new_set(Strategy::Bug, triple, solution);
        action.clue_cells_for_known(
            Verdict::Secondary,
            triple.peers() & board.candidate_cells(solution),
            solution,
        );

        effects.add_action(action);

        if effects.has_actions() {
            return Some(effects);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_board_returns_none() {
        let board = Board::new();
        assert!(find_bugs(&board, true).is_none());
    }

    #[test]
    fn no_pairs_returns_none() {
        let mut board = Board::new();
        let mut eff = Effects::new();
        board.set_known(
            crate::cell!("A1"),
            crate::layout::values::known::known!("1"),
            &mut eff,
        );

        assert!(find_bugs(&board, true).is_none());
    }

    #[test]
    fn bug_solver_delegates_to_find_bugs() {
        let board = Board::new();
        let solver = BugSolver;
        let a = solver.apply(&board, true);
        let b = find_bugs(&board, true);
        assert_eq!(a.is_some(), b.is_some());
    }
}
