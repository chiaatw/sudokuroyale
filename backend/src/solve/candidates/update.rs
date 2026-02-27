use crate::layout::{Cell, Known, KnownSet, ValueLike};
use crate::solve::validator::Board;

use super::Candidates;

pub fn recompute_all_candidates(board: &Board, candidates: &mut Candidates) {
    for cell in Cell::iter() {
        let v = board.get(cell);
        if v.is_known() {
            candidates.set(cell, KnownSet::empty());
            continue;
        }

        let mut allowed = KnownSet::full();

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

pub fn update_after_set_known(
    board: &Board,
    candidates: &mut Candidates,
    cell: Cell,
    known: Known,
) {
    candidates.set(cell, KnownSet::empty());

    for peer in cell.peers().iter() {
        if board.get(peer).is_unknown() {
            candidates.remove(peer, known);
        }
    }
}
