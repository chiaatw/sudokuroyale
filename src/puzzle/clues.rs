use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{self, Write};

use crate::layout::{Cell, CellSet, Known, KnownSet};
use crate::symbols::EMPTY_SET;

// Verdicts for clues with color-coded display
#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Verdict {
    #[default]
    None,
    Set,
    Erase,
    Related,
    Primary,
    Secondary,
    Tertiary,
}

impl Verdict {
    pub fn color_char(self, c: char) -> String {
        self.color(c.to_string())
    }

    pub fn color(self, str: String) -> String {
        use colored::Colorize;
        match self {
            Self::None => str,
            Self::Set => str.bright_green().bold().blink().to_string(),
            Self::Erase => str.bright_yellow().bold().blink().to_string(),
            Self::Related => str.bright_blue().bold().blink().to_string(),
            Self::Primary => str.bright_purple().bold().blink().to_string(),
            Self::Secondary => str.bright_cyan().bold().blink().to_string(),
            Self::Tertiary => str.bright_red().bold().blink().to_string(),
        }
    }
}

// A single clue linking cells, a known value, and a verdict
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Clue {
    verdict: Verdict,
    known: Known,
    cells: CellSet,
}

// Collection of clues
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Clues {
    clues: Vec<Clue>,
}

impl Clues {
    pub const fn new() -> Self {
        Self {
            clues: Vec::new(),
        }
    }
}

pub trait ClueCollection {
    fn clue_cell_for_known(&mut self, verdict: Verdict, cell: Cell, known: Known);
    fn clue_cells_for_known(&mut self, verdict: Verdict, cells: CellSet, known: Known);
    fn clue_cell_for_knowns(&mut self, verdict: Verdict, cell: Cell, knowns: KnownSet);
    fn clue_cells_for_knowns(&mut self, verdict: Verdict, cells: CellSet, knowns: KnownSet);
    fn is_empty(&self) -> bool;
    fn collect(&self) -> HashMap<Cell, HashMap<Known, Verdict>>;
    fn collect_for_known(&self, known: Known) -> HashMap<Cell, Verdict>;
    fn display(&self) -> String;
}

impl ClueCollection for Clues {
    fn clue_cell_for_known(&mut self, verdict: Verdict, cell: Cell, known: Known) {
        self.clue_cells_for_known(verdict, CellSet::empty() + cell, known);
    }

    fn clue_cells_for_known(&mut self, verdict: Verdict, cells: CellSet, known: Known) {
        let clue = Clue {
            verdict: verdict, 
            known: known, 
            cells: cells,
        };

        match self.clues.binary_search_by(|c| {
            match verdict.partial_cmp(&c.verdict) {
                Some(Ordering::Equal) => known.partial_cmp(&c.known),
                other => other,
            }
            .unwrap()
        }) {
            Ok(index) => self.clues[index].cells |= cells,
            Err(index) => self.clues.insert(index, clue)
        }
    }

    fn clue_cell_for_knowns(&mut self, verdict: Verdict, cell: Cell, knowns: KnownSet) {
        self.clue_cells_for_knowns(verdict, CellSet::empty() + cell, knowns);
    }

    fn clue_cells_for_knowns(&mut self, verdict: Verdict, cells: CellSet, knowns: KnownSet) {
        knowns.iter().for_each(|k| self.clue_cells_for_known(verdict, cells, k));
    }

    fn is_empty(&self) -> bool {
        self.clues.is_empty()
    }

    fn collect(&self) -> HashMap<Cell, HashMap<Known, Verdict>> {
        let mut map = HashMap::new();
        for clue in &self.clues {
            for cell in clue.cells.iter() {
                map.entry(cell).or_default().insert(clue.known, clue.verdict);
            }
        }
        map
    }

    fn collect_for_known(&self, known: Known) -> HashMap<Cell, Verdict> {
        let mut map = HashMap::new();
        for clue in self.clues.iter().filter(|c| c.known == known) {
            for cell in clue.cells.iter() {
                map.insert(cell, clue.verdict);
            }
        }
        map
    }

    fn display(&self) -> String {
        if self.is_empty() {
            return EMPTY_SET.to_string();
        }

        let mut first = true;
        let mut prev_color = Verdict::Secondary;
        let mut result = String::new();

        for clue in &self.clues {
            if first {
                first = false;
                write!(result, "{:?} [", clue.verdict).unwrap();
            } else if clue.verdict != prev_color {
                write!(result, "] {:?} [", clue.verdict).unwrap();
                prev_color = clue.verdict;
            } else {
                result.push_str(", ");
            }
            write!(result, "{}: {}", clue.known, clue.cells).unwrap();
        }

        result.push(']');
        result
    }
}

impl fmt::Display for Clues {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}