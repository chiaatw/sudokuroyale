mod user;
mod game_match;


#[macro_use]
extern crate rocket;

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

//repos aus user 
use crate::user::repository::UserRepository;
use crate::user::session_repository::SessionRepository;
use crate::user::reset_token_repository::ResetTokenRepository;

use crate::user::error::AuthError;
use crate::user::services::register_user;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}


#[get("/health")]
fn health(
    user: &State<Mutex<UserRepository>>,
    session: &State<Mutex<SessionRepository>>,
    token: &State<Mutex<ResetTokenRepository>>,
) -> Json<HealthResponse> {
    //zugriff testen
    let _users = user.lock().unwrap();
    let _sessions = session.lock().unwrap();
    let _tokens = token.lock().unwrap();

    Json(HealthResponse { status: "ok" })
}


#[derive(Deserialize)]
struct ResgisterRequest {
    username: String,
    email: String,
    password: String,
}

#[derive(Serialize)]
struct RegisterResponse {
    message: &'static str,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[post("/register", data = "<req>")]
fn register(
    req: Json<ResgisterRequest>,
    users: &State<Mutex<UserRepository>>,
) -> Result<Json<RegisterResponse>, (Status, Json<ErrorResponse>)> {
    let mut users= users.lock().unwrap();

    match register_user(&mut users, &req.username, &req.email, &req.password) {
        Ok(_) => Ok(Json(RegisterResponse {
            message: "User registered successfully",
        })),
        Err(err) => Err(err.into()),
    }
}

impl From<AuthError> for (Status, Json<ErrorResponse>) {
    fn from(err: AuthError) -> Self {
        let status = match err {
            AuthError::InvalidUsername
            | AuthError::InvalidEmail
            | AuthError::InvalidPassword => Status::BadRequest,

            AuthError::UsernameExists => Status::Conflict,

            _ => Status::Unauthorized,
        };

        ( status, Json(ErrorResponse {
            error: err.to_string(),
        }),
        )
    }
}

            
#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(Mutex::new(UserRepository::new()))
        .manage(Mutex::new(SessionRepository::new()))
        .manage(Mutex::new(ResetTokenRepository::new()))
        .mount("/", routes![health, register])
}

use user::repository::UserRepository;
use user::session_repository::SessionRepository;
use user::services::{register_user, login_user};
use game_match::repository::MatchRepository;
use user::services::get_user_from_session;
use game_match::services::{create_match, join_match};

fn main() {
    let mut users = UserRepository::new();
    let mut sessions = SessionRepository::new();

    // Test: Registrierung
    register_user(&mut users, "alice", "alice@test.de", "SecurePass1!")
        .expect("register failed");

    // Test: Login
    let session_id = login_user(&users, &mut sessions, "alice", "SecurePass1!")
        .expect("login failed");

    println!("Logged in! Session-ID: {}", session_id);

    let mut match_repo = MatchRepository::new();

    let match_id = create_match(&users, &sessions, &mut match_repo, &session_id)
        .expect("create match failed");
    println!("Match created: {}", match_id);
    
    register_user(&mut users, "bob", "bob@test.de", "SecurePass1!")
    .expect("register bob failed");

    let session_id_bob = login_user(&users, &mut sessions, "bob", "SecurePass1!")
    .expect("login bob failed");

    let ok = join_match(&users, &sessions, &mut match_repo, &session_id_bob, &match_id);
    println!("Join ok? {}", ok);
    
    let m = match_repo.find_by_id(&match_id).unwrap();
    println!("After join: status={:?}, p1={:?}, p2={:?}", m.status, m.player1_id, m.player2_id);
}

