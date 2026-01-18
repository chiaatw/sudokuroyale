mod user;
mod game_match;
mod api;

#[macro_use]
extern crate rocket;

use sqlx::PgPool;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use rocket::http::{Cookie, CookieJar};
use rocket::request::{FromRequest, Outcome};
use::rocket::Request;



// nur EINMAL, und zwar mit crate::
use crate::user::repository::UserRepository;
use crate::user::session_repository::SessionRepository;
use crate::user::reset_token_repository::ResetTokenRepository;
use crate::user::error::AuthError;
use crate::user::services::register_user;
use crate::user::services::get_user_from_session;
use crate::user::services::change_password;
use crate::user::services::request_password_reset;
use crate::user::services::login_user;
use crate::user::services::reset_password;



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

#[derive(Deserialize)]
struct RequestResetRequest {
    email: String,
}

#[derive(Deserialize)]
struct ResetPasswordRequest {
    token: String,
    new_password: String,
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
async fn me(
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

    match get_user_from_session(&sessions_guard, &users_guard, &session_id).await {
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
    sessions: &State<Mutex<SessionRepository>>, 
)
 -> Result<Json<RegisterResponse>, (Status, Json<ErrorResponse>)> {

        let sessions_guard = sessions.lock().unwrap();
        let mut users_guard = users.lock().unwrap();

        match change_password(
            &sessions_guard,          
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

#[post("/request-reset", data = "<req>")]
fn request_reset(
    req: Json<RequestResetRequest>,
    users: &State<Mutex<UserRepository>>,
    tokens: &State<Mutex<ResetTokenRepository>>,
) -> Result<Json<RegisterResponse>, (Status, Json<ErrorResponse>)> {

    let users_guard = users.lock().unwrap();
    let mut tokens_guard = tokens.lock().unwrap();

    let _ = request_password_reset(
        &users_guard,
        &mut tokens_guard,
        &req.email,
    );

    Ok(Json(RegisterResponse {
        message: "if the email exists, a reset link was sent",
    }))
}

#[post("/reset-password", data = "<req>")]
fn reset_password_endpoint(
    req: Json<ResetPasswordRequest>,
    users: &State<Mutex<UserRepository>>,
    tokens: &State<Mutex<ResetTokenRepository>>,
) -> Result<Json<RegisterResponse>, (Status, Json<ErrorResponse>)> {

    let mut users_guard = users.lock().unwrap();
    let mut tokens_guard = tokens.lock().unwrap();
    let token_id = uuid::Uuid::parse_str(&req.token).map_err(|_| {
    (
        Status::BadRequest,
        Json(ErrorResponse {
            error: "invalid reset token".to_string(),
        }),
    )
    })?;


    match reset_password(
        &mut users_guard,
        &mut tokens_guard,
        &token_id,
        &req.new_password,
    ) {
        Ok(_) => Ok(Json(RegisterResponse {
            message: "password reset successful",
        })),
        Err(err) => Err(err.into()),
    }
}

#[catch(401)]
fn unauthorized(_req: &Request) -> Json<ErrorResponse> {
    Json(ErrorResponse {
        error: "unauthorized".to_string(),
    })
}

#[catch(404)]
fn not_found(_req: &Request) -> Json<ErrorResponse> {
    Json(ErrorResponse {
        error: "not found".to_string(),
    })
}

#[catch(500)]
fn internal_error(_req: &Request) -> Json<ErrorResponse> {
    Json(ErrorResponse {
        error: "internal server error".to_string(),
    })
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
async fn rocket() -> _ {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to create database pool");

    rocket::build()
        .manage(pool)
        .manage(Mutex::new(UserRepository::new(pool.clone())))
        .manage(Mutex::new(SessionRepository::new()))
        .manage(Mutex::new(ResetTokenRepository::new()))
        .manage(std::sync::Mutex::new(game_match::repository::MatchRepository::new()))
        .mount(
            "/",
            routes![
                health,
                register,
                login,
                me,
                logout,
                change_password_endpoint,
                request_reset,
                reset_password_endpoint
            ],
        )
        .mount("/", api::routes())
        .register("/", catchers![unauthorized, not_found, internal_error])
}
