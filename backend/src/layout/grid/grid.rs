use crate::layout::{Cell, Value};

// Stellt ein 9x9 Sudoku-Raster als 81 Werte (zeilenweise) dar
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid {
    values: [Value; 81],
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            values: [Value::unknown(); 81],
        }
    }
}

impl Grid {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn get(&self, cell: Cell) -> Value {
        self.values[cell.usize()]
    }

    #[inline]
    pub fn set(&mut self, cell: Cell, value: Value) {
        self.values[cell.usize()] = value;
    }

    #[inline]
    pub fn values(&self) -> &[Value; 81] {
        &self.values
    }
}
