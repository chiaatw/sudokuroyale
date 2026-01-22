use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlayerIdDto {
    PlayerA,
    PlayerB,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MoveDto {
    Place { cell: u8, value: u8 },
    Clear { cell: u8 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyMoveRequest {
    pub player_id: PlayerIdDto,
    pub expected_revision: u64,
    pub move_id: String,
    pub mv: MoveDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyMoveResponse {
    pub outcome: MoveOutcomeDto,
    pub view: Option<GameViewDto>,
    pub replay: bool,
}

// Outcome DTO (wire-format)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MoveOutcomeDto {
    Applied { revision: u64, applied: AppliedMoveDto },
    Rejected { reason: RejectReasonDto, revision: u64 },
    Penalty { reason: PenaltyReasonDto, mistakes_left: u8, revision: u64 },
    Won { revision: u64 },
    Lost { revision: u64, reason: LoseReasonDto },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AppliedMoveDto {
    Placed,
    Cleared,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RejectReasonDto {
    NotInProgress,
    UnknownPlayer,
    RevisionMismatch { expected: u64, actual: u64 },
    GivenCell,
    InvalidValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PenaltyReasonDto {
    WrongValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LoseReasonDto {
    Timeout,
    TooManyMistakes,
    OpponentWon,
    // ggf. erweitern nach deinem core
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameViewDto {
    pub revision: u64,
    pub state: String, // oder enum dto, wenn du willst
    pub givens: Vec<u8>,  // length 81
    pub current: Vec<u8>, // length 81
    pub mistakes_left: u8,
    pub remaining_ms: u64,
    pub opponent_progress: Option<OpponentProgressDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpponentProgressDto {
    pub filled: u8,
    pub mistakes_left: u8,
    pub remaining_ms: u64,
}