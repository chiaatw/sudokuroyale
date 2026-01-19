use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use itertools::Itertools;

use super::*;
use crate::puzzle::{Action, Cell, CellSet, Known, KnownSet, Board, Effects, Strategy, Verdict};

use crate::layout::values::known::KnownLike;
use crate::layout::cells::cell_set::CellIteratorUnion;

/// Solver for the XY-Chain strategy
/// 
/// Detects XY-Chains on the board and produces candidate eliminations
pub struct XYChainSolver;

impl Solver for XYChainSolver {
    #[inline(always)]
    fn strategy(&self) -> Strategy {
        Strategy::XYChain
    }

    #[inline(always)]
    fn apply(&self, board: &Board, single: bool) -> Option<Effects> {
        find_xy_chains(board, single)
    }
}

/// Finds all XY-Chains on the board and returns their effects
pub fn find_xy_chains(board: &Board, single: bool) -> Option<Effects> {
    let mut effects = Effects::new();

    let bi_values = board.cells_with_n_candidates(2);
    let mut forest = Forest::new();

// Build graph nodes for all bi-value cells
    for cell in bi_values {
        forest.add_node(board, cell);
    }

    for k in Known::iter() {
        let candidates = board.candidate_cells(k);
        let mut found = Found::new(k);

        for graph in forest.graphs.values() {
            if graph.nodes.len() < 4 {
// Ignore too small graphs
                continue;
            }

            let erasables = candidates & graph.peers[k.usize()];
            if erasables.is_empty() {
                continue;
            }

// Find starting cells for chains
            let starts = erasables.iter().fold(CellSet::empty(), |acc, cell| {
                acc | (cell.peers() & candidates & graph.cells[k.usize()])
            });

            for start in starts {
                let mut chains: VecDeque<Rc<Chain>> = VecDeque::new();
                chains.push_back(Rc::new(Chain::new(&graph.nodes[&start], k)));

                while let Some(chain) = chains.pop_front() {
                    for end in chain.edges() {
                        let erasable = start.peers() & end.peers() & erasables;
                        let extended = Chain::extend(&chain, &graph.nodes[&end], erasable);

                        if !extended.erases.is_empty() {
                            found.add(&extended);
                        }

                        if !extended.edges().is_empty() {
                            chains.push_back(extended);
                        }
                    }
                }
            }
        }

        if found.resolve(single, &mut effects) {
            return Some(effects);
        }
    }

    if effects.has_actions() {
        Some(effects)
    } else {
        None
    }
}

/// Forest of connected graphs representing bi-value cells
struct Forest {
    graphs: HashMap<Cell, Graph>,
}

impl Forest {
    fn new() -> Self {
        Self {
            graphs: HashMap::new(),
        }
    }

/// Adds a new node to the appropriate graph, or creates a new graph if necessary
    fn add_node(&mut self, board: &Board, cell: Cell) {
        let node = Rc::new(Node::new(board, cell));

// Identify which graphs the node can connect to
        let mut sees = self
            .graphs
            .iter()
            .filter(|(_, g)| g.can_add_node(&node))
            .map(|(c, _)| *c)
            .union_cells();

        if sees.is_empty() {
            self.graphs.insert(cell, Graph::new(&node));
        } else if sees.len() == 1 {
            let root = sees.pop().unwrap();
            self.graphs.get_mut(&root).unwrap().add_node(&node);
        } else {
            let root = sees.pop().unwrap();
            let mut graph = self.graphs.remove(&root).unwrap();
            graph.add_node(&node);

            for seen in sees {
                graph.merge(self.graphs.remove(&seen).unwrap());
            }

            self.graphs.insert(root, graph);
        }
    }
}

/// Graph of connected bi-value cells
/// Index by candidate
#[allow(dead_code)]
struct Graph {
    root: Cell,
    cells: [CellSet; 9], 
    peers: [CellSet; 9],
    nodes: HashMap<Cell, Rc<Node>>,
}

impl Graph {
    fn new(node: &Rc<Node>) -> Self {
        let root = node.cell;
        let mut cells = [CellSet::empty(); 9];
        cells[0] = CellSet::of(&[root]);

        let mut peers = [CellSet::empty(); 9];
        peers[node.min_known.usize()] = root.peers();
        peers[node.max_known.usize()] = root.peers();

        let mut nodes = HashMap::new();
        nodes.insert(root, Rc::clone(node));

        Self {
            root,
            cells,
            peers,
            nodes,
        }
    }

    fn can_add_node(&self, node: &Rc<Node>) -> bool {
        self.peers[node.min_known.usize()].has(node.cell) || self.peers[node.max_known.usize()].has(node.cell)
    }

    fn add_node(&mut self, node: &Rc<Node>) {
        let cell = node.cell;
        let min_k = node.min_known.usize();
        let max_k = node.max_known.usize();

        self.cells[0].add(cell);
        self.cells[min_k].add(cell);
        self.cells[max_k].add(cell);

        let peers = cell.peers();
        self.peers[min_k].union_with(peers);
        self.peers[max_k].union_with(peers);

        self.nodes.insert(cell, Rc::clone(node));
    }

    fn merge(&mut self, other: Graph) {
        self.cells
            .iter_mut()
            .enumerate()
            .for_each(|(i, set)| set.union_with(other.cells[i]));

        for (i, peers) in other.peers.iter().enumerate() {
            self.peers[i].union_with(*peers);
        }

        self.nodes.extend(other.nodes);
    }
}

/// Node representing a single bi-value cell
#[allow(dead_code)]
struct Node {
    cell: Cell,
    pair: KnownSet,
    min_known: Known,
    min_edges: CellSet,
    max_known: Known,
    max_edges: CellSet,
}

impl Node {
    fn new(board: &Board, cell: Cell) -> Self {
        let edges = cell.peers() & board.cells_with_n_candidates(2);
        let pair = board.candidates(cell);
        let (min_known, max_known) = pair.as_pair().unwrap();

        Self {
            cell,
            pair,
            min_known,
            min_edges: (edges & board.candidate_cells(min_known)) - cell,
            max_known,
            max_edges: (edges & board.candidate_cells(max_known)) - cell,
        }
    }

    fn other(&self, known: Known) -> Known {
        if known == self.min_known {
            self.max_known
        } else if known == self.max_known {
            self.min_known
        } else {
            panic!("Node::other: known not in pair")
        }
    }

    fn edges(&self, known: Known) -> CellSet {
        if known == self.min_known {
            self.min_edges
        } else if known == self.max_known {
            self.max_edges
        } else {
            panic!("Node::edges: known not in pair")
        }
    } 
}

/// Chain of nodes for XY-Chain search
#[allow(dead_code)]
struct Chain {
    head: Rc<Link>,
    len: usize,
    start: Cell,
    start_known: Known,
    end: Cell,
    end_known: Known,
    visited: CellSet,
    erases: CellSet,
}

impl Chain {
    fn new(start: &Rc<Node>, known: Known) -> Self {
        let link = Rc::new(Link::new(start, known));
        Self {
            head: link.clone(),
            len: 1,
            start: start.cell,
            start_known: known,
            end: start.cell,
            end_known: link.known,
            visited: CellSet::empty() + start.cell,
            erases: CellSet::empty(),
        }
    }

    fn extend(&self, node: &Rc<Node>, erasable: CellSet) -> Rc<Self> {
        let head = Link::extend(&self.head, node);
        let len = head.len;
        let end_known = head.known;
        let erases = if len >= 4 && end_known == self.start_known {
            erasable
        } else {
            CellSet::empty()
        };

        Rc::new(Self {
            head,
            len,
            start: self.start,
            start_known: self.start_known,
            end: node.cell,
            end_known,
            visited: self.visited + node.cell,
            erases,
        })
    }

    fn edges(&self) -> CellSet {
        self.head.edges() - self.visited
    }
}

/// Shared link in a chain of nodes
#[allow(dead_code)]
struct Link {
    tail: Option<Rc<Link>>,
    tail_known: Known,
    len: usize,
    node: Rc<Node>,
    known: Known,
}

impl Link {
    fn new(start: &Rc<Node>, known: Known) -> Self {
        Self {
            tail: None,
            tail_known: known, 
            len: 1,
            node: Rc::clone(start),
            known: start.other(known),
        }
    }

    fn extend(tail: &Rc<Link>, node: &Rc<Node>) -> Rc<Self> {
        Rc::new(Self {
            tail: Some(Rc::clone(tail)),
            tail_known: tail.known,
            len: tail.len + 1,
            node: Rc::clone(node),
            known: node.other(tail.known),
        })
    }

    fn edges(&self) -> CellSet {
        self.node.edges(self.known)
    }
}

/// Tracks shortest unique chains for a starting known
#[allow(dead_code)]
struct Found {
    known: Known,
    erases: CellSet,
    chains: Vec<Rc<Chain>>,
}

impl Found {
    fn new(known: Known) -> Self {
        Self {
            known,
            erases: CellSet::empty(),
            chains: Vec::new(),
        }
    }

    fn add(&mut self, chain: &Rc<Chain>) {
        self.erases |= chain.erases;
        add_candidate(chain, &mut self.chains);
    }

    fn resolve(&self, single: bool, effects: &mut Effects) -> bool {
        let mut remaining = self.erases;
        for chain in self.chains.iter().sorted_by(|a, b| {
            a.len
                .cmp(&b.len)
                .then(a.erases.len().cmp(&b.erases.len()))
        }) {
            let mut action = Action::new_erase_cells(Strategy::XYChain, chain.erases, chain.start_known);

            let mut link = Some(&chain.head);
            while let Some(next) = link {
                let cell = next.node.cell;
                let known = next.node.other(next.known);
                action.clue_cell_for_known(Verdict::Secondary, cell, known);
                action.clue_cell_for_known(Verdict::Tertiary, cell, next.known);
                link = next.tail.as_ref();
            }

            if effects.add_action(action) && single {
                return true;
            }

            remaining -= chain.erases;
            if remaining.is_empty() {
                break;
            }
        }

        false
    }
}

/// Adds a new chain candidate, removing redundant chains
fn add_candidate(new: &Rc<Chain>, chains: &mut Vec<Rc<Chain>>) {
    let mut remove: Vec<usize> = Vec::new();
    let mut add = true;

    for (i, chain) in chains.iter().enumerate() {
        if new.len < chain.len {
            if new.erases.has_all(chain.erases) {
                remove.push(i);
            }
        } else if chain.erases.has_all(new.erases) {
            add = false;
            break;
        }
    }

    for i in remove.iter().rev() {
        chains.remove(*i);
    }

    if add {
        chains.push(Rc::clone(new));
    }
}

#[cfg(test)]
mod tests {
    use crate::io::{Parse, Parser};
    use crate::layout::cells::cell::cell;
    use crate::layout::cells::cell_set::cells;
    use crate::layout::values::known::known;

    use super::*;

    #[test]
    fn test_xy_chain_solver() {
        let parser = Parse::wiki().stop_on_error();
        let (board, ..) = parser.parse(
            "441181i402i4k4080h0g20g10884418411024c0c03o4100gs421g4p4o4410h09q403o030o6om091184o42go040p0og20o040031g0508g2g214a40ha409403020411403g108100g8188880g412011g402g4",
        );

        let solver = XYChainSolver;

        if let Some(got) = solver.apply(&board, true) {
            // Build the expected action manually
            let mut expected = Action::new(Strategy::XYChain);

            expected.erase(cell!("C4"), known!("9"));
            expected.clue_cells_for_known(Verdict::Secondary, cells!("B7 E5"), known!("2"));
            expected.clue_cells_for_known(Verdict::Tertiary, cells!("B5 C9"), known!("2"));
            expected.clue_cells_for_known(Verdict::Secondary, cells!("B5 F4"), known!("8"));
            expected.clue_cells_for_known(Verdict::Tertiary, cells!("B7 E5"), known!("8"));
            expected.clue_cells_for_known(Verdict::Secondary, cells!("C9"), known!("9"));
            expected.clue_cells_for_known(Verdict::Tertiary, cells!("F4"), known!("9"));

            // Compare the generated effect with expected
            assert_eq!(format!("{:?}", expected), format!("{:?}", got.actions()[0]));
        } else {
            panic!("XY-Chain solver found no effects");
        }
    }
    #[test]
    fn test_xy_chain_basic() {
        let parser = crate::io::Parse::grid().stop_on_error();
        let (board, _, failed) = parser.parse(
            "
            +-------+-------+-------+
            | 12 34 | 56 78 | 9 . . |
            | .  .  | 12 34 | 56 78 |
            | .  .  | .  .  | 12 34 |
            +-------+-------+-------+
            "
        );
        assert_eq!(None, failed);

        let solver = XYChainSolver;
        let effects = solver.apply(&board, true);
        assert!(effects.is_some(), "XY-Chain sollte gefunden werden");
        let effects = effects.unwrap();
        assert!(effects.has_actions(), "XY-Chain Effekte sollten Aktionen enthalten");
    }

    #[test]
    fn test_xy_chain_none() {
        let board = crate::layout::Board::new(); // leeres Board
        let solver = XYChainSolver;
        let effects = solver.apply(&board, true);
        assert!(effects.is_none(), "Kein XY-Chain sollte None zurückgeben");
    }

    #[test]
    fn test_xy_chain_multiple() {
        let parser = crate::io::Parse::grid().stop_on_error();
        let (board, _, failed) = parser.parse(
            "
            +-------+-------+-------+
            | 12 34 | 56 78 | 9 . . |
            | 12 34 | 56 78 | 9 . . |
            | .  .  | 12 34 | 56 78 |
            +-------+-------+-------+
            "
        );
        assert_eq!(None, failed);

        let solver = XYChainSolver;
        let effects = solver.apply(&board, false);
        assert!(effects.is_some(), "Mehrere XY-Chains sollten erkannt werden");
        let effects = effects.unwrap();
        assert!(effects.actions().len() > 1, "Mehrere Aktionen sollten vorhanden sein");
    }

    #[test]
    fn test_xy_chain_clues() {
        let parser = crate::io::Parse::grid().stop_on_error();
        let (board, _, failed) = parser.parse(
            "
            +-------+-------+-------+
            | 12 34 | 56 78 | 9 . . |
            | .  .  | 12 34 | 56 78 |
            | .  .  | .  .  | 12 34 |
            +-------+-------+-------+
            "
        );
        assert_eq!(None, failed);

        let solver = XYChainSolver;
        let effects = solver.apply(&board, true).unwrap();
        let action = &effects.actions()[0];
        assert!(!action.secondary_clues().is_empty(), "Secondary clues sollten gesetzt sein");
        assert!(!action.tertiary_clues().is_empty(), "Tertiary clues sollten gesetzt sein");
    }
}
