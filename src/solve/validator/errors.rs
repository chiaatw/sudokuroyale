use crate::layout::{Cell, Value};
use std::fmt;

/// Errors that can occur when validating a Sudoku board.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationError {
    /// A cell contains an invalid value (outside 1-9 or unknown where not allowed)
    InvalidCellValue {
        cell: Cell,
        value: Option<Value>,
    },

    /// A house (row, column, or block) contains duplicate values
    DuplicateInHouse {
        house_index: usize,
        value: Value,
    },

    /// A deadly rectangle (Unique Rectangle) is present
    DeadlyRectangle {
        rectangle_cells: [Cell; 4],
        value: Value,
    },

    /// A known value is missing in a house
    MissingValueInHouse {
        house_index: usize,
        value: Value,
    },

    /// A general board error not covered by other variants
    GeneralError(String),
}

impl ValidationError {
    /// Returns a human-readable description of the error
    pub fn description(&self) -> String {
        match self {
            ValidationError::InvalidCellValue { cell, value } => {
                format!("Invalid value {:?} in cell {}", value, cell)
            }
            ValidationError::DuplicateInHouse { house_index, value } => {
                format!("Duplicate value {} in house {}", value, house_index)
            }
            ValidationError::DeadlyRectangle { rectangle_cells, value } => {
                format!(
                    "Deadly rectangle with value {} in cells: {}, {}, {}, {}",
                    value, rectangle_cells[0], rectangle_cells[1], rectangle_cells[2], rectangle_cells[3]
                )
            }
            ValidationError::MissingValueInHouse { house_index, value } => {
                format!("Missing value {} in house {}", value, house_index)
            }
            ValidationError::GeneralError(msg) => format!("Board error: {}", msg),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::Cell;
    use crate::layout::Value;

    #[test]
    fn test_invalid_cell_value_error() {
        let cell = Cell::new(0);
        let err = ValidationError::InvalidCellValue {
            cell,
            value: Some(Value::One),
        };
        assert!(err.description().contains("Invalid value"));
    }

    #[test]
    fn test_duplicate_in_house_error() {
        let err = ValidationError::DuplicateInHouse {
            house_index: 3,
            value: Value::Two,
        };
        assert!(err.description().contains("Duplicate value"));
    }

    #[test]
    fn test_deadly_rectangle_error() {
        let cells = [
            Cell::new(0),
            Cell::new(1),
            Cell::new(2),
            Cell::new(3),
        ];
        let err = ValidationError::DeadlyRectangle {
            rectangle_cells: cells,
            value: Value::Three,
        };
        assert!(err.description().contains("Deadly rectangle"));
    }

    #[test]
    fn test_missing_value_in_house_error() {
        let err = ValidationError::MissingValueInHouse {
            house_index: 5,
            value: Value::Four,
        };
        assert!(err.description().contains("Missing value"));
    }

    #[test]
    fn test_general_error() {
        let err = ValidationError::GeneralError("Something went wrong".to_string());
        assert!(err.description().contains("Something went wrong"));
    }
}
