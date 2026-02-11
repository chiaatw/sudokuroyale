use uuid::Uuid;

use crate::game_match::model::GameMatch;
use crate::match_state::GameSession;

pub struct MatchRepository {
    sessions: Vec<GameSession>,
}

impl MatchRepository {
    pub fn new() -> Self {
        Self { sessions: Vec::new() }
    }

    pub fn add_match(&mut self, m: GameMatch) {
        self.sessions.push(GameSession::from_match(m));
    }

    pub fn add_session(&mut self, s: GameSession) {
        self.sessions.push(s);
    }

    pub fn find_by_id(&self, id: &Uuid) -> Option<&GameMatch> {
        for s in &self.sessions {
            if &s.meta.id == id {
                return Some(&s.meta);
            }
        }
        None
    }

    pub fn find_by_id_mut(&mut self, id: &Uuid) -> Option<&mut GameMatch> {
        for s in &mut self.sessions {
            if &s.meta.id == id {
                return Some(&mut s.meta);
            }
        }
        None
    }

    pub fn find_session_by_id(&self, id: &Uuid) -> Option<&GameSession> {
        self.sessions.iter().find(|s| &s.meta.id == id)
    }

    pub fn find_session_by_id_mut(&mut self, id: &Uuid) -> Option<&mut GameSession> {
        self.sessions.iter_mut().find(|s| &s.meta.id == id)
    }

    pub fn remove_match(&mut self, id: &Uuid) -> bool {
        if let Some(pos) = self.sessions.iter().position(|s| &s.meta.id == id) {
            self.sessions.swap_remove(pos);
            true
        } else {
            false
        }
    }
}
