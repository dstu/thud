use ::mcts::base::*;

use ::console_ui;
use ::game;
use ::search_graph;

use std::collections::HashSet;
use std::cmp;

pub enum Rollout<'a> {
    Terminal(MutNode<'a>),
    Unexpanded(EdgeExpander<'a>),
    Cycle(MutNode<'a>),
}

pub fn rollout<'a>(mut node: MutNode<'a>, state: &mut game::State, bias: f64) -> Rollout<'a> {
    let mut path = HashSet::new();
    path.insert(node.get_id());
    loop {
        node = match best_child_edge(node.get_child_list(), state.active_player().marker(), bias) {
            Some(i) => {
                let mut cycle = false;
                if let search_graph::Target::Expanded(n) = node.get_child_list().get_edge(i).get_target() {
                    cycle = !path.insert(n.get_id());
                }
                if cycle {
                    return Rollout::Cycle(node)
                }
                match node.to_child_list().to_edge(i).to_target() {
                    search_graph::Target::Unexpanded(e) => return Rollout::Unexpanded(e),
                    search_graph::Target::Expanded(n) => n,
                }
            },
            None => {
                println!("rollout: no children for (id {}) with board state:", node.get_id());
                console_ui::write_board(state.board());
                return Rollout::Terminal(node)
            },
        }
    }
}

fn best_child_edge<'a>(children: ChildList<'a>, player: game::PlayerMarker, bias: f64) -> Option<usize> {
    let parent_visits = cmp::max(1, children.get_source_node().get_data().statistics.visits) as f64;
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
                    if uct >= best_edge_uct {
                        // println!("best_child_edge: edge {}, value {} is new best edge (old index {:?}, value {})",
                        //          i, uct, best_edge_index, best_edge_uct);
                        // TODO tie-breaking.
                        best_edge_index = Some(i);
                        best_edge_uct = uct;
                    } // else {
                    //     println!("best_child_edge: edge {}, value {} skipped in favor of {:?}, value {}",
                    //              i, uct, best_edge_index, best_edge_uct);
                    // }
                }
            },
        }
    }
    best_edge_index
}
