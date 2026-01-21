pub mod algorithms;
pub mod candidates;
pub mod strategy_ord;
pub mod validator;

pub use strategy_ord::solver::Solver;
pub use strategy_ord::strategy::Strategy;

pub use strategy_ord::algorithms::find_brute_force;
pub use strategy_ord::algorithms::find_intersection_removals;

pub use strategy_ord::timing::Timings;

pub use strategy_ord::technique::Technique;
pub use strategy_ord::technique::NON_PEER_TECHNIQUES;

pub use strategy_ord::deadly_rectangles::creates_deadly_rectangles;
