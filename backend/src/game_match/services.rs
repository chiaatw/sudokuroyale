use chrono::Utc;
use uuid::Uuid;

use std::time::{Duration, Instant};

use crate::build::{Finder, Generator};
use crate::game::puzzle::Puzzle as GamePuzzle;
use crate::game_match::model::{GameMatch, MatchStatus};
use crate::game_match::repository::MatchRepository;
use crate::match_state::GameSession;
use crate::io::Cancelable;
use crate::layout::{Cell, Grid};
use crate::puzzle::{Board, Changer, Options}; // Board = solve::strategy_ord::Board
use crate::game::outcome::MoveOutcome;
use crate::game::player::PlayerId;
use crate::game::r#move::Move;
use crate::game::view::GameView;

/// Neues Match erstellen – synchron, bekommt user_id direkt.
/// Legt automatisch eine GameSession an (meta + game=None).
pub fn create_match(match_repo: &mut MatchRepository, user_id: &Uuid) -> Uuid {
    let meta = GameMatch::new(*user_id);
    let match_id = meta.id;

    // direkt Session erstellen statt nur Meta
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

pub fn leave_match_by_user(match_repo: &mut MatchRepository, user_id: &Uuid, match_id: &Uuid) -> bool {
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

        // falls das Spiel schon lief, kannst du entscheiden:
        // - abbrechen: session.game = None;
        // - oder keep: session.game bleibt, bis cleanup ihn entfernt
        // Für MVP: abbrechen ist meistens sinnvoll:
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
        // board.value(cell) muss einen Value liefern, der zu Grid::set passt
        grid.set(cell, board.value(cell));
    }
    grid
}

/// Startet ein Sudoku-Race: erzeugt Lösung + Givens (28 clues) und ruft session.start_game(...)
pub fn start_match_by_user(
    match_repo: &mut MatchRepository,
    user_id: &Uuid,
    match_id: &Uuid,
) -> bool {
    // 1) komplette Session holen (nicht nur Meta!)
    let session = match match_repo.find_session_by_id_mut(match_id) {
        Some(s) => s,
        None => return false,
    };

    // 2) Regeln: nur Player1 darf starten, Player2 muss da sein, Status READY
    if session.meta.player1_id != *user_id {
        return false;
    }
    if session.meta.player2_id.is_none() {
        return false;
    }
    if session.meta.status != MatchStatus::Ready {
        return false;
    }

    // 3) Lösung generieren (vollständig)
    let changer = Changer::new(Options::errors());
    let cancelable = Cancelable::new();

    let mut gen = Generator::new(true, false);
    let solved: Board = match gen.generate(&changer, &cancelable) {
        Some(b) if b.known_count() == 81 => b,
        _ => return false,
    };

    // WICHTIG: Solution behalten, bevor Finder das Board "verbraucht"
    let solved_clone = solved.clone();

    // 4) Givens finden: 28 clues, max 2 Sekunden Suche (Server-friendly)
    let mut finder = Finder::new(28, 5, false);
    let (givens_board, _effects) = finder.backtracking_find(solved);

    // Sicherheitscheck: wirklich <= 28 clues erreicht?
    // Finder kann früher stoppen (time limit etc.), dann wären es >28.
    // Für "Medium" ist >28 nicht schlimm, aber wenn du exakt willst: check.
    // Ich mache hier "<= 28 oder Abbruch".
    if givens_board.known_count() > 28 {
        return false;
    }

    // 5) Puzzle bauen: givens + solution passen garantiert zusammen
    let givens_grid = board_to_grid(&givens_board);
    let solution_grid = board_to_grid(&solved_clone);

    let puzzle = GamePuzzle::new(givens_grid, solution_grid);

    // 6) Start Game in Session
    let time_limit = Duration::from_secs(6 * 60); // 6 Minuten
    let now = Instant::now();

    // session.start_game setzt status + started_at + touch
    let ok = session.start_game(puzzle, time_limit, now);

    // optional: started_at wird in start_game gesetzt, aber wenn du doppelt sicher willst:
    if ok {
        session.meta.started_at = Some(Utc::now());
    }

    ok
}

fn player_id_for_user(session: &crate::match_state::GameSession, user_id: &uuid::Uuid) -> Option<PlayerId> {
    session.player_for_user(user_id)
}

/// GET /match/<id>/state
/// Gibt einen GameView zurück (enthält givens + current, aber nie solution).
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

/// POST /match/<id>/move  (mit expected_revision)
/// Liefert MoveOutcome + optional updated view.
/// - Wenn RevisionMismatch -> Rejected(RevisionMismatch) => Route gibt 409 zurück.
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
    // Nach dem Move immer neuen View erzeugen (auch fürs Polling gut)
    let view = game.view_for(player, now);

    session.touch();
    Some((outcome, view))
}