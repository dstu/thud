use ::mcts::base::*;
use ::mcts::ucb;

use ::console_ui;
use ::game;

use ::search_graph;
use ::rand::Rng;

use std::collections::HashSet;
use std::cmp::Ordering;

pub enum Rollout<'a> {
    Terminal(MutNode<'a>),
    Internal(EdgeExpander<'a>),
    Cycle(MutEdge<'a>),
    Err(ucb::Error),
}

pub fn rollout<'a, R: Rng>(mut node: MutNode<'a>, state: &mut game::State, bias: f64, rng: &mut R) -> Rollout<'a> {
    let mut path = HashSet::new();
    path.insert(node.get_id());
    loop {
        node = match ucb::find_best_child_edge(&node.get_child_list(), state.active_player().marker(), bias, rng) {
            Result::Ok(i) => {
                let mut cycle = false;
                if let search_graph::Target::Expanded(n) = node.get_child_list().get_edge(i).get_target() {
                    trace!("rollout: best child of node {} is edge {} (to node {})", node.get_id(), node.get_child_list().get_edge(i).get_id(), n.get_id());
                    cycle = !path.insert(n.get_id());
                }
                if cycle {
                    return Rollout::Cycle(node.to_child_list().to_edge(i))
                }
                let child_action = node.get_child_list().get_edge(i).get_data().action;
                let node_id = node.get_id();
                match node.to_child_list().to_edge(i).to_target() {
                    search_graph::Target::Unexpanded(e) => {
                        trace!("rollout: found unexpanded edge {} (from node {})", e.get_edge().get_id(), node_id);
                        return Rollout::Internal(e)
                    },
                    search_graph::Target::Expanded(n) => {
                        state.do_action(&child_action);
                        n
                    },
                }
            },
            Result::Err(e) => {
                error!("error {:?} computing maximal UCB child for id {} with board state:", e, node.get_id());
                console_ui::write_board(state.board());
                return Rollout::Err(e)
            },
        }
    }
}
