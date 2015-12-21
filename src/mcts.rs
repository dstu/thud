use ::actions;
use ::board;
use ::game;
use ::search_graph;

use rand::Rng;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Payoff {
    values: [usize; 2],
}

impl Payoff {
    pub fn new() -> Self {
        Payoff { values: [0; 2], }
    }

    pub fn score(&self, player: game::PlayerMarker) -> usize {
        self.values[player.index()]
    }
}

impl Default for Payoff {
    fn default() -> Self {
        Payoff::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Statistics {
    visits: usize,
    payoff: Payoff,
}

impl Statistics {
    pub fn new() -> Self {
        Statistics {
            visits: 0,
            payoff: Payoff::new(),
        }
    }

    pub fn increment_visit(&mut self, p: Payoff) {
        self.visits += 1;
        self.payoff.values[0] += p.values[0];
        self.payoff.values[1] += p.values[1];
    }
}

impl Default for Statistics {
    fn default() -> Self {
        Statistics::new()
    }
}

#[derive(Clone, Debug)]
pub struct ActionData {
    action: actions::Action,
    payoff: Statistics,
}

pub type Edge<'a> = search_graph::Edge<'a, Statistics, ActionData>;
pub type MutEdge<'a> = search_graph::MutEdge<'a, Statistics, ActionData>;
pub type Graph = search_graph::Graph<Statistics, ActionData>;
pub type Node<'a> = search_graph::Node<'a, Statistics, ActionData>;
pub type MutNode<'a> = search_graph::MutNode<'a, Statistics, ActionData>;
pub type ChildList<'a> = search_graph::ChildList<'a, Statistics, ActionData>;
pub type MutParentList<'a> = search_graph::MutParentList<'a, Statistics, ActionData>;

pub enum RolloutResult<'a> {
    Unexpanded(Edge<'a>),
    Terminal(Node<'a>),
    Cycle(Node<'a>),
}

pub fn rollout<'a>(mut node: Node<'a>, state: &mut game::State, bias: f64) -> RolloutResult<'a> {
    loop {
        let children = node.child_list();
        if children.is_empty() {
            return RolloutResult::Terminal(node)
        }
        match best_child_edge(children, state.active_player().marker(), bias) {
            None => return RolloutResult::Cycle(node),
            Some(edge) => match edge.target() {
                search_graph::Target::Unexpanded => return RolloutResult::Unexpanded(edge),
                search_graph::Target::Expanded(n) => node = n,
                search_graph::Target::Cycle(n) => panic!("Cycle detection failed"),
            },
        }
    }
}

pub fn best_child_edge<'a>(children: ChildList<'a>, player: game::PlayerMarker, bias: f64) -> Option<Edge<'a>> {
    let parent_visits = children.source_node().data().visits as f64;
    let mut best_edge = None;
    let mut best_edge_uct = 0.0;
    let mut i = 0;
    while i < children.len() {
        let edge = children.get_edge(i);
        match edge.target() {
            search_graph::Target::Unexpanded => return Some(edge),
            search_graph::Target::Cycle(_) => (),
            search_graph::Target::Expanded(child) => {
                let edge_visits = edge.data().payoff.visits as f64;
                let edge_payoff = edge.data().payoff.payoff.score(player) as f64;
                let uct = edge_payoff / edge_visits
                    + bias * f64::sqrt(f64::ln(parent_visits) / edge_visits);
                if uct > best_edge_uct {
                    // TODO tie-breaking.
                    best_edge = Some(edge);
                    best_edge_uct = uct;
                }
            },
        }
        i += 1;
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
        let parent_visits = edge.source().data().visits as f64;
        let edge_payoff = edge.data().payoff.payoff.score(player) as f64;
        let edge_visits = edge.data().payoff.visits as f64;
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

pub fn simulate<R>(mut state: game::State, mut rng: R) -> Payoff where R: Rng {
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
        let mut payoff = Payoff::new();
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

pub fn backprop<'a>(mut node: MutNode<'a>, payoff: Payoff, mut player: game::PlayerMarker, bias: f64) {
    loop {
        node.data_mut().increment_visit(payoff);
        if node.is_root() {
            break
        }
        let mut parent_edge = best_parent_edge(node.to_parent_list_mut(), player, bias);
        parent_edge.data_mut().payoff.increment_visit(payoff);
        node = parent_edge.to_source();
        player.toggle();
    }
}
