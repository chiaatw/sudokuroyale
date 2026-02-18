use chrono::Utc;
use uuid::Uuid;

use std::time::{Duration, Instant};

use crate::build::{Finder, Generator};
use crate::game::outcome::MoveOutcome;
use crate::game::player::PlayerId;
use crate::game::puzzle::Puzzle as GamePuzzle;
use crate::game::r#move::Move;
use crate::game::view::GameView;
use crate::game_match::model::{GameMatch, MatchStatus};
use crate::game_match::repository::MatchRepository;
use crate::io::Cancelable;
use crate::layout::{Cell, Grid};
use crate::match_state::GameSession;
use crate::puzzle::{Board, Changer, Options}; 

/// Neues Match erstellen – synchron, bekommt user_id direkt
/// Legt automatisch eine GameSession an (meta + game=None)
pub fn create_match(match_repo: &mut MatchRepository, user_id: &Uuid) -> Uuid {
    let meta = GameMatch::new(*user_id);
    let match_id = meta.id;

    let session = GameSession::from_match(meta);
    match_repo.add_session(session);

    match_id
}

pub fn join_match(match_repo: &mut MatchRepository, user_id: &Uuid, match_id: &Uuid) -> bool {
    // Session holen (nicht nur Meta)
    let session = match match_repo.find_session_by_id_mut(match_id) {
        Some(s) => s,
        None => return false,
    };

    eprintln!("start_match: user_id={user_id} match_id={match_id}");
    eprintln!(
        "start_match: meta status={:?}, p1={}, p2={:?}",
        session.meta.status, session.meta.player1_id, session.meta.player2_id
    );

    let m = &mut session.meta;

    if m.status != MatchStatus::Waiting {
        return false;
    }
    if m.player1_id == *user_id {
        return false;
    }

    m.player2_id = Some(*user_id);
    m.status = MatchStatus::Ready;

    session.touch();
    true
}

pub fn leave_match_by_user(
    match_repo: &mut MatchRepository,
    user_id: &Uuid,
    match_id: &Uuid,
) -> bool {
    // Session mut holen
    let session = match match_repo.find_session_by_id_mut(match_id) {
        Some(s) => s,
        None => return false,
    };

    let m = &mut session.meta;

    // Player1 geht -> Match komplett löschen
    if m.player1_id == *user_id {
        let id = m.id;
        return match_repo.remove_match(&id);
    }

    // Player2 geht -> slot freimachen + zurück auf Waiting
    if m.player2_id == Some(*user_id) {
        m.player2_id = None;
        m.status = MatchStatus::Waiting;

        session.game = None;

        session.touch();
        return true;
    }

    false
}

/// Hilfsfunktion: Solver-Board -> Game-Grid
fn board_to_grid(board: &Board) -> Grid {
    let mut grid = Grid::new();
    for i in 0..81 {
        let cell = Cell::new(i);
        grid.set(cell, board.value(cell));
    }
    grid
}

pub(crate) fn generate_puzzle_mvp() -> Option<GamePuzzle> {
    eprintln!("gen: start");
    let t0 = Instant::now();

    let changer = Changer::new(Options::errors());

    // Gesamt-Retries: erst solved finden, dann Finder anwenden
    for attempt in 1..=120 {
        eprintln!("gen: attempt {}", attempt);

        let cancelable = Cancelable::new();

        // 1) solved board erzeugen
        let mut gen = Generator::new(true, false);
        let solved = match gen.generate(&changer, &cancelable) {
            Some(b) if b.known_count() == 81 => b,
            Some(b) => {
                eprintln!(
                    "gen: generator returned incomplete board (known_count={})",
                    b.known_count()
                );
                continue;
            }
            None => {
                eprintln!("gen: generator returned None");
                continue;
            }
        };

        let solution = solved.clone();

        // 2) Finder: aus solved -> givens (Puzzle)
        let mut finder = Finder::new(28, 5, false);
        eprintln!("gen: starting finder...");
        let (givens_board, _effects) = finder.backtracking_find(solved);

        let clues = givens_board.known_count();
        eprintln!("gen: finder clues={}", clues);

        // 3) Guards: kaputte Ergebnisse verwerfen und weiter versuchen
        if clues < 17 || clues > 81 {
            eprintln!("gen: guard failed (clues={}), retrying", clues);
            continue;
        }

        // 4) Puzzle bauen
        let givens_grid = board_to_grid(&givens_board);
        let solution_grid = board_to_grid(&solution);

        eprintln!("gen: returning REAL puzzle total {:?}", t0.elapsed());
        return Some(GamePuzzle::new(givens_grid, solution_grid));
    }

    eprintln!("gen: FAIL after retries total {:?}", t0.elapsed());
    None
}

pub fn start_match_by_user(
    match_repo: &mut MatchRepository,
    user_id: &Uuid,
    match_id: &Uuid,
) -> bool {
    let session = match match_repo.find_session_by_id_mut(match_id) {
        Some(s) => s,
        None => return false,
    };

    if session.meta.player1_id != *user_id {
        return false;
    }
    if session.meta.player2_id.is_none() {
        return false;
    }
    if session.meta.status != MatchStatus::Ready {
        return false;
    }

    let mut solved_opt: Option<Board> = None;

    for attempt in 0..50 {
        let changer = Changer::new(Options::errors());
        let cancelable = Cancelable::new();
        let mut gen = Generator::new(true, false);

        match gen.generate(&changer, &cancelable) {
            Some(b) if b.known_count() == 81 => {
                eprintln!("start_match: solved board on attempt {}", attempt + 1);
                solved_opt = Some(b);
                break;
            }
            Some(b) => {
                eprintln!(
                    "start_match: attempt {} incomplete board (known_count={})",
                    attempt + 1,
                    b.known_count()
                );
            }
            None => {
                eprintln!(
                    "start_match: generator returned None on attempt {}",
                    attempt + 1
                );
            }
        }
    }

    let solved = match solved_opt {
        Some(b) => b,
        None => return false,
    };

    let solved_clone = solved.clone();

    let mut finder = Finder::new(28, 5, false);
    eprintln!("start_match: starting finder...");
    let (givens_board, _effects) = finder.backtracking_find(solved);
    eprintln!("start_match: finder finished");

    let clues = givens_board.known_count();
    eprintln!("start_match: givens clues={}", clues);

    if clues < 17 || clues > 81 {
        eprintln!("start_match: clues guard failed");
        return false;
    }

    let givens_grid = board_to_grid(&givens_board);
    let solution_grid = board_to_grid(&solved_clone);
    let puzzle = GamePuzzle::new(givens_grid, solution_grid);

    let time_limit = Duration::from_secs(6 * 60); 
    let now = Instant::now();

    let ok = session.start_game(puzzle, time_limit, now);
    eprintln!("start_match: session.start_game ok={}", ok);

    if ok {
        session.meta.started_at = Some(Utc::now());
    }

    ok
}

fn player_id_for_user(
    session: &crate::match_state::GameSession,
    user_id: &uuid::Uuid,
) -> Option<PlayerId> {
    session.player_for_user(user_id)
}

pub fn get_match_state_for_user(
    match_repo: &mut crate::game_match::repository::MatchRepository,
    user_id: &uuid::Uuid,
    match_id: &uuid::Uuid,
) -> Option<GameView> {
    let session = match_repo.find_session_by_id_mut(match_id)?;
    let player = player_id_for_user(session, user_id)?;

    let game = session.game.as_mut()?;
    let now = Instant::now();

    Some(game.view_for(player, now))
}

pub fn apply_move_for_user(
    match_repo: &mut crate::game_match::repository::MatchRepository,
    user_id: &uuid::Uuid,
    match_id: &uuid::Uuid,
    expected_revision: u64,
    mv: Move,
) -> Option<(MoveOutcome, GameView)> {
    let session = match_repo.find_session_by_id_mut(match_id)?;
    let player = player_id_for_user(session, user_id)?;

    let game = session.game.as_mut()?;
    let now = Instant::now();

    let outcome = game.apply_move(player, expected_revision, mv, now);
    let view = game.view_for(player, now);

    session.touch();
    Some((outcome, view))
}
