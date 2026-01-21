use super::Cell;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CellIndex {
    A1 = 0,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    A8,
    A9,
    B1,
    B2,
    B3,
    B4,
    B5,
    B6,
    B7,
    B8,
    B9,
    C1,
    C2,
    C3,
    C4,
    C5,
    C6,
    C7,
    C8,
    C9,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    D8,
    D9,
    E1,
    E2,
    E3,
    E4,
    E5,
    E6,
    E7,
    E8,
    E9,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    G1,
    G2,
    G3,
    G4,
    G5,
    G6,
    G7,
    G8,
    G9,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    H7,
    H8,
    H9,
    J1,
    J2,
    J3,
    J4,
    J5,
    J6,
    J7,
    J8,
    J9,
}

impl CellIndex {
    pub fn from_label(label: &str) -> Result<Self, String> {
        let upper = label.to_uppercase();
        if upper.len() != 2 {
            return Err(format!("Invalid cell: \"{}\"", label));
        }

        let mut chars = upper.chars();
        let row = chars.next().unwrap();
        let col = chars.next().unwrap();

        if !('1'..='9').contains(&col) {
            return Err(format!("Invalid cell: \"{}\"", label));
        }

        let col = (col as u8) - b'1';

        let row_index = match row {
            'A'..='H' => (row as u8) - b'A',
            'J' => 8,
            _ => return Err(format!("Invalid cell: \"{}\"", label)),
        };

        let index = row_index * 9 + col;

        Self::from_index(index).ok_or_else(|| format!("Invalid cell: \"{}\"", label))
    }

    pub const fn from_index(index: u8) -> Option<Self> {
        if index < 81 {
            Some(unsafe { std::mem::transmute(index) })
        } else {
            None
        }
    }

    pub const fn index(self) -> u8 {
        self as u8
    }

    pub const fn label(self) -> &'static str {
        CELL_LABELS[self as usize]
    }
}

pub fn index_from_label(label: &str) -> u8 {
    match try_index_from_label(label) {
        Ok(index) => index,
        Err(message) => panic!("{}", message),
    }
}

pub fn try_index_from_label(label: &str) -> Result<u8, String> {
    Ok(CellIndex::from_label(label)?.index())
}

pub const fn label_from_index(index: u8) -> &'static str {
    debug_assert!(index < Cell::COUNT);
    CELL_LABELS[index as usize]
}

#[rustfmt::skip]
const CELL_LABELS: [&str; 81] = [
    "A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8", "A9",
    "B1", "B2", "B3", "B4", "B5", "B6", "B7", "B8", "B9",
    "C1", "C2", "C3", "C4", "C5", "C6", "C7", "C8", "C9",
    "D1", "D2", "D3", "D4", "D5", "D6", "D7", "D8", "D9",
    "E1", "E2", "E3", "E4", "E5", "E6", "E7", "E8", "E9",
    "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9",
    "G1", "G2", "G3", "G4", "G5", "G6", "G7", "G8", "G9",
    "H1", "H2", "H3", "H4", "H5", "H6", "H7", "H8", "H9",
    "J1", "J2", "J3", "J4", "J5", "J6", "J7", "J8", "J9",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_label_valid() {
        // Erste Reihe
        let c = CellIndex::from_label("A1").unwrap();
        assert_eq!(c.index(), 0);
        assert_eq!(c.label(), "A1");

        let c = CellIndex::from_label("A9").unwrap();
        assert_eq!(c.index(), 8);
        assert_eq!(c.label(), "A9");

        // Mittlere Reihe
        let c = CellIndex::from_label("E5").unwrap();
        assert_eq!(c.index(), 4 * 9 + 4);
        assert_eq!(c.label(), "E5");

        // Letzte Reihe (J-Reihe)
        let c = CellIndex::from_label("J9").unwrap();
        assert_eq!(c.index(), 8 * 9 + 8); // J ist 8. Reihe (0-basiert)
        assert_eq!(c.label(), "J9");
    }

    #[test]
    fn test_from_label_invalid() {
        // Ungültige Buchstaben
        assert!(CellIndex::from_label("I1").is_err());
        assert!(CellIndex::from_label("K2").is_err());
        assert!(CellIndex::from_label("Z9").is_err());

        // Ungültige Zahlen
        assert!(CellIndex::from_label("A0").is_err());
        assert!(CellIndex::from_label("B10").is_err());

        // Falsche Länge
        assert!(CellIndex::from_label("").is_err());
        assert!(CellIndex::from_label("A12").is_err());
    }

    #[test]
    fn test_from_index_valid_and_invalid() {
        // Gültige Indizes
        for i in 0..81 {
            let c = CellIndex::from_index(i).unwrap();
            assert_eq!(c.index(), i);
        }

        // Ungültiger Index
        assert!(CellIndex::from_index(81).is_none());
        assert!(CellIndex::from_index(100).is_none());
    }

    #[test]
    fn test_label_from_index() {
        for i in 0..81 {
            let label = label_from_index(i);
            let index = index_from_label(label);
            assert_eq!(index, i);
        }
    }

    #[test]
    fn test_index_from_label_and_try_index_from_label() {
        let labels = ["A1", "B2", "C3", "H9", "J1", "J9"];
        for &label in &labels {
            let index1 = index_from_label(label);
            let index2 = try_index_from_label(label).unwrap();
            assert_eq!(index1, index2);

            let cell = CellIndex::from_index(index1).unwrap();
            assert_eq!(cell.label(), label);
        }

        // Ungültige Labels paniken oder geben Fehler zurück
        assert!(try_index_from_label("I1").is_err());
        // index_from_label panikt bei ungültigem Label
        let result = std::panic::catch_unwind(|| {
            index_from_label("I1");
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_case_insensitive_labels() {
        let c1 = CellIndex::from_label("a1").unwrap();
        let c2 = CellIndex::from_label("A1").unwrap();
        assert_eq!(c1.index(), c2.index());
        assert_eq!(c1.label(), c2.label());
    }
}
