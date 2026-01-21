use std::fmt;

/// Identifies the logic used to solve cells and remove candidates
///
/// Strategy is intentionally a simple enum
/// fast to copy
/// easy to match
/// exhaustive and compiler checked
///
///

#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Strategy {
    // The player or parser has provided a given clue
    Given,
    // The player has solved a cell
    Solve,
    // The player has erased a candidate from a cell
    Erase,

    // Remove candidates from peers of a solved cell
    Peer,

    // A cell with one candidate remaining may be solved
    NakedSingle,
    // A candidate that may only appear in one cell in a house may be solved
    HiddenSingle,

    NakedPair,
    HiddenPair,

    NakedTriple,
    HiddenTriple,

    NakedQuad,
    HiddenQuad,

    // Produces pointing pairs/triples and box/line reductions
    IntersectionRemoval,
    PointingPair,
    PointingTriple,
    BoxLineReduction,

    XWing,
    Swordfish,
    Jellyfish,

    Bug,
    AvoidableRectangle,
    TwoStringKite,
    SinglesChain,
    Skyscraper,
    YWing,
    XYZWing,
    WXYZWing,

    XYChain,
    UniqueRectangle,
    Fireworks,

    EmptyRectangle,

    BruteForce,
}

impl Strategy {
    /// Difficulty classification

    #[inline(always)]
    pub const fn difficulty(self) -> Difficulty {
        match self {
            Self::Given | Self::Solve | Self::Erase => Difficulty::Trivial,

            Self::Peer | Self::NakedSingle | Self::HiddenSingle => Difficulty::Trivial,

            Self::NakedPair
            | Self::HiddenPair
            | Self::NakedTriple
            | Self::HiddenTriple
            | Self::NakedQuad
            | Self::HiddenQuad
            | Self::IntersectionRemoval
            | Self::PointingPair
            | Self::PointingTriple
            | Self::BoxLineReduction => Difficulty::Basic,

            Self::XWing
            | Self::TwoStringKite
            | Self::SinglesChain
            | Self::YWing
            | Self::EmptyRectangle
            | Self::Swordfish
            | Self::XYZWing
            | Self::AvoidableRectangle
            | Self::Bug => Difficulty::Tough,

            Self::Jellyfish
            | Self::Skyscraper
            | Self::XYChain
            | Self::UniqueRectangle
            | Self::Fireworks
            | Self::WXYZWing => Difficulty::Diabolical,

            Self::BruteForce => Difficulty::Extreme,
        }
    }

    // Human-readable label
    #[inline(always)]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Given => "Given",
            Self::Solve => "Solve",
            Self::Erase => "Erase",
            Self::Peer => "Peer",

            Self::NakedSingle => "Naked Single",
            Self::HiddenSingle => "Hidden Single",

            Self::NakedPair => "Naked Pair",
            Self::HiddenPair => "Hidden Pair",
            Self::NakedTriple => "Naked Triple",
            Self::HiddenTriple => "Hidden Triple",
            Self::NakedQuad => "Naked Quad",
            Self::HiddenQuad => "Hidden Quad",

            Self::IntersectionRemoval => "Intersection Removal",
            Self::PointingPair => "Pointing Pair",
            Self::PointingTriple => "Pointing Triple",
            Self::BoxLineReduction => "Box/Line Reduction",

            Self::XWing => "X-Wing",
            Self::Swordfish => "Swordfish",
            Self::Jellyfish => "Jellyfish",

            Self::Bug => "BUG",
            Self::AvoidableRectangle => "Avoidable Rectangle",
            Self::TwoStringKite => "Two-String Kite",
            Self::SinglesChain => "Singles Chain",
            Self::Skyscraper => "Skyscraper",
            Self::YWing => "Y-Wing",
            Self::XYZWing => "XYZ-Wing",
            Self::WXYZWing => "WXYZ-Wing",

            Self::XYChain => "XY-Chain",
            Self::UniqueRectangle => "Unique Rectangle",
            Self::Fireworks => "Fireworks",
            Self::EmptyRectangle => "Empty Rectangle",

            Self::BruteForce => "Brute Force",
        }
    }
}

impl fmt::Display for Strategy {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

// Groups solvers by difficulty
#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Difficulty {
    Trivial,
    Basic,
    Tough,
    Diabolical,
    Extreme,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_difficulty() {
        // Trivial
        assert_eq!(Strategy::Given.difficulty(), Difficulty::Trivial);
        assert_eq!(Strategy::NakedSingle.difficulty(), Difficulty::Trivial);

        // Basic
        assert_eq!(Strategy::NakedPair.difficulty(), Difficulty::Basic);
        assert_eq!(Strategy::BoxLineReduction.difficulty(), Difficulty::Basic);

        // Tough
        assert_eq!(Strategy::XWing.difficulty(), Difficulty::Tough);
        assert_eq!(Strategy::Bug.difficulty(), Difficulty::Tough);

        // Diabolical
        assert_eq!(Strategy::Jellyfish.difficulty(), Difficulty::Diabolical);
        assert_eq!(Strategy::Skyscraper.difficulty(), Difficulty::Diabolical);

        // Extreme
        assert_eq!(Strategy::BruteForce.difficulty(), Difficulty::Extreme);
    }

    #[test]
    fn test_strategy_label() {
        assert_eq!(Strategy::Given.label(), "Given");
        assert_eq!(Strategy::NakedSingle.label(), "Naked Single");
        assert_eq!(Strategy::BoxLineReduction.label(), "Box/Line Reduction");
        assert_eq!(Strategy::XYChain.label(), "XY-Chain");
        assert_eq!(Strategy::BruteForce.label(), "Brute Force");
    }

    #[test]
    fn test_strategy_display() {
        let s = Strategy::HiddenSingle;
        assert_eq!(s.to_string(), "Hidden Single");

        let s = Strategy::Fireworks;
        assert_eq!(s.to_string(), "Fireworks");
    }

    #[test]
    fn test_difficulty_enum_variants() {
        let difficulties = [
            Difficulty::Trivial,
            Difficulty::Basic,
            Difficulty::Tough,
            Difficulty::Diabolical,
            Difficulty::Extreme,
        ];
        for &d in &difficulties {
            match d {
                Difficulty::Trivial => {}
                Difficulty::Basic => {}
                Difficulty::Tough => {}
                Difficulty::Diabolical => {}
                Difficulty::Extreme => {}
            }
        }
    }
}
