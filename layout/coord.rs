use std::fmt;

#[derive(
    Clone, Copy, Debug, Default, Hash,
    Eq, PartialEq, Ord, PartialOrd
)]

pub enum Coord {
    #[default] C0,
    C1, C2, C3,
    C4, C5, C6,
    C7, C8, 
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
        Self::from_index(coord)
    }

    pub const fn from_digit(digit: u8) -> Self {
        debug_assert!((1..=9).contains(&digit));
        Self::from_index(digit -1)
    }

    pub const fn from_index(index: u32) -> Self {
        debug_assert!(index <9);
        match index {
            0 => Coord::C0;
            1 => Coord::C1;
            2 => Coord::C2;
            3 => Coord::C3;
            4 => Coord::C4;
            5 => Coord::C5;
            6 => Coord::C6;
            7 => Coord::C7;
            8 => Coord::C8;
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

    pub const fn label (&self) -> char {
        (b'1' + self.index()) as char 
    }

    pub const fn min( self, other: Self ) -> Self {
        if self.index() <= other.index() { self } else { other }
    }

    pub const fn max(self, other: Self) -> Self {
        if self.index() >= other.index() { self } else { other }
    }

    impl From <i32> for Coord {
        fn from(coord: i32) -> Self {
            debug_assert! (coord >=0);
            Coord::new(coord as u8)
        }
    }
    impl From <u8> for Coord {
        fn from(coord: u8) -> Self {
            Coord::newcoord)
        }
    }

    impl From <usize> for Coord {
        fn from(coord: usize) -> Self {
            Coord::new(coord as u8)
        }
    }
    impl From <char> for Coord {
        fn from (coord: char) -> Self {
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
    macro_rules! coord {
        ($c:expr) => {
            Coord::new($c as u8 -1)
        };
    }

    pub(crate) use coord;
    

