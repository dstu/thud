use ::mcts::base::*;
use ::mcts::ucb;

use ::console_ui;
use ::game;

use ::search_graph;
use ::rand::Rng;

use std::collections::HashSet;
use std::cmp::Ordering;

pub enum Result<'a> {
    Terminal(MutNode<'a>),
    Internal(EdgeExpander<'a>),
    Cycle(MutNode<'a>),
    Err(ucb::UcbError),
}

pub fn rollout<'a, R: Rng>(mut node: MutNode<'a>, state: &mut game::State, bias: f64, epoch: usize,
                           rng: &mut R) -> Result<'a> {
    loop {
        if node.get_data().visited_in_rollout_epoch(epoch) {
            return Result::Cycle(node)
        }
        node = match ucb::find_best_child_edge(&node.get_child_list(),
                                               state.active_player().marker(), epoch, bias, rng) {
            ::std::result::Result::Ok(i) => {
                let children = node.to_child_list();
                state.do_action(&children.get_edge(i).get_data().action);
                match children.to_edge(i).to_target() {
                    search_graph::Target::Unexpanded(e) => return Result::Internal(e),
                    search_graph::Target::Expanded(n) => n,
                }
            },
            ::std::result::Result::Err(e) => return Result::Err(e),
        }
    }
}
