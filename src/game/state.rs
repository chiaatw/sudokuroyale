use crate::game::player::PlayerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameState {
    Waiting,
    InProgress,
    Won { player: PlayerId },
    Lost { player: PlayerId, reason: LoseReason },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoseReason {
    TooManyMistakes,
    TimeExpired,
}
