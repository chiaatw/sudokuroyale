use std::time::Duration;
use crate::game::game::{Game, MoveResult};
use crate::game::player::PlayerId;
use crate::game::state::{GameState, LoseReason};
use crate::layout::Sudoku;

fn empty_sudoku() -> Sudoku {
    Sudoku::empty()
}

#[test]
fn player_loses_after_three_mistakes() {
    let sudoku = empty_sudoku();
    let mut game = Game::new(sudoku, Duration::from_secs(300));

    for _ in 0..3 {
        game.apply_move(PlayerId::PlayerA, (0, 0).into(), 9.into());
    }

    assert_eq!(
        game.state(),
        &GameState::Lost {
            player: PlayerId::PlayerA,
            reason: LoseReason::TooManyMistakes
        }
    );
}

#[test]
fn player_loses_when_time_runs_out() {
    let sudoku = empty_sudoku();
    let mut game = Game::new(sudoku, Duration::from_secs(10));

    game.tick(PlayerId::PlayerA, Duration::from_secs(10));

    assert_eq!(
        game.state(),
        &GameState::Lost {
            player: PlayerId::PlayerA,
            reason: LoseReason::TimeExpired
        }
    );
}
