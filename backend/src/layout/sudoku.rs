use crate::layout::{Cell, Grid, Value, ValueLike};
#[derive(Clone, Debug)]
pub struct Sudoku {
    current: Grid,
    solution: Grid,
}

impl Sudoku {
    // Erstellt ein Sudoku aus einem Lösungsraster
    // Das aktuelle Raster beginnt leer
    pub fn from_solution(solution: Grid) -> Self {
        Self {
            current: Grid::new(),
            solution,
        }
    }

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
