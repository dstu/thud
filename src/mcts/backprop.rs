use ::mcts::base::*;
use ::game;
use ::mcts::payoff::Payoff;
use ::search_graph;

use std::collections::HashSet;
use std::cmp;

fn best_parent_edge<'a>(parents: ParentList<'a>, player: game::PlayerMarker, bias: f64) -> Option<usize> {
    let mut best_edge_index = None;
    let mut best_edge_uct = 0.0;
    for i in 0..parents.len() {
        let edge = parents.get_edge(i);
        let parent_visits = edge.get_source().get_data().statistics.visits as f64;
        if parent_visits == 0.0 {
            continue
        }
        let edge_payoff = edge.get_data().statistics.payoff.score(player) as f64;
        let edge_visits = edge.get_data().statistics.visits as f64;
        if edge_visits == 0.0 {
            println!("best_parent_edge: choosing parent {} because this edge hasn't been visited", i);
            return Some(i)
        } else {
            let uct = edge_payoff / edge_visits
                + bias * f64::sqrt(f64::ln(parent_visits) / edge_visits);
            if uct >= best_edge_uct {
                // TODO tie-breaking.
                best_edge_index = Some(i);
                best_edge_uct = uct;
            }
        }
    }
    best_edge_index
}

pub fn backprop_payoff<'a>(mut node: MutNode<'a>, payoff: Payoff, mut player: game::PlayerMarker, bias: f64) {
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
        node.get_data_mut().statistics.increment_visit(payoff);
        player.toggle();
    }
}

pub fn backprop_known_payoff<'a>(mut node: MutNode<'a>, p: Payoff) {
    let mut visited = HashSet::new();
    node.get_data_mut().known_payoff = Some(p);
    visited.insert(node.get_id());
    let mut parents = node.to_parent_list();
    for i in 0..parents.len() {
        let mut parent = parents.get_edge_mut(i).to_source();
        if !visited.insert(parent.get_id()) {
            set_known_payoff_from_children(&mut parent);
            match parent.get_data().known_payoff {
                Some(p) =>
                    // TODO fix potential stack exhaustion.
                    backprop_known_payoff(parent, p),
                None => (),
            }
        }
    }
}

fn set_known_payoff_from_children<'a>(node: &mut MutNode<'a>) {
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
                        None => {
                            payoff = None;
                            break
                        },
                        Some(known) => payoff = match payoff {
                            None => Some(known),
                            Some(p) => Some(Payoff { values: [cmp::max(p.values[0], known.values[0]),
                                                              cmp::max(p.values[1], known.values[1])], }),
                        },
                    },
            }
        }
    }
    node.get_data_mut().known_payoff = payoff;
}
