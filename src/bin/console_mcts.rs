use std::str::FromStr;

extern crate chrono;
extern crate fern;
extern crate log;
extern crate thud;

use ::thud::console_ui;
use ::thud::game;
use ::thud::game::board;
use ::thud::mcts;

extern crate rand;

pub fn initialize_search(state: game::State, graph: &mut mcts::Graph) {
    let actions: Vec<game::Action> = state.role_actions(state.active_player().role()).collect();
    let mut children = graph.add_root(state, Default::default()).to_child_list();
    for a in actions.into_iter() {
        children.add_child(mcts::EdgeData::new(a));
    }
}

fn main() {
    let logger_config = fern::DispatchConfig {
        format: Box::new(|msg: &str, level: &log::LogLevel, _location: &log::LogLocation| {
            format!("[{}][{}] {}", chrono::Local::now().to_rfc3339(), level, msg)
        }),
        output: vec![fern::OutputConfig::stdout()],
        level: log::LogLevelFilter::Trace,
    };
    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Trace) {
        panic!("Failed to initialize global logger: {}", e);
    }
    let mut rng = rand::thread_rng();
    let state = game::State::new(board::Cells::default(), String::from_str("Player 1").ok().unwrap(), String::from_str("Player 2").ok().unwrap());
    let mut graph = mcts::Graph::new();
    initialize_search(state.clone(), &mut graph);
    for _ in 0..795 {
        mcts::iterate_search(state.clone(), &mut graph, &mut rng, 1.0);
    }
    console_ui::write_search_graph(&graph, &state);
}
