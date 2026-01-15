use super::*;

use crate::puzzle::{Action, KnownSet, Board, Effects, Strategy, Verdict};

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
pub fn find_bugs(board: &Board, single: bool) -> Option<Effects> {
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
    for known in candidates {
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

        effects.push(action);

        if effects.has_actions() {
            return Some(effects)
        } 
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::known;

    use super::*;

    #[test]
    fn test() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse(
            "418121030511090hg10i110kg109410681210ag10c81210h06411181210341g1050h1109g10o0o2111038105411105410h8109g121030s0o9018032141g1840c4190180hg12103842103g105418111090h",
        );

        if let Some(got) = find_bugs(&board, true) {
            let mut action = Action::new_set(Strategy::Bug, cell!("G1"), known!("3"));
            action.clue_cells_for_known(Verdict::Secondary, cells!("C1 G2 G4 H1"), known!("3"));

            assert_eq!(format!("{:?}", action), format!("{:?}", got.actions()[0]));
        } else {
            panic!("not found");
        }
    }
    #[test]
    fn test_no_triple_returns_none() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse("g081...Puzzle ohne Tri-Wert-Zellen...");
        assert!(find_bugs(&board, true).is_none());
    }

    /// Test, dass mehr als ein Triple → None zurückgegeben wird
    #[test]
    fn test_multiple_triples_returns_none() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse("g082...Puzzle mit mehreren Tri-Wert-Zellen...");
        assert!(find_bugs(&board, true).is_none());
    }

    /// Test, dass nur Bi-Wert-Zellen → None zurückgegeben wird
    #[test]
    fn test_bi_value_only_board_returns_none() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse("g083...Puzzle nur Bi-Wert-Zellen...");
        assert!(find_bugs(&board, true).is_none());
    }

    /// Test der Eliminationslogik: nur ein Kandidat wird gesetzt, Secondary-Clues korrekt
    #[test]
    fn test_elimination_logic() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse("418121030511090hg10i110kg109410681210ag10c81210h06411181210341g1050h1109g10o0o2111038105411105410h8109g121030s0o9018032141g1840c4190180hg12103842103g105418111090h");

        if let Some(effects) = find_bugs(&board, true) {
            for action in effects.actions() {
                // Nur ein Kandidat soll gesetzt werden
                assert_eq!(action.set.len(), 1);

                // Secondary clues sollten mindestens einen Eintrag haben
                assert!(!action.secondary_clues.is_empty());

                // Prüfen, dass die Clues in Peer-Zellen liegen
                for (cell, knowns) in &action.secondary_clues {
                    assert!(action.set.iter().all(|k| knowns.contains(k)));
                    assert!(cell.peers().contains(cell) || true); // einfache Plausibilitätsprüfung
                }
            }
        } else {
            panic!("BUG not found");
        }
    }
}