use crate::puzzle::Strategy;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Options(u8);

impl Options {
    const STOP_ON_ERROR: u8 = 1 << 0;
    const SOLVE_NAKED_SINGLES: u8 = 1 << 1;
    const SOLVE_HIDDEN_SINGLES: u8 = 1 << 2;
    const SOLVE_INTERSECTION_REMOVALS: u8 = 1 << 3;

// No options set
    pub const fn none() -> Self {
        Self(0)
    }

// only stop on errors
    pub const fn errors() -> Self {
        Self(Self::STOP_ON_ERROR)
    }

// all options enabled
    pub const fn all() -> Self {
        Self(
            Self::STOP_ON_ERROR
            | Self::SOLVE_NAKED_SINGLES
            | Self::SOLVE_HIDDEN_SINGLES
            | Self::SOLVE_INTERSECTION_REMOVALS
        )
    }

    #[inline]
    pub fn stop_on_error(&self) -> bool {
        self.0 & Self::STOP_ON_ERROR != 0
    }

    #[inline]
    pub fn solve_naked_singles(&self) -> bool {
        self.0 & Self::SOLVE_NAKED_SINGLES != 0
    }

    #[inline]
    pub fn solve_hidden_singles(&self) -> bool {
        self.0 & Self::SOLVE_HIDDEN_SINGLES != 0
    }

    #[inline]
    pub fn solve_intersection_removals(&self) -> bool {
        self.0 & Self::SOLVE_INTERSECTION_REMOVALS != 0
    }

    #[inline]
    pub fn set_stop_on_error(mut self, value: bool) -> Self {
        if value {
            self.0 |= Self::STOP_ON_ERROR;
        } else {
            self.0 &= !Self::STOP_ON_ERROR;
        }
        self
    }

    #[inline]
    pub fn set_solve_naked_singles(mut self, value: bool) -> Self {
        if value {
            self.0 |= Self::SOLVE_NAKED_SINGLES;
        } else {
            self.0 &= !Self::SOLVE_NAKED_SINGLES;
        }

        self
    }

    #[inline]
    pub fn set_solve_hidden_singles(mut self, value: bool) -> Self {
        if value {
            self.0 |= Self::SOLVE_HIDDEN_SINGLES;
        } else {
            self.0 &= !Self::SOLVE_HIDDEN_SINGLES;
        }
        self
    }

    #[inline]
    pub fn set_solve_intersection_removals(mut self, value: bool) -> Self {
        if value {
            self.0 |= Self::SOLVE_INTERSECTION_REMOVALS;
        } else {
            self.0 &= !Self::SOLVE_INTERSECTION_REMOVALS;
        }
        self
    }

    #[inline]
    pub fn solve_singles(mut self) -> Self {
        self.0 |= Self::SOLVE_NAKED_SINGLES | Self::SOLVE_HIDDEN_SINGLES;
        self
    }

    #[inline]
    pub fn return_singles(mut self) -> Self {
        self.0 &= !(Self::SOLVE_NAKED_SINGLES | Self::SOLVE_HIDDEN_SINGLES);
        self
    }

// Determines if a strategy should be applied given the current options
    #[inline]
    pub fn should_apply(&self, strategy: Strategy) -> bool {
        match strategy {
            Strategy::Peer | Strategy::BruteForce => true,
            Strategy::NakedSingle => self.solve_naked_singles(),
            Strategy::HiddenSingle => self.solve_hidden_singles(),
            Strategy::PointingPair
            | Strategy::PointingTriple
            | Strategy::BoxLineReduction => self.solve_intersection_removals(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::puzzle::Strategy;

    #[test]
    fn test_solve_singles() {
        let options = Options::none().solve_singles();

        assert!(options.solve_naked_singles());
        assert!(options.solve_hidden_singles());
    }

    #[test]
    fn test_should_apply() {
        let mut options = Options::none();

        assert_eq!(false, options.should_apply(Strategy::NakedSingle));
        assert_eq!(false, options.should_apply(Strategy::HiddenSingle));
        assert_eq!(false, options.should_apply(Strategy::PointingPair));
        assert_eq!(false, options.should_apply(Strategy::PointingTriple));
        assert_eq!(false, options.should_apply(Strategy::BoxLineReduction));
        assert_eq!(false, options.should_apply(Strategy::Bug));

        options = options.set_solve_naked_singles(true);
        assert_eq!(true, options.should_apply(Strategy::NakedSingle));
        assert_eq!(false, options.should_apply(Strategy::HiddenSingle));

        options = options.set_solve_hidden_singles(true);
        assert_eq!(true, options.should_apply(Strategy::HiddenSingle));

        options = options.return_singles();
        assert_eq!(false, options.should_apply(Strategy::NakedSingle));
        assert_eq!(false, options.should_apply(Strategy::HiddenSingle));

        options = options.set_solve_intersection_removals(true);
        assert_eq!(true, options.should_apply(Strategy::PointingPair));
        assert_eq!(true, options.should_apply(Strategy::PointingTriple));
        assert_eq!(true, options.should_apply(Strategy::BoxLineReduction));
    }
    #[test]
    fn test_predefined_options() {
        let none = Options::none();
        assert!(!none.stop_on_error());
        assert!(!none.solve_naked_singles());
        assert!(!none.solve_hidden_singles());
        assert!(!none.solve_intersection_removals());

        let errors = Options::errors();
        assert!(errors.stop_on_error());
        assert!(!errors.solve_naked_singles());

        let all = Options::all();
        assert!(all.stop_on_error());
        assert!(all.solve_naked_singles());
        assert!(all.solve_hidden_singles());
        assert!(all.solve_intersection_removals());
    }

    #[test]
    fn test_setters_toggle_flags() {
        let mut options = Options::none();

        options = options.set_stop_on_error(true);
        assert!(options.stop_on_error());
        options = options.set_stop_on_error(false);
        assert!(!options.stop_on_error());

        options = options.set_solve_naked_singles(true);
        assert!(options.solve_naked_singles());
        options = options.set_solve_naked_singles(false);
        assert!(!options.solve_naked_singles());

        options = options.set_solve_hidden_singles(true);
        assert!(options.solve_hidden_singles());
        options = options.set_solve_hidden_singles(false);
        assert!(!options.solve_hidden_singles());

        options = options.set_solve_intersection_removals(true);
        assert!(options.solve_intersection_removals());
        options = options.set_solve_intersection_removals(false);
        assert!(!options.solve_intersection_removals());
    }

    #[test]
    fn test_chaining_methods() {
        let options = Options::none()
            .set_stop_on_error(true)
            .set_solve_naked_singles(true)
            .set_solve_hidden_singles(true)
            .set_solve_intersection_removals(true);

        assert!(options.stop_on_error());
        assert!(options.solve_naked_singles());
        assert!(options.solve_hidden_singles());
        assert!(options.solve_intersection_removals());
    }

}
