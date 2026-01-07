use std::process::exit;
use std::time::Instant;

use clap::Args;
use itertools::Itertools;

use crate::build::{Finder, Generator};
use crate::io::{
    format_runtime, print_all_and_single_candidates, print_known_values, Cancelable, Parse, Parser,
};
use crate::puzzle::{Changer, Options};

#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Randomize the cells before generating 
    #[clap(short, long)]
    randomize: bool,

    /// Stop once a puzzle with the given number of clues is found
    #[clap(short, long)]
    clues: Option<usize>,

    /// Stop after the given number of seconds
    #[clap(short, long)]
    time: Option<u64>,

    /// Show a progress bar while running
    #[clap(short, long)]
    bar: bool,

    /// The completed puzzle to use as a starting point
    #[clap(short, long)]
    solution: Option<String>,
}

/// Centralized failure path:
/// Prints the board state for debugging
/// Prints a clear error message
/// Exits with a non-zero status
/// 
fn fail(board: &crate::puzzle::Board, message: &str) -> ! {
    print_all_and_single_candidates(board);
    eprintln!("\n==> {message}");
    exit(1);
}

/// Creates a new puzzle and prints it to stdout using the given solution and/or generated solution
pub fn create_puzzle(args: CreateArgs) {
    // Used to support user cancellation
    let cancelable = Cancelable::new();

    let options = Options::all();

    // First obtain a fully solved board
    let board = match args.solution {
        // User provided a solution string
        Some(solution) => {
            let parser = Parse::packed_with_options(options.clone());
            let (board, effects, failure) = parser.parse(&solution);

            // Parsing failed due to a contradictory assignment
            if let Some((cell, known)) = failure {
                print_all_and_single_candidates(&board);
                eprintln!("\n==> Setting {} to {} will cause errors\n", cell, known);
                effects.print_errors();
                exit(1);
            }

            // Ensure the provided board is fully solved
            if !board.is_fully_solved() {
                fail(&board, "You must provide a complete solution");
            }

            board
        }

        // No solution provided: generate one
        None => {
            let changer = Changer::new(options);
            let mut generator = Generator::new(args.randomize, args.bar);

            match generator.generate(&changer) {
                Some(board) => {
                    // Check for user cancellation after generation
                    if cancelable.is_canceled() {
                        fail(&board, "Puzzle generation canceled");
                    }

                    // Sanity check: generator must produce a full solution
                    if !board.is_fully_solved() {
                        fail(&board, "Failed to generate a complete solution");
                    }

                    board
                }
                None => {
                    eprintln!("\n==> Failed to generate a complete solution");
                    exit(1);
                }
            }
        }
    };

    // Step 2, show the completed solution
    print_known_values(&board);
    println!(
        "\n==> Seeking a starting puzzle for {} ...",
        board.packed_string()
    );

    // Step 3, search for a minimal starting puzzle
    let runtime = Instant::now();
    let mut finder = Finder::new(
        args.clues.unwrap_or(22),
        args.time.unwrap_or(10),
        args.bar,
    );

    let (start, actions) = finder.backtracking_find(board);

    // Step 4, output result
    println!();
    print_all_and_single_candidates(&start);
    println!(
        "\n==> Created puzzle with {} clues in {} µs\n\n    {}\n",
        start.known_count(),
        format_runtime(runtime.elapsed()),
        start.packed_string()
    );

    // Step 5, print strategy usage statistics
    let counts = actions.action_counts();
    counts
        .iter()
        .sorted_by(|a, b| a.0.cmp(b.0))
        .for_each(|(strategy, count)| {
            println!("- {:>2} {:?}", count, strategy);
        });
}
