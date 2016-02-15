mod base;
mod backprop;
mod payoff;
mod rollout;
mod statistics;
mod simulation;

pub use self::base::*;
use self::backprop::*;
use self::payoff::*;
pub use self::statistics::*;
use self::rollout::*;
use self::simulation::*;

use ::rand::Rng;
use ::console_ui;
use ::game;
use ::search_graph;

pub fn iterate_search<'a, R>(mut state: game::State, graph: &'a mut Graph, rng: &mut R, bias: f64) where R: Rng {
    if let Some(node) = graph.get_node_mut(&state) {
        match rollout(node, &mut state, bias) {
            Rollout::Unexpanded(expander) => {
                let action = expander.get_edge().get_data().action;
                state.do_action(&action);
                match expander.expand(state.clone(), Default::default, || { EdgeData::new(action) }).to_target() {
                    search_graph::Target::Expanded(mut node) => {
                        {
                            let mut children = node.get_child_list_mut();
                            for a in state.role_actions(state.active_player().role()).into_iter() {
                                children.add_child(EdgeData::new(a));
                            }
                        }
                        let payoff = simulate(&mut state, rng);
                        let payoff_player = state.active_player().marker();
                        // TODO: make sure correct player is getting payoff. We
                        // may get payoff *after* the player getting the payoff
                        // has moved.
                        backprop_payoff(node, payoff, payoff_player, bias);
                    },
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
