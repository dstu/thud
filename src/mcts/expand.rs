use ::game;
use ::mcts::base::*;
use ::mcts::payoff::{Payoff, payoff};
use ::mcts::statistics::EdgeData;
use ::game::board::format_board;
use ::rand::Rng;

pub fn expand<'a, R: Rng>(expander: EdgeExpander<'a>, state: State, rng: &mut R,
                          simulation_count: usize) -> (MutNode<'a>, game::Role, Payoff) {
    match expander.expand_to_edge(state.clone(), Default::default) {
        ::search_graph::Expanded::New(e) => {
            let mut target = e.to_target();
            trace!("expand: made new node {}; now to play: {:?}; board: {}",
                   target.get_id(), state.active_player(), format_board(state.board()));
            expand_children(&mut target, &state);
            let mut payoff = Payoff { weight: 0, values: [0, 0], };
            for _ in 0..simulation_count {
                payoff += simulate(&mut state.clone(), rng);
            }
            trace!("expand: simulation of new node {} gives payoff {:?}", target.get_id(), payoff);
            (target, state.active_player().role(), payoff)
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
            trace!("expand: edge {} given payoff {:?} from extant node {}", e.get_id(), payoff, e.get_target().get_id());
            (e.to_source(), state.active_player().role().toggle(), payoff)
        },
    }
}

pub fn simulate<R>(state: &mut State, rng: &mut R) -> Payoff where R: Rng {
    loop {
        let action = match payoff(&state) {
            None => {
                let actions: Vec<game::Action> =
                    state.role_actions(state.active_player().role()).collect();
                match rng.choose(&actions) {
                    Some(a) => *a,
                    None => panic!("terminal state has no payoff (role: {:?}, actions: {:?}); board: {}",
                                   state.active_player().role(), actions, format_board(state.board())),
                }
            },
            Some(p) => return p,
        };
        state.do_action(&action);
    }
}

fn expand_children<'a>(node: &mut MutNode<'a>, state: &State) {
    trace!("expanding moves at node {} for {:?}", node.get_id(), state.active_player());
    let mut children = node.get_child_list_mut();
    let mut i = 0;
    for a in state.role_actions(state.active_player().role()).into_iter() {
        let _ = children.add_child(EdgeData::new(a));
        i += 1;
    }
    trace!("expand_children: added {} children (now {} total)", i, children.len());
}
