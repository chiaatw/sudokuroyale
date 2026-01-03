//! This module defines the [Board] type which tracks the current state of a Sudoku puzzle
//! along with the tools for modifying it and reporting the consequences of those modifications
//! 
//! Modifying a board such as setting givens (starting clues) removing candidates or solving cells
//! can trigger follow on [Action]s
//! or produce [Error]s
//! All of these are collected in an [Effects] object
//! 
//! A [Changer] can be used to apply actions to a board and optionally propagate follow on actions
//! automatically according to its [Options]
//! Currently the board automatically removes candidates from neighboring cells when a cell becomes known
//! While this behavior could be made optional most players likely appreciate having obvious pencil marks cleared automatically
//! 
//! The [Strategy] enum represents the various ways the board can be modified as well as the types of deductions produced by different solving [algorithms]
//! 
//! A [PseudoCell] represents multiple cells treated as one by certain solving algorithms
//! Currently this is used only by the Avoidable Rectangle strategy but other strategies could leverage it to 
//! derive additional deductions
//! 
//! For details on the components that make up a board see the [layout] module

pub use action::Action;
pub use board::{Board, Change};
pub use changer::{ChangeResult, Changer};
pub use clues::{Clues, Verdict};
pub use effects::Effects;
pub use error::Error;
pub use options::Options;
pub use pseudo_cell::PseudoCell;
pub use strategy::{Difficuulty, Strategy};

mod action;
mod board;
mod changer;
mod clues;
mod effects;
mod error;
mod options;
mod pseudo_cell;
mod strategy;