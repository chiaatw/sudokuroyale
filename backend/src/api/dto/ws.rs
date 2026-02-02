use serde::{Deserialize, Serialize};

use super::game::GameViewDto;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WsServerEvent {
    Snapshot { view: GameViewDto },
    RevisionChanged { revision: u64, view: GameViewDto },
}
