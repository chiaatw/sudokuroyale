use chrono::Utc;
use uuid::Uuid;

use crate::game_match::model::{GameMatch, MatchStatus};
use crate::game_match::repository::MatchRepository;

/// Neues Match erstellen – synchron, bekommt user_id direkt.
/// (Auth & Session wird in der Route gemacht via AuthUser)
pub fn create_match(match_repo: &mut MatchRepository, user_id: &Uuid) -> Uuid {
    let m = GameMatch::new(*user_id);
    let match_id = m.id;
    match_repo.add_match(m);
    match_id
}

pub fn join_match(match_repo: &mut MatchRepository, user_id: &Uuid, match_id: &Uuid) -> bool {
    let m = match match_repo.find_by_id_mut(match_id) {
        Some(m) => m,
        None => return false,
    };

    if m.status != MatchStatus::Waiting {
        return false;
    }
    if m.player1_id == *user_id {
        return false;
    }

    m.player2_id = Some(*user_id);
    m.status = MatchStatus::Ready;
    true
}

pub fn leave_match_by_user(repo: &mut MatchRepository, user_id: &Uuid, match_id: &Uuid) -> bool {
    let m = match repo.find_by_id_mut(match_id) {
        Some(m) => m,
        None => return false,
    };

    // Player1 geht -> Match löschen
    if m.player1_id == *user_id {
        let id = m.id;
        return repo.remove_match(&id);
    }

    // Player2 geht -> slot freimachen
    if m.player2_id == Some(*user_id) {
        m.player2_id = None;
        m.status = MatchStatus::Waiting;
        return true;
    }

    false
}

pub fn start_match_by_user(match_repo: &mut MatchRepository, user_id: &Uuid, match_id: &Uuid) -> bool {
    // Fürs MVP setzt das nur Running.
    // Nächster Schritt danach: auf GameSession umstellen + session.start_game(...)
    let m = match match_repo.find_by_id_mut(match_id) {
        Some(m) => m,
        None => return false,
    };

    if m.player1_id != *user_id {
        return false;
    }
    if m.player2_id.is_none() {
        return false;
    }
    if m.status != MatchStatus::Ready {
        return false;
    }

    m.status = MatchStatus::Running;
    m.started_at = Some(Utc::now());
    true
}
