use crate::layout::{Cell, CellSet, Value};
use super::error::ValidationError;

/// Validator for the given starting cells of a Sudoku board
pub struct GivensValidator;

impl GivensValidator {
    /// Validates that all given cells have valid values and no duplicates in any house.
    pub fn validate(givens: &[(Cell, Value)], houses: &[CellSet]) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        let mut seen = vec![CellSet::empty(); 9]; // Tracks which values have been seen in each house

        for &(cell, value) in givens {
            // Check value is in 1-9
            if value.is_none() {
                errors.push(ValidationError::InvalidCellValue {
                    cell,
                    value: Some(value),
                });
                continue;
            }

            // Check duplicates in each house
            for (house_index, house) in houses.iter().enumerate() {
                if house.contains(cell) {
                    if seen[house_index].contains(cell) {
                        errors.push(ValidationError::DuplicateInHouse {
                            house_index,
                            value,
                        });
                    } else {
                        seen[house_index].insert(cell);
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Cell, CellSet, Value};

    fn create_house(cells: &[usize]) -> CellSet {
        let mut set = CellSet::empty();
        for &i in cells {
            set.insert(Cell::new(i));
        }
        set
    }

    #[test]
    fn test_valid_givens() {
        let houses = vec![
            create_house(&[0, 1, 2, 3, 4, 5, 6, 7, 8]), // first row
        ];

        let givens = vec![
            (Cell::new(0), Value::One),
            (Cell::new(1), Value::Two),
            (Cell::new(2), Value::Three),
        ];

        let result = GivensValidator::validate(&givens, &houses);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_value() {
        let houses = vec![
            create_house(&[0, 1, 2, 3, 4, 5, 6, 7, 8]),
        ];

        let givens = vec![
            (Cell::new(0), Value::None),
        ];

        let result = GivensValidator::validate(&givens, &houses);
        assert!(matches!(result, Err(errors) if errors.iter().any(|e| matches!(e, ValidationError::InvalidCellValue {..}))));
    }

    #[test]
    fn test_duplicate_in_house() {
        let houses = vec![
            create_house(&[0, 1, 2, 3, 4, 5, 6, 7, 8]),
        ];

        let givens = vec![
            (Cell::new(0), Value::One),
            (Cell::new(0), Value::One),
        ];

        let result = GivensValidator::validate(&givens, &houses);
        assert!(matches!(result, Err(errors) if errors.iter().any(|e| matches!(e, ValidationError::DuplicateInHouse {..}))));
    }
}
