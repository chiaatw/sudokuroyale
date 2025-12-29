use super::*;

// X-Wing solver wrapper for the engine
pub struct XWingSolver;

impl Solver for XWingSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::XWing
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_x_wings(board, single)
    }
}

pub struct SwordfishSolver;

impl Solver for SwordfishSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::Swordfish
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_swordfish(board, single)
    }
}

// Jellyfish solver wrapper for the engine
pub struct JellyfishSolver;

impl Solver for JellyfishSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::Jellyfish
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_jellyfish(board, single)
    }
}

// Entry function for X-Wing strategy
pub fn find_x_wings(board: &Board, single: bool) -> Option<Effects> {
    find_fish(board, single, 2, Strategy::XWing)
}

// Entry function for Swordfish strategy
pub fn find_swordfish(board: &Board, single: bool) -> Option<Effects> {
    find_fish(board, single, 3, Strategy::Swordfish)
}

// Entry function for Jellyfish strategy
pub fn find_jellyfish(board: &Board, single: bool) -> Option<Effects> {
    find_fish(board, single, 4, Strategy::Jellyfish)
}

/// Generic Fish detection logic
/// Handles X-Wing (size 2), Swordfish (size 3), Jellyfish (size 4)
fn find_fish(board: &Board, single: bool, size: usize, strategy: Strategy) -> Option<Effects> {
    let mut effects = Effects::new(); 

// First check rows, then columns if no single-action early exit
    if !check_houses(board, single, size, strategy, Shape::Row, &mut effects) {
        check_houses(board, single, size, strategy, Shape::Column, &mut effects);
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

// Core logic for detecting Fish patterns along a given shape (row or column)
fn check_houses(
    board: &Board,
    single: bool,
    size: usize,
    strategy: Strategy,
    shape: Shape,
    effects: &mut Effects,
) -> bool {
    for known in Known::iter() {
        let candidate_cells = board.candidate_cells(known);

// Iterate over all combinations of houses of the given size
        for candidates in shape
            .house_iter()
            .map(|house| (house, house.cells() & candidate_cells))
// Only consider houses that have 2..=size candidates for this known
            .filter(|(_, cells)| 2 <= cells.len() && cells.len() <= size)
            .map(|(house, cells)| (house, cells, house.crossing_houses(cells)))
            .combinations(size)
        {
// Union of all crossing houses (the other orientation)
            let crosses = candidates
                .iter()
                .map(|(_, _, crosses)| *crosses)
                .union_houses();

            if crosses.len() != size {
                continue;
            }

// Skip degenerate intermediate combinations for Swordfish and Jellyfish
        if size > 2 && candidates
            .iter()
            .map(|(_, _, crosses)| *crosses)
            .filter(|crosses| crosses.len() < 3)
            .combinations(2)
            .map(|pair| pair[0] | pair[1])
            .any(|union| union.len() <= 2)
        {
            continue;
        }

        if size > 3 && candidates
            .iter()
            .map(|(_, _, crosses)| *crosses)
            .filter(|crosses| crosses.len() < 4)
            .combinations(3)
            .map(|pair| pair[0] | pair[1] | pair[2])
            .any(|union| union.len() <= 3)
        {
            continue;
        }

        let main_cells = candidates.iter().map(|(_, cells, _)| *cells).union_cells();
        let cross_cells = crosses.cells() & candidate_cells;
        let erase = cross_cells - main_cells;

        if erase.is_empty() {
            continue;
        }

// Construct the action to erase candidate cells and add clues
        let mut action = Action::new(strategy);
        action.erase_cells(erase, known);

        candidates.iter().for_each(|(house, cells, _)| {
            action.clue_cells_for_known(Verdict::Secondary, *cells, known);
            action.clue_cells_for_known(
                Verdict::Related,
                house.cells() - main_cells - board.knowns(),
                known,
            );
        });

// Early exit if only a single solution is required     
        if effects.add_action(action) && single {       
            return true;
            }
        }
    }
    false
}