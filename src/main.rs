mod user;
mod game_match;

#[macro_use]
extern crate rocket;

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use rocket::http::{Cookie, CookieJar};
use crate::user::services::login_user;
use rocket::request::{FromRequest, Outcome};


// nur EINMAL, und zwar mit crate::
use crate::user::repository::UserRepository;
use crate::user::session_repository::SessionRepository;
use crate::user::reset_token_repository::ResetTokenRepository;
use crate::user::error::AuthError;
use crate::user::services::register_user;
use crate::user::services::get_user_from_session;
use crate::user::services::change_password;


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
    let _users = user.lock().unwrap();
    let _sessions = session.lock().unwrap();
    let _tokens = token.lock().unwrap();

    Json(HealthResponse { status: "ok" })
}

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}
#[derive(Deserialize)]
struct ChangePasswordRequest {
    old_password: String,
    new_password: String,
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
    req: Json<RegisterRequest>,
    users: &State<Mutex<UserRepository>>,
) -> Result<Json<RegisterResponse>, (Status, Json<ErrorResponse>)> {
    let mut users = users.lock().unwrap();

    match register_user(&mut users, &req.username, &req.email, &req.password) {
        Ok(_) => Ok(Json(RegisterResponse {
            message: "User registered successfully",
        })),
        Err(err) => Err(err.into()),
    }
}

#[derive(Serialize)]
struct LoginResponse {
    message: &'static str,
}

#[post("/login", data = "<req>")]
fn login(
    req: Json<RegisterRequest>,
    users: &State<Mutex<UserRepository>>,
    sessions: &State<Mutex<SessionRepository>>,
    cookies: &CookieJar<'_>,
) -> Result<Json<LoginResponse>, (Status, Json<ErrorResponse>)> {
    let users_guard = users.lock().unwrap();
    let mut sessions_guard = sessions.lock().unwrap();

    match login_user(&users_guard, &mut sessions_guard, &req.username, &req.password) {
        Ok(session_id) => {
            let cookie = Cookie::build(("session_id", session_id.to_string()))
                .path("/")
                .http_only(true)
                .build();

            cookies.add(cookie);

            Ok(Json(LoginResponse {
                message: "User logged in successfully",
            }))

        },
        Err(err) => Err(err.into()),
    }
}

#[derive(Serialize)]
struct MeResponse {
    id: String,
    username: String,
    email: String,
}

struct AuthUser {
    user_id: uuid::Uuid,
}
#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ();

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let cookies = match req.cookies().get("session_id") {
            Some(c) => c,
            None => return Outcome::Error((Status::Unauthorized, ())),
        };

        let session_id = match uuid::Uuid::parse_str(cookies.value()) {
            Ok(id) => id,
            Err(_) => return Outcome::Error((Status::Unauthorized, ())),
        };

        let sessions = match req.rocket().state::<Mutex<SessionRepository>>() {
            Some(s) => s,
            None => return Outcome::Error((Status::InternalServerError, ())),
        };

        let sessions_guard = sessions.lock().unwrap();
        match sessions_guard.find_by_session_id(&session_id) {
            Some(session) => Outcome::Success(AuthUser { user_id: session.user_id }),

            None => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}

#[get("/me")]
fn me(
    users: &State<Mutex<UserRepository>>,
    sessions: &State<Mutex<SessionRepository>>,
    cookies: &CookieJar<'_>,
) -> Result<Json<MeResponse>, (Status, Json<ErrorResponse>)> {

    let session_cookie = cookies.get("session_id").ok_or_else(|| {
        (
            Status::Unauthorized,
            Json(ErrorResponse {
                error: "not authenticated".to_string(),
            }),
        )
    })?;

    let session_id = uuid::Uuid::parse_str(session_cookie.value()).map_err(|_| {
        (
            Status::Unauthorized,
            Json(ErrorResponse {
                error: "invalid session id".to_string(),
            }),
        )
    })?;

    let users_guard = users.lock().unwrap();
    let sessions_guard = sessions.lock().unwrap();

    match get_user_from_session(&sessions_guard, &users_guard, &session_id) {
        Some(user) => Ok(Json(MeResponse {
            id: user.id.to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
        })),
        None => Err((
            Status::Unauthorized,
            Json(ErrorResponse {
                error: "invalid session".to_string(),
            }),
        )),
    }
}

#[post("/logout")]
fn logout(
    sessions: &State<Mutex<SessionRepository>>,
    cookies: &CookieJar<'_>,
) -> Result<Json<RegisterResponse>, (Status, Json<ErrorResponse>)> {

    // 1) Cookie lesen
    let session_cookie = cookies.get("session_id").ok_or_else(|| {
        (
            Status::Unauthorized,
            Json(ErrorResponse {
                error: "not authenticated".to_string(),
            }),
        )
    })?;

    // 2) Session-ID parsen
    let session_id = uuid::Uuid::parse_str(session_cookie.value()).map_err(|_| {
        (
            Status::Unauthorized,
            Json(ErrorResponse {
                error: "invalid session id".to_string(),
            }),
        )
    })?;

    // 3) Session löschen
    let mut sessions_guard = sessions.lock().unwrap();
    sessions_guard.remove_session(&session_id);

    // 4) Cookie entfernen
    cookies.remove(Cookie::from("session_id"));

    Ok(Json(RegisterResponse {
        message: "logout successful",
    }))
}


#[post("/change-password", data = "<req>")]
fn change_password_endpoint(
    auth: AuthUser,
    req: Json<ChangePasswordRequest>,
    users: &State<Mutex<UserRepository>>,
    sessions: &State<Mutex<SessionRepository>>, // ← NEU
)
 -> Result<Json<RegisterResponse>, (Status, Json<ErrorResponse>)> {

        let sessions_guard = sessions.lock().unwrap();
        let mut users_guard = users.lock().unwrap();

        match change_password(
            &sessions_guard,          // ← FEHLENDES ARGUMENT
            &mut users_guard,
            &auth.user_id,
            &req.new_password,
            &req.old_password,
        ) {
        Ok(_) => Ok(Json(RegisterResponse {
            message: "password changed successfully",
        })),
        Err(err) => Err(err.into()),
    }
}




impl From<AuthError> for (Status, Json<ErrorResponse>) {
    fn from(err: AuthError) -> Self {
        let status = match err {
            AuthError::InvalidUsername | AuthError::InvalidEmail | AuthError::InvalidPassword => Status::BadRequest,
            AuthError::UsernameExists => Status::Conflict,
            _ => Status::Unauthorized,
        };

        (status, Json(ErrorResponse { error: err.to_string() }))
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(Mutex::new(UserRepository::new()))
        .manage(Mutex::new(SessionRepository::new()))
        .manage(Mutex::new(ResetTokenRepository::new()))
        .mount("/", routes![health, register, login, me, logout, change_password_endpoint])
}