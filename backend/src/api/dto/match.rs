use serde::{Deserialize, Serialize};

use uuid::Uuid;

use crate::api::dto::game::GameViewDto;
use crate::api::dto::ws::WsServerEvent;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMatchResponse {
    pub match_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinMatchRequest {
    pub match_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinMatchResponse {
    pub ok: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchInfoResponse {
    pub match_id: String,
    pub status: String,
    pub player1_id: String,
    pub player2_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaveMatchRequest {
    pub match_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaveMatchResponse {
    pub ok: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartMatchRequest {
    pub match_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartMatchResponse {
    pub ok: bool,
}

pub fn snapshot_from_meta(
    match_id: Uuid,
    meta: &crate::game_match::model::GameMatch,
    view: Option<GameViewDto>,
) -> WsServerEvent {
    WsServerEvent::Snapshot {
        match_id,
        status: meta.status,
        player1_id: meta.player1_id,
        player2_id: meta.player2_id,
        started_at: meta.started_at,
        view,
    }
}
