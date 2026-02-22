use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::game::GameViewDto;
use crate::game_match::model::MatchStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WsServerEvent {
    Snapshot {
        match_id: Uuid,
        status: MatchStatus,
        player1_id: Uuid,
        player2_id: Option<Uuid>,
        started_at: Option<chrono::DateTime<chrono::Utc>>,
        view: Option<GameViewDto>, 
    },

    RevisionChanged {
        revision: u64,
    },
}
