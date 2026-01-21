use crate::game::state::LoseReason;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectReason {
    NotInProgress,
    UnknownPlayer,
    RevisionMismatch { expected: u64, actual: u64 },
    GivenCell,
    InvalidValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PenaltyReason {
    WrongValue,
}

/// Optional: Gibt der GUI/API Kontext, was genau angewendet wurde.
/// (Kannst du auch entfernen, wenn du es minimal halten willst.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppliedMove {
    Placed,
    Cleared,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveOutcome {
    Applied {
        revision: u64,
        applied: AppliedMove,
    },
    Rejected {
        reason: RejectReason,
    },
    Penalty {
        reason: PenaltyReason,
        mistakes_left: u8,
        revision: u64,
    },
    Won {
        revision: u64,
    },
    Lost {
        revision: u64,
        reason: LoseReason,
    },
}