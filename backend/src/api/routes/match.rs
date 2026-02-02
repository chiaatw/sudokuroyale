use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket::{get, post};
use std::sync::Mutex;
use uuid::Uuid;

use crate::auth::AuthUser;

use crate::api::dto::r#match::{
    CreateMatchResponse, JoinMatchRequest, JoinMatchResponse, LeaveMatchRequest,
    LeaveMatchResponse, MatchInfoResponse, StartMatchRequest, StartMatchResponse,
};

use crate::game_match::repository::MatchRepository;
use crate::game_match::services::{
    create_match, join_match, leave_match_by_user, start_match_by_user,
};

#[post("/match/create")]
pub fn create_match_route(
    auth: AuthUser,
    matches: &State<Mutex<MatchRepository>>,
) -> Result<Json<CreateMatchResponse>, Status> {
    let mut matches_guard = matches.lock().map_err(|_| Status::InternalServerError)?;
    let match_id = create_match(&mut matches_guard, &auth.user_id);

    Ok(Json(CreateMatchResponse {
        match_id: match_id.to_string(),
    }))
}

#[post("/match/join", data = "<req>")]
pub fn join_match_route(
    auth: AuthUser,
    req: Json<JoinMatchRequest>,
    matches: &State<Mutex<MatchRepository>>,
) -> Result<Json<JoinMatchResponse>, Status> {
    let match_id = Uuid::parse_str(&req.match_id).map_err(|_| Status::BadRequest)?;
    let mut matches_guard = matches.lock().map_err(|_| Status::InternalServerError)?;

    let ok = join_match(&mut matches_guard, &auth.user_id, &match_id);

    Ok(Json(JoinMatchResponse { ok }))
}

#[get("/match/<match_id>")]
pub fn get_match_route(
    match_id: &str,
    matches: &State<Mutex<MatchRepository>>,
) -> Result<Json<MatchInfoResponse>, Status> {
    let match_uuid = Uuid::parse_str(match_id).map_err(|_| Status::BadRequest)?;
    let matches_guard = matches.lock().map_err(|_| Status::InternalServerError)?;

    let m = matches_guard
        .find_by_id(&match_uuid)
        .ok_or(Status::NotFound)?;

    Ok(Json(MatchInfoResponse {
        match_id: m.id.to_string(),
        status: format!("{:?}", m.status),
        player1_id: m.player1_id.to_string(),
        player2_id: m.player2_id.map(|id| id.to_string()),
    }))
}

#[post("/match/leave", data = "<req>")]
pub fn leave_match_route(
    auth: AuthUser,
    req: Json<LeaveMatchRequest>,
    matches: &State<Mutex<MatchRepository>>,
) -> Result<Json<LeaveMatchResponse>, Status> {
    let match_id = Uuid::parse_str(&req.match_id).map_err(|_| Status::BadRequest)?;
    let mut repo = matches.lock().map_err(|_| Status::InternalServerError)?;

    let ok = leave_match_by_user(&mut repo, &auth.user_id, &match_id);

    Ok(Json(LeaveMatchResponse { ok }))
}

#[post("/match/start", data = "<req>")]
pub fn start_match_route(
    auth: AuthUser,
    req: Json<StartMatchRequest>,
    matches: &State<Mutex<MatchRepository>>,
) -> Result<Json<StartMatchResponse>, Status> {
    let match_id = Uuid::parse_str(&req.match_id).map_err(|_| Status::BadRequest)?;
    let mut matches_guard = matches.lock().map_err(|_| Status::InternalServerError)?;

    let ok = start_match_by_user(&mut matches_guard, &auth.user_id, &match_id);

    Ok(Json(StartMatchResponse { ok }))
}
