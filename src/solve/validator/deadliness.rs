use crate::layout::{Cell, CellSet, Known, KnownSet, Rectangle, Value};

/// Utilities for detecting deadly rectangles (Unique Rectangles) in Sudoku.
#[derive(Clone, Debug)]
pub struct Deadliness;

impl Deadliness {
    /// Returns true if the rectangle contains a deadly pattern for the given candidate `value`.
    ///
    /// A deadly rectangle occurs if a rectangle of four cells forms a situation where
    /// a candidate appears in such a way that it would lead to multiple solutions.
    pub fn is_deadly(rect: &Rectangle, value: Value) -> bool {
        let mut count = 0;
        for cell in rect.cells().iter() {
            if cell.candidates().contains(value) {
                count += 1;
            }
        }
        count == 2
    }

    /// Returns all deadly values for the given rectangle.
    pub fn deadly_values(rect: &Rectangle) -> KnownSet {
        let mut deadly = KnownSet::empty();
        for value in Value::all() {
            if Self::is_deadly(rect, value) {
                deadly.insert(Known::from(value));
            }
        }
        deadly
    }

    /// Returns true if any rectangle in the set is deadly for any candidate.
    pub fn any_deadly(rects: &[Rectangle]) -> bool {
        rects.iter().any(|rect| !Self::deadly_values(rect).is_empty())
    }

    /// Returns the count of deadly rectangles in a list of rectangles.
    pub fn count_deadly(rects: &[Rectangle]) -> usize {
        rects.iter().filter(|rect| !Self::deadly_values(rect).is_empty()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Cell, CellSet, Value, Rectangle};

    fn make_rectangle(values: [[Option<Value>; 2]; 2]) -> Rectangle {
        let mut cells = vec![];
        for row in 0..2 {
            for col in 0..2 {
                let mut cell = Cell::default();
                if let Some(v) = values[row][col] {
                    cell.set(v);
                }
                cells.push(cell);
            }
        }
        Rectangle::from(cells[0], cells[1], cells[2], cells[3])
    }

    #[test]
    fn test_is_deadly() {
        // Rectangle with candidate in exactly two cells
        let rect = make_rectangle([[Some(Value::One), Some(Value::Two)], [Some(Value::One), None]]);
        assert!(Deadliness::is_deadly(&rect, Value::One));
        assert!(!Deadliness::is_deadly(&rect, Value::Two));
    }

    #[test]
    fn test_deadly_values() {
        let rect = make_rectangle([[Some(Value::One), Some(Value::Two)], [Some(Value::One), None]]);
        let deadly = Deadliness::deadly_values(&rect);
        assert!(deadly.contains(Known::from(Value::One)));
        assert!(!deadly.contains(Known::from(Value::Two)));
    }

    #[test]
    fn test_any_deadly_and_count() {
        let rect1 = make_rectangle([[Some(Value::One), Some(Value::Two)], [Some(Value::One), None]]);
        let rect2 = make_rectangle([[Some(Value::Three), Some(Value::Four)], [None, None]]);
        let rects = vec![rect1, rect2];

        assert!(Deadliness::any_deadly(&rects));
        assert_eq!(Deadliness::count_deadly(&rects), 1);
    }
}
