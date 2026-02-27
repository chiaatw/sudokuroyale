use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::game::puzzle::Puzzle;
use crate::game::Game;
use crate::game::PlayerId;
use crate::game_match::model::{GameMatch, MatchStatus};

/// Laufender Match-Zustand
#[derive(Debug, Clone)]
pub struct GameSession {
    pub meta: GameMatch,
    pub game: Option<Game>,
    pub last_activity_at: DateTime<Utc>,
}

impl GameSession {
    pub fn new(player1_id: Uuid) -> Self {
        Self {
            meta: GameMatch::new(player1_id),
            game: None,
            last_activity_at: Utc::now(),
        }
    }

    pub fn from_match(meta: GameMatch) -> Self {
        Self {
            meta,
            game: None,
            last_activity_at: Utc::now(),
        }
    }

    pub fn player_for_user(&self, user_id: &Uuid) -> Option<PlayerId> {
        if self.meta.player1_id == *user_id {
            return Some(PlayerId::PlayerA);
        }
        if self.meta.player2_id == Some(*user_id) {
            return Some(PlayerId::PlayerB);
        }
        None
    }

    pub fn touch(&mut self) {
        self.last_activity_at = Utc::now();
    }

    pub fn start_game(&mut self, puzzle: Puzzle, time_limit: Duration, now: Instant) -> bool {
        if self.meta.status != MatchStatus::Ready {
            return false;
        }
        if self.meta.player2_id.is_none() {
            return false;
        }

        let mut game = Game::new(puzzle, time_limit);
        game.start(now);

        self.game = Some(game);
        self.meta.status = MatchStatus::InProgress;
        self.meta.started_at = Some(Utc::now());
        self.touch();
        true
    }
}
