mod base;
mod backprop;
mod payoff;
mod rollout;
mod statistics;
mod simulation;

use self::backprop::*;
use self::payoff::*;
use self::rollout::*;
use self::simulation::*;

pub use self::base::*;
pub use self::statistics::*;

use ::rand::Rng;
use ::console_ui;
use ::game;

pub fn iterate_search<'a, R>(mut state: game::State, graph: &'a mut Graph, rng: &mut R, bias: f64) where R: Rng {
    if let Some(node) = graph.get_node_mut(&state) {
        match rollout(node, &mut state, bias) {
            Rollout::Unexpanded(expander) => {
                let action = expander.get_edge().get_data().action;
                state.do_action(&action);
                let mut expanded_node =
                    expander.expand_to_target(
                        state.clone(), Default::default, || EdgeData::new(action));
                {
                    let mut children = expanded_node.get_child_list_mut();
                    for a in state.role_actions(state.active_player().role()).into_iter() {
                        children.add_child(EdgeData::new(a));
                    }
                }
                let payoff = simulate(&mut state, rng);
                let payoff_player = state.active_player().marker();
                // TODO: make sure correct player is getting payoff. We
                // may get payoff *after* the player getting the payoff
                // has moved.
                backprop_payoff(expanded_node, payoff, payoff_player, bias);
            },
            Rollout::Terminal(node) => match payoff(&state) {
                None => {
                    println!("I am confused by this board state, which is terminal but has no payoff:");
                    console_ui::write_board(state.board());
                    panic!("Terminal node has no payoff")
                },
                Some(p) => backprop_known_payoff(node, p),
            },
            Rollout::Cycle(_) => panic!("Hit cycle in search"),
        }
    } else {
        panic!("Unknown root state")
    }
}
