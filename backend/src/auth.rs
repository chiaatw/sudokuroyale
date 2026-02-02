use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use uuid::Uuid;

use std::sync::Mutex;

use crate::user::session_repository::SessionRepository;

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

        let session_id = match Uuid::parse_str(cookie.value()) {
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
