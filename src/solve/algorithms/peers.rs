use super::*;

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

        let mut action = Action::new_erase_cells(
            Strategy::Peer,
            peers,
            known,
        );

        // The known cell is the logical cause of the elimination
        action.clue_cell_for_known(
            Verdict::Secondary,
            cell,
            known,
        );

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