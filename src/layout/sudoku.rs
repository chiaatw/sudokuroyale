use crate::layout::{Cell, Grid, Value, ValueLike};

/// Runtime Sudoku state used by the game layer.
///
/// For Sudoku Royale we need:
/// - the current grid (player-filled)
/// - the solution grid (for correctness checks)
#[derive(Clone, Debug)]
pub struct Sudoku {
    current: Grid,
    solution: Grid,
}

impl Sudoku {
    /// Creates a Sudoku from a solution grid.
    /// The current grid starts empty (all unknown).
    pub fn from_solution(solution: Grid) -> Self {
        Self { current: Grid::new(), solution }
    }

    /// Creates a Sudoku from both current and solution grids.
    pub fn new(current: Grid, solution: Grid) -> Self {
        Self { current, solution }
    }

    #[inline]
    pub fn is_correct_move(&self, cell: Cell, value: Value) -> bool {
        self.solution.get(cell) == value && value.is_known()
    }

    #[inline]
    pub fn set(&mut self, cell: Cell, value: Value) {
        self.current.set(cell, value);
    }

    #[inline]
    pub fn is_solved(&self) -> bool {
        self.current == self.solution
    }

    pub fn current(&self) -> &Grid {
        &self.current
    }
}
