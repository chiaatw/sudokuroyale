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

pub mod action;
pub mod algorithms;
pub mod board;
pub mod cancelable;
pub mod changer;
pub mod clues;
pub mod deadly_rectangles;
pub mod effects;
pub mod error;
pub mod option;
pub mod pseudo_cell;
pub mod reporter;
pub mod solver;
pub mod strategy;
pub mod technique;
pub mod timing;

pub use avoidable_rectangles::{AvoidableRectanglesSolver, find_avoidable_rectangles};
pub use brute_force::{BruteForceSolver, BruteForceResult};
pub use bugs::{BugSolver, find_bugs};
pub use empty_rectangles::{EmptyRectangleSolver, find_empty_rectangles};
pub use fireworks::{FireworksSolver, find_fireworks};

pub use fish::{
    XWingSolver, SwordfishSolver, JellyfishSolver,
    find_x_wings, find_swordfish, find_jellyfish,
};

pub use hidden_singles::{HiddenSingleSolver, find_hidden_singles};


pub use hidden_tuples::{
    HiddenPairSolver, HiddenTripleSolver, HiddenQuadSolver,
    find_hidden_pairs, find_hidden_triples, find_hidden_quads,
    find_hidden_tuples, is_degenerate,
};

pub use intersection_removals::{IntersectionSolver, find_intersection_removals};
pub use naked_singles::{NakedSingleSolver, find_naked_singles};
pub use naked_tuples::{NakedPairSolver, NakedTripleSolver, NakedQuadSolver, find_naked_tuples};
pub use peers::{PeerSolver, find_peers};
pub use singles_chains::{SinglesChainSolver, find_singles_chains};
pub use skyscrapers::SkyscraperSolver;
pub use two_string_kites::TwoStringKiteSolver;
pub use unique_rectangles::{UniqueRectangleSolver, find_unique_rectangles};
pub use wxyz_wings::{WXYZWingSolver, find_wxyz_wings};
pub use xy_chains::XYChainSolver;
pub use xyz_wings::XYZWingSolver;
pub use y_wings::YWingSolver;

pub use action::Action;
pub use algorithms::Algorithms;
pub use board::Board;
pub use cancelable::Cancelable;
pub use changer::Changer;
pub use clues::Clues;
pub use deadly_rectangles::DeadlyRectangles;
pub use effects::Effects;
pub use error::Error;
pub use option::Option;
pub use pseudo_cell::PseudoCell;
pub use reporter::Reporter;
pub use solver::Solver;
pub use strategy::Strategy;
pub use technique::Technique;
pub use timing::Timing;
