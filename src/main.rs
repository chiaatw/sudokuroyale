mod user;

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
