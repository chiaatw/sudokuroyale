use crate::puzzle::{Board, Effects, Strategy};

pub trait Solver {
    fn strategy(&self) -> Strategy;
    fn apply(&self, board: &Board, single: bool) -> Option<Effects>;
}

pub mod avoidable_rectangles;
pub mod brute_force;
pub mod bugs;
pub mod empty_rectangles;
pub mod fireworks;
pub mod fish;
pub mod hidden_singles;
pub mod hidden_tuples;
pub mod intersection_removals;
pub mod naked_singles;
pub mod naked_tuples;
pub mod peers;
pub mod singles_chains;
pub mod skyscrapers;
pub mod two_string_kites;
pub mod unique_rectangles;
pub mod wxyz_wings;
pub mod xy_chains;
pub mod xyz_wings;
pub mod y_wings;
