use crate::game_match::model::GameMatch;
use uuid::Uuid;

pub struct MatchRepository {
    matches: Vec<GameMatch>,
}

impl MatchRepository {
    pub fn new() -> Self {
        Self {
            matches: Vec::new(),
        }
    }

    pub fn add_match(&mut self, m: GameMatch) {
        self.matches.push(m);
    }

    pub fn find_by_id(&self, id: &Uuid) -> Option<&GameMatch> {
        self.matches.iter().find(|m| &m.id == id)
    }

    pub fn find_by_id_mut(&mut self, id: &Uuid) -> Option<&mut GameMatch> {
        self.matches.iter_mut().find(|m| &m.id == id)
    }

    pub fn remove_match(&mut self, id: &Uuid) -> bool {
        if let Some(pos) = self.matches.iter().position(|m| &m.id == id) {
            self.matches.swap_remove(pos);
            true
        } else {
            false
        }
    }
}
