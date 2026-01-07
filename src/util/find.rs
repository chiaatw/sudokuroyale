use std::io::{stdin, BufRead};
use std::process::exit;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::thread::{available_parallelism, spawn};
use std::time::Instant;

use clap::Args;
use itertools::Itertools;

use crate::io::{
    format_number, format_runtime, print_all_and_single_candidates, Cancelable, Parse, Parser,
};
use crate::layout::CellSet;
use crate::puzzle::{Board, Changer, Difficulty, Effects, Options};
use crate::solve::{Resolution, Solver, Timings};

/// Command-line arguments for find operation
#[derive(Debug, Args)]
pub struct FindArgs {
    /// Display the strategies used to solve each puzzle
    #[clap(short, long)]
    pub actions: bool,

    /// Worker thread count negative values are relative to core count
    #[clap(short, long)]
    pub threads: Option<isize>,

    /// The completed puzzle to use as a starting point
    pub solution: String,
}

/// Represents the result of attempting a pattern
#[derive(Debug)]
enum PatternResult {
    Success {
        pattern: String,
        board: Board,
        actions: Effects,
        difficulty: Difficulty,
    },
    Failure {
        pattern: String,
        board: Board,
    }
}

/// Encapsulates the find solution logic with multi threaded workers
pub struct SolutionFinder {
    board: Board,
    num_workers: usize,
    show_actions: bool,
}

impl SolutionFinder {
    /// Construct a new SolutionFinder from command line arguments
    pub fn new(args: FindArgs) -> Self {
        let board = Self::parse_puzzle_or_exit(args.solution);
        let num_workers = Self::determine_worker_count(args.threads);
        Self {
            board,
            num_workers,
            show_actions: args.actions,
        }
    }

    /// Runs the solution finding process
    pub fn run(&self) {
        let runtime = Instant::now();

        // Channels for pattern distribution and result collection
        let (pattern_tx, pattern_rx) = channel::<String>();
        let (result_tx, result_rx) = channel::<PatternResult>();

        // Shared pattern receiver for worker threads
        let pattern_rx: Arc<Mutex<Receiver<String>>> = Arc::new(Mutex::new(pattern_rx));

        // Spawn worker threads
        let workers: Vec<_> = (1..=self.num_workers)
            .map(|id| {
                let pattern_rx = Arc::clone(&pattern_rx);
                let result_tx = result_tx.clone();
                let board = self.board.clone();

                spawn(move || {
                    Self::worker_loop(id, board, pattern_rx, result_tx);
                })
            })
            .collect();

        // Drop the extra sender to let workers terminate when all patterns are sent
        drop(result_tx);

        // Spawn stdin reader thread
        {
            let pattern_tx = pattern_tx.clone();
            spawn(move || {
                Self::read_patterns_from_stdin(pattern_tx);
            });
        }
        // Close original sender in main thread
        drop(pattern_tx);

        // Process results
        self.collect_results(result_rx, runtime);

        // Wait for workers
        for worker in workers {
            worker.join().unwrap();
        }
    }

    /// Worker thread main loop
    fn worker_loop(
        id: usize,
        board: Board,
        pattern_rx: Arc<Mutex<Receiver<String>>>,
        result_tx: std::sync::mpsc::Sender<PatternResult>,
    ) {
        let solver = Solver::new(false);
        let cancelable = Cancelable::new();
        let mut count = 0;
        let mut timings = Timings::new();
        let runtime = Instant::now();

        while !cancelable.is_canceled() {
            let pattern = match pattern_rx.lock().unwrap().recv() {
                Ok(p) => p,
                Err(_) => break,
            };

            let (start_board, effects) = board.with_givens(CellSet::new_from_pattern(&pattern));

            let result = match solver.solve(&start_board, &effects, &mut timings) {
                Resolution::Solved(_, actions, difficulty) => PatternResult::Success {
                    pattern,
                    board: start_board,
                    actions,
                    difficulty,
                },
                Resolution::Canceled(..) => break,
                _ => PatternResult::Failure {
                    pattern,
                    board: start_board,
                },
            };

            result_tx.send(result).unwrap();
            count += 1;
        }

        println!(
            "Worker {} processed {} patterns in {} µs - {} p/s",
            id,
            format_number(count),
            format_runtime(runtime.elapsed()),
            format_number((count as f64 / runtime.elapsed().as_secs_f64()) as u128)
        );
    }

    /// Reads patterns from stdin and sends them to the channel
    fn read_patterns_from_stdin(pattern_tx: std::sync::mpsc::Sender<String>) {
        let cancelable = Cancelable::new();
        for line in stdin().lock().lines().map_while(Result::ok) {
            if cancelable.is_canceled() {
                break;
            }
            pattern_tx.send(line).unwrap();
        }
        // Close the sender
        drop(pattern_tx);
    }

    /// Collects results from worker threads and prints summary statistics
    fn collect_results(&self, result_rx: Receiver<PatternResult>, runtime: Instant) {
        let cancelable = Cancelable::new();

        let mut count = 0usize;
        let mut solved = 0usize;
        let mut easiest: Option<Board> = None;
        let mut easiest_actions = 10000;
        let mut hardest: Option<Board> = None;
        let mut hardest_actions = 0;

        for processed in result_rx {
            if cancelable.is_canceled() {
                break;
            }
            count += 1;

            if let PatternResult::Success {
                board, actions,
                difficulty,
                ..
            } = processed
            {
                solved += 1;
                println!("{} {:?}", board.packed_string(), difficulty);

                let action_count = actions.action_count();
                if action_count < easiest_actions {
                    easiest = Some(board.clone());
                    easiest_actions = action_count;
                }
                if action_count > hardest_actions {
                    hardest = Some(board.clone());
                    hardest_actions = action_count;
                }

                if self.show_actions {
                    actions
                        .action_counts()
                        .iter()
                        .sorted_by(|a, b| a.0.cmp(b.0))
                        .for_each(|(strategy, count)| {
                            println!("\n- {:>2} {:?}\n", count, strategy);
                        });
                }
            }
        }

        if count > 0 {
            println!(
                "\n==> Found {} solvable puzzles from {} patterns in {} µs\n",
                format_number(solved),
                format_number(count),
                format_runtime(runtime.elapsed())
            );

            println!(
                "    Easiest: {} - {} actions\n    Hardest: {} - {} actions",
                easiest.unwrap().packed_string(),
                easiest_actions,
                hardest.unwrap().packed_string(),
                hardest_actions
            );
        }
    }

    /// Determines the number of worker threads to use
    fn determine_worker_count(requested: Option<isize>) -> usize {
        let num_cores = available_parallelism().unwrap().get() as isize;
        let count = match requested {
            Some(c) if c < 0 => num_cores + c,
            Some(c) => c,
            None => num_cores - 1,
        };
        count.max(1) as usize
    }

    /// Parses a puzzle solution string or exits on error
    fn parse_puzzle_or_exit(solution: String) -> Board {
        let changer = Changer::new(Options::errors());
        let parser = Parse::packed_with_player(changer);
        let (board, effects, failure) = parser.parse(&solution);

        if let Some((cell, known)) = failure {
            print_all_and_single_candidates(&board);
            eprintln!("\n==> Setting {} to {} will cause errors\n", cell, known);
            effects.print_errors();
            exit(1);
        }
        if !board.is_fully_solved() {
            print_all_and_single_candidates(&board);
            eprintln!("\n==> You must provide a complete solution");
            exit(1);
        }

        board
    }
}