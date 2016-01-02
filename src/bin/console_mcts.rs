use std::str::FromStr;

extern crate thud;
use ::thud::actions;
use ::thud::board;
use ::thud::console_ui;
use ::thud::game;
use ::thud::mcts;

extern crate rand;

pub fn initialize_search(state: game::State, graph: &mut mcts::Graph) {
    let actions: Vec<actions::Action> = state.role_actions(state.active_player().role()).collect();
    let mut adder = graph.add_root(state, Default::default()).to_child_adder();
    for a in actions.into_iter() {
        adder.add(mcts::EdgeData::new(a));
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    let state = game::State::new(board::Cells::default(), String::from_str("Player 1").ok().unwrap(), String::from_str("Player 2").ok().unwrap());
    let mut graph = mcts::Graph::new();
    initialize_search(state.clone(), &mut graph);
    for _ in 0..10000 {
        mcts::iterate_search(state.clone(), &mut graph, &mut rng, 1.0);
        console_ui::write_search_graph(&graph, &state);
    }
}
