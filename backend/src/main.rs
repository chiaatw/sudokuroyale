#[macro_use]
extern crate rocket;

use rocket::http::{Cookie, CookieJar, Status};
use rocket::serde::json::Json;
use rocket::State;

use sqlx::PgPool;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use dotenvy::dotenv;




use sudokuroyale::api::dto::error::ApiError;
use sudokuroyale::api::dto::user::{
    HealthResponse, LoginRequest, LoginResponse, MeResponse, RegisterRequest, RegisterResponse,
};

use sudokuroyale::user::error::AuthError;
use sudokuroyale::user::repository::UserRepository;
use sudokuroyale::user::reset_token_repository::ResetTokenRepository;
use sudokuroyale::user::services::{authenticate_user, create_session_for_user, register_user};
use sudokuroyale::user::session_repository::SessionRepository;

use sudokuroyale::game_match::repository::MatchRepository;

fn auth_error_to_response(err: AuthError) -> (Status, Json<ApiError>) {
    let (status, code) = match err {
        AuthError::InvalidUsername => (Status::BadRequest, "invalid_username"),
        AuthError::InvalidEmail => (Status::BadRequest, "invalid_email"),
        AuthError::InvalidPassword => (Status::BadRequest, "invalid_password"),

        AuthError::UsernameExists => (Status::Conflict, "username_exists"),
        AuthError::EmailExists => (Status::Conflict, "email_exists"),

        AuthError::UserNotFound => (Status::Unauthorized, "user_not_found"),
        AuthError::InvalidPasswordLogin => (Status::Unauthorized, "invalid_password"),
        AuthError::SessionExpired => (Status::Unauthorized, "session_expired"),

        AuthError::TokenInvalid => (Status::BadRequest, "token_invalid"),
        AuthError::TokenExpired => (Status::BadRequest, "token_expired"),

        AuthError::PasswordHashingFailed => (Status::InternalServerError, "password_hashing_failed"),
        AuthError::DatabaseError => (Status::InternalServerError, "database_error"),
    };

    (
        status,
        Json(ApiError {
            code: code.into(),
            message: err.to_string(),
            details: None,
        }),
    )
}

#[get("/health")]
fn health(
    users: &State<UserRepository>,
    sessions: &State<Mutex<SessionRepository>>,
    tokens: &State<Mutex<ResetTokenRepository>>,
    matches: &State<Arc<Mutex<MatchRepository>>>,
) -> Json<HealthResponse> {
    let _ = users;
    
    drop(sessions.lock().unwrap());
    drop(tokens.lock().unwrap());
    drop(matches.lock().unwrap());

    Json(HealthResponse { status: "ok" })
}

#[post("/register", data = "<req>")]
async fn register(
    req: Json<RegisterRequest>,
    users: &State<UserRepository>,
) -> Result<Json<RegisterResponse>, (Status, Json<ApiError>)> {
    register_user(users.inner(), &req.username, &req.email, &req.password)
        .await
        .map(|_| {
            Json(RegisterResponse {
                message: "User registered successfully",
            })
        })
        .map_err(auth_error_to_response)
}

#[post("/login", data = "<req>")]
async fn login(
    req: Json<LoginRequest>,
    users: &State<UserRepository>,
    sessions: &State<Mutex<SessionRepository>>,
    cookies: &CookieJar<'_>,
) -> Result<Json<LoginResponse>, (Status, Json<ApiError>)> {
    // 1) async: nur DB check (keine Locks!)
    let user_id = authenticate_user(users.inner(), &req.username, &req.password)
        .await
        .map_err(auth_error_to_response)?;

    // 2) sync: Session erstellen (Lock nur kurz, kein await)
    let session_id = {
        let mut sessions_guard = sessions.lock().unwrap();
        create_session_for_user(&mut sessions_guard, user_id)
    };

    cookies.add(
        Cookie::build(("session_id", session_id.to_string()))
            .path("/")
            .http_only(true)
            .build(),
    );

    Ok(Json(LoginResponse {
        message: "User logged in successfully",
    }))
}

#[get("/me")]
async fn me(
    users: &State<UserRepository>,
    sessions: &State<Mutex<SessionRepository>>,
    cookies: &CookieJar<'_>,
) -> Result<Json<MeResponse>, (Status, Json<ApiError>)> {
    let cookie = cookies.get("session_id").ok_or_else(|| {
        (
            Status::Unauthorized,
            Json(ApiError {
                code: "unauthorized".into(),
                message: "not authenticated".into(),
                details: None,
            }),
        )
    })?;

    let session_id = Uuid::parse_str(cookie.value()).map_err(|_| {
        (
            Status::Unauthorized,
            Json(ApiError {
                code: "unauthorized".into(),
                message: "not authenticated".into(),
                details: None,
            }),
        )
    })?;

    // user_id kopieren (kein borrow über guard)
    let user_id = {
        let sessions_guard = sessions.lock().unwrap();
        match sessions_guard.find_by_session_id(&session_id) {
            Some(s) if s.is_valid() => s.user_id,
            _ => {
                return Err((
                    Status::Unauthorized,
                    Json(ApiError {
                        code: "unauthorized".into(),
                        message: "not authenticated".into(),
                        details: None,
                    }),
                ))
            }
        }
    };

    let user = users
        .find_by_id(&user_id)
        .await
        .map_err(|_| auth_error_to_response(AuthError::DatabaseError))?
        .ok_or_else(|| {
            (
                Status::Unauthorized,
                Json(ApiError {
                    code: "unauthorized".into(),
                    message: "not authenticated".into(),
                    details: None,
                }),
            )
        })?;

    Ok(Json(MeResponse {
        id: user.id.to_string(),
        username: user.username,
        email: user.email,
    }))
}

#[post("/logout")]
fn logout(
    sessions: &State<Mutex<SessionRepository>>,
    cookies: &CookieJar<'_>,
) -> Result<Json<RegisterResponse>, (Status, Json<ApiError>)> {
    let cookie = cookies.get("session_id").ok_or_else(|| {
        (
            Status::Unauthorized,
            Json(ApiError {
                code: "unauthorized".into(),
                message: "not authenticated".into(),
                details: None,
            }),
        )
    })?;

    let session_id = Uuid::parse_str(cookie.value()).map_err(|_| {
        (
            Status::Unauthorized,
            Json(ApiError {
                code: "unauthorized".into(),
                message: "not authenticated".into(),
                details: None,
            }),
        )
    })?;

    sessions.lock().unwrap().remove_session(&session_id);
    cookies.remove(Cookie::from("session_id"));

    Ok(Json(RegisterResponse {
        message: "logout successful",
    }))
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok();    
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to create database pool");

    rocket::build()
        // DB Repo: ohne Mutex
        .manage(UserRepository::new(pool.clone()))
        // In-Memory: mit Mutex
        .manage(Mutex::new(SessionRepository::new()))
        .manage(Mutex::new(ResetTokenRepository::new()))
        .manage(Arc::new(Mutex::new(MatchRepository::new())))
        .mount("/", routes![health, register, login, me, logout])
        // API aus der Library (Match-Routes etc.)
        .manage(Arc::new(sudokuroyale::api::ws_hub::WsHub::new(64)))
        .mount("/", sudokuroyale::api::routes::routes())
}
