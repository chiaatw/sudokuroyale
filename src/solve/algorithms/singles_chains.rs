use std::collections::HashMap;
use itertools::Itertools;

use super::*;

/// Solver for Singles Chain (Strong-Weak Link / X-Chain) strategy
/// 
/// Detecs chains of candidates confined to pairs across houses and removes candidates
/// that can be logically deduced from the chain
pub struct SinglesChainSolver;

impl Solver for SinglesChainSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::SinglesChain
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_singles_chains(board, single)
    }
}

// Finds al Singles Chains on the board and returns candidate erasures and clues
pub fn find_singles_chains(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

// Ignore cells that already have a single candidate
    let ignore = board.cells_with_n_candidates(1);

    for known in Known::iter() {
        let possibles = board.candidate_cells(known) - ignore;
        if possibles.is_empty() {
            continue;
        }

// Build the candidate nodes and peer graph
        let mut nodes = CellSet::empty();
        let mut peer_graph: HashMap<Cell, CellSet> = HashMap::new();

        for house in House::iter() {
            let house_cells = board.house_candidate_cells(house, known);
            if house_cells.len() == 2 {
                nodes |= house_cells;
                let pair = house_cells.as_pair().unwrap();
                let (a, b) = pair;
                *peer_graph.entry(a).or_default() += b;
                *peer_graph.entry(b).or_default() += a;
            }
        }

// Identify candidate cells that see each other
        let candidates = possibles & nodes
            .iter()
            .combinations(2)
            .fold(CellSet::empty(), |acc, pair| acc | (pair[0].peers() & pair[1].peers()));

        let mut chains:Vec<Chain> = Vec::new();
        let mut cell_chains: HashMap<Cell, (usize, usize)> = HashMap::new();

        for candidate in candidates {
            let sees = nodes & candidate.peers();
            let mut chain = Chain::new(candidate);
            let mut stack = vec![sees];
            let mut shortest = cell_chains
                .get(&candidate)
                .map_or(usize::MAX, |(_, len)| *len);

            while !stack.is_empty() {
                let pool = stack.last_mut().unwrap();
                if pool.is_empty() || chain.nodes.len() + 1 >= shortest {
                    if !chain.nodes.is_empty() {
                        chain.pop();
                    }
                    stack.pop();
                    continue;
                }

                let node = pool.pop().unwrap();
                if node == candidate || chain.has(node) {
                    continue;
                }

                chain.push(node);

                if sees[node] && chain.is_mismatched() {
                    if chain.all_nodes_in_same_block() {
// degenerate hidden pair, ignore
                        cell_chains.remove(&candidate);
                        break;
                    }

                    shortest = chain.nodes.len();
                    chains.push(chain.clone());

                    (candidates & chain.sees()).iter().for_each(|cell| {
                        cell_chains.insert(cell, (chains.len() - 1, chain.len()));
                    });

                    chain.pop();
                    continue;
                }

                let next = peer_graph[&node] - chain.nodes - candidate;
                if !next.is_empty() {
                    stack.push(next);
                } else {
                    chain.pop();
                }
            }
        }

// Group cells y chain index and create actions
        let mut grouped: HashMap<usize, CellSet> = HashMap::new();
        cell_chains.iter().for_each(|(cell, (index, _))| {
            *grouped.entry(*index).or_default()+= *cell;
        });

        for (_, cells) in grouped {
            let mut action = Action::new(Strategy::SinglesChain);
            action.erase_cells(cells, known);
            if effects.add_action(action) && single {
                return Some(effects);
            }
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

// Tracks a candidate chain
#[derive(Debug, Clone)]
struct Chain {
    candidate: Cell,
    nodes: CellSet,
    colors: Colors,
    stack: Vec<Cell>,
    end: Option<Cell>,
    color: Color,
}

impl Chain {
    pub fn new(candidate: Cell) -> Self {
        Self {
            candidate,
            nodes: CellSet::empty(),
            colors: Colors::new(),
            stack: Vec::new(),
            end: None,
            color: Color::Green,
        }
    }

//Returns true if the chain is mismatched (strong/weak link logic)
    pub fn is_mismatched(&self) -> bool {
        match self.color {
            Color::Red => false,
            Color::Green => true,
        }
    }

// Returns true if all nodes of the chain are in the same block (degenerate case)
    pub fn all_nodes_in_same_block(&self) -> bool {
        let mut block: Option<House> = None;
        for cell in self.nodes {
            match block {
                None => block = Some(cell.block()),
                Some(b) => if b != cell.block() {
                    return false;
                }
            }
        }
        true
    }

    pub fn has(&self, node: Cell) -> bool {
        self.nodes.has(node)
    }

    pub fn push(&mut self, node: Cell) {
        self.color.flip();
        self.end = Some(node);
        self.nodes += node;
        self.colors.add(node, self.color);
        self.stack.push(node);
    }

    pub fn pop(&mut self) {
        if let Some(end) = self.end {
            self.stack.pop();
            self.color.flip();
            self.nodes -= end;
            self.colors.remove(end);
            self.end = self.stack.last().copied();
        }
    }

    pub fn len(&self) -> usize {
        self.stack.len().saturating_sub(1)
    }

// Returns the intersection of peers of first and last nodes in the stack
    pub fn sees(&self) -> CellSet {
        self.stack.first().unwrap().peers() & self.stack.last().unwrap().peers()
    }
}

// Strong/Weak link coloring
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Color {
    Red,
    Green
}

impl Color {
    pub fn flip(&mut self) {
        *self = match *self {
            Color::Red => Color::Green,
            Color::Green => Color::Red,
        }
    }
}

// Tracks nodes by color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Colors((CellSet, CellSet));

impl Colors {
    pub fn new() -> Self {
        Self((CellSet::empty(), CellSet::empty()))
    }

    pub fn add(&mut self, node: Cell, color: Color) {
        match color {
            Color::Red => self.0 .0 += node,
            Color::Green => self.0 .1 += node,
        }
    }

    pub fn remove(&mut self, cell: Cell) {
        self.0 .0 -= cell;
        self.0 .1 -= cell;
    }
}