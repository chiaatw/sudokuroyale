use crate::layout::{Cell, CellSet, Known, KnownSet, Value};

/// Utilities for working with sets of Sudoku cells.
#[derive(Clone, Debug)]
pub struct Cells;

impl Cells {
    /// Checks if all cells in the given `CellSet` are known (have a `Known` value).
    pub fn all_known(cells: &CellSet) -> bool {
        cells.iter().all(|cell| cell.is_known())
    }

    /// Returns a `KnownSet` representing all known values in the given cells.
    pub fn known_values(cells: &CellSet) -> KnownSet {
        let mut knowns = KnownSet::empty();
        for cell in cells.iter() {
            if let Some(value) = cell.known_value() {
                knowns.insert(value);
            }
        }
        knowns
    }

    /// Returns true if all known values in the set are unique.
    pub fn all_unique(cells: &CellSet) -> bool {
        let mut seen = KnownSet::empty();
        for cell in cells.iter() {
            if let Some(value) = cell.known_value() {
                if seen.contains(value) {
                    return false;
                }
                seen.insert(value);
            }
        }
        true
    }

    /// Returns true if the given `CellSet` contains a specific value.
    pub fn contains_value(cells: &CellSet, value: Value) -> bool {
        cells.iter().any(|cell| cell.value() == Some(value))
    }

    /// Returns the number of known cells in a set.
    pub fn count_known(cells: &CellSet) -> usize {
        cells.iter().filter(|cell| cell.is_known()).count()
    }

    /// Returns the number of unknown cells in a set.
    pub fn count_unknown(cells: &CellSet) -> usize {
        cells.len() - Self::count_known(cells)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Cell, CellSet, Value};

    fn make_cells(values: &[Option<Value>]) -> CellSet {
        let mut set = CellSet::empty();
        for &v in values {
            let mut cell = Cell::default();
            if let Some(val) = v {
                cell.set(val);
            }
            set.insert(cell);
        }
        set
    }

    #[test]
    fn test_all_known() {
        let cells = make_cells(&[Some(Value::One), Some(Value::Two)]);
        assert!(Cells::all_known(&cells));

        let cells2 = make_cells(&[Some(Value::One), None]);
        assert!(!Cells::all_known(&cells2));
    }

    #[test]
    fn test_known_values() {
        let cells = make_cells(&[Some(Value::One), Some(Value::Two), None]);
        let known = Cells::known_values(&cells);
        assert!(known.contains(Known::from(Value::One)));
        assert!(known.contains(Known::from(Value::Two)));
        assert_eq!(known.len(), 2);
    }

    #[test]
    fn test_all_unique() {
        let cells = make_cells(&[Some(Value::One), Some(Value::Two)]);
        assert!(Cells::all_unique(&cells));

        let cells2 = make_cells(&[Some(Value::One), Some(Value::One)]);
        assert!(!Cells::all_unique(&cells2));
    }

    #[test]
    fn test_contains_value() {
        let cells = make_cells(&[Some(Value::One), Some(Value::Two)]);
        assert!(Cells::contains_value(&cells, Value::One));
        assert!(!Cells::contains_value(&cells, Value::Three));
    }

    #[test]
    fn test_count_known_and_unknown() {
        let cells = make_cells(&[Some(Value::One), None, Some(Value::Two)]);
        assert_eq!(Cells::count_known(&cells), 2);
        assert_eq!(Cells::count_unknown(&cells), 1);
    }
}
