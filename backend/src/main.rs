mod api;
mod game_match;
mod user;

#[macro_use]
extern crate rocket;

use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::{FromRequest, Outcome};
use rocket::serde::json::Json;
use rocket::{Request, State};

use sqlx::PgPool;
use std::sync::Mutex;

use crate::api::dto::error::ApiError;
use crate::api::dto::user::{
    ChangePasswordRequest, HealthResponse, LoginRequest, LoginResponse, MeResponse,
    RegisterRequest, RegisterResponse, RequestResetRequest, ResetPasswordRequest,
};

use crate::user::error::AuthError;
use crate::user::repository::UserRepository;
use crate::user::reset_token_repository::ResetTokenRepository;
use crate::user::services::*;
use crate::user::session_repository::SessionRepository;
use crate::game_match::repository::MatchRepository;

use uuid::Uuid;

#[get("/health")]
fn health(
    users: &State<Mutex<UserRepository>>,
    sessions: &State<Mutex<SessionRepository>>,
    tokens: &State<Mutex<ResetTokenRepository>>,
) -> Json<HealthResponse> {
    let _ = users.lock().unwrap();
    let _ = sessions.lock().unwrap();
    let _ = tokens.lock().unwrap();

    Json(HealthResponse { status: "ok" })
}

#[post("/register", data = "<req>")]
async fn register(
    req: Json<RegisterRequest>,
    users: &State<Mutex<UserRepository>>,
) -> Result<Json<RegisterResponse>, (Status, Json<ApiError>)> {
    let result = {
        let users = users.lock().unwrap();
        register_user(&users, &req.username, &req.email, &req.password).await
    };

    result.map(|_| {
        Json(RegisterResponse {
            message: "User registered successfully",
        })
    })
    .map_err(|e| e.into())
}

#[post("/login", data = "<req>")]
async fn login(
    req: Json<LoginRequest>,
    users: &State<Mutex<UserRepository>>,
    sessions: &State<Mutex<SessionRepository>>,
    cookies: &CookieJar<'_>,
) -> Result<Json<LoginResponse>, (Status, Json<ApiError>)> {
    let session_id = {
        let users = users.lock().unwrap();
        let mut sessions = sessions.lock().unwrap();
        login_user(&users, &mut sessions, &req.username, &req.password).await
    }?;

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

pub struct AuthUser {
    pub user_id: Uuid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ();

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let cookie = match req.cookies().get("session_id") {
            Some(c) => c,
            None => return Outcome::Error((Status::Unauthorized, ())),
        };

        let session_id = match uuid::Uuid::parse_str(cookie.value()) {
            Ok(id) => id,
            Err(_) => return Outcome::Error((Status::Unauthorized, ())),
        };

        let sessions = match req.rocket().state::<Mutex<SessionRepository>>() {
            Some(s) => s,
            None => return Outcome::Error((Status::InternalServerError, ())),
        };

        let sessions = sessions.lock().unwrap();


        match sessions.find_by_session_id(&session_id) {
            Some(session) => Outcome::Success(AuthUser {
                user_id: session.user_id,
            }),
            None => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}


#[get("/me")]
async fn me(
    users: &State<Mutex<UserRepository>>,
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

    let session = {
        let sessions = sessions.lock().unwrap();
        sessions.find_by_session_id(&session_id)
    };

    let user = match session {
        Some(session) => {
            let users = users.lock().unwrap();
            users.find_by_id(&session.user_id).await.ok().flatten()
        }
        None => None,
    };

    match user {
        Some(user) => Ok(Json(MeResponse {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
        })),
        None => Err((
            Status::Unauthorized,
            Json(ApiError {
                code: "unauthorized".into(),
                message: "not authenticated".into(),
                details: None,
            }),
        )),
    }
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


impl From<AuthError> for (Status, Json<ApiError>) {
    fn from(err: AuthError) -> Self {
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

            AuthError::PasswordHashingFailed => {
                (Status::InternalServerError, "password_hashing_failed")
            }

            AuthError::DatabaseError => {
                (Status::InternalServerError, "database_error")
            }
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
}


#[launch]
async fn rocket() -> _ {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to create database pool");

    rocket::build()
        .manage(Mutex::new(UserRepository::new(pool.clone())))
        .manage(Mutex::new(SessionRepository::new()))
        .manage(Mutex::new(ResetTokenRepository::new()))
        .manage(Mutex::new(MatchRepository::new()))
        .mount(
            "/",
            routes![health, register, login, me, logout],
        )
        .mount("/", api::routes::routes())
}
