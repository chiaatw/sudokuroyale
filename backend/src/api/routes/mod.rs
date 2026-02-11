use rocket::routes;
use rocket::Route;

pub mod r#match;
pub mod user;

pub fn routes() -> Vec<Route> {
    routes![
        // match
        r#match::create_match_route,
        r#match::join_match_route,
        r#match::get_match_route,
        r#match::leave_match_route,
        r#match::start_match_route,
        r#match::get_match_state_route,
        r#match::apply_move_route,
        r#match::match_ws_route,

        //user
        /* user::login,
        user::register, */
    ]
}
