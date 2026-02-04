use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket::{get, post};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use rocket::futures::{SinkExt, StreamExt};
use rocket_ws::{WebSocket, Channel, Message};

use crate::api::ws_hub::WsHub;
use crate::api::dto::ws::WsServerEvent;

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
use crate::game_match::model::MatchStatus;
use std::time::{Duration, Instant};
use chrono::Utc;

use crate::game_match::services::{
    create_match, join_match, leave_match_by_user, get_match_state_for_user, apply_move_for_user, generate_puzzle_mvp
};

#[post("/match/create")]
pub fn create_match_route(
    auth: AuthUser,
    matches: &State<Arc<Mutex<MatchRepository>>>,
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
    matches: &State<Arc<Mutex<MatchRepository>>>,
) -> Result<Json<JoinMatchResponse>, Status> {
    let match_id = Uuid::parse_str(&req.match_id).map_err(|_| Status::BadRequest)?;
    let mut matches_guard = matches.lock().map_err(|_| Status::InternalServerError)?;

    let ok = join_match(&mut matches_guard, &auth.user_id, &match_id);

    Ok(Json(JoinMatchResponse { ok }))
}

#[get("/match/<match_id>")]
pub fn get_match_route(
    match_id: &str,
    matches: &State<Arc<Mutex<MatchRepository>>>,
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
    matches: &State<Arc<Mutex<MatchRepository>>>,
) -> Result<Json<LeaveMatchResponse>, Status> {
    let match_id = Uuid::parse_str(&req.match_id).map_err(|_| Status::BadRequest)?;
    let mut repo = matches.inner().lock().map_err(|_| Status::InternalServerError)?;

    let ok = leave_match_by_user(&mut repo, &auth.user_id, &match_id);

    Ok(Json(LeaveMatchResponse { ok }))
}

#[post("/match/start", data = "<req>")]
pub fn start_match_route(
    auth: AuthUser,
    req: Json<StartMatchRequest>,
    matches: &State<Arc<Mutex<MatchRepository>>>,
) -> Result<Json<StartMatchResponse>, Status> {
    eprintln!("start_route: ENTER");

    let match_id = Uuid::parse_str(&req.match_id).map_err(|_| Status::BadRequest)?;

    // -------- Phase A: kurz locken & validieren --------
    {
        eprintln!("start_route: before lock (validate)");
        let mut repo = matches.lock().map_err(|_| Status::InternalServerError)?;
        eprintln!("start_route: after lock (validate)");

        let session = repo
            .find_session_by_id_mut(&match_id)
            .ok_or(Status::NotFound)?;

        eprintln!(
        "start_route: meta status={:?} p1={} p2={:?} authed={}",
        session.meta.status,
        session.meta.player1_id,
        session.meta.player2_id,
        auth.user_id
        );

        if session.meta.player1_id != auth.user_id {
            eprintln!("start_route: reject not player1");
            return Ok(Json(StartMatchResponse { ok: false }));
        }
        if session.meta.player2_id.is_none() {
            eprintln!("start_route: reject no player2");
            return Ok(Json(StartMatchResponse { ok: false }));
        }
        if session.meta.status != MatchStatus::Ready {
            eprintln!("start_route: reject status not Ready");
            return Ok(Json(StartMatchResponse { ok: false }));
        }
        // lock wird hier automatisch gedroppt
    }

    // -------- Phase B: Puzzle generieren OHNE lock --------
    eprintln!("start_route: generating puzzle (no lock)");

    let puzzle_opt = generate_puzzle_mvp(); // implementieren (siehe unten)
    let puzzle = match puzzle_opt {
        Some(p) => p,
        None => return Ok(Json(StartMatchResponse { ok: false })),
    };

    // -------- Phase C: kurz locken & starten --------
    {
        eprintln!("start_route: before lock (start_game)");
        let mut repo = matches.lock().map_err(|_| Status::InternalServerError)?;
        eprintln!("start_route: after lock (start_game)");

        let session = repo
            .find_session_by_id_mut(&match_id)
            .ok_or(Status::NotFound)?;

        // Status kann sich geändert haben (race) -> nochmal prüfen
        if session.meta.status != MatchStatus::Ready {
            return Ok(Json(StartMatchResponse { ok: false }));
        }

        let time_limit = Duration::from_secs(6 * 60);
        let now = Instant::now();

        let ok = session.start_game(puzzle, time_limit, now);
        eprintln!("start_route: start_game returned {}", ok);

        if ok {
            session.meta.started_at = Some(Utc::now());
            session.meta.status = MatchStatus::InProgress; // falls du das Enum hast
            session.touch();
        }
        eprintln!("start_route: done ok={}", ok);
        return Ok(Json(StartMatchResponse { ok }));
    }
}

#[get("/match/<match_id>/state")]
pub fn get_match_state_route(
    auth: AuthUser,
    match_id: &str,
    matches: &State<Arc<Mutex<MatchRepository>>>,
) -> Result<Json<GameViewDto>, Status> {
    let match_id = Uuid::parse_str(match_id).map_err(|_| Status::BadRequest)?;
    let mut repo = matches.inner().lock().map_err(|_| Status::InternalServerError)?;

    let view = get_match_state_for_user(&mut repo, &auth.user_id, &match_id)
        .ok_or(Status::NotFound)?;

    Ok(Json(game_view_to_dto(view)))
}

#[post("/match/<match_id>/move", data = "<req>")]
pub fn apply_move_route(
    auth: AuthUser,
    match_id: &str,
    req: Json<ApplyMoveRequest>,
    matches: &State<Arc<Mutex<MatchRepository>>>,
    hub: &State<Arc<WsHub>>,
) -> Result<Json<ApplyMoveResponse>, Status> {
    let match_id = Uuid::parse_str(match_id).map_err(|_| Status::BadRequest)?;

    // MoveDto -> Move (domain)
    let mv = move_dto_to_domain(&req.mv).ok_or(Status::BadRequest)?;

    let mut repo = matches.inner().lock().map_err(|_| Status::InternalServerError)?;

    let (outcome, view) = apply_move_for_user(
        &mut repo,
        &auth.user_id,
        &match_id,
        req.expected_revision,
        mv,
    )
    .ok_or(Status::NotFound)?;

    // Revision mismatch => 409 Conflict
    if matches_revision_mismatch(&outcome) {
        return Err(Status::Conflict);
    }

    // dto view nur einmal bauen
    let dto_view = game_view_to_dto(view);

    // Alles, was in tokio::spawn geht, muss 'static sein: Arc + owned values
    let hub_for_ws = hub.inner().clone();          // Arc<WsHub>
    let dto_view_for_ws = dto_view.clone();        // GameViewDto (Clone)
    let match_id_for_ws = match_id;                // Uuid (Copy/Clone-safe)

    tokio::spawn(async move {
        hub_for_ws
            .publish(
                match_id_for_ws,
                WsServerEvent::RevisionChanged {
                    revision: dto_view_for_ws.revision,
                    view: dto_view_for_ws,
                },
            )
            .await;
    });

    Ok(Json(ApplyMoveResponse {
        outcome: move_outcome_to_dto(outcome),
        view: Some(dto_view),
        replay: false,
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


#[get("/match/<match_id>/ws")]
pub fn match_ws_route(
    auth: AuthUser,
    match_id: &str,
    ws: WebSocket,
    hub: &State<Arc<WsHub>>,
    matches: &State<Arc<Mutex<MatchRepository>>>,
) -> Result<Channel<'static>, Status> {
    let match_id = Uuid::parse_str(match_id).map_err(|_| Status::BadRequest)?;


    // WICHTIG: Arc-Klone ziehen, damit die Closure nur 'static Dinge capturt
    let hub = hub.inner().clone();
    let matches = matches.inner().clone();

    // Wenn AuthUser nicht Clone ist, nimm nur user_id raus:
    let user_id = auth.user_id;

    Ok(ws.channel(move |mut stream| {
        let hub = hub.clone();
        let matches = matches.clone();

        Box::pin(async move {
            // 1) Snapshot erzeugen
            let domain_view = {
                let mut repo = matches.lock().map_err(|_| {
                    rocket_ws::result::Error::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "match repo lock poisoned",
                    ))
                })?;

                get_match_state_for_user(&mut repo, &user_id, &match_id)
                    .ok_or_else(|| rocket_ws::result::Error::ConnectionClosed)?
            };

            // 2) Snapshot senden
            let snapshot = WsServerEvent::Snapshot {
                view: game_view_to_dto(domain_view),
            };
            stream
                .send(Message::Text(serde_json::to_string(&snapshot).unwrap()))
                .await?;

            // 3) Subscribe auf Room
            let mut rx = hub.subscribe(match_id).await;

            // 4) Events weiterleiten + Verbindung offen halten
            loop {
                tokio::select! {
                    msg = stream.next() => {
                        match msg {
                            None => break,
                            Some(Ok(_)) => { /* ignore */ }
                            Some(Err(e)) => return Err(e),
                        }
                    }

                    ev = rx.recv() => {
                        match ev {
                            Ok(ev) => {
                                stream.send(Message::Text(serde_json::to_string(&ev).unwrap())).await?;
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                                // Snapshot neu senden (robust)
                                let domain_view = {
                                    let mut repo = matches.lock().map_err(|_| {
                                        rocket_ws::result::Error::Io(std::io::Error::new(
                                            std::io::ErrorKind::Other,
                                            "match repo lock poisoned",
                                        ))
                                    })?;

                                    get_match_state_for_user(&mut repo, &user_id, &match_id)
                                        .ok_or_else(|| rocket_ws::result::Error::ConnectionClosed)?
                                };

                                let snapshot = WsServerEvent::Snapshot {
                                    view: game_view_to_dto(domain_view),
                                };

                                stream.send(Message::Text(serde_json::to_string(&snapshot).unwrap())).await?;
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                        }
                    }
                }
            }

            Ok(())
        })
    }))
}