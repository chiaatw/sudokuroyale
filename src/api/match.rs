use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::game_match::repository::MatchRepository;
use crate::game_match::services::{create_match, join_match};
use crate::user::repository::UserRepository;
use crate::user::session_repository::SessionRepository;
use crate::AuthUser;
use rocket::http::CookieJar;
use uuid::Uuid;


#[derive(Serialize)]
pub struct CreateMatchResponse {
    pub match_id: String,
}

#[post("/match/create")]
pub fn create_match_route(
    cookies: &CookieJar<'_>,
    users: &State<Mutex<UserRepository>>,
    sessions: &State<Mutex<SessionRepository>>,
    matches: &State<Mutex<MatchRepository>>,
) -> Result<Json<CreateMatchResponse>, Status> {
    // 1) session_id Cookie holen
    let session_cookie = cookies.get("session_id").ok_or(Status::Unauthorized)?;
    let session_id = Uuid::parse_str(session_cookie.value()).map_err(|_| Status::Unauthorized)?;

    // 2) Locks holen
    let users_guard = users.lock().map_err(|_| Status::InternalServerError)?;
    let sessions_guard = sessions.lock().map_err(|_| Status::InternalServerError)?;
    let mut matches_guard = matches.lock().map_err(|_| Status::InternalServerError)?;

    // 3) Match erstellen
    let match_id = create_match(&users_guard, &sessions_guard, &mut matches_guard, &session_id)
        .ok_or(Status::Unauthorized)?;

    Ok(Json(CreateMatchResponse {
        match_id: match_id.to_string(),
    }))
}

#[derive(Deserialize)]
pub struct JoinMatchRequest {
    pub match_id: String,
}

#[derive(Serialize)]
pub struct JoinMatchResponse {
    pub ok: bool,
}

#[post("/match/join", data = "<req>")]
pub fn join_match_route(
    req: Json<JoinMatchRequest>,
    cookies: &CookieJar<'_>,
    users: &State<Mutex<UserRepository>>,
    sessions: &State<Mutex<SessionRepository>>,
    matches: &State<Mutex<MatchRepository>>,
) -> Result<Json<JoinMatchResponse>, Status> {
    let session_cookie = cookies.get("session_id").ok_or(Status::Unauthorized)?;
    let session_id = Uuid::parse_str(session_cookie.value()).map_err(|_| Status::Unauthorized)?;

    let match_id = Uuid::parse_str(&req.match_id).map_err(|_| Status::BadRequest)?;

    let users_guard = users.lock().map_err(|_| Status::InternalServerError)?;
    let sessions_guard = sessions.lock().map_err(|_| Status::InternalServerError)?;
    let mut matches_guard = matches.lock().map_err(|_| Status::InternalServerError)?;

    let ok = join_match(&users_guard, &sessions_guard, &mut matches_guard, &session_id, &match_id);

    Ok(Json(JoinMatchResponse { ok }))
}

#[derive(Serialize)]
pub struct MatchInfoResponse {
    pub match_id: String,
    pub status: String,
    pub player1_id: String,
    pub player2_id: Option<String>,
}

#[get("/match/<match_id>")]
pub fn get_match_route(
    match_id: &str,
    matches: &State<Mutex<MatchRepository>>,
) -> Result<Json<MatchInfoResponse>, Status> {
    let match_uuid = Uuid::parse_str(match_id).map_err(|_| Status::BadRequest)?;
    let matches_guard = matches.lock().map_err(|_| Status::InternalServerError)?;

    let m = matches_guard.find_by_id(&match_uuid).ok_or(Status::NotFound)?;

    Ok(Json(MatchInfoResponse {
        match_id: m.id.to_string(),
        status: format!("{:?}", m.status),
        player1_id: m.player1_id.to_string(),
        player2_id: m.player2_id.map(|id| id.to_string()),
    }))
}


#[derive(Deserialize)]
pub struct LeaveMatchRequest {
    pub match_id: String,
}

#[derive(Serialize)]
pub struct LeaveMatchResponse {
    pub ok: bool,
}

#[post("/match/leave", data = "<req>")]
pub fn leave_match_route(
    auth: AuthUser,
    req: Json<LeaveMatchRequest>,
    matches: &State<Mutex<MatchRepository>>,
) -> Result<Json<LeaveMatchResponse>, Status> {

    let match_id = Uuid::parse_str(&req.match_id).map_err(|_| Status::BadRequest)?;

    let mut repo = matches.lock().map_err(|_| Status::InternalServerError)?;

    let ok = crate::game_match::services::leave_match_by_user(&mut repo, &auth.user_id, &match_id);

    Ok(Json(LeaveMatchResponse { ok }))
}