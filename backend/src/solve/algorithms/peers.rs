use super::*;

use crate::puzzle::{Action, Board, Effects, Strategy, Verdict};
use crate::layout::CellSet;
use crate::layout::values::known_set::KnownSetLike;
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

    for (cell, known) in board.known_iter() {
        // Statt: cell.peers() & board.candidate_cells(known)
        let peers: CellSet = cell
            .peers()
            .iter()
            .filter(|&p| board.candidates(p).has(known))
            .collect();

        if peers.is_empty() {
            continue;
        }

        let mut action = Action::new_erase_cells(Strategy::Peer, peers, known);
        action.clue_cell_for_known(Verdict::Secondary, cell, known);

        if effects.add_action(action) && single {
            return Some(effects);
        }
    }

    effects.has_actions().then_some(effects)
}

#[cfg(test)]
mod peer_tests {
    use super::*;

    use crate::cell;
    use crate::layout::values::known::known;
    use crate::layout::values::known_set::KnownSetLike;


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
    fn set_known_removes_known_from_peers_candidates() {
        let mut board = Board::new();
        let mut eff = Effects::new();

        board.set_known(cell!("A1"), known!("5"), &mut eff);
        assert!(!eff.has_errors());

        // 5 muss aus allen Peers von A1 verschwunden sein
        for p in cell!("A1").peers().iter() {
            assert!(!board.candidates(p).has(known!("5")));
        }
    }
}
