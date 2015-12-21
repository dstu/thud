use ::actions;
use ::board;
use ::game;
use ::search_graph;

use std::cmp;

use std::fmt::Debug;
use rand::Rng;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug)]
pub struct NodeData {
    cycle: bool,
    known_payoff: Option<Payoff>,
    statistics: Statistics,
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
    action: actions::Action,
    cycle: bool,
    known_payoff: Option<Payoff>,
    statistics: Statistics,
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
pub type MutNode<'a> = search_graph::MutNode<'a, game::State, NodeData, EdgeData>;
pub type ChildList<'a> = search_graph::ChildList<'a, game::State, NodeData, EdgeData>;
pub type MutParentList<'a> = search_graph::MutParentList<'a, game::State, NodeData, EdgeData>;

pub enum Rollout<'a> {
    Unexpanded(Edge<'a>),
    Terminal(Node<'a>),
    Cycle(Node<'a>),
}

pub fn rollout<'a>(mut node: Node<'a>, state: &mut game::State, bias: f64) -> Rollout<'a> {
    loop {
        let children = node.child_list();
        if children.is_empty() {
            return Rollout::Terminal(node)
        }
        match best_child_edge(children, state.active_player().marker(), bias) {
            None => return Rollout::Cycle(node),
            Some(edge) => match edge.target() {
                search_graph::Target::Unexpanded(_) => return Rollout::Unexpanded(edge),
                search_graph::Target::Expanded(n) => node = n,
                search_graph::Target::Cycle(n) => panic!("Cycle detection failed"),
            },
        }
    }
}

pub fn best_child_edge<'a>(children: ChildList<'a>, player: game::PlayerMarker, bias: f64) -> Option<Edge<'a>> {
    let parent_visits = children.source_node().data().statistics.visits as f64;
    let mut best_edge = None;
    let mut best_edge_uct = 0.0;
    for i in 0..children.len() {
        let edge = children.get_edge(i);
        match edge.target() {
            search_graph::Target::Unexpanded(_) => return Some(edge),
            search_graph::Target::Cycle(_) => (),
            search_graph::Target::Expanded(child) => {
                let edge_visits = edge.data().statistics.visits as f64;
                let edge_payoff = edge.data().statistics.payoff.score(player) as f64;
                let uct = edge_payoff / edge_visits
                    + bias * f64::sqrt(f64::ln(parent_visits) / edge_visits);
                if uct > best_edge_uct {
                    // TODO tie-breaking.
                    best_edge = Some(edge);
                    best_edge_uct = uct;
                }
            },
        }
    }
    best_edge
}

pub fn best_parent_edge<'a>(parents: MutParentList<'a>, player: game::PlayerMarker, bias: f64) -> MutEdge<'a> {
    assert!(!parents.is_empty());
    let mut best_edge_index = 0;
    let mut best_edge_uct = 0.0;
    let mut i = 0;
    while i < parents.len() {
        let edge = parents.get_edge(i);
        let parent_visits = edge.source().data().statistics.visits as f64;
        let edge_payoff = edge.data().statistics.payoff.score(player) as f64;
        let edge_visits = edge.data().statistics.visits as f64;
        let uct = edge_payoff / edge_visits
            + bias * f64::sqrt(f64::ln(parent_visits) / edge_visits);
        if uct > best_edge_uct {
            // TODO tie-breaking.
            best_edge_index = i;
            best_edge_uct = uct;
        }
        i += 1;
    }
    parents.to_edge_mut(i)
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
                selected.expect("Terminal state has no payoff")
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
        node.data_mut().statistics.increment_visit(payoff);
        if node.is_root() {
            break
        }
        let mut parent_edge = best_parent_edge(node.to_parent_list_mut(), player, bias);
        parent_edge.data_mut().statistics.increment_visit(payoff);
        node = parent_edge.to_source();
        player.toggle();
    }
}

fn backprop_cycle<'a>(mut node: MutNode<'a>) {
    // loop {
        node.data_mut().cycle = true;
        let mut parents = node.to_parent_list_mut();
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
                search_graph::Target::Expanded(ref node) if !node.data().cycle => return,
                _ => (),
            }
        }
    }
    node.data_mut().cycle = true;
}

fn backprop_known_payoff<'a>(mut node: MutNode<'a>, p: Payoff) {
    // loop {
        node.data_mut().known_payoff = Some(p);
        let mut parents = node.to_parent_list_mut();
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
                    match node.data().known_payoff {
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
    node.data_mut().known_payoff = payoff;
}

pub fn expand<'a>(mut edge: MutEdge<'a>, state: game::State) -> MutEdge<'a> {
    if let search_graph::Target::Unexpanded(e) = edge.to_target() {
        e.expand(state, EdgeData::new, Default::default)
    } else {
        panic!("Edge is already expanded");
    }
}

pub fn iterate_search<'a, R>(rng: &mut R, graph: &'a mut Graph, mut state: game::State, bias: f64) where R: Rng {
    if let Some(node) = graph.get_node(&state) {
        match rollout(node, &mut state, bias) {
            Rollout::Unexpanded(edge) => {
                state.do_action(&edge.data().action);
                match graph.promote_edge_mut(edge).to_target() {
                    search_graph::Target::Unexpanded(expander) => {
                        let expanded = expander.expand(
                            state.clone(), EdgeData::new, Default::default);
                        if let search_graph::Target::Expanded(mut_node) = expanded.to_target() {
                            let payoff = simulate(&mut state, rng);
                            backprop_payoff(mut_node, payoff, state.active_player().marker(), bias);
                        } else {
                            panic!("Edge expansion failed");
                        }
                    },
                    _ => panic!("Edge is actually expanded"),
                }
            },
            Rollout::Cycle(node) => {
                let mut_node = graph.promote_node_mut(node);
                backprop_cycle(mut_node);
            },
            Rollout::Terminal(node) => {
                let mut_node = graph.promote_node_mut(node);
                match payoff(&state) {
                    None => panic!("Terminal node has no payoff"),
                    Some(p) => backprop_known_payoff(mut_node, p),
                }
            },
        }
    }
}
