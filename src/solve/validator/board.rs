use crate::layout::{Cell, Value};

/// Simple board representation: 81 values (row-major).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    cells: [Value; 81],
}

impl Board {
    /// New empty board (all unknown).
    pub fn new() -> Self {
        Self {
            cells: [Value::unknown(); 81],
        }
    }

    /// Get value at cell.
    #[inline(always)]
    pub fn get(&self, cell: Cell) -> Value {
        self.cells[cell.usize()]
    }

    /// Set value at cell (no validation here).
    #[inline(always)]
    pub fn set(&mut self, cell: Cell, value: Value) {
        self.cells[cell.usize()] = value;
    }

    /// Clear a cell (set to unknown).
    #[inline(always)]
    pub fn clear(&mut self, cell: Cell) {
        self.cells[cell.usize()] = Value::unknown();
    }

    /// Returns internal slice (useful for debugging/serialization).
    pub fn as_slice(&self) -> &[Value] {
        &self.cells
    }

    /// Parse from 81-char string.
    /// Allowed: '1'..'9', '.' or '0' for unknown. Whitespace is ignored.
    pub fn from_str(s: &str) -> Result<Self, String> {
        let chars: Vec<char> = s.chars().filter(|c| !c.is_whitespace()).collect();
        if chars.len() != 81 {
            return Err(format!(
                "Expected 81 cells, got {} chars (after trimming whitespace)",
                chars.len()
            ));
        }

        let mut b = Board::new();
        for (i, ch) in chars.into_iter().enumerate() {
            let v = match ch {
                '.' | '0' => Value::unknown(),
                '1'..='9' => Value::new(ch.to_digit(10).unwrap() as u8),
                _ => return Err(format!("Invalid char '{}' at index {}", ch, i)),
            };
            b.cells[i] = v;
        }
        Ok(b)
    }
}
