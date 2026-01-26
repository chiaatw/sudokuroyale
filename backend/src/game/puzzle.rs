use crate::layout::{Cell, Grid, Value, ValueLike};

#[derive(Clone, Debug)]
pub struct Puzzle {
    givens: Grid,
    solution: Grid,
}

impl Puzzle {
    pub fn new(givens: Grid, solution: Grid) -> Self {
        Self { givens, solution }
    }

    #[inline]
    pub fn givens(&self) -> &Grid {
        &self.givens
    }

    #[inline]
    pub fn solution(&self) -> &Grid {
        &self.solution
    }

    #[inline]
    pub fn is_given(&self, cell: Cell) -> bool {
        self.givens.get(cell).is_known()
    }

    #[inline]
    pub fn is_correct_value(&self, cell: Cell, value: Value) -> bool {
        self.solution.get(cell) == value && value.is_known()
    }
}
