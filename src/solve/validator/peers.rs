use crate::solve::validator::{Board, ValidationError};
use crate::layout::{Cell, Value};

pub fn validate_peers(board: &Board) -> Result<(), ValidationError> {
    for cell in Cell::ALL {
        let v = board.get(cell);
        if v.is_unknown() { continue; }

        for peer in cell.peers() {
            if board.get(peer) == v {
                return Err(ValidationError::ConflictWithPeer {
                    cell, value: v, peer
                });
            }
        }
    }
    Ok(())
}
