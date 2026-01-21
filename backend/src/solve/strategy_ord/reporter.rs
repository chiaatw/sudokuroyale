use std::collections::HashMap;
use std::time::Duration;

use crate::layout::{Cell, Known};
use crate::puzzle::{Action, Board, Difficulty, Effects, Strategy};

/// Strategy execution counters used by the solver.
type StrategyCounts = HashMap<Strategy, i32>;

/// One of these methods is called for each puzzle run through the solver.
pub trait Reporter {
    /// The givens for a puzzle create an invalid puzzle.
    fn invalid(
        &self,
        givens: &str,
        start: &Board,
        errors: &Effects,
        cell: Cell,
        known: Known,
        runtime: Duration,
    );

    /// One of the solver techniques produced an invalid puzzle.
    #[allow(clippy::too_many_arguments)]
    fn failed(
        &self,
        givens: &str,
        start: &Board,
        stopped: &Board,
        action: &Action,
        errors: &Effects,
        runtime: Duration,
        counts: &StrategyCounts,
    );

    /// The puzzle could not be solved using the given techniques.
    fn unsolved(
        &self,
        givens: &str,
        start: &Board,
        stopped: &Board,
        runtime: Duration,
        counts: &StrategyCounts,
    );

    /// The puzzle was fully solved.
    fn solved(
        &self,
        givens: &str,
        start: &Board,
        solution: &Board,
        difficulty: Difficulty,
        runtime: Duration,
        counts: &StrategyCounts,
    );
}
