pub mod r#match;
pub mod user;

use rocket::Route;

pub fn routes() -> Vec<Route> {
    routes![
        r#match::create_match_route,
        r#match::join_match_route,
        r#match::get_match_route,
        r#match::leave_match_route,
    ]
}