use rocket::http::Status;

#[options("/<_..>")]
pub fn all_options() -> Status {
    Status::NoContent
}