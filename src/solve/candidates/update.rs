use crate::layout::{Cell, Known, KnownSet, ValueLike};
use crate::solve::validator::Board;

use super::Candidates;

/// Recomputes candidates for all cells from scratch.
/// Rule: candidates(cell) = {1..9} minus values in peers, if cell is unknown.
/// If cell is known: candidates = empty.
pub fn recompute_all_candidates(board: &Board, candidates: &mut Candidates) {
    for cell in Cell::iter() {
        let v = board.get(cell);
        if v.is_known() {
            candidates.set(cell, KnownSet::empty());
            continue;
        }

        let mut allowed = KnownSet::full();

        // remove digits already present in peers
        for peer in cell.peers().iter() {
            let pv = board.get(peer);
            if pv.is_known() {
                let k = Known::new(pv.raw());
                allowed -= k;
            }
        }

        candidates.set(cell, allowed);
    }
}

/// Incremental update after setting a known value into a cell.
/// - Set cell candidates to empty.
/// - Remove this digit from all peer cells candidates.
pub fn update_after_set_known(board: &Board, candidates: &mut Candidates, cell: Cell, known: Known) {
    // if board says it's known -> candidates empty
    if board.get(cell).is_known() {
        candidates.set(cell, KnownSet::empty());
    }

    // peers cannot have that digit anymore
    for peer in cell.peers().iter() {
        // Only matters for unknown peer cells. If peer is known, its candidates should already be empty.
        if board.get(peer).is_unknown() {
            candidates.remove(peer, known);
        }
    }
}
