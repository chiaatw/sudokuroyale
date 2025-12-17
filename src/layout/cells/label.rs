use super::Cell;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CellIndex {
    A1 = 0, A2, A3, A4, A5, A6, A7, A8, A9,
    B1, B2, B3, B4, B5, B6, B7, B8, B9,
    C1, C2, C3, C4, C5, C6, C7, C8, C9,
    D1, D2, D3, D4, D5, D6, D7, D8, D9,
    E1, E2, E3, E4, E5, E6, E7, E8, E9,
    F1, F2, F3, F4, F5, F6, F7, F8, F9,
    G1, G2, G3, G4, G5, G6, G7, G8, G9,
    H1, H2, H3, H4, H5, H6, H7, H8, H9,
    J1, J2, J3, J4, J5, J6, J7, J8, J9,
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

        Self::from_index(index)
            .ok_or_else(|| format!("Invalid cell: \"{}\"", label))
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

