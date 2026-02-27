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

#[cfg(test)]
mod tests {
    use super::AuthUser;
    use rocket::http::{Cookie, Status};
    use rocket::local::asynchronous::Client;
    use rocket::{get, routes};
    use std::sync::Mutex;
    use uuid::Uuid;

    use crate::user::session::Session;
    use crate::user::session_repository::SessionRepository;

    #[get("/protected")]
    fn protected_route(user: AuthUser) -> String {
        user.user_id.to_string()
    }

    fn rocket_with_sessions(repo: SessionRepository) -> rocket::Rocket<rocket::Build> {
        rocket::build()
            .manage(Mutex::new(repo))
            .mount("/", routes![protected_route])
    }

    #[rocket::async_test]
    async fn authuser_missing_cookie_returns_401() {
        let repo = SessionRepository::new();
        let rocket = rocket_with_sessions(repo);
        let client = Client::tracked(rocket).await.expect("client");

        let resp = client.get("/protected").dispatch().await;
        assert_eq!(resp.status(), Status::Unauthorized);
    }

    #[rocket::async_test]
    async fn authuser_invalid_uuid_cookie_returns_401() {
        let repo = SessionRepository::new();
        let rocket = rocket_with_sessions(repo);
        let client = Client::tracked(rocket).await.expect("client");

        let resp = client
            .get("/protected")
            .cookie(Cookie::new("session_id", "not-a-uuid"))
            .dispatch()
            .await;

        assert_eq!(resp.status(), Status::Unauthorized);
    }

    #[rocket::async_test]
    async fn authuser_unknown_session_returns_401() {
        let repo = SessionRepository::new();
        let rocket = rocket_with_sessions(repo);
        let client = Client::tracked(rocket).await.expect("client");

        let random_session_id = Uuid::new_v4();

        let resp = client
            .get("/protected")
            .cookie(Cookie::new("session_id", random_session_id.to_string()))
            .dispatch()
            .await;

        assert_eq!(resp.status(), Status::Unauthorized);
    }

    #[rocket::async_test]
    async fn authuser_valid_session_returns_200_and_user_id() {
        let user_id = Uuid::new_v4();
        let session = Session::new(user_id);
        let session_id = session.id;

        let mut repo = SessionRepository::new();
        repo.add_session(session);

        let rocket = rocket_with_sessions(repo);
        let client = Client::tracked(rocket).await.expect("client");

        let resp = client
            .get("/protected")
            .cookie(Cookie::new("session_id", session_id.to_string()))
            .dispatch()
            .await;

        assert_eq!(resp.status(), Status::Ok);
        let body = resp.into_string().await.expect("body");
        assert_eq!(body, user_id.to_string());
    }

    #[rocket::async_test]
    async fn authuser_expired_session_returns_401() {
        use chrono::{Duration, Utc};

        let user_id = Uuid::new_v4();
        let mut session = Session::new(user_id);
        session.expires_at = Utc::now() - Duration::minutes(1);
        let session_id = session.id;

        let mut repo = SessionRepository::new();
        repo.add_session(session);

        let rocket = rocket_with_sessions(repo);
        let client = Client::tracked(rocket).await.expect("client");

        let resp = client
            .get("/protected")
            .cookie(Cookie::new("session_id", session_id.to_string()))
            .dispatch()
            .await;
        assert_eq!(resp.status(), Status::Unauthorized);
    }
}