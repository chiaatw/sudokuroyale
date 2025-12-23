type Bits = u128;
type Size = u8;

const ALL_CELLS: std::ops::Range<Size> = 0..Cell::COUNT;
const ALL_SET: Bits = (1 << Cell::COUNT) - 1;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct CellSet(CellSetRepr);

#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
enum CellSetRepr {
    Empty,
    Full,
    Bits(Bits),
}

impl Default for CellSet {
    fn default() -> Self {
        Self::empty()
    }
}

impl CellSet {
    #[inline]
    const fn from_bits(bits: Bits) -> Self {
        debug_assert!(bits <= ALL_SET);
        match bits {
            0 => Self(CellSetRepr::Empty),
            ALL_SET => Self(CellSetRepr::Full),
            _ => Self(CellSetRepr::Bits(bits)),
        }
    }

    #[inline]
    const fn bits(&self) -> Bits {
        match self.0 {
            CellSetRepr::Empty => 0,
            CellSetRepr::Full => ALL_SET,
            CellSetRepr::Bits(bits) => bits,
        }
    }

    pub const fn empty() -> Self {
        Self(CellSetRepr::Empty)
    }

    pub const fn full() -> Self {
        Self(CellSetRepr::Full)
    }

    pub const fn is_empty(&self) -> bool {
        matches!(self.0, CellSetRepr::Empty)
    }

    pub const fn is_full(&self) -> bool {
        matches!(self.0, CellSetRepr::Full)
    }

    pub const fn len(&self) -> usize {
        self.bits().count_ones() as usize
    }

    pub const fn has(&self, cell: Cell) -> bool {
        self.bits() & cell.bit().bit() != 0
    }

    pub const fn with(&self, cell: Cell) -> Self {
        Self::from_bits(self.bits() | cell.bit().bit())
    }

    pub const fn without(&self, cell: Cell) -> Self {
        Self::from_bits(self.bits() & !cell.bit().bit())
    }

    pub const fn union(&self, other: Self) -> Self {
        if self.bits() == other.bits() {
            *self
        } else {
            Self::from_bits(self.bits() | other.bits())
        }
    }

    pub const fn intersect(&self, other: Self) -> Self {
        if self.bits() == other.bits() {
            *self
        } else {
            Self::from_bits(self.bits() & other.bits())
        }
    }

    pub const fn minus(&self, other: Self) -> Self {
        if self.bits() == other.bits() {
            Self::empty()
        } else {
            Self::from_bits(self.bits() & !other.bits())
        }
    }

    pub const fn inverted(&self) -> Self {
        Self::from_bits(!self.bits() & ALL_SET)
    }

    pub const fn first(&self) -> Option<Cell> {
        if self.is_empty() {
            None
        } else {
            Some(Cell::new(self.bits().trailing_zeros() as u8))
        }
    }

    pub fn add(&mut self, cell: Cell) {
        *self = self.with(cell);
    }

    pub fn remove(&mut self, cell: Cell) {
        *self = self.without(cell);
    }

    pub fn pop(&mut self) -> Option<Cell> {
        let cell = self.first()?;
        self.remove(cell);
        Some(cell)
    }
}
/// Returns true if all cells in this set are in the same `shape` house.
pub fn share_house(&self, shape: Shape) -> bool {
    if self.is_empty() {
        false
    } else {
        let house = self.first().unwrap().house(shape);
        for cell in self.iter() {
            if cell.house(shape) != house {
                return false;
            }
        }
        true
    }
}

/// Returns the minimal set of rows containing the members of this set.
pub fn rows(&self) -> HouseSet {
    self.houses(Shape::Row)
}

/// Returns the minimal set of columns containing the members of this set.
pub fn columns(&self) -> HouseSet {
    self.houses(Shape::Column)
}

/// Returns the minimal set of blocks containing the members of this set.
pub fn blocks(&self) -> HouseSet {
    self.houses(Shape::Block)
}

/// Returns the minimal set of `shape` houses containing the members of this set.
pub fn houses(&self, shape: Shape) -> HouseSet {
    self.iter()
        .fold(HouseSet::empty(shape), |set, cell| set + cell.house(shape))
}

/// Returns the common peers of all members of this set.
pub fn peers(&self) -> CellSet {
    self.iter()
        .fold(CellSet::full(), |set, cell| set & cell.peers())
}

/// Returns an iterator over the members of this set in row-then-column order.
pub const fn iter(&self) -> CellIter {
    CellIter {
        iter: self.bit_iter(),
    }
}

/// Returns an iterator over the members of this set as bits in row-then-column order.
pub const fn bit_iter(&self) -> BitIter {
    BitIter { bits: self.bits() }
}

/// Returns a packed pattern string with a `1` for each member of this set.
pub fn pattern_string(&self) -> String {
    (0..Cell::COUNT)
        .map(|i| if self.has(Cell::new(i)) { '1' } else { '.' })
        .collect()
}

/// Returns the size and bits of this set as a debug string.
pub fn debug(&self) -> String {
    format!(
        "{:02}:{:081b}",
        self.len(),
        self.bits().reverse_bits() >> (128 - 81)
    )
}

