use rocket::Route;

pub mod r#match;

pub fn routes() -> Vec<Route> {
    routes![
        // match
        r#match::create_match_route,
        r#match::join_match_route,
        r#match::get_match_route,
        r#match::leave_match_route,
        r#match::start_match_route,
    ]
}