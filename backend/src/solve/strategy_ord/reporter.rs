use std::collections::HashMap;
use std::time::Duration;

use crate::layout::{Cell, Known};
use crate::puzzle::{Action, Board, Difficulty, Effects, Strategy};

type StrategyCounts = HashMap<Strategy, i32>;

pub trait Reporter {
    fn invalid(
        &self,
        givens: &str,
        start: &Board,
        errors: &Effects,
        cell: Cell,
        known: Known,
        runtime: Duration,
    );

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

    fn unsolved(
        &self,
        givens: &str,
        start: &Board,
        stopped: &Board,
        runtime: Duration,
        counts: &StrategyCounts,
    );

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
