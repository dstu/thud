use std::str::FromStr;

extern crate chrono;
extern crate fern;
#[macro_use]
extern crate log;
extern crate rand;
extern crate thud;

use ::thud::console_ui;
use ::thud::game;
use ::thud::game::board;
use ::thud::mcts;

use std::env::args;

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
    if let Err(e) = fern::init_global_logger(logger_config, log::LogLevelFilter::Info) {
        panic!("Failed to initialize global logger: {}", e);
    }
    let mut rng = rand::thread_rng();
    let state = game::State::new(board::Cells::default(), String::from_str("Player 1").ok().unwrap(), String::from_str("Player 2").ok().unwrap());
    let mut graph = mcts::Graph::new();

    let iteration_count = args().skip(1).next().unwrap().parse::<usize>().ok().unwrap();
    initialize_search(state.clone(), &mut graph);
    for iteration in 0..iteration_count {
        if iteration % 1000 == 0 {
            info!("iteration: {} / {} = {}%", iteration, iteration_count,
                  ((10000.0 * (iteration as f64) / (iteration_count as f64)) as usize as f64) / 100.0);
        }
        mcts::iterate_search(state.clone(), &mut graph, &mut rng, 0.1);
    }
    println!("top-level choices:");
    for child in graph.get_node(&state).unwrap().get_child_list().iter() {
        println!("{:?}: [{}, {}] = {:?} / {}", child.get_data().action,
                 (child.get_data().statistics.payoff.values[0] as f64) / (child.get_data().statistics.visits as f64),
                 (child.get_data().statistics.payoff.values[1] as f64) / (child.get_data().statistics.visits as f64),
                 child.get_data().statistics.payoff, child.get_data().statistics.visits);
    }
    // console_ui::write_search_graph(&graph, &state);
}
