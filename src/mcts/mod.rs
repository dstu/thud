mod base;
mod backprop;
mod payoff;
mod rollout;
mod statistics;
mod simulation;
mod ucb;

use self::backprop::*;
use self::payoff::*;
use self::rollout::*;
use self::simulation::*;

pub use self::base::*;
pub use self::statistics::*;

use ::rand::Rng;
use ::console_ui;
use ::game;

use std::collections::HashSet;

pub fn iterate_search<'a, R>(state: game::State, graph: &'a mut Graph, rng: &mut R, bias: f64) where R: Rng {
    if let Some(node) = graph.get_node_mut(&state) {
        iterate_search_helper(state, node, rng, bias, HashSet::new());
    } else {
        console_ui::write_board(state.cells());
        panic!("Unknown root state")
    }
}

fn iterate_search_helper<'a, R>(mut state: game::State, node: MutNode<'a>,
                                rng: &mut R, bias: f64, bad_children: HashSet<usize>) where R: Rng {
    let node_id = node.get_id();
    if bad_children.len() == node.get_child_list().len() {
        panic!("ran out of children to look at [node paths are all cycles]")
    }
    match rollout(node, &mut state, bias, rng) {
        Rollout::Internal(expander) => {
            let action = expander.get_edge().get_data().action;
            // info!("iterate_search: expanding from id {}", expander.get_edge().get_source().get_id());
            // console_ui::write_board(state.cells());
            state.do_action(&action);
            let mut expanded_node =
                expander.expand_to_target(state.clone(), Default::default);
            // trace!("iterate_search: after expansion, {} parents", expanded_node.get_parent_list().len());
            // info!("iterate_search: expanding to id {}", expanded_node.get_id());
            // console_ui::write_board(state.cells());
            {
                let mut children = expanded_node.get_child_list_mut();
                for a in state.role_actions(state.active_player().role()).into_iter() {
                    let mut child = children.add_child(EdgeData::new(a));
                }
            }
            let mut backprop_player = state.active_player().marker();
            backprop_player.toggle();
            let payoff = simulate(&mut state, rng);
            backprop_payoff(expanded_node, payoff, backprop_player, bias, rng);
        },
        Rollout::Terminal(node) => {
            trace!("iterate_search: found terminal node");
            match payoff(&state) {
                None => {
                    error!("I am confused by this board state, which is terminal but has no payoff:");
                    console_ui::write_board(state.board());
                    panic!("Terminal node has no payoff")
                },
                Some(p) => backprop_known_payoff(node, p),
            }
        },
        Rollout::Cycle(mut cyclic_edge) => {
            trace!("iterate_search: hit cycle in search");
            match cyclic_edge.get_target() {
                ::search_graph::Target::Expanded(child_node) =>
                    if child_node.get_id() == node_id {
                        // Cycle wrapped back around to here.
                        trace!("cycle back to root");
                    } else {
                        // Cycle intersected with some child of original node.
                        trace!("cycle to intermediate vertex: {}", child_node.get_id());
                    },
                ::search_graph::Target::Unexpanded(_) =>
                    panic!("encountered an unexpanded node in a graph cycle"),
            }
            // We "punish" the last edge in the cycle and the vertex it comes
            // from by pretending we've visited them without having any payoff,
            // thereby diluting their statistics and discouraging a visit in the
            // immediate future.
            // TODO: This is a hack. Most problematically, it doesn't adequately
            // handle the case of all paths looping back to root. In that case,
            // we are stuck in a loop incrementing the visit count ad infinitum.
            cyclic_edge.get_data_mut().statistics.visits += 1;
            cyclic_edge.get_source_mut().get_data_mut().statistics.visits += 1;
        },
        Rollout::Err(e) =>
            panic!("error in UCB computation {:?}", e),
    }
}
