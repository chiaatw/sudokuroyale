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
        let mut map: HashMap<Cell, HashMap<Known, Verdict>> = HashMap::new();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Cell, CellSet, Known};

    fn cell(i: usize) -> Cell {
        Cell::new(i as u8)
    }

    fn known(n: u8) -> Known {
        // FIX: bei dir ist Known::new(...) korrekt, nicht Known::from(u8)
        Known::new(n)
    }

    #[test]
    fn clues_start_empty() {
        let clues = Clues::new();
        assert!(clues.is_empty());
        assert_eq!(clues.collect().len(), 0);
        assert_eq!(clues.display(), EMPTY_SET.to_string());
    }

    #[test]
    fn add_single_clue_cell() {
        let mut clues = Clues::new();
        clues.clue_cell_for_known(Verdict::Set, cell(0), known(1));

        assert!(!clues.is_empty());

        let collected = clues.collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(
            collected.get(&cell(0)).unwrap().get(&known(1)),
            Some(&Verdict::Set)
        );
    }

    #[test]
    fn add_multiple_clues_same_known() {
        let mut clues = Clues::new();
        let cells = CellSet::from_iter([cell(0), cell(1), cell(2)]);
        clues.clue_cells_for_known(Verdict::Erase, cells, known(2));

        let collected = clues.collect();
        assert_eq!(collected.len(), 3);
        for c in [cell(0), cell(1), cell(2)] {
            assert_eq!(
                collected.get(&c).unwrap().get(&known(2)),
                Some(&Verdict::Erase)
            );
        }
    }

    #[test]
    fn add_clues_for_multiple_knowns() {
        let mut clues = Clues::new();
        let cells = CellSet::from_iter([cell(3), cell(4)]);
        let knowns = KnownSet::from_iter([known(1), known(2)]);

        clues.clue_cells_for_knowns(Verdict::Primary, cells, knowns);

        let collected = clues.collect();
        for c in [cell(3), cell(4)] {
            assert_eq!(
                collected.get(&c).unwrap().get(&known(1)),
                Some(&Verdict::Primary)
            );
            assert_eq!(
                collected.get(&c).unwrap().get(&known(2)),
                Some(&Verdict::Primary)
            );
        }
    }

    #[test]
    fn collect_for_known_returns_correct_cells() {
        let mut clues = Clues::new();
        clues.clue_cell_for_known(Verdict::Secondary, cell(0), known(1));
        clues.clue_cell_for_known(Verdict::Secondary, cell(1), known(1));
        clues.clue_cell_for_known(Verdict::Tertiary, cell(2), known(2));

        let known1_map = clues.collect_for_known(known(1));
        assert_eq!(known1_map.len(), 2);
        assert_eq!(known1_map.get(&cell(0)), Some(&Verdict::Secondary));
        assert_eq!(known1_map.get(&cell(1)), Some(&Verdict::Secondary));

        let known2_map = clues.collect_for_known(known(2));
        assert_eq!(known2_map.len(), 1);
        assert_eq!(known2_map.get(&cell(2)), Some(&Verdict::Tertiary));
    }

    #[test]
    fn duplicate_cells_merge() {
        let mut clues = Clues::new();
        clues.clue_cell_for_known(Verdict::Set, cell(0), known(1));
        clues.clue_cell_for_known(Verdict::Set, cell(0), known(1)); // duplicate

        let collected = clues.collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected.get(&cell(0)).unwrap().len(), 1);
    }

    #[test]
    fn display_nonempty_contains_verdicts_and_cells() {
        let mut clues = Clues::new();
        clues.clue_cell_for_known(Verdict::Set, cell(0), known(1));
        clues.clue_cell_for_known(Verdict::Erase, cell(1), known(2));

        let display_str = clues.display();
        assert!(display_str.contains("Set"));
        assert!(display_str.contains("Erase"));
        assert!(display_str.contains(&cell(0).to_string()));
        assert!(display_str.contains(&cell(1).to_string()));
    }

    #[test]
    fn verdict_color_char_returns_string() {
        let c = 'X';
        assert_eq!(Verdict::None.color_char(c), "X".to_string());
        // ANSI-Farben nicht stabil testbar, aber String sollte nicht leer sein
        assert!(!Verdict::Set.color_char(c).is_empty());
        assert!(!Verdict::Erase.color_char(c).is_empty());
        assert!(!Verdict::Primary.color_char(c).is_empty());
    }

    #[test]
    fn clue_cell_for_knowns_handles_multiple_knowns() {
        let mut clues = Clues::new();
        let knowns = KnownSet::from_iter([known(1), known(2)]);
        clues.clue_cell_for_knowns(Verdict::Related, cell(0), knowns);

        let collected = clues.collect();
        assert_eq!(collected.get(&cell(0)).unwrap().len(), 2);
        assert_eq!(
            collected.get(&cell(0)).unwrap().get(&known(1)),
            Some(&Verdict::Related)
        );
        assert_eq!(
            collected.get(&cell(0)).unwrap().get(&known(2)),
            Some(&Verdict::Related)
        );
    }
}