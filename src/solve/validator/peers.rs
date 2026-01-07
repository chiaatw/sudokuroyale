use crate::layout::{Cell, CellSet, House, Shape};
use super::error::ValidationError;

/// Validator for cell peers to ensure no conflicts in Sudoku rules.
pub struct PeersValidator;

impl PeersValidator {
    /// Validates that no two cells in the same peer group have the same value.
    ///
    /// # Arguments
    /// * `givens` - A slice of `(Cell, value)` tuples representing the filled cells.
    /// * `houses` - A slice of `House` representing rows, columns, and blocks.
    ///
    /// # Returns
    /// * `Ok(())` if all peers are valid.
    /// * `Err(Vec<ValidationError>)` containing conflicts if found.
    pub fn validate(givens: &[(Cell, u8)], houses: &[House]) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        for house in houses {
            let mut seen: CellSet = CellSet::empty();

            for cell in house.cells() {
                if let Some(&(_, value)) = givens.iter().find(|&&(c, _)| c == cell) {
                    // Check if value already seen in this house
                    if seen.contains(cell) {
                        errors.push(ValidationError::PeerConflict { cell, value });
                    } else {
                        seen.insert(cell);
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
    use crate::layout::{Cell, CellSet, House, Shape};

    fn create_row(indices: &[usize]) -> House {
        let mut house = House::empty(Shape::Row);
        for &i in indices {
            house.insert(Cell::new(i));
        }
        house
    }

    #[test]
    fn test_no_peer_conflicts() {
        let row = create_row(&[0, 1, 2, 3, 4, 5, 6, 7, 8]);
        let givens = vec![
            (Cell::new(0), 1),
            (Cell::new(1), 2),
            (Cell::new(2), 3),
        ];

        let result = PeersValidator::validate(&givens, &[row]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_peer_conflict() {
        let row = create_row(&[0, 1, 2, 3, 4, 5, 6, 7, 8]);
        let givens = vec![
            (Cell::new(0), 1),
            (Cell::new(1), 1),
        ];

        let result = PeersValidator::validate(&givens, &[row]);
        assert!(matches!(result, Err(errors) if errors.iter().any(|e| matches!(e, ValidationError::PeerConflict { .. }))));
    }
}
