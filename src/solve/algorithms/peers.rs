use super::*;

use crate::puzzle::{Action, Board, Effects, Strategy, Verdict};
// Solver wrapper for the Peer (simple elimination) strategy

pub struct PeerSolver;

impl Solver for PeerSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::Peer
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_peers(board, single)
    }
}

pub fn find_peers(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    // For each fixed value on the board, eliminate it from all peers
    for (cell, known) in board.known_iter() {
        let peers = cell.peers() & board.candidate_cells(known);

        // Nothing to eliminate
        if peers.is_empty() {
            continue;
        }

        let mut action = Action::new_erase_cells(Strategy::Peer, peers, known);

        // The known cell is the logical cause of the elimination
        action.clue_cell_for_known(Verdict::Secondary, cell, known);

        if effects.add_action(action) && single {
            return Some(effects);
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

#[cfg(test)]
mod peer_tests {
    use super::*;

    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::known;
    use crate::layout::values::known::Known;
    use crate::layout::values::known_set::{KnownSet, KnownSetLike};

    // lokales knowns! Makro (falls du keins global hast)
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
    fn no_knowns_returns_none() {
        let board = Board::new();
        assert!(find_peers(&board, false).is_none());
    }

    #[test]
    fn solver_delegates_to_find() {
        let board = Board::new();
        let solver = PeerSolver;

        let via_solver = solver.apply(&board, true);
        let via_fn = find_peers(&board, true);

        assert_eq!(via_solver.is_some(), via_fn.is_some());
    }

    #[test]
    fn single_mode_returns_at_most_one_action() {
        let mut board = Board::new();
        let mut eff = Effects::new();

        board.set_known(cell!("A1"), known!("5"), &mut eff);
        assert!(!eff.has_errors());

        if let Some(found) = find_peers(&board, true) {
            assert!(found.actions().len() <= 1);
        }
    }

    #[test]
    fn peer_elimination_removes_known_from_peers_candidates() {
        let mut board = Board::new();
        let mut eff = Effects::new();

        // Setze eine Zahl fest
        board.set_known(cell!("A1"), known!("5"), &mut eff);
        assert!(!eff.has_errors());

        // Wir können Kandidaten nicht direkt setzen,
        // aber wir können Kandidaten gezielt "vorbereiten" indem wir alles ANDERE entfernen,
        // so dass in manchen Peer-Zellen die 5 garantiert noch drin ist.
        //
        // Board::new() hat typischerweise volle Kandidaten,
        // daher sorgen wir dafür, dass z.B. A2/B1/B2 auf {5, x} reduziert werden,
        // indem wir alle anderen außer 5 und x entfernen.
        let keep_a2 = knowns!("1 5");
        let keep_b1 = knowns!("2 5");
        let keep_b2 = knowns!("3 4 5");

        // remove = full - keep
        board.remove_candidates_from_cells(cells!("A2"), KnownSet::full() - keep_a2, &mut eff);
        board.remove_candidates_from_cells(cells!("B1"), KnownSet::full() - keep_b1, &mut eff);
        board.remove_candidates_from_cells(cells!("B2"), KnownSet::full() - keep_b2, &mut eff);
        assert!(!eff.has_errors());

        // Jetzt Peer-Elimination finden + anwenden
        let found = find_peers(&board, false).expect("expected peer eliminations");
        let mut after = board;
        found.apply_all(&mut after);

        // 5 muss aus den Peers verschwinden
        assert!(!after.candidates(cell!("A2")).has(known!("5")));
        assert!(!after.candidates(cell!("B1")).has(known!("5")));
        assert!(!after.candidates(cell!("B2")).has(known!("5")));

        // Und die anderen Kandidaten bleiben plausibel erhalten
        assert!(after.candidates(cell!("A2")).has(known!("1")));
        assert!(after.candidates(cell!("B1")).has(known!("2")));
    }
}
