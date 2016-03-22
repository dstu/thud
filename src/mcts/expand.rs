use ::console_ui;
use ::game;
use ::mcts::base::*;
use ::mcts::payoff::{Payoff, payoff};
use ::mcts::statistics::EdgeData;
use ::rand::Rng;

pub fn expand<'a, R>(expander: EdgeExpander<'a>, mut state: game::State, rng: &mut R)
                     -> (MutNode<'a>, Payoff) where R: Rng{
    state.do_action(&expander.get_edge().get_data().action);
    match expander.expand_to_edge(state.clone(), Default::default) {
        ::search_graph::Expanded::New(mut e) => {
            let mut target = e.to_target();
            trace!("expand: made new node {}", target.get_id());
            expand_children(&mut target, &state);
            (target, simulate(&mut state, rng))
        },
        ::search_graph::Expanded::Extant(e) => {
            let payoff = {
                let target = e.get_target();
                trace!("expand: hit extant node {}", target.get_id());
                let stats = target.get_data().statistics.get();
                Payoff { weight: stats.visits,
                         values: stats.payoff.values.clone(), }
            };
            {
                let mut edge_stats = e.get_data().statistics.get();
                edge_stats.increment_visit(payoff);
                e.get_data().statistics.set(edge_stats);
            }
            (e.to_source(), payoff)
        },
    }
}

pub fn simulate<R>(state: &mut game::State, rng: &mut R) -> Payoff where R: Rng {
    loop {
        let action = match payoff(&state) {
            None => {
                let actions: Vec<game::Action> =
                    state.role_actions(state.active_player().role()).collect();
                match rng.choose(&actions) {
                    Some(a) => *a,
                    None => {
                        console_ui::write_board(state.board());
                        panic!("terminal state has no payoff")
                    },
                }
            },
            Some(p) => return p,
        };
        state.do_action(&action);
    }
}

fn expand_children<'a>(node: &mut MutNode<'a>, state: &game::State) {
    let mut children = node.get_child_list_mut();
    let mut i = 0;
    for a in state.role_actions(state.active_player().role()).into_iter() {
        let _ = children.add_child(EdgeData::new(a));
        i += 1;
    }
    trace!("expand_children: added {} children (now {} total)", i, children.len());
}
