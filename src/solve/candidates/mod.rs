//! Candidates handling for the solver.
//!
//! This module maintains candidate sets (possible digits) for each cell.
//! It is designed to work with:
//! - `layout::Cell` as an index (0..80)
//! - `layout::{Known, KnownSet}` as digit + digit-set bitset
//! - `solve::validator::Board` as board state (values + givens)

mod store;
mod update;
mod elimination;
mod intersection;
mod singles;
mod utils;

pub use store::Candidates;
pub use update::{recompute_all_candidates, update_after_set_known};
