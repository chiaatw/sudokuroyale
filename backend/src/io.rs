use crate::solve::strategy_ord::board::Board;

pub use crate::solve::strategy_ord::cancelable::{create_signal, Cancelable};

use std::time::Duration;

pub fn show_progress(_current: usize, _total: usize) {}

pub fn format_for_fancy_console(board: &Board) -> String {
    board.packed_string()
}
pub fn format_number(n: usize) -> String {
    n.to_string()
}
pub fn format_runtime(d: Duration) -> String {
    format!("{:.3}s", d.as_secs_f64())
}

pub fn print_all_and_single_candidates<T>(_t: &T) {}
