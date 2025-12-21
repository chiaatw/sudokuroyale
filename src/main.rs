mod user;

#[macro_use]
extern crate rocket;

use rocket::serde::json::Json;
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[get("/health")]
fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![health])
}
