use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket::{get, post};
use std::sync::Mutex;
use uuid::Uuid;

use crate::auth::AuthUser;

use crate::api::dto::game::{
    ApplyMoveRequest, ApplyMoveResponse, GameViewDto, MoveDto, MoveOutcomeDto,
    AppliedMoveDto, RejectReasonDto, PenaltyReasonDto, LoseReasonDto, OpponentProgressDto
};

use crate::api::dto::r#match::{
    CreateMatchResponse, JoinMatchRequest, JoinMatchResponse, LeaveMatchRequest,
    LeaveMatchResponse, MatchInfoResponse, StartMatchRequest, StartMatchResponse,
};

use crate::game::outcome::{MoveOutcome, AppliedMove, RejectReason, PenaltyReason};
use crate::game::r#move::Move;
use crate::layout::{Cell, Value};

use crate::game_match::repository::MatchRepository;
use crate::game_match::services::{
    create_match, join_match, leave_match_by_user, start_match_by_user, get_match_state_for_user, apply_move_for_user
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

#[get("/match/<match_id>/state")]
pub fn get_match_state_route(
    auth: AuthUser,
    match_id: &str,
    matches: &State<Mutex<MatchRepository>>,
) -> Result<Json<GameViewDto>, Status> {
    let match_id = Uuid::parse_str(match_id).map_err(|_| Status::BadRequest)?;
    let mut repo = matches.lock().map_err(|_| Status::InternalServerError)?;

    let view = get_match_state_for_user(&mut repo, &auth.user_id, &match_id)
        .ok_or(Status::NotFound)?;

    Ok(Json(game_view_to_dto(view)))
}

#[post("/match/<match_id>/move", data = "<req>")]
pub fn apply_move_route(
    auth: AuthUser,
    match_id: &str,
    req: Json<ApplyMoveRequest>,
    matches: &State<Mutex<MatchRepository>>,
) -> Result<Json<ApplyMoveResponse>, Status> {
    let match_id = Uuid::parse_str(match_id).map_err(|_| Status::BadRequest)?;

    // MoveDto -> Move (domain)
    let mv = move_dto_to_domain(&req.mv).ok_or(Status::BadRequest)?;

    let mut repo = matches.lock().map_err(|_| Status::InternalServerError)?;

    let (outcome, view) = apply_move_for_user(&mut repo, &auth.user_id, &match_id, req.expected_revision, mv)
        .ok_or(Status::NotFound)?;

    // Revision mismatch => 409 Conflict
    if matches_revision_mismatch(&outcome) {
        return Err(Status::Conflict);
    }

    Ok(Json(ApplyMoveResponse {
        outcome: move_outcome_to_dto(outcome),
        view: Some(game_view_to_dto(view)),
        replay: false, // später: move_id dedupe möglich
    }))
}

fn move_dto_to_domain(dto: &MoveDto) -> Option<Move> {
    match dto {
        MoveDto::Clear { cell } => {
            if *cell > 80 { return None; }
            Some(Move::Clear { cell: Cell::new(*cell) })
        }
        MoveDto::Place { cell, value } => {
            if *cell > 80 { return None; }
            if *value < 1 || *value > 9 { return None; }
            Some(Move::Place {
                cell: Cell::new(*cell),
                value: Value::new(*value),
            })
        }
    }
}

fn grid_to_vec(grid: &crate::layout::Grid) -> Vec<u8> {
    let mut out = Vec::with_capacity(81);
    for i in 0..81 {
        let cell = Cell::new(i);
        out.push(grid.get(cell).value());
    }
    out
}

fn game_view_to_dto(view: crate::game::view::GameView) -> GameViewDto {
    GameViewDto {
        revision: view.revision,
        state: format!("{:?}", view.state),
        givens: grid_to_vec(&view.givens),
        current: grid_to_vec(&view.current),
        mistakes_left: view.mistakes_left,
        remaining_ms: view.remaining_time.as_millis() as u64,
        opponent_progress: view.opponent_progress.map(|op| OpponentProgressDto {
            filled: op.filled,
            mistakes_left: op.mistakes_left,
            remaining_ms: op.remaining_time.as_millis() as u64,
        }),
    }
}

fn matches_revision_mismatch(outcome: &MoveOutcome) -> bool {
    matches!(
        outcome,
        MoveOutcome::Rejected { reason: RejectReason::RevisionMismatch { .. } }
    )
}

fn move_outcome_to_dto(outcome: MoveOutcome) -> MoveOutcomeDto {
    match outcome {
        MoveOutcome::Applied { revision, applied } => MoveOutcomeDto::Applied {
            revision,
            applied: match applied {
                AppliedMove::Placed => AppliedMoveDto::Placed,
                AppliedMove::Cleared => AppliedMoveDto::Cleared,
            },
        },
        MoveOutcome::Rejected { reason } => {
            let (reason_dto, revision) = match reason {
                RejectReason::NotInProgress => (RejectReasonDto::NotInProgress, 0),
                RejectReason::UnknownPlayer => (RejectReasonDto::UnknownPlayer, 0),
                RejectReason::GivenCell => (RejectReasonDto::GivenCell, 0),
                RejectReason::InvalidValue => (RejectReasonDto::InvalidValue, 0),
                RejectReason::RevisionMismatch { expected, actual } => (
                    RejectReasonDto::RevisionMismatch { expected, actual },
                    actual,
                ),
            };

            MoveOutcomeDto::Rejected { reason: reason_dto, revision }
        }
        MoveOutcome::Penalty { reason, mistakes_left, revision } => MoveOutcomeDto::Penalty {
            reason: match reason {
                PenaltyReason::WrongValue => PenaltyReasonDto::WrongValue,
            },
            mistakes_left,
            revision,
        },
        MoveOutcome::Won { revision } => MoveOutcomeDto::Won { revision },
        MoveOutcome::Lost { revision, reason } => MoveOutcomeDto::Lost {
            revision,
            reason: match reason {
                crate::game::state::LoseReason::TimeExpired => LoseReasonDto::Timeout,
                crate::game::state::LoseReason::TooManyMistakes => LoseReasonDto::TooManyMistakes,
            },
        },
    }
}