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

#[cfg(test)]
mod peer_tests {
    use super::*;
    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::known;

    #[test]
    fn peer_elimination_single() {
        let mut board = Board::new();

        // Set a known value in cell A1
        board.set_known(cell!("A1"), known!("5"));

        // Place candidates in peers of A1 including 5
        board.set_candidates(cell!("A2"), known!("1 5"), &mut Effects::new());
        board.set_candidates(cell!("B1"), known!("2 5"), &mut Effects::new());
        board.set_candidates(cell!("B2"), known!("3 4 5"), &mut Effects::new());

        let effects = find_peers(&board, false).unwrap();
        effects.apply_all(&mut board);

        // The known value 5 should be removed from all peers of A1
        assert_eq!(board.candidates(cell!("A2")), known!("1"));
        assert_eq!(board.candidates(cell!("B1")), known!("2"));
        assert_eq!(board.candidates(cell!("B2")), known!("3 4"));
    }

    #[test]
    fn peer_elimination_multiple() {
        let mut board = Board::new();

        // Known values
        board.set_known(cell!("A1"), known!("5"));
        board.set_known(cell!("B1"), known!("3"));

        // Candidates in row/column/box
        board.set_candidates(cell!("A2"), known!("3 5"), &mut Effects::new());
        board.set_candidates(cell!("B2"), known!("3 5"), &mut Effects::new());

        let effects = find_peers(&board, false).unwrap();
        effects.apply_all(&mut board);

        // 5 removed because of A1, 3 removed because of B1
        assert_eq!(board.candidates(cell!("A2")), KnownSet::empty());
        assert_eq!(board.candidates(cell!("B2")), known!("5"));
    }

    #[test]
    fn peer_no_elimination_when_empty() {
        let mut board = Board::new();

        // No knowns on the board, should return None
        let effects = find_peers(&board, false);
        assert!(effects.is_none());
    }
}
