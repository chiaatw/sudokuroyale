use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MatchStatus {
    Waiting,
    Ready,
    InProgress,
    Finished,
}

#[derive(Debug, Clone)]
pub struct GameMatch {
    pub id: Uuid,
    pub player1_id: Uuid,
    pub player2_id: Option<Uuid>,
    pub status: MatchStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
}

impl GameMatch {
    pub fn new(player1_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            player1_id,
            player2_id: None,
            status: MatchStatus::Waiting,
            created_at: Utc::now(),
            started_at: None,
        }
    }
}
