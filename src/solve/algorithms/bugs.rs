use super::*;

use crate::puzzle::{Action, KnownSet, Board, Effects, Strategy, Verdict};
use crate::layout::values::known_set::KnownSetLike;

// Solver wrapper for the BUG (Bi-value Universal Grave) strategy
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

// Find BUG patterns and returns the corresponding effects
pub fn find_bugs(board: &Board, _single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

// Bi-value and tri-value cells
    let pairs = board.cells_with_n_candidates(2);
    let triples = board.cells_with_n_candidates(3);

// Only proceed if there are pairs and exactly on triple
    if pairs.is_empty() || triples.len() != 1 {
        return None;
    }

// No other candidate counts allowed in a BUG
    for count in [1, 4, 5, 6 , 7, 8, 9] {
        if !board.cells_with_n_candidates(count).is_empty() {
            return None;
        }
    }

    let triple = triples.as_single().unwrap();
    let candidates = board.candidates(triple);
    let mut eliminated = KnownSet::empty();

// Determine which candidates can be safely removed
    for known in candidates.iter() {
        for house in triple.houses() {
            if board.house_candidate_cells(house, known).len() == 2 {
                eliminated += known;
                break;
            }
        }
    }

// Only one solution remains
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
            return Some(effects)
        } 
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_board_returns_none() {
        // Auf einem frischen Board haben viele Zellen nicht genau 2 Kandidaten
        // oder es gibt nicht genau einen Triple-Cell => None ist korrekt.
        let board = Board::new();
        assert!(find_bugs(&board, true).is_none());
    }

    #[test]
    fn no_pairs_returns_none() {
        // Künstlich: wir setzen ein Known, damit sich Kandidatenlage ändert,
        // aber ohne gezielte Kandidaten-Konstruktion sollte weiterhin kein BUG entstehen.
        let mut board = Board::new();
        let mut eff = Effects::new();
        // irgendein Given/Known setzen (wenn möglich)
        // (falls set_known nicht existiert, nimm set_given oder lass es weg)
        board.set_known(crate::layout::cells::cell::cell!("A1"), crate::layout::values::known::known!("1"), &mut eff);

        assert!(find_bugs(&board, true).is_none());
    }

    #[test]
    fn bug_solver_delegates_to_find_bugs() {
        let board = Board::new();
        let solver = BugSolver;
        let a = solver.apply(&board, true);
        let b = find_bugs(&board, true);
        // Beide sollten identisch sein (hier None)
        assert_eq!(a.is_some(), b.is_some());
    }
}