use ::actions;
use ::board;
use ::game;
use ::search_graph;

use rand::Rng;
use std::ops::Add;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Payoff {
    values: [usize; 2],
}

impl Payoff {
    pub fn new() -> Self {
        Payoff { values: [0; 2], }
    }

    pub fn increment(&mut self, player: game::PlayerMarker, value: usize) {
        self.values[player.index()] += value
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

impl Add for Payoff {
    type Output = Payoff;

    fn add(self, other: Payoff) -> Payoff {
        Payoff { values: [self.values[0] + other.values[0],
                          self.values[1] + other.values[1]], }
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

impl Add for Statistics {
    type Output = Statistics;

    fn add(self, other: Statistics) -> Statistics {
        Statistics { visits: self.visits + other.visits,
                     payoff: self.payoff + other.payoff, }
    }
}

#[derive(Clone, Debug)]
pub struct ActionData {
    action: actions::Action,
    payoff: Statistics,
}

pub type Edge<'a> = search_graph::Edge<'a, Statistics, ActionData>;
pub type Graph = search_graph::Graph<Statistics, ActionData>;
pub type Node<'a> = search_graph::Node<'a, Statistics, ActionData>;

pub enum RolloutResult<'a> {
    Unexpanded(Edge<'a>),
    Terminal(Node<'a>),
    Cycle,
}

pub fn rollout<'a>(mut node: Node<'a>, state: &mut game::State) -> RolloutResult<'a> {
    loop {
        let children = node.child_list();
        if children.is_empty() {
            return RolloutResult::Terminal(node)
        }
        let mut i = 0;
        let mut best_child_index = 0;
        let mut best_child_score = 0.0;
        while i < children.len() {
            let edge = children.get_edge(i);
            match edge.target() {
                search_graph::Target::Unexpanded => {
                    state.do_action(&edge.data().action);
                    return RolloutResult::Unexpanded(edge)
                },
                search_graph::Target::Cycle(_) => (),
                search_graph::Target::Expanded(_) => {
                    let child_score = 0.0;
                    if child_score > best_child_score {
                        // TODO tie-breaking.
                        best_child_score = child_score;
                        best_child_index = i;
                    }
                },
            }
            i += 1;
        }
        let edge = children.get_edge(best_child_index);
        match edge.target() {
            search_graph::Target::Expanded(child) => {
                state.do_action(&edge.data().action);
                node = child;
            },
            _ => return RolloutResult::Cycle,
        }
    }
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
