use uuid::Uuid;
use chrono::Utc;

use crate::game_match::model::{GameMatch, MatchStatus};
use crate::game_match::repository::MatchRepository;

use crate::user::repository::UserRepository;
use crate::user::session_repository::SessionRepository;
use crate::user::services::get_user_from_session;

pub fn create_match(
    users: &UserRepository,
    sessions: &SessionRepository,
    match_repo: &mut MatchRepository,
    session_id: &Uuid,
) -> Option<Uuid> {
    let user = get_user_from_session(sessions, users, session_id)?;
    let m = GameMatch::new(user.id);
    let match_id = m.id;
    match_repo.add_match(m);
    Some(match_id)
}

pub fn join_match(
    users: &UserRepository,
    sessions: &SessionRepository,
    match_repo: &mut MatchRepository,
    session_id: &Uuid,
    match_id: &Uuid,
) -> bool {
    let user = match get_user_from_session(sessions, users, session_id) {
        Some(u) => u,
        None => return false,
    };

    let m = match match_repo.find_by_id_mut(match_id) {
        Some(m) => m,
        None => return false,
    };

    if m.status != MatchStatus::Waiting {
        return false;
    }
    if m.player1_id == user.id {
        return false;
    }

    m.player2_id = Some(user.id);
    true
}

pub fn leave_match_by_user(
    repo: &mut MatchRepository,
    user_id: &Uuid,
    match_id: &Uuid,
) -> bool {
    // 1) Match mut holen
    let m = match repo.find_by_id_mut(match_id) {
        Some(m) => m,
        None => return false,
    };

    // 2) Wenn Player1 geht -> löschen
    if m.player1_id == *user_id {
        let id = m.id;
        return repo.remove_match(&id);
    }

    // 3) Wenn Player2 geht -> player2 entfernen + status zurück
    if m.player2_id == Some(*user_id) {
        m.player2_id = None;
        m.status = MatchStatus::Waiting;
        return true;
    }

    false
}