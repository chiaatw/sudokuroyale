use std::fmt;

#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]

pub enum Coord {
    #[default]
    C0,
    C1,
    C2,
    C3,
    C4,
    C5,
    C6,
    C7,
    C8,
}

impl Coord {
    pub const COUNT: u8 = 9;

    pub const fn index(self) -> u8 {
        match self {
            Coord::C0 => 0,
            Coord::C1 => 1,
            Coord::C2 => 2,
            Coord::C3 => 3,
            Coord::C4 => 4,
            Coord::C5 => 5,
            Coord::C6 => 6,
            Coord::C7 => 7,
            Coord::C8 => 8,
        }
    }

    pub const fn new(coord: u8) -> Self {
        debug_assert!(coord < 9);
        Self::from_index(coord as u32)
    }

    pub const fn from_digit(digit: u8) -> Self {
        debug_assert!(digit >= 1 && digit <= 9);
        Self::from_index((digit - 1) as u32)
    }

    pub const fn from_index(index: u32) -> Self {
        debug_assert!(index < 9);
        match index {
            0 => Coord::C0,
            1 => Coord::C1,
            2 => Coord::C2,
            3 => Coord::C3,
            4 => Coord::C4,
            5 => Coord::C5,
            6 => Coord::C6,
            7 => Coord::C7,
            8 => Coord::C8,
            _ => unreachable!(),
        }
    }

    pub const fn u8(&self) -> u8 {
        self.index()
    }

    pub const fn usize(&self) -> usize {
        self.index() as usize
    }

    pub const fn bit(&self) -> u16 {
        1 << self.index()
    }

    pub const fn label(&self) -> char {
        (b'1' + self.index()) as char
    }

    pub const fn min(self, other: Self) -> Self {
        if self.index() <= other.index() {
            self
        } else {
            other
        }
    }

    pub const fn max(self, other: Self) -> Self {
        if self.index() >= other.index() {
            self
        } else {
            other
        }
    }
}

impl From<i32> for Coord {
    fn from(coord: i32) -> Self {
        debug_assert!(coord >= 0);
        Coord::new(coord as u8)
    }
}
impl From<u8> for Coord {
    fn from(coord: u8) -> Self {
        Coord::new(coord)
    }
}

impl From<usize> for Coord {
    fn from(coord: usize) -> Self {
        Coord::new(coord as u8)
    }
}
impl From<char> for Coord {
    fn from(coord: char) -> Self {
        Coord::new(coord as u8 - b'1')
    }
}
impl From<&str> for Coord {
    fn from(label: &str) -> Self {
        Coord::from(label.chars().next().unwrap())
    }
}
impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[macro_export]
macro_rules! coord {
    ($c:expr) => {
        $crate::layout::Coord::from_digit($c)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_and_bit() {
        let c = Coord::C0;
        assert_eq!(c.index(), 0);
        assert_eq!(c.u8(), 0);
        assert_eq!(c.usize(), 0);
        assert_eq!(c.bit(), 1);

        let c = Coord::C3;
        assert_eq!(c.index(), 3);
        assert_eq!(c.bit(), 1 << 3);

        let c = Coord::C8;
        assert_eq!(c.index(), 8);
        assert_eq!(c.bit(), 1 << 8);
    }

    #[test]
    fn test_new_and_from_index() {
        for i in 0..9 {
            let c = Coord::new(i);
            assert_eq!(c.index(), i);
            assert_eq!(Coord::from_index(i as u32), c);
        }
    }

    #[test]
    fn test_from_digit() {
        for d in 1..=9 {
            let c = Coord::from_digit(d);
            assert_eq!(c.index(), d - 1);
        }
    }

    #[test]
    fn test_from_traits() {
        // i32
        assert_eq!(Coord::from(0i32), Coord::C0);
        assert_eq!(Coord::from(8i32), Coord::C8);

        // u8
        assert_eq!(Coord::from(1u8), Coord::C1);
        assert_eq!(Coord::from(5u8), Coord::C5);

        // usize
        assert_eq!(Coord::from(2usize), Coord::C2);
        assert_eq!(Coord::from(7usize), Coord::C7);

        // char
        assert_eq!(Coord::from('1'), Coord::C0);
        assert_eq!(Coord::from('9'), Coord::C8);

        // &str
        assert_eq!(Coord::from("3"), Coord::C2);
        assert_eq!(Coord::from("8"), Coord::C7);
    }

    #[test]
    fn test_label() {
        for i in 0..9 {
            let c = Coord::new(i);
            assert_eq!(c.label(), (b'1' + i) as char);
        }
    }

    #[test]
    fn test_min_max() {
        let a = Coord::C1;
        let b = Coord::C4;
        assert_eq!(a.min(b), a);
        assert_eq!(a.max(b), b);

        let a = Coord::C8;
        let b = Coord::C0;
        assert_eq!(a.min(b), b);
        assert_eq!(a.max(b), a);
    }

    #[test]
    fn test_display() {
        let c = Coord::C5;
        assert_eq!(format!("{}", c), "6");

        let c = Coord::C0;
        assert_eq!(format!("{}", c), "1");
    }

    #[test]
    fn test_macro_coord() {
        let c = coord!(1);
        assert_eq!(c, Coord::C0);

        let c = coord!(9);
        assert_eq!(c, Coord::C8);
    }
}
