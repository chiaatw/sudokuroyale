//! Refactored play module – enum-based command handling (block 1)

use std::io::{stdout, Write};
use std::time::Instant;

use clap::Args;

use crate::build::{Finder, Generator};
use crate::io::{
    format_for_fancy_console, format_for_wiki, format_grid, format_packed,
    format_runtime, print_all_and_single_candidates,
    print_all_and_single_candidates_with_highlight,
    print_candidate, print_givens, print_known_values,
    Cancelable, Parse, Parser, SUDOKUWIKI_URL,
};
use crate::layout::{Cell, CellSet, Known, KnownSet};
use crate::puzzle::{Board, ChangeResult, Changer, Effects, Options, Strategy};
use crate::solve::{find_brute_force, BruteForceResult, TECHNIQUES};
use crate::symbols::{MISSING, UNKNOWN_VALUE};

const MAXIMUM_SOLUTIONS: usize = 100;

#[derive(Debug, Args)]
#[clap(disable_help_flag = true)]
pub struct PlayArgs {
    #[clap(long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,

    #[clap(short, long)]
    naked: bool,

    #[clap(short, long)]
    hidden: bool,

    #[clap(short, long)]
    singles: bool,

    #[clap(short, long)]
    intersection: bool,

    puzzle: Option<String>,
}

impl PlayArgs {
    pub fn options(&self) -> Options {
        Options {
            stop_on_error: true,
            solve_naked_singles: self.naked || self.singles,
            solve_hidden_singles: self.hidden || self.singles,
            solve_intersection_removals: self.intersection,
        }
    }
}



struct PlayerState {
    boards: Vec<Board>,
    deductions: Option<Effects>,
    highlight: Option<crate::puzzle::Action>,
    show_board: bool,
}

impl PlayerState {
    fn new(initial: Board) -> Self {
        Self {
            boards: vec![initial],
            deductions: None,
            highlight: None,
            show_board: true,
        }
    }

    fn board(&self) -> &Board {
        self.boards.last().expect("board stack empty")
    }

    fn push(&mut self, board: Board) {
        self.boards.push(board);
        self.deductions = None;
        self.highlight = None;
        self.show_board = true;
    }
}



#[derive(Debug)]
enum Command {
    Options(Vec<char>),
    NewPuzzle,
    CreateRandom,
    Print(Option<char>),
    Export(Option<char>),
    Wiki,
    Grid,
    Give { cells: CellSet, digit: Known },
    Solve { cells: CellSet, digit: Known },
    Erase { cells: CellSet, digits: KnownSet },
    Verify,
    Find(Option<FindFilter>),
    Highlight(usize),
    Apply(Option<usize>),
    BruteSolve,
    Reset,
    Undo,
    Help,
    Quit,
    Unknown(String),
}

#[derive(Debug)]
enum FindFilter {
    Cell(Cell),
    Digit(Known),
}



impl Command {
    fn parse(input: &str) -> Self {
        let parts: Vec<_> = input.split_whitespace().collect();
        let head = parts[0].to_uppercase();

        match head.as_str() {
            "O" => Command::Options(
                parts.get(1)
                    .map(|s| s.to_uppercase().chars().collect())
                    .unwrap_or_default(),
            ),
            "N" => Command::NewPuzzle,
            "C" => Command::CreateRandom,
            "P" => Command::Print(parts.get(1).and_then(|s| s.chars().next())),
            "X" => Command::Export(parts.get(1).and_then(|s| s.chars().next())),
            "W" => Command::Wiki,
            "M" => Command::Grid,
            "V" => Command::Verify,
            "B" => Command::BruteSolve,
            "R" => Command::Reset,
            "Z" => Command::Undo,
            "?" => Command::Help,
            "Q" => Command::Quit,

            "G" if parts.len() == 3 => {
                match Known::try_from(parts[2]) {
                    Ok(digit) => Command::Give {
                        cells: CellSet::from(parts[1]),
                        digit,
                    },
                    Err(_) => Command::Unknown(input.into()),
                }
            }

            "S" if parts.len() == 3 => {
                match Known::try_from(parts[2]) {
                    Ok(digit) => Command::Solve {
                        cells: CellSet::from(parts[1]),
                        digit,
                    },
                    Err(_) => Command::Unknown(input.into()),
                }
            }

            "E" if parts.len() == 3 => Command::Erase {
                cells: CellSet::from(parts[1]),
                digits: KnownSet::from(parts[2]),
            },

            "F" => {
                let filter = parts.get(1).and_then(|s| {
                    if s.len() == 2 {
                        Cell::try_from(*s).ok().map(FindFilter::Cell)
                    } else if s.len() == 1 {
                        Known::try_from(*s).ok().map(FindFilter::Digit)
                    } else {
                        None
                    }
                });
                Command::Find(filter)
            }

            "H" => parts
                .get(1)
                .and_then(|s| s.parse::<usize>().ok())
                .map(Command::Highlight)
                .unwrap_or(Command::Unknown(input.into())),

            "A" => parts
                .get(1)
                .and_then(|s| s.parse::<usize>().ok())
                .map(|n| Command::Apply(Some(n)))
                .unwrap_or(Command::Apply(None)),

            _ => Command::Unknown(input.into()),
        }
    }
}



pub fn start_player(args: PlayArgs) {
    let cancelable = Cancelable::new();
    let mut changer = Changer::new(args.options());

    let initial_board = match args.puzzle {
        Some(clues) => {
            let parser = Parse::packed_with_player(changer);
            let (board, effects, failure) = parser.parse(&clues);

            if let Some((cell, known)) = failure {
                println!();
                print_all_and_single_candidates(&board);
                println!("\n==> Setting {} to {} will cause errors\n", cell, known);
                effects.print_errors();
            }
            board
        }
        None => {
            print_help();
            Board::new()
        }
    };

    let mut state = PlayerState::new(initial_board);

    loop {
        render_board(&state);

        print_prompt(state.board());
        let input = read_input();
        if input.is_empty() {
            continue;
        }

        let command = Command::parse(&input);
        if dispatch_command(command, &mut state, &mut changer, &cancelable) {
            break;
        }
    }
}



fn render_board(state: &PlayerState) {
    if !state.show_board {
        return;
    }

    let board = state.board();

    if board.is_fully_solved() {
        print_known_values(board);
        println!("\n==> Congratulations!\n");
        return;
    }

    if let Some(action) = &state.highlight {
        print_all_and_single_candidates_with_highlight(board, action);
    } else {
        print_all_and_single_candidates(board);
    }
    println!();
}

fn print_prompt(board: &Board) {
    print!(
        "[ {} solved - {} unsolved ] ",
        board.known_count(),
        board.unknown_count()
    );
    let _ = stdout().flush();
}

fn read_input() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}



/// Returns `true` if the player should quit
fn dispatch_command(
    command: Command,
    state: &mut PlayerState,
    changer: &mut Changer,
    cancelable: &Cancelable,
) -> bool {
    match command {
        Command::Quit => return true,

        Command::Help => {
            print_help();
        }

        Command::Options(flags) => {
            toggle_options(flags, changer);
        }

        Command::NewPuzzle => {
            if let Some(board) = create_new_puzzle(*changer) {
                state.push(board);
            }
        }

        Command::CreateRandom => {
            create_random_puzzle(state, changer, cancelable);
        }

        Command::Print(arg) => {
            handle_print(state.board(), arg);
        }

        Command::Export(arg) => {
            handle_export(state.board(), arg);
        }

        Command::Wiki => {
            println!("\n==> {}{}\n", SUDOKUWIKI_URL, format_for_wiki(state.board()));
        }

        Command::Grid => {
            println!("\n{}\n", format_grid(state.board()));
        }

        Command::Give { cells, digit } => {
            apply_cells(state, changer, cells, digit, Strategy::Given);
        }

        Command::Solve { cells, digit } => {
            apply_cells(state, changer, cells, digit, Strategy::Solve);
        }

        Command::Erase { cells, digits } => {
            erase_candidates(state, changer, cells, digits);
        }

        Command::Verify => {
            verify_board(state.board(), cancelable);
        }

        Command::Find(filter) => {
            find_deductions(state, filter);
        }

        Command::Highlight(n) => {
            highlight_deduction(state, n);
        }

        Command::Apply(n) => {
            apply_deductions(state, changer, n);
        }

        Command::BruteSolve => {
            brute_solve(state, cancelable);
        }

        Command::Reset => {
            reset_board(state);
        }

        Command::Undo => {
            if state.boards.len() > 1 {
                println!("\n==> Undoing last move\n");
                state.boards.pop();
                state.show_board = true;
            }
        }

        Command::Unknown(cmd) => {
            println!("\n==> Unknown command: {}\n", cmd);
        }
    }

    false
}



fn toggle_options(flags: Vec<char>, changer: &mut Changer) {
    for flag in flags {
        match flag {
            'N' => changer.options.solve_naked_singles =
                !changer.options.solve_naked_singles,
            'H' => changer.options.solve_hidden_singles =
                !changer.options.solve_hidden_singles,
            'I' => changer.options.solve_intersection_removals =
                !changer.options.solve_intersection_removals,
            _ => {}
        }
    }

    println!(
        concat!(
            "\n==> Options\n\n",
            "  N - {} naked singles\n",
            "  H - {} hidden singles\n",
            "  I - {} intersection removals\n",
        ),
        if changer.options.solve_naked_singles { "solving" } else { "not solving" },
        if changer.options.solve_hidden_singles { "solving" } else { "not solving" },
        if changer.options.solve_intersection_removals { "solving" } else { "not solving" },
    );
}



fn apply_cells(
    state: &mut PlayerState,
    changer: &mut Changer,
    cells: CellSet,
    digit: Known,
    strategy: Strategy,
) {
    let mut clone = *state.board();
    let mut changed = false;

    for cell in cells {
        let result = match strategy {
            Strategy::Given => changer.set_given(&clone, strategy, cell, digit),
            Strategy::Solve => changer.set_known(&clone, strategy, cell, digit),
            _ => unreachable!(),
        };

        match result {
            ChangeResult::None => {
                println!("\n==> {} is not a candidate for {}\n", digit, cell);
            }
            ChangeResult::Valid(after, _) => {
                clone = *after;
                changed = true;
            }
            ChangeResult::Invalid(_, _, _, errors) => {
                println!("\n==> Invalid move\n");
                errors.print_errors();
            }
        }
    }

    if changed {
        state.push(clone);
    }
}

fn erase_candidates(
    state: &mut PlayerState,
    changer: &mut Changer,
    cells: CellSet,
    digits: KnownSet,
) {
    let mut clone = *state.board();
    let mut changed = false;

    for cell in cells {
        for digit in digits {
            match changer.remove_candidate(&clone, Strategy::Erase, cell, digit) {
                ChangeResult::None => {
                    println!("\n==> {} is not a candidate for {}\n", digit, cell);
                }
                ChangeResult::Valid(after, _) => {
                    clone = *after;
                    changed = true;
                }
                ChangeResult::Invalid(_, _, _, errors) => {
                    println!("\n==> Invalid move\n");
                    errors.print_errors();
                }
            }
        }
    }

    if changed {
        state.push(clone);
    }
}



fn verify_board(board: &Board, cancelable: &Cancelable) {
    let runtime = Instant::now();

    match find_brute_force(board, false, 0, MAXIMUM_SOLUTIONS) {
        BruteForceResult::AlreadySolved => {
            println!("\n==> The puzzle is already solved\n");
        }
        BruteForceResult::TooFewKnowns => {
            println!("\n==> The puzzle needs at least 17 solved cells to verify\n");
        }
        BruteForceResult::UnsolvableCells(cells) => {
            println!(
                "\n==> The puzzle cannot be solved with these {} empty cells\n\n    {}\n",
                cells.len(),
                cells
            );
        }
        BruteForceResult::Canceled => {
            println!(
                "\n==> The verification was canceled - took {} µs\n",
                format_runtime(runtime.elapsed())
            );
            cancelable.clear();
        }
        BruteForceResult::Unsolvable => {
            println!(
                "\n==> The puzzle cannot be solved - took {} µs\n",
                format_runtime(runtime.elapsed())
            );
        }
        BruteForceResult::Solved(_) => {
            println!(
                "\n==> The puzzle is solvable - took {} µs\n",
                format_runtime(runtime.elapsed())
            );
        }
        BruteForceResult::MultipleSolutions(solutions) => {
            println!(
                "\n==> The puzzle has {}{} solutions - took {} µs\n",
                if solutions.len() > MAXIMUM_SOLUTIONS {
                    "at least "
                } else {
                    ""
                },
                solutions.len(),
                format_runtime(runtime.elapsed())
            );
        }
    }
}

fn brute_solve(state: &mut PlayerState, cancelable: &Cancelable) {
    let runtime = Instant::now();

    match find_brute_force(state.board(), false, 0, MAXIMUM_SOLUTIONS) {
        BruteForceResult::Solved(solution) => {
            println!(
                "\n==> The puzzle was solved - took {} µs",
                format_runtime(runtime.elapsed())
            );
            state.push(*solution);
        }
        BruteForceResult::Canceled => {
            println!(
                "\n==> The solution was canceled - took {} µs\n",
                format_runtime(runtime.elapsed())
            );
            cancelable.clear();
        }
        BruteForceResult::MultipleSolutions(solutions) => {
            println!(
                "\n==> The puzzle has {}{} solutions - took {} µs\n",
                if solutions.len() > MAXIMUM_SOLUTIONS {
                    "at least "
                } else {
                    ""
                },
                solutions.len(),
                format_runtime(runtime.elapsed())
            );
        }
        other => {
            verify_board(state.board(), cancelable);
            if matches!(other, BruteForceResult::Solved(_)) {
                return;
            }
        }
    }
}



fn find_deductions(state: &mut PlayerState, filter: Option<FindFilter>) {
    if state.deductions.is_none() {
        let mut found = Effects::new();
        TECHNIQUES.iter().for_each(|solver| {
            if let Some(actions) = solver.solve(state.board(), false) {
                found.take_actions(actions);
            }
        });
        state.deductions = Some(found);
    }

    let found = state.deductions.as_ref().unwrap();

    let filtered = match filter {
        Some(FindFilter::Cell(cell)) => found.affecting_cell(cell),
        Some(FindFilter::Digit(digit)) => found.affecting_known(digit),
        None => found.clone(),
    };

    if filtered.is_empty() {
        println!("\n==> No deductions found\n");
        return;
    }

    println!(
        "\n==> Found {}\n",
        pluralize(filtered.action_count(), "deduction")
    );

    for (i, action) in filtered.actions().iter().enumerate() {
        println!("{:>4} - {}", i + 1, action);
    }
    println!();
}

fn highlight_deduction(state: &mut PlayerState, n: usize) {
    if let Some(found) = &state.deductions {
        if n == 0 || n > found.action_count() {
            println!(
                "\n==> Enter a deduction number 1 - {}\n",
                found.action_count()
            );
            return;
        }
        let action = found.actions()[n - 1].clone();
        state.highlight = Some(action);
        state.show_board = true;
    } else {
        println!("\n==> Find deductions first with F\n");
    }
}

fn apply_deductions(
    state: &mut PlayerState,
    changer: &mut Changer,
    single: Option<usize>,
) {
    if let Some(found) = &state.deductions {
        if let Some(n) = single {
            if n == 0 || n > found.action_count() {
                println!(
                    "\n==> Enter a deduction number 1 - {}\n",
                    found.action_count()
                );
                return;
            }

            let action = &found.actions()[n - 1];
            match changer.apply(state.board(), action) {
                ChangeResult::Valid(after, _) => {
                    println!("\n==> Applied {}\n", action);
                    state.push(*after);
                }
                ChangeResult::None => {
                    println!("\n==> Did not apply {}\n", action);
                }
                ChangeResult::Invalid(_, _, _, errors) => {
                    println!("\n==> Applying {} will cause errors\n", action);
                    errors.print_errors();
                }
            }
            return;
        }

        let mut clone = *state.board();
        let mut applied_any = false;

        for solver in TECHNIQUES {
            if let Some(actions) = solver.solve(state.board(), false) {
                let mut applied = 0;
                for action in actions.actions() {
                    if let ChangeResult::Valid(after, _) =
                        changer.apply(&clone, action)
                    {
                        clone = *after;
                        applied += 1;
                    }
                }
                if applied > 0 {
                    println!(
                        "\n==> Applied {}",
                        pluralize(applied, solver.label())
                    );
                    applied_any = true;
                }
            }
        }

        if applied_any {
            state.push(clone);
        } else {
            println!("\n==> No deductions applied\n");
        }
    } else {
        println!("\n==> Find deductions first with F\n");
    }
}



fn reset_board(state: &mut PlayerState) {
    let mut reset = Board::new();
    let mut effects = Effects::new();

    for (cell, known) in state.board().known_iter() {
        reset.set_given(cell, known, &mut effects);
    }

    if effects.has_errors() {
        println!("\n==> Invalid board\n");
        effects.print_errors();
        return;
    }

    state.push(reset);
}



fn print_help() {
    println!(concat!(
        "\n==> Help\n",
        "\n",
        "  O [option]          - view or toggle an option\n",
        "  N                   - start or input a new puzzle\n",
        "  C                   - create a new random puzzle\n",
        "\n",
        "  P [G | K | digit]   - print the full puzzle, givens, knowns, or a single candidate\n",
        "  X [char]            - export the puzzle with optional character for unsolved cells\n",
        "  W                   - print URL to play on SudokuWiki.org\n",
        "  M                   - print the puzzle as a grid suitable for email\n",
        "\n",
        "  G <cells> <digit>   - set the given (clue) for a cell\n",
        "  S <cells> <digit>   - solve a cell\n",
        "  E <cells> <digits>  - erase one or more candidates\n",
        "\n",
        "  F [cell | digit]    - find deductions\n",
        "  H <num>             - highlight a single deduction\n",
        "  A [num]             - apply a single or all deductions\n",
        "  V                   - verify that puzzle is solvable\n",
        "  B                   - use Bowman's Bingo to solve the puzzle if possible\n",
        "  R                   - reset candidates based on solved cells\n",
        "  Z                   - undo last change\n",
        "\n",
        "  ?                   - this help message\n",
        "  Q                   - quit\n",
        "\n",
        "      <option> - H, N or I\n",
        "      <cell>   - A1 to J9\n",
        "      <digit>  - 1 to 9\n",
        "      <num>    - any positive number\n",
        "      <char>   - any single character\n",
        "      [...]    - optional\n",
        "\n",
        "  Commands and cells are not case-sensitive\n",
    ));
}

fn create_new_puzzle(changer: Changer) -> Option<Board> {
    println!(concat!(
        "\n==> Enter the givens\n\n",
        "  - enter up to 81 digits\n",
        "  - use period or zero to leave a cell blank\n",
        "  - spaces are ignored\n",
        "  - leave empty to cancel\n",
        "  - enter 'E' for an empty puzzle\n",
    ));

    loop {
        print!("> ");
        let _ = stdout().flush();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let cleaned = input
            .trim()
            .replace(' ', "")
            .replace(MISSING, ".");

        if cleaned.is_empty() {
            println!();
            return None;
        }

        if cleaned.eq_ignore_ascii_case("E") {
            println!("\n==> Starting an empty puzzle\n");
            return Some(Board::new());
        }

        let parser: Option<Box<dyn Parser>> = match cleaned.len() {
            162 => Some(Box::new(Parse::wiki())),
            0..=81 => Some(Box::new(Parse::packed_with_player(changer))),
            _ => None,
        };

        if let Some(parser) = parser {
            let (board, effects, failure) = parser.parse(&cleaned);

            println!();
            print_all_and_single_candidates(&board);

            if let Some((cell, known)) = failure {
                println!("\n==> Setting {} to {} will cause errors\n", cell, known);
                effects.print_errors();
            }

            return Some(board);
        }

        println!(
            concat!(
                "\n==> Expected 81 or 162 digits, got {}\n\n",
                "{}\n",
                "        |        |        |        |        |        |        |        |        |\n",
            ),
            cleaned.len(),
            cleaned
        );
    }
}



fn pluralize(count: usize, label: &str) -> String {
    if count == 1 {
        format!("{} {}", count, label)
    } else if ES_SUFFIXES.iter().any(|s| label.ends_with(s)) {
        format!("{} {}es", count, label)
    } else {
        format!("{} {}s", count, label)
    }
}

const ES_SUFFIXES: [&str; 1] = ["sh"];
