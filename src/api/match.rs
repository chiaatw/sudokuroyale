use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Serialize)]
pub struct CreateMatchResponse {
    pub match_id: String,
}

#[post("/match/create")]
pub fn create_match_route() -> Result<Json<CreateMatchResponse>, Status> {
    // Dummy erstmal, damit es kompiliert
    Ok(Json(CreateMatchResponse {
        match_id: "test".to_string(),
    }))
}