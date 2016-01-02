use ::actions;
use ::board;
use ::console_ui;
use ::game;
use ::search_graph;

use std::cmp;
use std::fmt;

use rand::Rng;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Payoff {
    values: [usize; 2],
}

impl Payoff {
    pub fn score(&self, player: game::PlayerMarker) -> usize {
        self.values[player.index()]
    }
}

impl Default for Payoff {
    fn default() -> Self {
        Payoff { values: [0; 2], }
    }
}

impl fmt::Debug for Payoff {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "[{}, {}]", self.values[0], self.values[1])
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Statistics {
    visits: usize,
    payoff: Payoff,
}

impl Statistics {
    pub fn increment_visit(&mut self, p: Payoff) {
        self.visits += 1;
        self.payoff.values[0] += p.values[0];
        self.payoff.values[1] += p.values[1];
    }
}

impl Default for Statistics {
    fn default() -> Self {
        Statistics { visits: 0, payoff: Default::default(), }
    }
}

impl fmt::Debug for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Statistics(visits: {}, payoff: {:?})", self.visits, self.payoff)
    }
}

#[derive(Clone, Debug)]
pub struct NodeData {
    pub statistics: Statistics,
    cycle: bool,
    known_payoff: Option<Payoff>,
}

impl Default for NodeData {
    fn default() -> Self {
        NodeData {
            cycle: false,
            known_payoff: None,
            statistics: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct EdgeData {
    pub action: actions::Action,
    pub statistics: Statistics,
    cycle: bool,
    known_payoff: Option<Payoff>,
}

impl EdgeData {
    pub fn new(action: actions::Action) -> Self {
        EdgeData {
            action: action,
            cycle: false,
            known_payoff: None,
            statistics: Default::default(),
        }
    }
}

pub type Edge<'a> = search_graph::Edge<'a, game::State, NodeData, EdgeData>;
pub type MutEdge<'a> = search_graph::MutEdge<'a, game::State, NodeData, EdgeData>;
pub type Graph = search_graph::Graph<game::State, NodeData, EdgeData>;
pub type Node<'a> = search_graph::Node<'a, game::State, NodeData, EdgeData>;
pub type ChildList<'a> = search_graph::ChildList<'a, game::State, NodeData, EdgeData>;
pub type ParentList<'a> = search_graph::ParentList<'a, game::State, NodeData, EdgeData>;
pub type MutNode<'a> = search_graph::MutNode<'a, game::State, NodeData, EdgeData>;
pub type MutChildList<'a> = search_graph::MutChildList<'a, game::State, NodeData, EdgeData>;
pub type MutParentList<'a> = search_graph::MutParentList<'a, game::State, NodeData, EdgeData>;
pub type EdgeExpander<'a> = search_graph::EdgeExpander<'a, game::State, NodeData, EdgeData>;

pub enum Rollout<'a> {
    Terminal(MutNode<'a>),
    Unexpanded(EdgeExpander<'a>),
    Cycle(MutNode<'a>),
}

pub fn rollout<'a>(mut node: MutNode<'a>, state: &mut game::State, bias: f64) -> Rollout<'a> {
    loop {
        node = match best_child_edge(node.get_child_list(), state.active_player().marker(), bias) {
            Some(i) => match node.to_child_list().to_edge(i).to_target() {
                search_graph::Target::Expanded(n) => n,
                search_graph::Target::Unexpanded(e) => return Rollout::Unexpanded(e),
                search_graph::Target::Cycle(e) => return Rollout::Cycle(e),
            },
            None => {
                println!("rollout finds no children for (id {}):", node.get_id());
                console_ui::write_board(state.board());
                return Rollout::Terminal(node)
            },
        }
    }
}

fn best_child_edge<'a>(children: ChildList<'a>, player: game::PlayerMarker, bias: f64) -> Option<usize> {
    let parent_visits = children.get_source_node().get_data().statistics.visits as f64;
    let mut best_edge_index = None;
    let mut best_edge_uct = 0.0;
    println!("best_child_edge: visiting {} children", children.len());
    for i in 0..children.len() {
        let edge = children.get_edge(i);
        match edge.get_target() {
            search_graph::Target::Unexpanded(_) => {
                println!("best_child_edge: edge {} is unexpanded", i);
                return Some(i)
            },
            search_graph::Target::Cycle(_) => {
                println!("best_child_edge: ignoring edge {}, which is cycle", i);
                ()
            },
            search_graph::Target::Expanded(e) => {
                let edge_visits = e.get_data().statistics.visits as f64;
                if edge_visits == 0.0 {
                    println!("best_child_edge: edge {} because it hasn't been visited yet", i);
                    best_edge_index = Some(i);
                    best_edge_uct = 0.0;
                } else {
                    let edge_payoff = e.get_data().statistics.payoff.score(player) as f64;
                    let uct = edge_payoff / edge_visits
                        + bias * f64::sqrt(f64::ln(parent_visits) / edge_visits);
                    if uct > best_edge_uct {
                        println!("best_child_edge: edge {}, value {} is new best edge (old index {:?}, value {})",
                                 i, uct, best_edge_index, best_edge_uct);
                        // TODO tie-breaking.
                        best_edge_index = Some(i);
                        best_edge_uct = uct;
                    }
                }
            },
        }
    }
    best_edge_index
}

// fn best_parent_edge<'a>(parents: ParentList<'a>, player: game::PlayerMarker, bias: f64) -> Option<usize> {
//     let mut best_edge_index = None;
//     let mut best_edge_uct = 0.0;
//     for i in 0..parents.len() {
//         println!("best_parent_edge: examining edge {}/{}", i, parents.len());
//         let edge = parents.get_edge(i);
//         let parent_visits = edge.get_source().get_data().statistics.visits as f64;
//         if parent_visits == 0.0 {
//             continue
//         }
//         let edge_payoff = edge.get_data().statistics.payoff.score(player) as f64;
//         let edge_visits = edge.get_data().statistics.visits as f64;
//         if edge_visits == 0.0 {
//             println!("best_parent_edge: choosing parent {} because this edge hasn't been visited", i);
//             return Some(i)
//         } else {
//             let uct = edge_payoff / edge_visits
//                 + bias * f64::sqrt(f64::ln(parent_visits) / edge_visits);
//             if uct > best_edge_uct {
//                 println!("best_parent_edge: new best edge {}, value {} (old best edge {:?}, value {})",
//                          i, uct, best_edge_index, best_edge_uct);
//                 // TODO tie-breaking.
//                 best_edge_index = Some(i);
//                 best_edge_uct = uct;
//             }
//         }
//     }
//     best_edge_index
// }

fn best_parent_edge<'a>(parents: ParentList<'a>, player: game::PlayerMarker, bias: f64) -> Option<usize> {
    match parents.len() {
        0 => None,
        1 => Some(0),
        _ => {
            println!("defaulting to first parent because best parent selection not done yet");
            Some(0)
        },
    }
}

pub fn simulate<R>(state: &mut game::State, rng: &mut R) -> Payoff where R: Rng {
    loop {
        let action = match payoff(&state) {
            None => {
                let mut actions = state.role_actions(state.active_player().role());
                let mut selected = None;
                let mut i = 0;
                loop {
                    match actions.next() {
                        None => break,
                        Some(a) => {
                            if i == 0 {
                                selected = Some(a);
                            } else if rng.next_f64() < (1.0 / (i as f64)) {
                                selected = Some(a);
                            }
                            i += 1;
                        },
                    }
                }
                if let Some(s) = selected {
                    s
                } else {
                    console_ui::write_board(state.board());
                    panic!("Terminal state has no payoff");
                }
            },
            Some(p) => return p,
        };
        state.do_action(&action);
    }
}

fn role_payoff(r: game::Role) -> usize {
    match r {
        game::Role::Dwarf => 1,
        game::Role::Troll => 4,
    }
}

pub fn payoff(state: &game::State) -> Option<Payoff> {
    if state.terminated() {
        let player_1_role = state.player(game::PlayerMarker::One).role();
        let player_1_role_payoff = role_payoff(player_1_role);
        let player_2_role = state.player(game::PlayerMarker::Two).role();
        let player_2_role_payoff = role_payoff(player_2_role);
        let mut payoff: Payoff = Default::default();
        let mut i = state.board().cells_iter();
        loop {
            match i.next() {
                Some((_, board::Content::Occupied(t))) if t.role() == Some(player_1_role) =>
                    payoff.values[0] += player_1_role_payoff,
                Some((_, board::Content::Occupied(t))) if t.role() == Some(player_2_role) =>
                    payoff.values[1] += player_2_role_payoff,
                None => break,
                _ => (),
            }
        }
        Some(payoff)
    } else {
        None
    }
}

fn backprop_payoff<'a>(mut node: MutNode<'a>, payoff: Payoff, mut player: game::PlayerMarker, bias: f64) {
    loop {
        node.get_data_mut().statistics.increment_visit(payoff);
        node = match best_parent_edge(node.get_parent_list(), player, bias) {
            None => break,
            Some(i) => {
                let mut to_parent = node.to_parent_list().to_edge(i);
                to_parent.get_data_mut().statistics.increment_visit(payoff);
                to_parent.to_source()
            },
        };
        player.toggle();
    }
}

fn backprop_cycle<'a>(mut node: MutNode<'a>) {
    // loop {
        node.get_data_mut().cycle = true;
        let mut parents = node.to_parent_list();
        for i in 0..parents.len() {
            set_cycle_from_children(parents.get_edge_mut(i).get_source_mut());
        }
    // }
}

fn set_cycle_from_children<'a>(mut node: MutNode<'a>) {
    {
        let mut children = node.get_child_list_mut();
        for i in 0..children.len() {
            match children.get_edge_mut(i).get_target_mut() {
                search_graph::Target::Unexpanded(_) => return,
                search_graph::Target::Expanded(ref node) if !node.get_data().cycle => return,
                _ => (),
            }
        }
    }
    node.get_data_mut().cycle = true;
}

fn backprop_known_payoff<'a>(mut node: MutNode<'a>, p: Payoff) {
    // loop {
        node.get_data_mut().known_payoff = Some(p);
        let mut parents = node.to_parent_list();
        for i in 0..parents.len() {
            set_known_payoff_from_children(parents.get_edge_mut(i).get_source_mut());
        }
    // }
}

fn set_known_payoff_from_children<'a>(mut node: MutNode<'a>) {
    let mut payoff = None;
    {
        let mut children = node.get_child_list_mut();
        for i in 0..children.len() {
            match children.get_edge_mut(i).get_target_mut() {
                search_graph::Target::Unexpanded(_) => {
                    payoff = None;
                    break
                },
                search_graph::Target::Expanded(node) =>
                    match node.get_data().known_payoff {
                        None => return,
                        Some(known) => payoff = match payoff {
                            None => Some(known),
                            Some(p) => Some(Payoff { values: [cmp::max(p.values[0], known.values[0]),
                                                              cmp::max(p.values[1], known.values[1])], }),
                        },
                    },
                _ => (),
            }
        }
    }
    node.get_data_mut().known_payoff = payoff;
}

pub fn iterate_search<'a, R>(mut state: game::State, graph: &'a mut Graph, rng: &mut R, bias: f64) where R: Rng {
    if let Some(node) = graph.get_node_mut(&state) {
        match rollout(node, &mut state, bias) {
            Rollout::Unexpanded(expander) => {
                state.do_action(&expander.get_edge().get_data().action);
                // println!("after action {:?}:", expander.get_edge().get_data().action);
                // console_ui::write_board(state.board());
                match expander.expand(state.clone(), Default::default).to_target() {
                    search_graph::Target::Expanded(mut node) => {
                        let payoff = simulate(&mut state, rng);
                        let payoff_player = state.active_player().marker();
                        {
                            let mut adder = node.get_child_adder();
                            state.toggle_active_player();
                            for a in state.role_actions(state.active_player().role()).into_iter() {
                                adder.add(EdgeData::new(a));
                            }
                        }
                        backprop_payoff(node, payoff, payoff_player, bias);
                    },
                    search_graph::Target::Cycle(node) => backprop_cycle(node),
                    search_graph::Target::Unexpanded(_) => panic!("Edge expansion failed"),
                }
            },
            Rollout::Terminal(node) => match payoff(&state) {
                None => {
                    println!("i am confused by this board state:");
                    console_ui::write_board(state.board());
                    panic!("Terminal node has no payoff")
                },
                Some(p) => backprop_known_payoff(node, p),
            },
            Rollout::Cycle(_) => panic!("Hit cycle in search"),
        }
    } else {
        panic!("Unknown state")
    }
}
