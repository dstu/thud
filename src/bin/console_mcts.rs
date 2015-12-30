use std::str::FromStr;

extern crate thud;
use ::thud::board;
use ::thud::console_ui;
use ::thud::game;
use ::thud::mcts;

extern crate rand;

fn main() {
    let mut rng = rand::thread_rng();
    let state = game::State::new(board::Cells::default(), String::from_str("Player 1").ok().unwrap(), String::from_str("Player 2").ok().unwrap());
    let mut graph = mcts::Graph::new();
    graph.add_root(state.clone(), Default::default());
    for _ in 0..10 {
        mcts::iterate_search(&mut rng, &mut graph, state.clone(), 1.0);
        console_ui::write_search_graph(&graph, &state);
    }
}
