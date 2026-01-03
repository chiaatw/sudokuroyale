//! Provides utilities for parsing and formatting Sudoku puzzles as well as other I/O helpers
//! 
//! The main tools are [Parse] for constructing a [Board] from a string
//! and [Format] for producing a string representation of a board
//! Several string formats are supported for sharing puzzles
//! 
//! The packed format is a compact 81 character string. Each digit represents a known cell and a period
//! represents an unknown cell
//! This format cannot distinguish between given and solved cells
//! Parsing ignores all other characters
//! When formatting spaces can optionallybe inserted between rows
//! 
//! [Cancelable] detects when the user presses Ctrl C allowing long running processes
//! to be stopped without terminating the program
//! [show_progress] displays a progress bar while building or solving puzzles
//! [format_runtime] and [format_number] helpers for logging and formatting numbers or elapsed time

pub use cancelable::{create_signal, Cancelable};
pub use format::{format_for_fancy_console, format_for_wiki, format_grid, format_packed, Format};
pub use numbers::{format_number, format_runtime};
pub use parse::{Parse, ParsePacked, Parser};
pub use print::{
    print_all_and_single_candidates, print_all_and_single_candidates_with_highlight,
    print_candidate, print_givens, print_known_values,
};
pub use progress::show_progress;

mod cancelable;
mod format;
mod numbers;
mod parse;
mod print;
mod progress;

pub const SUDOKUWIKI_URL: &str = "https://www.sudokuwiki.org/sudoku.htm?bd=";
