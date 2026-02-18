/*  use rocket::serde::{json::Json, Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub ok: bool,
    pub message: String,
}

#[rocket::post("/login", data = "<req>")]
pub fn login(req: Json<LoginRequest>) -> Json<AuthResponse> {
    Json(AuthResponse {
        ok: true,
        message: format!("Login received for {}", req.email),
    })
}

#[rocket::post("/register", data = "<req>")]
pub fn register(req: Json<RegisterRequest>) -> Json<AuthResponse> {
    Json(AuthResponse {
        ok: true,
        message: format!("Register received for {}", req.username),
    })
}*/
