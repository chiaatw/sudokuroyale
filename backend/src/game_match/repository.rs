use uuid::Uuid;

use crate::game_match::model::GameMatch;
use crate::match_state::GameSession;

pub struct MatchRepository {
    sessions: Vec<GameSession>,
}

impl MatchRepository {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
        }
    }

    /// Backwards-compatible: bestehender Code kann weiter GameMatch reinreichen.
    pub fn add_match(&mut self, m: GameMatch) {
        self.sessions.push(GameSession::from_match(m));
    }

    /// Neu: direkt eine Session hinzufügen (falls du später so arbeiten willst)
    pub fn add_session(&mut self, s: GameSession) {
        self.sessions.push(s);
    }

    /// Backwards-compatible: gibt nur Meta zurück (wie vorher)
    pub fn find_by_id(&self, id: &Uuid) -> Option<&GameMatch> {
        self.sessions
            .iter()
            .find(|s| &s.meta.id == id)
            .map(|s| &s.meta)
    }

    /// Backwards-compatible: gibt nur Meta mut zurück (wie vorher)
    pub fn find_by_id_mut(&mut self, id: &Uuid) -> Option<&mut GameMatch> {
        self.sessions
            .iter_mut()
            .find(|s| &s.meta.id == id)
            .map(|s| &mut s.meta)
    }

    /// Neu: Zugriff auf die komplette Session (inkl. game)
    pub fn find_session_by_id(&self, id: &Uuid) -> Option<&GameSession> {
        self.sessions.iter().find(|s| &s.meta.id == id)
    }

    /// Neu: Zugriff auf die komplette Session (inkl. game) mut
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
